use ::gameboy::cpu::interrupts::{InterruptLine, Interrupt};

pub type SerialCallback = (FnMut(bool) -> bool) + Send;

pub struct Serial {
	/// Callback that is called when a bit is shifted through the serial port using the internal clock.
	/// If an external clock is used to drive the serial port, this callback function will not be called.
	/// The parameter is the bit that was shifted out of the serial port.
	/// The return value is the bit to be shifted in.
	callback: Option<Box<SerialCallback>>,

	/// Serial Data register ($FF01).
	/// When a transfer is active, the value in this register will be shifted out bit by bit, and on every
	/// shift a new bit will also be shifted in from the other end.
	sb: u8,

	/// Serial Control register ($FF02).
	/// Bit 7 - Transfer Status
	///     0: No transfer in progress
	///     1: Start transfer
	/// Bit 1 - Clock Speed (CGB only)
	///     0: Normal
	///     1: Fast
	/// Bit 0 - Internal/External Clock
	///     0: External Clock
	///     1: Internal Clock
	sc: u8,

	/// Counts how many 4MHz cycles the current transfer has been active for.
	transfer_cycle_counter: usize,

	/// Counts how many 4MHz cycles since the last bit has been shifted out
	current_bit_cycles: usize,

	/// Counts how many bits have been shifted during the current transfer (0...8)
	bits_shifted: u8
}

impl Serial {
	pub fn new() -> Serial {
		Serial {
			callback: None,
			sb: 0xFF,
			sc: 0,
			transfer_cycle_counter: 0,
			current_bit_cycles: 0,
			bits_shifted: 0
		}

	}

	/// Read a byte from the serial data register ($FF01).
	pub fn read_sb(&self) -> u8 {
		self.sb
	}

	/// Read a byte from the serial control register ($FF02).
	/// Unused bits are probably all 1 (TODO: confirm this).
	pub fn read_sc(&self) -> u8 {
		self.sc | 0x7C
	}

	/// Write a byte to the serial data register ($FF01).
	/// TODO: what happens if you write to sb during a transfer?
	pub fn write_sb(&mut self, value: u8) {
		self.sb = value;
	}

	/// Write a byte to the serial control register ($FF02).
	/// TODO: what happens when you set bit 7 when there is already an active transfer? Does it restart?
	/// TODO: what happens when you change the value of bit 1 (clock speed) in the middle of a transfer that is using the internal clock.
	pub fn write_sc(&mut self, value: u8) {
		self.sc = value & 0x83;
		if value & 0x80 == 0x80 {
			self.transfer_cycle_counter = 0;
			self.current_bit_cycles = 0;
			self.bits_shifted = 0;
		}
	}

	pub fn register_callback(&mut self, cb: Box<SerialCallback>) {
		self.callback = Some(cb);
	}

	pub fn remove_callback(&mut self) {
		self.callback = None;
	}

	/// Shift a bit in from the other end of the serial port, and returns the bit shifted out.
	/// This is ignored if the current transfer is using the internal clock (it will still read the next bit to be shifted out).
	/// TODO: if bit 7 of SC is cleared, does this still shift a bit in when external clock is selected. (assuming yes for now)
	pub fn shift_bit_in(&mut self, bit: bool, interrupt_line: &mut InterruptLine) -> bool {
		let out = self.sb & 0x80 == 0x80;

		if self.sc & 1 == 0 {
			self.sb = self.sb << 1;
			self.sb |= bit as u8;
			self.bits_shifted += 1;
			// if bits_shifted >= 8 and bit 7 of sc is set an interrupt needs to be requested and bit 7 needs to be cleared.
			if self.bits_shifted >= 8 && self.sc & 0x80 == 0x80 {
				interrupt_line.request_interrupt(Interrupt::Serial);
				self.sc &= 0x7F;
				self.bits_shifted = 0;
			}
		}

		out
	}

	/// Emulate the serial port behaviour for 1 cycle.
	/// TODO: different transfer speeds for CGB mode.
	pub fn emulate_hardware(&mut self, interrupt_line: &mut InterruptLine) {
		if self.sc & 0x80 == 0x80 {
			// transfer active
			if self.sc & 1 == 1 {
				//internal clock
				if self.current_bit_cycles >= 64 {
					//shift bit out
					let out = self.sb & 0x80 == 0x80;
					let in_bit: bool = if let Some(ref mut cb) = self.callback {
						cb(out)
					}
					else {
						true
					};
					self.sb = (self.sb << 1) | (in_bit as u8);
					self.current_bit_cycles = 0;
					self.bits_shifted += 1;
					if self.bits_shifted >= 8 {
						interrupt_line.request_interrupt(Interrupt::Serial);
						self.sc &= 0x7F;
						self.bits_shifted = 0;
					}
				}
				else {
					self.current_bit_cycles += 1;
				}
			}
			self.transfer_cycle_counter += 1;
		}
	}
}
