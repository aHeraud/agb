#[cfg(feature = "no_std")]
use collections::vec::Vec;

#[cfg(not(feature = "no_std"))]
use std::vec::Vec;

use gameboy::Gameboy;
use gameboy::cpu::{ Registers, Register, RegisterPair };
use gameboy::mmu::Mmu;
use gameboy::assembly;

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
				if JR.contains(&next_opcode) {
					//next instruction is jr
				}
				else if JP.contains(&next_opcode) {
					//next instruction is jp
				}
				else if CALL.contains(&next_opcode) {
					//next instruction is call
				}
				else if RET.contains(&next_opcode) {
					//next instruction is ret
					//the return address is at the top of the stack
				}
				else if RST.contains(&next_opcode) {
					//next instrcution is rst
					//look up the address
				}
				else {
					panic!("oops");
				}
			}

			//TODO: read
			//TODO: write
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
}
