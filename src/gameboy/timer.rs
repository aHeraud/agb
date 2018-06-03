use gameboy::cpu::interrupts::{Interrupt, InterruptLine};

const FREQ: [u16; 4] = [512, 8, 32, 128];

const DIV_ADDRESS: u16 = 0xFF04;
const TIMA_ADDRESS: u16 = 0xFF05;
const TMA_ADDRESS: u16 = 0xFF06;
const TAC_ADDRESS: u16 = 0xFF07;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TimerRegister {
	Div, Tima, Tma, Tac
}

impl TimerRegister {
	pub fn address(&self) -> u16 {
		use self::TimerRegister::*;
		match self {
			Div => DIV_ADDRESS,
			Tima => TIMA_ADDRESS,
			Tma => TMA_ADDRESS,
			Tac => TAC_ADDRESS
		}
	}

	pub fn map_address(address: u16) -> Option<TimerRegister> {
		use self::TimerRegister::*;
		match address {
			DIV_ADDRESS => Some(Div),
			TIMA_ADDRESS => Some(Tima),
			TMA_ADDRESS => Some(Tma),
			TAC_ADDRESS => Some(Tac),
			_ => None
		}
	}
}

///http://gbdev.gg8.se/wiki/articles/Timer_and_Divider_Registers
///http://gbdev.gg8.se/wiki/articles/Timer_Obscure_Behaviour
///FF04: DIV -  16 bit divider register (only high 8 bytes are visible),
///             when written to, resets to 0.
///FF05: TIMA - Incremented by a frequency specified by TAC (FF07), when it overflows, it gets reset
///             to the value of TMA.
///FF06: TMA  - The value to load into TIMA when it overflows.
///FF07: TAC  - Bit 2 is the Timer Enable, and bits 1-0 are the clock select, as displayed below
///             0: CPU Clock / 1024
///             1: CPU Clock / 16
///             2: CPU Clock / 64
///             3: CPU Clock / 256
#[derive(Clone)]
pub struct Timer {
	/// Divider register (DIV) - incremented every cpu clock (4MHz)
	/// the high 8-bits of DIV are mapped to memory at address 0xFF04.
	/// Writing to DIV from the cpu causes it to reset.
	pub div: u16,

	/// 0xFF05: TIMA - Incremented at a division of the main clock specified by the TAC register.
	/// When tima overflows the value of tma is loaded and an interrupt will be requested.
	pub tima: u16,

	/// 0xFF06: TMA - The value to load into TIMA when it overflows.
	pub tma: u8,

	/// 0xFF07: TAC
	/// Bit 2: timer enable
	/// Bits 1-0: clock select
	///     0: CPU Clock / 1024
	///     1: CPU Clock / 16
	///     2: CPU Clock / 64
	///     3: CPU Clock / 256
	pub tac: u8,

	/// There is a 4 cycle (1 M-Cycle) delay between
	/// tima overflowing and it being reloaded and the interrupt firing, so this
	/// keeps track of how long ago tima overflowed
	pub tima_overflow_delay: Option<i8>
}

impl Timer {
	pub fn new() -> Timer {
		Timer {
			div: 0,
			tima: 0,
			tma: 0,
			tac: 0,
			tima_overflow_delay: None
		}
	}

	pub fn reset(&mut self) {
		self.div = 0;
		self.tima = 0;
		self.tma = 0;
		self.tac = 0;
		self.tima_overflow_delay = None
	}

	///Inspect the value of the internal div register
	pub fn get_div(&self) -> u16 {
		self.div
	}

	/// Emulate the timer for a cycle (increment div, trigger interrupts, etc...).
	/// Called every T-Cycle (4 MHz clock)
	pub fn emulate_hardware(&mut self, interrupt_line: &mut InterruptLine) {
		let old_div = self.div;
		self.div = self.div.wrapping_add(1);
		let freq = FREQ[(self.tac & 3) as usize];

		if let Some(delay) = self.tima_overflow_delay {
			if delay > 0 {
				self.tima_overflow_delay = Some(delay - 1);
			}
			else {
				//reload and request interrupt
				self.tima = self.tma as u16;
				interrupt_line.request_interrupt(Interrupt::Timer);
				self.tima_overflow_delay = None;
			}
		}
		// increment tima when current freq bit in div goes from high to low
		else if (self.tac & 4 == 4) && ((old_div & freq == freq) && (self.div & freq) == 0) {
			self.tima += 1;
			if self.tima > 0xFF {
				self.tima = 0;
				self.tima_overflow_delay = Some(4);
			}
		}
	}

	/// Read from one of the timers memory mapped io registers.
	pub fn read_io(&self, reg: TimerRegister) -> u8 {
		use self::TimerRegister::*;
		match reg {
			Div => (self.div >> 8) as u8,
			Tima => self.tima as u8,
			Tma => self.tma,
			Tac => self.tac
		}
	}

	/// Write a value to one of the timer's memory mapped io registers.
	/// Writing a value to the divider register ($FF04) will reset it to 0 (which might cause tima to be incremented).
	pub fn write_io(&mut self, reg: TimerRegister, value: u8) {
		use self::TimerRegister::*;
		match reg {
			Div => {
				let old_div = self.div;
				self.div = 0; //Writing to $FF04 resets divider to 0
				let freq = FREQ[(self.tac & 3) as usize];

				// if freq bit goes from high to low -> increment value in tima
				if (self.tac & 4 == 4) && (old_div & freq == freq) {
					self.tima += 1;
					if self.tima > 0xFF {
						self.tima = 0;
						self.tima_overflow_delay = Some(4);
					}
				}
			},
			Tima => self.tima = value as u16,
			Tma => self.tma = value,
			Tac => self.tac = value
		};
	}
}
