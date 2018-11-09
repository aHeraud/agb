use gameboy::Mode;

pub mod interrupts;
pub mod registers;
pub mod alu;

use self::registers::Registers;
use self::interrupts::{InterruptFlag, InterruptEnable};

pub const ZERO_FLAG_MASK: u8 = 1 << 7;
pub const SUBTRACTION_FLAG_MASK: u8 = 1 << 6;
pub const HALF_CARRY_FLAG_MASK: u8 = 1 << 5;
pub const CARRY_FLAG_MASK: u8 = 1 << 4;

const HRAM_SIZE: usize = 127;

///On the DMG/CGB the EI instruction, the value of ime isn't changed until after the next instruction,
///I assume this is because of instruction pipelining, and the next instruction has been fetched before
///interrupts have been enabled.
///On the CGB, the same applies to the DI instruction (but not on the DMG) allegedly
#[derive(Serialize, Deserialize)]
pub struct CPU {
	pub registers: Registers,
	pub ime: bool,
	pub next_ime_state: bool,
	pub interrupt_flag: InterruptFlag, //Interrupt Flag - $FF0F
	pub interrupt_enable: InterruptEnable, //Interrupt Enable Register - $FFFF
	pub stop: bool,
	pub halt: bool,
	pub hram: Box<[u8]>,
	pub double_speed_mode: bool,
	pub cycle_counter: usize,
}

impl CPU {
	pub fn new() -> CPU {
		CPU {
			registers: Registers::new(),
			ime: false,	//TODO: default value of ime
			next_ime_state: false,
			interrupt_flag: InterruptFlag::new(),
			interrupt_enable: InterruptEnable::new(),
			stop: false,
			halt: false,
			hram: Box::new([0; HRAM_SIZE]),
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
		self.interrupt_flag.reset();
		self.interrupt_enable.reset();
		self.stop = false;
		self.halt = false;
		self.double_speed_mode = false;
	}

	pub fn read_byte_hram(&self, offset: u16) -> u8 {
		self.hram[offset as usize]
	}

	pub fn write_byte_hram(&mut self, offset: u16, value: u8) {
		self.hram[offset as usize] = value
	}
}
