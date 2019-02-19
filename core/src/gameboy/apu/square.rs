#![allow(dead_code)]
use std::f32::consts::PI;

const DUTY: [u8; 4] = [ 0b10000000, 0b11000000, 0b11110000, 0b11111100 ]; /* Duty cycles, 1 = on, 0 = off */

///Generate $desired_samples samples, and place them in $buffer
///sample_rate: Sample rate in Hz
///frequency_shift: Ammount in Hz to shift the frequency of the square wave
///base_time: The base time in seconds
///speed: A fraction that represents the playback speed, use 1.0_f32 for 60fps, otherwise it's
///		fps / 60.
///Returns how many samples were actually written to the buffer (this is only less than desired_samples is larger than the buffer can hold)
pub fn generate_square_wave(frequency: f32,
		volume: f32,
		buffer: &mut[f32],
		sample_rate: u32,
		desired_samples: u32,
		base_time: f32) -> usize {

	if desired_samples == 0 || frequency == 0f32 {
		return 0
	}

	let mut samples_written: usize = 0;

	for i in 0..desired_samples {
		if i as usize >= buffer.len() {
			break;
		}

		let time_offset = i as f32 / sample_rate as f32;
		let time = base_time + time_offset;
		let sin = f32::sin(time * frequency * 2.0_f32 * PI /* * 0.9230769_f32 */ );
		let square = f32::signum(sin);
		let sample = square * volume;
		buffer[i as usize] = sample;
		samples_written += 1;
	}

	samples_written
}

pub struct Sweep {
	last_freq: u16,
	current_freq: u16,
	sweep_step: u8,
}

impl Sweep {
	pub fn new(frequency: u16, steps: u8) -> Sweep {
		Sweep {
			last_freq: frequency,
			current_freq: frequency,
			sweep_step: steps,
		}
	}
}

pub struct VolumeEnvelope {
	volume: i8,	/* in the range [0,15], where 0 means no sound */
	num_steps: i8,	/* in the range [0,7], zero stops envelope operation */
	direction: i8, /* +1 or -1, this is added to the volume every step (direction of the envelope) */
}

impl VolumeEnvelope {
	pub fn new(volume: i8, direction: i8, num_steps: i8) -> VolumeEnvelope {
		VolumeEnvelope {
			volume: volume,
			direction: direction,
			num_steps: num_steps,
		}
	}

	///Bits 7 - 4: Initial volume
	///Bit 3: direction (0 = decrease, 1 = increase)
	///Bit 2 - 0: Number of envelope steps
	pub fn set(&mut self, val: u8) {
		self.volume = ((val >> 4) & 0x0F) as i8;
		self.direction = if val & 8 == 0 {
			-1
		} else {
			1
		};
		self.num_steps = (val & 7) as i8;
	}

	///The volume envelope is clocked at 64Hz (1/8 of apu frame sequencer clock)
	pub fn step(&mut self) {
		if self.num_steps != 0 {
			self.num_steps -= 1;
			let next_volume = self.volume + self.direction;
			if next_volume >= 0 && next_volume <= 15 {
				self.volume = next_volume;
			}
			else {
				/* volume out of range, stop envelope operation */
				self.num_steps = 0;
			}
		}
	}

	///Returns the volume as a float in the range [0,1] (0 = off, 1 = max)
	pub fn get_volume(&self) -> f32 {
		self.volume as f32 / 15.0_f32
	}
}

pub struct SquareChannel {
	frame_counter: u32, /* counter incremented at 512 Hz*/

	pub frequency: u16,

	pub length_enable: bool, //should the length counter expiring stop playback
	pub length: i8,

	sweep: Sweep,
	envelope: VolumeEnvelope,

}

impl SquareChannel {
	pub fn new() -> SquareChannel {
		SquareChannel {
			frame_counter: 0,

			frequency: 0,

			length_enable: false,
			length: 0,

			sweep: Sweep::new(0, 0),
			envelope: VolumeEnvelope::new(0,-1,0),
		}
	}

	///This should be called every 1/512 seconds, since the frame sequencer is powered by a
	///512 Hz clock
	pub fn step(&mut self) {
		//Length counter: 256 Hz (512 / 2)
		//Volume Envelope: 64 Hz (512 / 8)
		//Sweep: 128 Hz (512 / 4)

		self.frame_counter = self.frame_counter.wrapping_add(1);
		if self.frame_counter % 2 == 0 {
			//length counter
			if self.length > 0 {
				self.length -= 1;
			}
		}

		if self.frame_counter % 4 == 0 {
			//sweep
		}

		if self.frame_counter % 8 == 0 {
			//volume envelope
			self.envelope.step();
		}
	}

	pub fn set_envelope(&mut self, val: u8) {
		self.envelope.set(val);
	}

	pub fn sample(&mut self,
			buffer: &mut[f32],
			sample_rate: u32,
			desired_samples: u32,
			base_time: f32,
			speed: f32) -> usize {

		let mut volume = self.envelope.get_volume();
		let frequency = (131072/(2048 - ((self.frequency as u32) & 2047))) as f32;

		if self.length_enable && self.length <= 0 {
			/* channel disabled, set volume to 0 */
			volume = 0f32;
		}

		//let frequency = 440.0_f32;	//test tone
		//let volume = 0.4f32;

		generate_square_wave(frequency, volume, buffer, sample_rate, desired_samples, base_time)
	}
}
