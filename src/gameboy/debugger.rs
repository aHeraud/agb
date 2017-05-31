use std::vec::Vec;

use gameboy::Gameboy;
use gameboy::cpu::{ Registers, Register, RegisterPair };
use gameboy::mmu::Mmu;
use gameboy::assembly;
use gameboy::ppu::Bitmap;

type BreakpointCallback = FnMut(Breakpoint);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AccessType {
	Read, Write, Execute, Jump,
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq, Eq,PartialOrd, Ord)]
pub struct Breakpoint {
	pub address: u16,	/* Address of the breakpoint */
	//pub bank: Option<u16>, /* The bank (if there is one) of the breakpoint */
	pub access_type: AccessType,
}

impl Breakpoint {
	pub fn new(address: u16, /* bank: Option<u16>, */ access_type: AccessType) -> Breakpoint {
		Breakpoint {
			address: address,
			//bank: bank,
			access_type: access_type
		}
	}
}

pub struct Debugger {
	enabled: bool,
	hit_breakpoint: bool,
	breakpoints: Vec<Breakpoint>,
	breakpoint_callback: Option<Box<BreakpointCallback>>,
}

impl Debugger {
	pub fn new() -> Debugger {
		Debugger {
			enabled: false,
			hit_breakpoint: false,
			breakpoints: Vec::new(),
			breakpoint_callback: None,
		}
	}

	pub fn enable(&mut self) {
		self.enabled = true;
	}

	pub fn disable(&mut self) {
		self.enabled = false;
	}

	pub fn enabled(&self) -> bool {
		self.enabled
	}

	pub fn breakpoint_callback(&mut self, param: Breakpoint) {
		if let Some(ref mut callback) = self.breakpoint_callback {
			(callback)(param);
		}
	}

	pub fn hit_breakpoint(&self) -> bool {
		self.hit_breakpoint
	}
}

pub trait DebuggerInterface {
	fn add_breakpoint(&mut self, breakpoint: Breakpoint);
	fn remove_breakpoint(&mut self, index: usize) -> Result<Breakpoint,()>;
	fn get_breakpoints(&self) -> Vec<Breakpoint>;
	fn register_breakpoint_callback<CB>(&mut self, cb: CB) where CB: 'static + FnMut(Breakpoint);
	fn clear_breakpoint_callback(&mut self);
	fn breakpoint_lookahead(&self) -> Option<Breakpoint>;

	fn debug_step(&mut self) -> Option<Breakpoint>;

	fn get_registers(&self) -> Registers;
	fn set_register(&mut self, register: Register, value: u8);
	fn set_register_pair(&mut self, register_pair: RegisterPair, value: u16);

	fn read_memory(&self, address: u16) -> u8;
	fn write_memory(&mut self, address: u16, value: u8);

	fn read_range(&self, address_start: u16, address_end: u16) -> Result<Box<[u8]>, ()>;
	fn write_range(&mut self, address_start: u16, values: &[u8]);

	fn get_assembly(&self, ins: &[u8]) -> Vec<String>;

	fn dump_tiles(&self) -> Bitmap<u32>;
	fn dump_bg(&self) -> Bitmap<u32>;

	fn reset(&mut self);
}

impl DebuggerInterface for Gameboy {
	///add a new breakpoint (if it doesn't already exist)
	fn add_breakpoint(&mut self, breakpoint: Breakpoint) {
		if let Err(index) = self.debugger.breakpoints.binary_search(&breakpoint) {
			self.debugger.breakpoints.insert(index, breakpoint);
		}
	}

	///remove a breakpoint (if it exists)
	fn remove_breakpoint(&mut self, index: usize) -> Result<Breakpoint,()> {
		if index >= self.debugger.breakpoints.len() {
			Err(())
		}
		else {
			Ok(self.debugger.breakpoints.remove(index))
		}
	}

	///get the list of breakpoints
	fn get_breakpoints(&self) -> Vec<Breakpoint> {
		self.debugger.breakpoints.to_vec()
	}

	///register a callback to be called when a breakpoint is encountered
	fn register_breakpoint_callback<CB>(&mut self, cb: CB) where CB: 'static + FnMut(Breakpoint) {
		self.debugger.breakpoint_callback = Some(Box::new(cb));
	}

	fn clear_breakpoint_callback(&mut self) {
		self.debugger.breakpoint_callback = None;
	}

	fn debug_step(&mut self) -> Option<Breakpoint> {
		self.interrupt_service_routine();
		let result = self.breakpoint_lookahead();
		self.execute();
		result
	}

	fn get_registers(&self) -> Registers {
		self.cpu.registers
	}

	fn set_register(&mut self, register: Register, value: u8) {
		match register {
			Register::A => self.cpu.registers.a = value,
			Register::F => self.cpu.registers.f = value,
			Register::B => self.cpu.registers.b = value,
			Register::C => self.cpu.registers.c = value,
			Register::D => self.cpu.registers.d = value,
			Register::E => self.cpu.registers.e = value,
			Register::H => self.cpu.registers.h = value,
			Register::L => self.cpu.registers.l = value,
			_ => { /* should you be able to poke at memory with (HL), it's not really a register*/},
		};
	}

	fn set_register_pair(&mut self, register: RegisterPair, value: u16) {
		self.cpu.registers.set_register_pair(register, value);
	}

	///Check whether the next step of the interpreter would trigger a breakpoint,
	///and if it would, return a copy of the breakpoint.
	///TODO: finish implementing this
	///TODO: it's possible to hit multiple breakpoints in a step, so it might be
	///better to return a a vector of breakpoints. Although, perhaps it would make sense
	///for interrupts to have a priority of Execute > Jump > Write > Read
	fn breakpoint_lookahead(&self) -> Option<Breakpoint> {
		if self.debugger.breakpoints.len() > 0 {
			//TODO: what if the breakpoint is not aligned
			//ex:  what if the next instruction is JP 0x4000 (0xC3 0x00 0x40, 3 bytes)
			//     starting at pc= 0x1000.  If there is a breakpoint at 0x1001, which
			//     is in the middle of the instruction, should it be hit when the
			//     instruction hits?
			let next_opcode = self.read_byte(self.cpu.registers.pc);

			let execute = Breakpoint::new(self.cpu.registers.pc, AccessType::Execute);
			if let Ok(index) = self.debugger.breakpoints.binary_search(&execute) {
				//there is a breakpoint on the next instruction
				return Some(self.debugger.breakpoints[index]);
			}

			//jump: check if the next instruction is JR, JP, CALL, RET, or RST,
			//then look at it's destination (for RET look at stack)
			const JR:   [u8;5] = [0x18, 0x20, 0x28, 0x30, 0x38];
			const JP:   [u8;6] = [0xC2, 0xC3, 0xCA, 0xD2, 0xDA, 0xE9];
			const CALL: [u8;5] = [0xC4, 0xCC, 0xCD, 0xD4, 0xDC];
			const RET:  [u8;6] = [0xC0, 0xC8, 0xC9, 0xD0, 0xD8, 0xD9];
			const RST:  [u8;8] = [0xC7, 0xCF, 0xD7, 0xDF, 0xE7, 0xEF, 0xF7, 0xFF];
			const JUMPS:[u8;30] = [0x18, 0x20, 0x28, 0x30, 0x38, 0xC0, 0xC2, 0xC3,
			                       0xC4, 0xC7, 0xC8, 0xC9, 0xCA, 0xCC, 0xCD, 0xCF,
			                       0xD0, 0xD2, 0xD4, 0xD7, 0xD8, 0xD9, 0xDA, 0xDC,
			                       0xDF, 0xE7, 0xE9, 0xEF, 0xF7, 0xFF];
			if let Ok(_) = JUMPS.binary_search(&next_opcode) {
				//the next instruction is some sort of jump, look at where it points.
				let address =
				if JR.contains(&next_opcode) {
					//next instruction is jr, so the address is relative (signed byte offset)
					let pc = self.cpu.registers.pc;
					let offset = self.read_byte(pc + 1);
					((pc as i32) + ((offset as i8) as i32)) as u16
				}
				else if JP.contains(&next_opcode) || CALL.contains(&next_opcode) {
					//next instruction is jp or call (1 byte opcode followed by 2 byte address)
					let pc = self.cpu.registers.pc;
					let high = self.read_byte(pc + 2) as u16;
					let low = self.read_byte(pc + 1) as u16;
					(high << 8) | low
				}
				else if RET.contains(&next_opcode) {
					//next instruction is ret
					//the return address is at the top of the stack
					let sp = self.cpu.registers.sp;
					let high = self.read_byte(sp + 1) as u16;
					let low = self.read_byte(sp) as u16;
					(high << 8) | low
				}
				else if RST.contains(&next_opcode) {
					//next instrcution is rst
					//look up the address
					match next_opcode {
						0xC7 => 0,
						0xCF => 0x8,
						0xD7 => 0x10,
						0xDF => 0x18,
						0xE7 => 0x20,
						0xEF => 0x2F,
						0xF7 => 0x30,
						0xFF => 0x48,
						_ => panic!()
					}
				}
				else {
					panic!("oops");
				};

				let breakpoint = Breakpoint::new(address, AccessType::Jump);
				if let Ok(index) = self.debugger.breakpoints.binary_search(&breakpoint) {
					//there is a breakpoint on the next instruction
					return Some(self.debugger.breakpoints[index]);
				}
			}

			const READ_AT_BC: u8 = 0x0A;
			const READ_AT_DE: u8 = 0x1A;
			const READ_IO_A8: u8 = 0xF0;
			const READ_IO_C: u8 = 0xF2; //read from 0xFF00 + C
			const READ_A16: u8 = 0xFA;
			const READ_AT_HL: [u8;20] =
			[0x2A, 0x34, 0x35, 0x3A, 0x46, 0x4E, 0x56, 0x5E, 0x66, 0x6E, 0x7E, 0x86, 0x8E, 0x96, 0x9E,
			0xA6, 0xAE, 0xB6, 0xBE, 0xE9];
			const READ_AT_HL_EXTENDED: [u8;16] =
			[0x06, 0x0E, 0x16, 0x1E, 0x26, 0x2E, 0x36, 0x3E, 0x46, 0x4E, 0x56, 0x5E, 0x66, 0x6E, 0x76, 0x7E];

			const READS: [u8;25] = [READ_AT_BC, READ_AT_DE, 0x2A, 0x34, 0x35, 0x3A, 0x46, 0x4E, 0x56, 0x5E, 0x66, 0x6E, 0x7E, 0x86, 0x8E, 0x96, 0x9E,
			0xA6, 0xAE, 0xB6, 0xBE, 0xE9, READ_IO_A8, READ_IO_C, READ_A16];

			let mut address: Option<u16> = None;
			if let Ok(_) = READS.binary_search(&next_opcode) {
				if next_opcode == READ_AT_BC {
					let b = self.cpu.registers.b as u16;
					let c = self.cpu.registers.c as u16;
					address = Some((b << 8) | c);
				}
				else if next_opcode == READ_AT_DE {
					let d = self.cpu.registers.d as u16;
					let e = self.cpu.registers.e as u16;
					address = Some((d << 8) | e);
				}
				else if next_opcode == READ_IO_A8 {
					address = Some(0xFF00 + (self.read_byte(self.cpu.registers.pc + 1) as u16));
				}
				else if next_opcode == READ_IO_C {
					address = Some(0xFF00 + (self.cpu.registers.c as u16));
				}
				else if next_opcode == READ_A16 {
					let pc = self.cpu.registers.pc + 1;
					address = Some((self.read_byte(pc) as u16) | ((self.read_byte(pc + 1) as u16) << 8));
				}
				else if let Ok(_) = READ_AT_HL.binary_search(&next_opcode) {
					let h = self.cpu.registers.h as u16;
					let l = self.cpu.registers.l as u16;
					address = Some((h << 8) | l);
				}
				else {
					panic!("oops");
				}
			}
			else if next_opcode == 0xCB {
				let next_opcode = self.read_byte(self.cpu.registers.pc + 1);
				if let Ok(_) = READ_AT_HL_EXTENDED.binary_search(&next_opcode) {
					let h = self.cpu.registers.h as u16;
					let l = self.cpu.registers.l as u16;
					address = Some((h << 8) | l);
				}
			}
			if let Some(addr) = address {
				let breakpoint = Breakpoint::new(addr, AccessType::Read);
				if let Ok(index) = self.debugger.breakpoints.binary_search(&breakpoint) {
					return Some(self.debugger.breakpoints[index]);
				}
			}

			const WRITE_AT_BC: u8 = 0x02;
			const WRITE_AT_DE: u8 = 0x12;
			const WRITE_A16: [u8;2] = [0x08, 0xEA];
			const WRITE_IO_A8: u8 = 0xE0;
			const WRITE_IO_C: u8 = 0xE2;
			const WRITE_AT_HL: [u8;12] = [0x22, 0x32, 0x34, 0x35, 0x36, 0x70, 0x71, 0x72, 0x73,
			                              0x74, 0x75, 0x77];
			const WRITE_AT_HL_EXTENDED: [u8;24] = [0x06, 0x0E, 0x16, 0x1E, 0x26, 0x2E, 0x36, 0x3E,
			                                       0x86, 0x8E, 0x96, 0x9E, 0xA6, 0xAE, 0xB6, 0xBE,
			                                       0xC6, 0xCE, 0xD6, 0xDE, 0xE6, 0xEE, 0xF6, 0xFE];
			const WRITES: [u8;18] = [0x02, 0x08, 0x12, 0x22, 0x32, 0x34, 0x35, 0x36, 0x70, 0x71,
			                         0x72, 0x73, 0x74, 0x75, 0x77, 0xE0, 0xE2, 0xEA];

			let mut address: Option<u16> = None;
			if let Ok(_) = WRITES.binary_search(&next_opcode) {
				if let Ok(_) = WRITE_AT_HL.binary_search(&next_opcode) {
					let h = self.cpu.registers.h as u16;
					let l = self.cpu.registers.l as u16;
					address = Some((h << 8) | l);
				}
				else if let Ok(_) = WRITE_A16.binary_search(&next_opcode) {
					let pc = self.cpu.registers.pc + 1;
					address = Some((self.read_byte(pc) as u16) | ((self.read_byte(pc + 1) as u16) << 8));
				}
				else if next_opcode == WRITE_IO_A8 {
					address = Some((self.read_byte(self.cpu.registers.pc + 1) as u16) + 0xFF00);
				}
				else if next_opcode == WRITE_AT_BC {
					let b = self.cpu.registers.b as u16;
					let c = self.cpu.registers.c as u16;
					address = Some((b << 8) | c);
				}
				else if next_opcode == WRITE_AT_DE {
					let d = self.cpu.registers.d as u16;
					let e = self.cpu.registers.e as u16;
					address = Some((d << 8) | e);
				}
				else if next_opcode == WRITE_IO_C {
					address = Some(0xFF00 + (self.cpu.registers.c as u16));
				}
			}
			else if next_opcode == 0xCB {
				let next_opcode = self.read_byte(self.cpu.registers.pc + 1);
				if let Ok(_) = WRITE_AT_HL_EXTENDED.binary_search(&next_opcode) {
					let h = self.cpu.registers.h as u16;
					let l = self.cpu.registers.l as u16;
					address = Some((h << 8) | l);
				}
			}
			if let Some(address) = address {
				let breakpoint = Breakpoint::new(address, AccessType::Write);
				if let Ok(index) = self.debugger.breakpoints.binary_search(&breakpoint) {
					return Some(self.debugger.breakpoints[index]);
				}
			}
		}
		None
	}

	fn read_memory(&self, address: u16) -> u8 {
		self.read_byte(address)
	}

	///the reson we don't just call self.write_byte here is because
	///we want to be able to patch the cartridge rom
	fn write_memory(&mut self, address: u16, value: u8) {
		match address {
			0x0000...0x3FFF => {
				let mut rom = self.rom_mut();
				if (address as usize) < rom.len() {
					rom[address as usize] = value;
				}
			},
			0x4000...0x7FFF => {
				let mut rom = self.banked_rom_mut();
				if (address as usize) < rom.len() {
					rom[(address as usize) - 0x4000] = value;
				}
			},
			_ => { self.write_byte(address, value); },
		}
	}

	fn read_range(&self, address_start: u16, address_end: u16) -> Result<Box<[u8]>, ()> {
		//TODO: maybe implement this more efficiently
		if address_start > address_end {
			//TODO: is this really an error, or should it wrap around?
			Err(())
		}
		else {
			let size = (address_end - address_start + 1) as usize;
			let mut bytes = Vec::with_capacity(size);
			for address in address_start...address_end {
				bytes.push(self.read_byte(address));
			}
			Ok(bytes.into_boxed_slice())
		}
	}

	fn write_range(&mut self, address: u16, values: &[u8]) {
		//TODO: maybe implement this more efficiently
		for (index, value) in values.iter().enumerate() {
			self.write_byte((index as u16) + address, *value);
		}
	}

	fn get_assembly(&self, ins: &[u8]) -> Vec<String> {
		assembly::get_assembly(ins)
	}

	fn reset(&mut self) {
		use gameboy::Mode;
		let mode: Mode = match self.cart.get_cart_info().cgb {
			true => Mode::CGB,
			false => Mode::DMG,
		};
		self.cpu.reset(mode);
		self.timer.reset();
		self.ppu.reset();
		self.oam_dma_active = false;
		self.oam_dma_start_address = 0;
		self.oam_dma_current_offset = 0;
	}

	fn dump_tiles(&self) -> Bitmap<u32> {
		self.ppu.dump_tiles()
	}

	fn dump_bg(&self) -> Bitmap<u32> {
		self.ppu.dump_bg(&self.io)
	}
}
