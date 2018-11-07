use std::fmt;

#[derive(Default, Copy, Clone)]
#[derive(Serialize, Deserialize)]
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

#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub enum Register {
	B, C, D, E, H, L, AT_HL, A, F
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
