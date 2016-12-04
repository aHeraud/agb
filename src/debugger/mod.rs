use std::vec::Vec;

pub use gameboy::Register;
pub use gameboy::cpu::RegisterPair;
use gameboy::Gameboy;
mod instructions;

//pub enum AccessType {
	//Read, Write, Execute, Jump,
//}

/*
#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub struct Breakpoint {
	pub address: u16,	/* Address of the breakpoint */
	//pub bank: Option<u8>, /* The bank (if there is one) of the breakpoint */
	//pub original_data: u8, /* The value that was originally at $address */
	//pub access_type: AccessType,
}
*/

#[allow(dead_code)]
#[derive(Debug)]
pub struct Debugger {
	//breakpoints: Vec<Breakpoint>,
	breakpoints: Vec<u16>,
}

#[allow(dead_code)]
impl Debugger {
	pub fn new() -> Debugger {
		Debugger {
			breakpoints: Vec::new(),
		}
	}

	pub fn add_breakpoint(&mut self, address: u16) {
		//The vector of breakpoints is ordered, to make searching through it faster,
		//so new breakpoints must be inserted in order.
		let result = self.breakpoints.binary_search(&address);
		match result {
			Ok(_) => { /* Breakpoint aleady exists */ },
			Err(index) => {
				/* Insert new breakpoint at index */
				self.breakpoints.insert(index, address);
			}
		};
	}

	pub fn remove_breakpoint(&mut self, address: u16) {
		let result = self.breakpoints.binary_search(&address);
		match result {
			Ok(index) => {
				self.breakpoints.remove(index);
			},
			Err(_) => { /* No breakpoint with that address exists */ }
		};
	}

	///Get an immutable reference to the vector of breakpoints
	pub fn get_breakpoints(&self) -> &Vec<u16> {
		&self.breakpoints
	}

	///Executes for ~70224 clock cycles (the duration of a full screen refresh)
	///Stops execution when a breakpoint is hit
	pub fn step_frame(&mut self, gb: &mut Gameboy) {
		//A complete frame occurs every ~70224 clock cycles (140448 in gbc double speed mode)
		const FRAME_CLOCKS: usize = 70224;
		let mut counter: usize = 0;

		while counter < FRAME_CLOCKS {
			if self.step(gb, false).is_some() {
					break;
			}
			if gb.cpu.double_speed_mode {
				if self.step(gb, false).is_some() {
						break;
				}
			}
			counter += 4;
		}
	}

	///Returns Some(pc) if you hit a breakpoint, otherwise return none
	///You can step over a breakpoint by setting step_over to true
	pub fn step(&mut self, gb: &mut Gameboy, step_over: bool) -> Option<u16> {
		let pc: u16 = gb.cpu.registers.pc;
		let index_result = self.breakpoints.binary_search(&pc);
		match index_result {
			Ok(_) => {
				/* Hit a breakpoint, step over? */
				if step_over {
					gb.step();
					None
				}
				else {
					Some(pc)
				}
			}
			Err(_) => {
				/* No breakpoint at pc */
				gb.step();
				None
			}
		}
	}

	///TODO: don't use the public read methods since they are meant
	///for the cpu and have certain conditions where a memory region
	///can not be accessed
	pub fn read_byte(gb: &Gameboy, address: u16) -> u8 {
		match address {
			0x0000...0x7FFF => gb.cart.read_byte_rom(address),
			0x8000...0x9FFF => gb.ppu.read_byte_vram(&gb.io, address),
			0xA000...0xBFFF => gb.cart.read_byte_ram(address),
			0xC000...0xDFFF => gb.read_byte_wram(address),
			0xE000...0xFDFF => gb.read_byte_wram(address - 0x2000),	//Mirror of wram
			0xFE00...0xFE9F => gb.ppu.read_byte_oam(&gb.io, address),
			0xFF00...0xFF7F => gb.read_byte_io(address),
			0xFF80...0xFFFE => gb.cpu.read_byte_hram(address),
			0xFFFF => gb.cpu.ier,
			_ => 0xFF,
		}
	}

	///TODO: don't use the public write methods since they are meant
	///for the cpu and have certain conditions where a memory region
	///can not be accessed
	pub fn write_byte(gb: &mut Gameboy, address: u16, value: u8) {
		match address {
			0x0000...0x7FFF => gb.cart.write_byte_rom(address, value),
			0x8000...0x9FFF => gb.ppu.write_byte_vram(&gb.io, address, value),
			0xA000...0xBFFF => gb.cart.write_byte_ram(address, value),
			0xC000...0xDFFF => gb.write_byte_wram(address, value),
			0xE000...0xFDFF => gb.write_byte_wram(address - 0x2000, value),	//Mirror of wram
			0xFE00...0xFE9F => gb.ppu.write_byte_oam(&gb.io, address, value),
			0xFF00...0xFF7F => gb.write_byte_io(address, value),
			0xFF80...0xFFFE => gb.cpu.write_byte_hram(address, value),
			0xFFFF => gb.cpu.ier = value,
			_ => return,
		};
	}

	pub fn get_register(gb: &Gameboy, reg: Register) -> u8 {
		match reg {
			Register::B => gb.cpu.registers.b,
			Register::C => gb.cpu.registers.c,
			Register::D => gb.cpu.registers.d,
			Register::E => gb.cpu.registers.e,
			Register::H => gb.cpu.registers.h,
			Register::L => gb.cpu.registers.l,
			Register::AT_HL => {
				let hl: u16 = gb.cpu.registers.get_register_pair(RegisterPair::HL);
				let value = Debugger::read_byte(gb, hl);
				value
			},
			Register::A => gb.cpu.registers.a,
		}
	}

	pub fn set_register(gb: &mut Gameboy, reg: Register, val: u8) {
		match reg {
			Register::B => gb.cpu.registers.b = val,
			Register::C => gb.cpu.registers.c = val,
			Register::D => gb.cpu.registers.d = val,
			Register::E => gb.cpu.registers.e = val,
			Register::H => gb.cpu.registers.h = val,
			Register::L => gb.cpu.registers.l = val,
			Register::AT_HL => {
				let hl: u16 = gb.cpu.registers.get_register_pair(RegisterPair::HL);
				Debugger::write_byte(gb, hl, val);
			},
			Register::A => gb.cpu.registers.a = val,
		};
	}

	/* Read the value of one of the register pairs */
	pub fn get_register_pair(gb: &Gameboy, reg: RegisterPair) -> u16 {
		gb.cpu.registers.get_register_pair(reg)
	}

	/* Set the value of one of the register pairs*/
	pub fn set_register_pair(gb: &mut Gameboy, reg: RegisterPair, val: u16) {
		gb.cpu.registers.set_register_pair(reg, val);
	}
}
