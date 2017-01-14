use std::vec;
use std::num;

const AUDIO_BUFFER_LENGTH: usize = 4096;

pub struct Square {
	wave_pattern_duty: u8,
	length: u8,

	/* Volume Envelope: 0xFF12 */
	initial_volume: u8,
	envelope_increase: bool,
	envelope_sweeps: u8,//If this is zero, stop envelope operation

	frequency: u16,
	//frequency_low: u8,
	//frequency_high: u8,
	repeat: bool,
}

impl Square {
	pub fn new() -> Square {
		/* TODO: Initial values */
		Square {
			wave_pattern_duty: 0,
			length: 0,
			initial_volume: 0,
			envelope_increase: false,
			envelope_sweeps: 0,
			frequency: 0,
			//frequency_low: 0,
			//frequency_high: 0,
			repeat: false,
		}
	}

	pub fn sample(&self, sample_rate: u32 /* Sample rate in Hz */, counter: u32) {
		//Square wave generator
		let volume: f32 = 0f32;
		//let mut sample: f32 = 0f32 = volume * num::signum(sin());
	}
}

pub struct Sweep {
	sweep_time: u8,
	sweep_decrease: bool,
	sweep_shifts: u8,
	initial_frequency: u16,
	square: Square,
}

impl Sweep {
	pub fn new() -> Sweep {
		Sweep {
			sweep_time: 0,
			sweep_decrease: false,
			sweep_shifts: 0,
			initial_frequency: 0,
			square: Square::new(),
		}
	}
}

pub struct APU {
	audio_buffer: Box<[f32]>,
	counter: u32,
	sample_rate: u32,
	square_1: Sweep,
	square_2: Square,
}

impl APU {
	pub fn new() -> APU {
		APU {
			audio_buffer: Vec::with_capacity(AUDIO_BUFFER_LENGTH).into_boxed_slice(),
			counter: 0,
			sample_rate: 41000,
			square_1: Sweep::new(),
			square_2: Square::new(),
		}
	}

	pub fn emulate_hardware(&mut self, io: &[u8]) {

	}

	///Write a byte to the sound registers
	///The sound registers are mapped  to 0xFF10 - 0xFF3F
	///Panics if address is out of range
	pub fn write_to_sound_registers(&mut self, address: u16, value: u8) {
		match address {
			/* Channel 1 */
			0xFF10 => {
				/* NR10: Sweep Register */
				self.square_1.sweep_time = (value >> 4) & 7;
				self.square_1.sweep_decrease = value & 8 == 8;
				self.square_1.sweep_shifts = value & 7;
			},
			0xFF11 => {
				/* NR11: Sound Length/Wave Pattern Duty */
				self.square_1.square.wave_pattern_duty = (value >> 6) & 3;
				self.square_1.square.length = value & 0x3F;
			},
			0xFF12 => {
				/* NR12: Volume envelope */
				self.square_1.square.initial_volume = (value >> 4) & 0x0F;
				self.square_1.square.envelope_increase = (value & 8) == 8;
				self.square_1.square.envelope_sweeps = value & 7;
			},
			0xFF13 => {
				/* NR13: Frequency Low */
				self.square_1.square.frequency = (self.square_1.square.frequency & 0xFF00) | (value as u16);
			},
			0xFF14 => {
				/* NR14: Frequency High */
				self.square_1.square.frequency = (self.square_1.square.frequency & 0x00FF) | (((value & 0x7) as u16) << 8);
				self.square_1.square.repeat = value & 64 == 0;
				if value & 128 == 128 {
					//Restart sound
					//TODO
				}
			},

			/* Channel 2 */
			0xFF15 => { /* NR20: Not Used */ },
			0xFF16 => {
				/* NR21: Sound Length / Wave Pattern Duty */
				self.square_2.wave_pattern_duty = (value >> 6) & 3;
				self.square_2.length = value & 0x3F;
			},
			0xFF17 => {
				/* NR22: Channel 2 Volume Envelope */
				self.square_2.initial_volume = (value >> 4) & 0x0F;
				self.square_2.envelope_increase = (value & 8) == 8;
				self.square_2.envelope_sweeps = value & 7;
			},
			0xFF18 => {
				/* NR23: Channel 2 Frequency Low */
				self.square_2.frequency = (self.square_1.square.frequency & 0xFF00) | (value as u16);
			}
			0xFF19 => {
				/* NR24: Channel 2 Frequency High */
				self.square_2.frequency = (self.square_1.square.frequency & 0x00FF) | (((value & 0x7) as u16) << 8);
				self.square_2.repeat = value & 64 == 0;
				if value & 128 == 128 {
					//Restart sound
					//TODO
				}
			}
			0xFF20...0xFF35 => { /* TODO */ },
			_ => panic!("Invalid address, address must be in the range [0xFF10 - 0xFF3F]."),
		}
	}

	///Read from the sound registers (0xFF10...0xFF35)
	pub fn read_from_sound_registers(address: u16) -> u8 {
		/* TODO */
		match address {
			_ => 0xFF
		}
	}
}
