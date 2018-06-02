use gameboy::util::wrapping_add;
use gameboy::cpu::interrupts::{Interrupt, InterruptLine};

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
const FREQ: [u16; 4] = [1024, 16, 64, 256];

pub struct Timer {
	div: u16,
	tima: u16,
	tma: u8,
	tac: u8,
	tima_overflow: bool
}

impl Timer {
	pub fn new() -> Timer {
		Timer {
			div: 0,
			tima: 0,
			tma: 0,
			tac: 0,
			tima_overflow: false
		}
	}

	pub fn reset(&mut self) {
		self.div = 0;
		self.tima = 0;
		self.tma = 0;
		self.tac = 0;
		self.tima_overflow = false;
	}

	///Writing to $FF04 resets divider to 0
	pub fn reset_div(&mut self) {
		self.div = 0;
	}

	///Inspect the value of the internal div register
	pub fn get_div(&self) -> u16 {
		self.div
	}

	///Called every M-Cycle (4 clock cycles)
	pub fn emulate_hardware(&mut self, io: &mut [u8], interrupt_line: &mut InterruptLine) {
		//Read back registers
		self.tima = io[0x05] as u16;
		self.tma = io[0x06];
		self.tac = io[0x07] & 0x07;

		self.div = wrapping_add(self.div, 4);

		let freq = FREQ[(self.tac & 3) as usize];

		if self.tima_overflow {
			self.tima_overflow = false;
			//Tima overflow, load TMA
			self.tima = self.tma as u16;
			//Request Timer Interrupt
			interrupt_line.request_interrupt(Interrupt::Timer);
		}

		//Incremented on rising edge
		if (self.tac & 4 == 4) && (self.div % freq == 0) {
			//inc tima
			self.tima += 1;

			if self.tima > 0xFF {
				//when tima overflows there is a 1 m-cycle delay before
				//it is reloaded and the interrupt is fired
				self.tima = 0;
				self.tima_overflow = true;
			}
		}

		//Write back registers
		io[0x04] = (self.div >> 8) as u8;
		io[0x05] = self.tima as u8;
		io[0x06] = self.tma;
		io[0x07] = self.tac;
	}
}
