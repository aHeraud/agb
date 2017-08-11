use gameboy::Mode;

pub const ZERO_FLAG: u8 = 1 << 7;
pub const SUBTRACTION_FLAG: u8 = 1 << 6;
pub const HALF_CARRY_FLAG: u8 = 1 << 5;
pub const CARRY_FLAG: u8 = 1 << 4;

#[cfg(feature = "no_std")]
use collections::fmt;

#[cfg(not(feature = "no_std"))]
use std::fmt;

const HRAM_SIZE: usize = 127;

///On the DMG/CGB the EI instruction, the value of ime isn't changed until after the next instruction,
///I assume this is because of instruction pipelining, and the next instruction has been fetched before
///interrupts have been enabled.
///On the CGB, the same applies to the DI instruction (but not on the DMG) allegedly

#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub enum Register {
	B, C, D, E, H, L, AT_HL, A, F
}

pub struct CPU {
	pub registers: Registers,
	pub ime: bool,
	pub next_ime_state: bool,
	pub stop: bool,
	pub halt: bool,
	pub hram: [u8; HRAM_SIZE],
	pub ier: u8,
	pub double_speed_mode: bool,
	pub cycle_counter: usize,
}

impl CPU {
	pub fn new() -> CPU {
		CPU {
			registers: Registers::new(),
			ime: false,	//TODO: default value of ime
			next_ime_state: false,
			ier: 0,	//TODO: default value of ier
			stop: false,
			halt: false,
			hram: [0; HRAM_SIZE],
			double_speed_mode: false,
			cycle_counter: 0
		}
	}

	pub fn reset(&mut self, mode: Mode) {
		match mode {
			Mode::DMG => self.registers.init_dmg(),
			Mode::CGB => self.registers.init_cgb(),
		};
		self.ime = false;
		self.next_ime_state = false;
		self.ier = 0;
		self.stop = false;
		self.halt = false;
		self.double_speed_mode = false;
	}

	pub fn read_byte_hram(&self, address: u16) -> u8 {
		match address {
			0xFF80...0xFFFE => self.hram[(address - 0xFF80) as usize],
			_ => panic!("cpu::read_byte_hram: invalid arguments, address must be in the range [0xFF80, 0xFFFE]"),
		}
	}

	pub fn write_byte_hram(&mut self, address: u16, value: u8) {
		match address {
			0xFF80...0xFFFE => self.hram[(address - 0xFF80) as usize] = value,
			_ => panic!("cpu::read_byte_hram: invalid arguments, address must be in the range [0xFF80, 0xFFFE]"),
		};
	}
}

#[derive(Default, Copy, Clone)]
pub struct Registers {
	pub a: u8,
	pub f: u8,
	pub b: u8,
	pub c: u8,
	pub d: u8,
	pub e: u8,
	pub h: u8,
	pub l: u8,
	pub sp: u16,
	pub pc: u16,
}

#[derive(Copy, Clone)]
pub enum RegisterPair {
	AF, BC, DE, HL, SP
}

impl fmt::Debug for Registers {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f,
			"Registers {{\n\
				\tA:{:#X}\n\
				\tF:{:#X}\n\
				\tB:{:#X}\n\
				\tC:{:#X}\n\
				\tD:{:#X}\n\
				\tE:{:#X}\n\
				\tH:{:#X}\n\
				\tL:{:#X}\n\
				\tSP:{:#X}\n\
				\tPC:{:#X}\n\
			}}",
			self.a, self.f, self.b, self.c, self.d, self.e, self.h, self.l, self.sp, self.pc)
	}
}

impl Registers {
	pub fn new() -> Registers {
		let mut regs: Registers = Default::default();
		regs.init_dmg();
		//regs.init_cgb();
		regs
	}

	///Sets the values of the registers to what they would be
	///at the end of the dmg bootrom
	#[allow(dead_code)]
	pub fn init_dmg(&mut self) {
		self.a = 0x01;
		self.f = 0xB0;
		self.b = 0x00;
		self.c = 0x13;
		self.d = 0x00;
		self.e = 0xD8;
		self.h = 0x01;
		self.l = 0x4D;
		self.sp = 0xFFFE;
		self.pc = 0x0100;
	}

	///Sets the values of the registers to what they would be
	///at the end of the cgb bootrom
	pub fn init_cgb(&mut self) {
		//TODO: double check to actual values at the end of cgb bootrom
		self.a = 0x11;
		self.f = 0x80;
		self.b = 0x00;
		self.c = 0x00;
		self.d = 0xFF;
		self.e = 0x56;
		self.h = 0x00;
		self.l = 0x0D;
		self.sp = 0xFFFE;
		self.pc = 0x0100;
	}

	pub fn get_register_pair(&self, reg: RegisterPair) -> u16 {
		match reg {
			RegisterPair::AF => ((self.a as u16) << 8) | (self.f as u16),
			RegisterPair::BC => ((self.b as u16) << 8) | (self.c as u16),
			RegisterPair::DE => ((self.d as u16) << 8) | (self.e as u16),
			RegisterPair::HL => ((self.h as u16) << 8) | (self.l as u16),
			RegisterPair::SP => self.sp,
		}
	}

	pub fn set_register_pair(&mut self, reg: RegisterPair, value: u16) {
		match reg {
			RegisterPair::AF => {
				self.a = (value >> 8) as u8;
				self.f = value as u8;
			},
			RegisterPair::BC => {
				self.b = (value >> 8) as u8;
				self.c = value as u8;
			},
			RegisterPair::DE => {
				self.d = (value >> 8) as u8;
				self.e = value as u8;
			},
			RegisterPair::HL => {
				self.h = (value >> 8) as u8;
				self.l = value as u8;
			},
			RegisterPair::SP => self.sp = value,
		};
	}
}

/* Alu functions */
pub mod alu {
	use super::{ZERO_FLAG, SUBTRACTION_FLAG, HALF_CARRY_FLAG, CARRY_FLAG};

	#[cfg(feature = "no_std")]
	use core::num::Wrapping;

	#[cfg(not(feature = "no_std"))]
	use std::num::Wrapping;

	///Adds the values of 2 8-bit registers together, returns the result as a u8.
	///The resulting value of the flags register is: Z 0 H C
	pub fn add(register: u8, other: u8, flags: &mut u8) -> u8 {
		let result: u16 = (register as u16) + (other as u16);

		*flags = 0;
		*flags |= !(((result & 0x007F) + 0x007F) | result) as u8 & ZERO_FLAG;
		*flags |= (((register & 0x000F) + (other & 0x000F)) << 1) & HALF_CARRY_FLAG;
		*flags |= (result >> 4) as u8 & CARRY_FLAG;

		(result & 0xFF) as u8
	}

	///Adds the values of 2 8-bit registers together, returns the result as a u8.
	///The value of the carry flag is used as a carry in to the lower 4-bit adder.
	///The resulting value of the flags register is: Z 0 H C
	pub fn adc(register: u8, other: u8, flags: &mut u8) -> u8 {
		let cy: u8 = (*flags & CARRY_FLAG) >> 4;
		let result: u16 = register as u16 + other as u16 + cy as u16;

		*flags = 0;
		*flags |= !(((result & 0x007F) + 0x007F) | result) as u8 & ZERO_FLAG;
		*flags |= (((register & 0x000F) + (other & 0x000F) + cy) << 1) & HALF_CARRY_FLAG;
		*flags |= (result >> 4) as u8 & CARRY_FLAG;

		(result & 0xFF) as u8
	}

	///Subtracts the value of the second register from the first register, and returns the result as a u8.
	///The resulting value of the flags register is: Z 1 H C
	pub fn sub(register: u8, other: u8, flags: &mut u8) -> u8 {
		let result: u32 = (register as u32).wrapping_sub(other as u32);

		*flags = 0;
		if (other & 0x0F) > (register & 0x0F) {
			*flags |= HALF_CARRY_FLAG;
		}
		*flags |= !(((result & 0x7F) + 0x7F) | result) as u8 & ZERO_FLAG;
		*flags |= SUBTRACTION_FLAG;
		*flags |= (result >> 4) as u8 & CARRY_FLAG;

		(result & 0xFF) as u8
	}

	///Subtracts the value of the second register from the first register, and returns the result as a u8.
	///Also subtracts 1 if the carry flag is set.
	///The resulting value of the flags register is: Z 1 H C
	pub fn sbc(register: u8, other: u8, flags: &mut u8) -> u8 {
		let cy: u32 = ((*flags & CARRY_FLAG) >> 4) as u32;
		let result: u32 = (register as u32).wrapping_sub(other as u32).wrapping_sub(cy);

		*flags = SUBTRACTION_FLAG;
		*flags |= !(((result & 0x007F) + 0x007F) | result) as u8 & ZERO_FLAG;
		//*flags |= (((register & 0x000F) - (other & 0x000F) - (cy as u8)) << 1) & HALF_CARRY_FLAG;	//rust no like overflow
		*flags |= (((register & 0x000F).wrapping_sub((other & 0x000F)).wrapping_sub((cy as u8))) << 1) & HALF_CARRY_FLAG;	//TODO: test this
		*flags |= (result >> 4) as u8 & CARRY_FLAG;

		(result & 0xFF) as u8
	}

	///Performs a bitwise AND of 2 8-bit registers, and returns the result as a u8.
	///The resulting value of the flags register is: Z 0 1 0
	pub fn and(register: u8, other: u8, flags: &mut u8) -> u8 {
		let result: u8 = register & other;

		*flags = 0;
		*flags |= HALF_CARRY_FLAG;
		*flags |= !(((result & 0x7F) + 0x7F) | result) as u8 & ZERO_FLAG;

		(result & 0xFF) as u8
	}

	///Performs a bitwise XOR of 2 8-bit registers, and returns the result as a u8.
	///The resulting value of the flags register is: Z 0 0 0
	pub fn xor(register: u8, other: u8, flags: &mut u8) -> u8 {
		let result: u16 = (register ^ other) as u16;
		*flags = !(((result & 0x007F) + 0x007F) | result) as u8 & ZERO_FLAG;;
		result as u8
	}

	///Performs a bitwise OR of 2 8-bit registers, and returns the result as a u8.
	///The resulting value of the flags register is: Z 0 0 0
	pub fn or(register: u8, other: u8, flags: &mut u8) -> u8 {
		let result: u16 = (register | other) as u16;
		*flags = !(((result & 0x007F) + 0x007F) | result) as u8 & ZERO_FLAG;
		result as u8
	}

	///Subtracts the value of other from register, and sets the flags accoring to the definition of the sub operation.
	///The result of the subtraction is discarded, and only the value of the flags is kept.
	pub fn cp(register: u8, other: u8, flags: &mut u8) {
		sub(register, other, flags);
	}

	///Increment an 8-bit register by 1. The new value of the register is retured as a u8
	///The previous value of the Carry Flag is preserved.
	///The resulting value of the flags register is: Z 0 H -
	pub fn inc(register: u8, flags: &mut u8) -> u8 {
		let result: u16 = register as u16 + 1;

		//preserve Carry Flag
		*flags &= CARRY_FLAG;
		*flags |= !(((result & 0x007F) + 0x007F) | result) as u8 & ZERO_FLAG;
		*flags |= (((register as u8 & 0x0F) + 1) << 1) & HALF_CARRY_FLAG;

		(result & 0xFF) as u8
	}

	///Decrements an 8-bit register by 1. The new value of the register is returned as a u8.
	///The previous value of the Carry Flag is preserved.
	///The resulting value of the flags register is: Z 1 H -
	pub fn dec(register: u8, flags: &mut u8) -> u8 {
		let mut temp_flags = 0;
		let result: u8 = sub(register, 1, &mut temp_flags);
		temp_flags &= 0b11100000;
		*flags &= CARRY_FLAG;
		*flags |= temp_flags;

		result
	}

	///Performs an 8-bit left rotate on the register. The new value of the register is returned as a u8.
	///The resulting value of the flags register is: Z 0 0 C
	///The bit that is shifted out from the msb is placed into the carry (as well as moved to the lsb)
	pub fn rlc(register: u8, flags: &mut u8) -> u8 {
		let msb: u8 = register & 128;
		let result: u8 = (register << 1) | (msb >> 7);

		*flags = 0;
		*flags |= msb >> 3;
		*flags |= !(((result & 0x7F) + 0x7F) | result) as u8 & ZERO_FLAG;

		(result & 0xFF) as u8
	}

	///Performs an 8-bit right rotate on the register. The new value of the register is returned as a u8.
	///The bit that is shifted out from the lsb is placed into the carry (as well as moved to the msb)
	///The resulting value of the flags register is: Z 0 0 C
	pub fn rrc(register: u8, flags: &mut u8) -> u8 {
		let lsb: u16 = (register & 1) as u16;
		let result: u16 = ((register >> 1) as u16) | (lsb << 7);

		*flags = 0;
		*flags |= (lsb as u8) << 4;
		*flags |= !(((result & 0x007F) + 0x007F) | result) as u8 & ZERO_FLAG;

		(result & 0xFF) as u8
	}

	///Performs a 9-bit left rotate throught the carry flag on the register.
	///The new value of the register is returned as a u8.
	///The msb that is rotated out is put in the carry flag, and the previous value of the carry flag
	///is rotated into the lsb.
	///The resulting value of the flags register is: Z 0 0 C
	pub fn rl(register: u8, flags: &mut u8) -> u8 {
		let msb: u8 = register & 128;
		let result: u8 = (register << 1) | ((*flags & CARRY_FLAG)>> 4);

		*flags = 0;
		*flags |= msb >> 3;	//CY
		*flags |= !(((result & 0x7F) + 0x7F) | result) & ZERO_FLAG;

		result
	}

	///Performs a 9-bit right rotate throught the carry flag on the register.
	///The new value of the register is returned as a u8.
	///The lsb that is rotated out is put in the carry flag, and the previous value of the carry flag
	///is rotated into the msb.
	///The resulting value of the flags register is: Z 0 0 C
	pub fn rr(register: u8, flags: &mut u8) -> u8 {
		let lsb: u8 = register & 1;
		let result: u8 = (register >> 1) | ((*flags & CARRY_FLAG) << 3);

		*flags = 0;
		*flags |= lsb << 4;	//CY
		*flags |= !(((result & 0x7F) + 0x7F) | result) & ZERO_FLAG;

		result
	}

	///Performs a left shift on the register, a 0 is shifted into the lsb.
	///The new value of the register is returned as a u8.
	///The resulting value of the flags register is: Z 0 0 C
	pub fn sla(register: u8, flags: &mut u8) -> u8 {
		let result: u16 = (register as u16) << 1;

		*flags = 0;
		*flags |= ((result & 0x100) >> 4) as u8;
		*flags |= !(((result & 0x007F) + 0x007F) | result) as u8 & ZERO_FLAG;
		(result & 0xFF) as u8
	}

	///Performs an arithmetic (signed) right shift (the value of the msb stays the same).
	///The new value of the register is returned as a u8.
	///The resulting value of the flags register is: Z 0 0 C
	pub fn sra(register: u8, flags: &mut u8) -> u8 {
		let msb: u16 = register as u16 & 128;
		let result: u16 = (register >> 1) as u16 | msb;

		*flags = (register & 1) << 4;
		*flags |= !(((result & 0x007F) + 0x007F) | result) as u8 & ZERO_FLAG;

		(result & 0xFF) as u8
	}

	///Performs a logical (unsigned) right shift (a 0 is shifted in on the left).
	///The new value of the register is returned as a u8.
	///The resulting value of the flags register is: Z 0 0 C
	pub fn srl(register: u8, flags: &mut u8) -> u8 {
		let result: u16 = (register as u16) >> 1;

		*flags = 0;
		*flags |= (register << 4) & CARRY_FLAG;
		*flags |= !(((result & 0x007F) + 0x007F) | result) as u8 & ZERO_FLAG;

		(result & 0xFF) as u8
	}

	///Spaws the high and low nibble of the register.
	///The new value of the register is returned as a u8.
	///The resulting value of the flags register is: Z 0 0 0
	pub fn swap(register: u8, flags: &mut u8) -> u8 {
		let result: u16 = ((register << 4) | (register >> 4)) as u16;
		*flags = !(((result & 0x007F) + 0x007F) | result) as u8 & ZERO_FLAG;
		(result & 0xFF) as u8
	}

	///Tests bit n in the register.
	///The Carry Flag is preserved.
	///Flags: Z 0 1 -
	pub fn bit(register: u8, flags: &mut u8, bit: u8) {
		let bitmask: u8 = 1 << bit;
		let result: u8 = register & bitmask;

		*flags &= CARRY_FLAG;
		*flags |= HALF_CARRY_FLAG;
		*flags |= !(((result & 0x7F) + 0x7F) | result) & ZERO_FLAG;
	}

	///Set bit n
	///Returns the result as a u8.
	pub fn set(register: u8, bit: u8) -> u8 {
		register | 1u8 << bit
	}

	///Reset bit n
	///Returns the result as a u8.
	pub fn res(register: u8, bit: u8) -> u8 {
		register & !(1 << bit)
	}

	///Adds a 16 bit value to HL, not to be confused with ADD SP, r8 or LD HL, SP+r8
	///The Zero Flag is preserved
	///Flags: - 0 H C
	pub fn add16(hl: u16, other: u16, flags: &mut u8) -> u16 {
		let result: u32 = (hl as u32) + (other as u32);
		*flags &= ZERO_FLAG;	//Prezerve zero flag
		if (hl & 0x0FFF) + (other & 0x0FFF) > 0x0FFF {
			//set Half Carry
			*flags |= HALF_CARRY_FLAG;
		}
		if result > 0xFFFF {
			//set Carry Flag
			*flags |= CARRY_FLAG;
		}
		result as u16
	}

	///For 0xE8: ADD SP, r8
	///Add a signed byte to sp
	///The HC and C flags are set like regular 8-bit addition
	///Flags: 0 0 H C
	pub fn add_sp_nn(sp: u16, other: u8, flags: &mut u8) -> u16 {
		*flags = 0;
		if (sp & 0x0F) as u8 + (other & 0x0F) > 0x0F {
			*flags |= HALF_CARRY_FLAG;
		}
		if (sp & 0x00FF) + (other as u16) > 0x00FF {
			*flags |= CARRY_FLAG;
		}

		(Wrapping(sp as i16) + Wrapping((other as i8) as i16)).0 as u16
	}
}
