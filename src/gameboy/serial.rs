use std::sync::mpsc::{Sender, Receiver, channel};
use ::gameboy::cpu::interrupts::{InterruptLine, Interrupt};

//pub type SerialCallback = (FnMut(u8) -> u8) + Send;

#[derive(Serialize, Deserialize)]
pub struct Serial {
	// Callback that is called when a byte is shifted through the serial port using the internal clock.
	// If an external clock is used to drive the serial port, this callback function will not be called.
	// The parameter is the byte that was shifted out of the serial port.
	// The return value is the byte to be shifted in.
	// removed in favor of using channels to handle serial communication.
	//callback: Option<Box<SerialCallback>>,

	/// Serial Data register ($FF01).
	/// When a transfer is active, the value in this register will be shifted out bit by bit, and on every
	/// shift a new bit will also be shifted in from the other end. (for performance reasons, the new data will only be loaded after all 8bits have been shifted out)
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

	/// Counts how many 4MHz cycles since the last bit has been shifted out
	current_bit_cycles: usize,

	/// Counts how many bits have been shifted during the current transfer (0...8)
	bits_shifted: u8,

	/// Stores bits shifted out during the current transfer so they can all be sent at once.
	data_out: u8,

	#[serde(skip)] // public so we can preserve the serial connection when a save state is loaded
	pub channels: Option<(Sender<u8>, Receiver<u8>)>
}

impl Serial {
	pub fn new() -> Serial {
		Serial {
			channels: None,
			sb: 0xFF,
			sc: 0,
			current_bit_cycles: 0,
			bits_shifted: 0,
			data_out: 0
		}
	}

	pub fn reset(&mut self) {
		self.sb = 0xFF;
		self.sc = 0;
		self.current_bit_cycles = 0;
		self.bits_shifted = 0;
		self.data_out = 0;
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
			self.current_bit_cycles = 0;
			self.bits_shifted = 0;
			self.data_out = 0;
		}
	}

	/// Create channels to handle async serial transfers.
	/// When the remote device wants to shift a byte over the serial port, send the byte using the sender, and then then a corresponding response will be sent to the reciever channel.
	/// When the local device wants to shift a byte out during a transfer driven by the internal clock, it will send the byte and block until a byte is recieved in response.
	/// If one of the channels returned by this function is dropped, then it is assumed that the external device has been disconnected.
	pub fn create_channels(&mut self) -> (Sender<u8>, Receiver<u8>) {
		let input: (Sender<u8>, Receiver<u8>) = channel();
		let output: (Sender<u8>, Receiver<u8>) = channel();
		let (input_send, input_recv) = input;
		let (output_send, output_recv) = output;
		self.channels = Some((output_send, input_recv));
		(input_send, output_recv)
	}

	/// Emulate the serial port behaviour for 1 cycle.
	/// TODO: different transfer speeds for CGB mode.
	pub fn emulate_hardware(&mut self, interrupt_line: &mut InterruptLine) {
		if let Some((ref mut sender, ref mut reciever)) = self.channels {
			// handle externaly driven transfers
			if let Ok(byte) = reciever.try_recv() {
				if self.sc & 1 == 0 {
					//externally driven transfer
					let out = self.sb;
					self.sb = byte;
					self.bits_shifted += 8;
					// if bits_shifted >= 8 and bit 7 of sc is set an interrupt needs to be requested and bit 7 needs to be cleared.
					if self.sc & 0x80 == 0x80 {
						interrupt_line.request_interrupt(Interrupt::Serial);
						self.sc &= 0x7F;
						self.bits_shifted = 0;
					}

					if let Err(_) = sender.send(out) {
						// the channel on the other end was closed
						self.channels = None;
					}
				}
				else {
					//internally driven transfer -> ignore
					let out = if self.sb & 0x80 == 0x80 {
						0xFF
					}
					else {
						0
					};

					if let Err(_) = sender.send(out) {
						// the channel on the other end was closed
						self.channels = None;
					}
				}
			}
		}
		if self.sc & 0x80 == 0x80 {
			// transfer active
			if self.sc & 1 == 1 {
				//internal clock
				if self.current_bit_cycles >= 64 {
					//shift bit out
					self.data_out |= self.sb & (0x80 >> (self.bits_shifted % 8));
					self.current_bit_cycles = 0;
					self.bits_shifted += 1;
					if self.bits_shifted >= 8 {
						// send data to connected device & get data back. (if anything is connected)
						if let Some((ref mut sender, ref mut receiver)) = self.channels {
							match sender.send(self.data_out) { //send byte out through channel
								Ok(_) => {
									match receiver.recv() { //block while waiting for response
										Ok(byte) => self.sb = byte,
										Err(_) => {
											self.channels = None; /* assume disconnected */
											self.sb = 0xFF;
										}
									};
								},
								Err(_) => { // assume disconnected
									self.sb = 0xFF;
									self.channels = None;
								}
							};
						}
						else {
							self.sb = 0xFF; //if no serial device is connected load 0xFF
						}

						interrupt_line.request_interrupt(Interrupt::Serial);
						self.sc &= 0x7F;
						self.current_bit_cycles = 0;
						self.bits_shifted = 0;
					}
				}
				else {
					self.current_bit_cycles += 1;
				}
			}
		}
	}
}
