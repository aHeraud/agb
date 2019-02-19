mod square;

use gameboy::apu::square::SquareChannel;

const AUDIO_BUFFER_LENGTH: usize = 8192;

pub struct APU {
	audio_buffer: Box<[f32]>,
	buffer_index: usize,
	counter: u32, /* Increments at 4.2 Mhz */
	timer: u32, /* Increments at 512 Hz*/
	sample_rate: u32,	/* Sample per second (Hz) */
	square_1: SquareChannel,
	square_2: SquareChannel,
}

impl APU {
	pub fn new() -> APU {
		let buff: [f32; AUDIO_BUFFER_LENGTH] = [0.0_f32; AUDIO_BUFFER_LENGTH];
		APU {
			audio_buffer: Box::new(buff),
			buffer_index: 0,
			counter: 0,
			timer: 0,
			sample_rate: 41000,
			square_1: SquareChannel::new(),
			square_2: SquareChannel::new(),
		}
	}

	///Get the content of the audio buffer as a slice.
	///It's assumed that the caller will copy the data.
	pub fn get_audio_buffer(&mut self) -> &[f32] {
		let index = self.buffer_index;
		self.buffer_index = 0;
		&self.audio_buffer[0..index]
	}

	pub fn emulate_hardware(&mut self, double_speed_mode: bool, div: u16, last_div: u16) {
		/*
			From what i understand the sound clock is actually the cpu clock divided by 8192
			(16,384 in double speed mode) which equals ~512 Hz.
			http://gbdev.gg8.se/wiki/articles/Timer_Obscure_Behaviour
		*/

		if double_speed_mode {
			self.counter = self.counter.wrapping_add(2);
		}
		else {
			self.counter = self.counter.wrapping_add(4);
		}

		/* sample at 512 Hz */
		if self.counter % 8192 == 0 {
			self.timer = self.timer.wrapping_add(1);

			let base_time = (self.timer as f32) / 512.0_f32;
			let sample_count = self.sample_rate / 512;	/* 1 sound frame is 1/512 second */

			let samples_generated = self.square_2.sample(
				&mut self.audio_buffer[self.buffer_index .. AUDIO_BUFFER_LENGTH],
				self.sample_rate,
				sample_count,
				base_time,
				1.0_f32
			);
			self.buffer_index += samples_generated;
		}

		/* Falling edge detector for 512 Hz timer driven by divider register */
		if (double_speed_mode && (last_div & 16384 == 16) && (div & 16384 == 0)) ||
			((double_speed_mode == false) && (last_div & 8192 == 8192)  && (div & 8192 == 0)) {

			self.square_1.step();
			self.square_2.step();
		}
	}

	///Write a byte to the sound registers
	///The sound registers are mapped  to 0xFF10 - 0xFF3F
	///Panics if address is out of range
	pub fn write_to_sound_registers(&mut self, io: &mut[u8], address: u16, value: u8) {
		match address {
			0xFF10...0xFF3F => {
				io[(address as usize) - 0xFF10] = value;
				match address {
					/* Square 1 */
					0xFF10 => {
						/* NR10: Sweep period, negate, shift */
					},
					0xFF11 => {
						/* NR11: Duty, Length load (64-L) */
						//TODO: duty
						self.square_1.length = (value & 63) as i8;
					},
					0xFF12 => {
						/* NR12: Starting volume, envelope add mode, period */
						self.square_1.set_envelope(value);
					},
					0xFF13 => {
						/* NR13: Frequency lsb */
						let x: u16 = (value as u16) | (((io[0x14] & 7) as u16) << 8);
						self.square_1.frequency = x;
					},
					0xFF14 => {
						/* NR14: Trigger, length enable, frequency msb */
						//TODO: trigger
						self.square_1.length_enable = value & 64 == 1;
						let x: u16 = (((value & 7) as u16) << 8) | (io[0x13] as u16);
						self.square_1.frequency = x;
					},

					/* Square 2 */
					0xFF16 => {
						/* NR21: Duty, Length load (64 - L) */
						//TODO: duty
						self.square_2.length = (value & 63) as i8;
					},
					0xFF17 => {
						/* NR22: Starting volume, Envelope add mode, period */
						self.square_2.set_envelope(value);
					},
					0xFF18 => {
						/* NR23: Frequency lsb */
						let x: u16 = (value as u16) | (((io[0x14] & 7) as u16) << 8);
						self.square_2.frequency = x;
					},
					0xFF19 => {
						/* NR24: Trigger, length  enable, frequency msb */
						//TODO: trigger
						self.square_2.length_enable = value & 64 == 1;
						let x: u16 = (((value & 7) as u16) << 8) | (io[0x13] as u16);
						self.square_2.frequency = x;
					},

					/* Control registers */
					0xFF24 => {
						/* NR50: Channel control, on-off, volume */

					}

					_ => {}
				};
			}
			_ => {
				println!("Attempted to write value {} to address {:#4X}.", value, address);
				panic!("Invalid address, address must be in the range [0xFF10 - 0xFF3F].")
			},
		};
	}

	///Read from the sound registers (0xFF10...0xFF35)
	pub fn read_from_sound_registers(&self, io: &[u8], address: u16) -> u8 {
		/* TODO */
		match address {
			_ => 0xFF
		}
	}
}
