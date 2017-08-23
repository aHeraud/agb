use gameboy::Mode;
use self::registers::Registers;

pub mod registers;
pub mod alu;

pub const ZERO_FLAG_MASK: u8 = 1 << 7;
pub const SUBTRACTION_FLAG_MASK: u8 = 1 << 6;
pub const HALF_CARRY_FLAG_MASK: u8 = 1 << 5;
pub const CARRY_FLAG_MASK: u8 = 1 << 4;

const HRAM_SIZE: usize = 127;

///On the DMG/CGB the EI instruction, the value of ime isn't changed until after the next instruction,
///I assume this is because of instruction pipelining, and the next instruction has been fetched before
///interrupts have been enabled.
///On the CGB, the same applies to the DI instruction (but not on the DMG) allegedly

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
