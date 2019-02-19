use std::cell::Cell;
use gameboy::cartridge::{ROM_BANK_SIZE, RAM_BANK_SIZE};
use super::MemoryBankController;
use time;

/* What happens if you play at a different speed, the rtc should reflect that... */
/* Can you write to the rtc registers? If you write while there is an rtc value latched does it change the latched value or the real value? */
/* What happens when you try to set the ram_bank to an invalid value (something not in the range (0h...3h)U(8h...Ch)?*/

#[derive(Clone, Copy, Debug)]
#[derive(Serialize, Deserialize)]
pub struct Duration {
	seconds: usize, /* 0 -  59 */
	minutes: usize, /* 0 -  59 */
	hours: usize,   /* 0 -  23 */
	days: usize     /* 0 - 511 */
}

impl Duration {
	pub fn new() -> Duration {
		Duration {
			seconds: 0,
			minutes: 0,
			hours: 0,
			days: 0
		}
	}

	pub fn from(seconds: usize) -> Duration {
		Duration {
			seconds: seconds % 60,
			minutes: (seconds / 60) % 60,
			hours: (seconds / 3600) % 24,
			days: (seconds / 86400) % 512
		}
	}

	///Get the duration this object represents in seconds
	pub fn get_seconds(&self) -> usize {
		self.seconds + (self.minutes * 60) + (self.hours * 3600) + (self.days * 86400)
	}

	///Adds some seconds to the seconds of this duration, and returns a new duration that represents the sum
	pub fn add_seconds(&self, seconds: usize) -> Duration {
		Duration::from(self.get_seconds() + seconds)
	}
}

#[derive(Clone, Copy, Debug)]
#[derive(Serialize, Deserialize)]
pub struct RTC {
	last: i64, //the last time the timer was updated (unix timestamp)
	duration: Duration,   //the value of the rtc as of the last time update was called
	latched: Option<Duration>, //the latched duration (if any)
	halt: bool,
	day_carry: bool
}

impl RTC {
	pub fn new() -> RTC {
		RTC {
			last: time::now_utc().to_timespec().sec,
			duration: Duration::new(),
			latched: None,
			halt: false, //if the rtc is halted, the duration isn't updated in the update method
			day_carry: false
		}
	}

	pub fn update(&mut self) {
		let time = time::now_utc().to_timespec().sec; //current unix timestamp
		let delta = self.last - time;
		if delta > 0 { //if now is before (or the same) the last time it was updated then something has gone wrong
			self.last = time;
			if !self.halt {
				let new_duration = self.duration.add_seconds(delta as usize);
				if self.duration.days > new_duration.days {
					//day overflowed (day is 9 bits, 0 - 511)
					self.day_carry = true;
				}
				self.duration = new_duration;
			}
		}
	}

	///Latch the current duration
	pub fn latch(&mut self) {
		self.update();
		self.latched = Some(self.duration);
	}

	pub fn unlatch(&mut self) {
		self.latched = None;
	}

	///Read from the RTC_S register
	pub fn seconds(&self) -> u8 {
		match self.latched {
			Some(duration) => duration.seconds as u8,
			None => self.duration.seconds as u8
		}
	}

	///Write to the RTC_S register
	pub fn set_seconds(&mut self, value: u8) {
		self.duration.seconds = (value as usize) % 60;
	}

	///Read from the RTC_M register
	pub fn minutes(&self) -> u8 {
		match self.latched {
			Some(duration) => duration.minutes as u8,
			None => self.duration.minutes as u8
		}
	}

	///Write to the RTC_M register
	pub fn set_minutes(&mut self, value: u8) {
		self.duration.minutes = (value as usize) % 60;
	}

	///Read from the RTC_H register
	pub fn hours(&self) -> u8 {
		match self.latched {
			Some(duration) => duration.hours as u8,
			None => self.duration.hours as u8
		}
	}

	///Write to the RTC_H register
	pub fn set_hours(&mut self, value: u8) {
		self.duration.hours = (value as usize) % 24;
	}

	///Read from the RTC_DL register
	pub fn days_low(&self) -> u8 {
		match self.latched {
			Some(duration) => duration.days as u8,
			None => self.duration.days as u8
		}
	}

	///Write to the RTC_DL register
	pub fn set_days_low(&mut self, value: u8) {
		let days = self.duration.days;
		self.duration.days = (value as usize) | (days & 256);
	}

	///Read from the RTC_DH register
	///Bit 0: High bit of day counter
	///Bit 6: Halt RTC (0=running, 1=halted)
	///Bit 7: Day Counter Overflow
	pub fn days_high(&self) -> u8 {
		let days = match self.latched {
			Some(duration) => duration.days,
			None => self.duration.days
		};
		let halt = match self.halt {
			true => 64,
			false => 0
		};
		let day_carry = match self.day_carry {
			true => 128,
			false => 0
		};
		((days >> 8) as u8 & 1) | halt | day_carry
	}

	///Write to the RTC_DH register
	pub fn set_days_high(&mut self, value: u8) {
		self.duration.days = (self.duration.days & 255) | (((value as usize) >> 8) & 256);
		self.halt = value & 64 != 0;
		self.day_carry = value & 128 != 0;
	}
}

/* Ram bank numbers used to access the different rtc registers */
const RTC_S: u8 = 0x08;
const RTC_M: u8 = 0x09;
const RTC_H: u8 = 0x0A;
const RTC_DL: u8 = 0x0B;
const RTC_DH: u8 = 0x0C;

#[derive(Serialize, Deserialize)]
pub struct MBC3 {
	rom_bank: u8,      /* current rom bank (7 bits, can't be 0) */
	ram_bank: u8,      /* current ram bank or rtc register */
	latch_written: Option<u8>, /* last value written to the address range 0x6000-0x7FFF, when 00h and then 01h are written in succession the clock is latched */
	rtc: Option<Cell<RTC>>,
	ram_timer_enable: bool
}

impl MBC3 {
	pub fn new(rtc_enabled: bool) -> MBC3 {
		let rtc: Option<Cell<RTC>> = if rtc_enabled{
			Some(Cell::new(RTC::new()))
		}
		else {
			None
		};

		MBC3 {
			rom_bank: 1,
			ram_bank: 0,
			latch_written: None,
			rtc: rtc,
			ram_timer_enable: false
		}
	}
}

impl MemoryBankController for MBC3 {
	fn read_byte_rom(&self, rom: &Box<[u8]>, rom_size: usize, offset: u16) -> u8 {
		if offset < 0x4000 {
			//0...0x4000 is permanately mapped to bank 0
			if (offset as usize) < rom_size {
				rom[offset as usize]
			}
			else {
				0xFF
			}
		}
		else {
			let bank_offset: usize = (self.rom_bank as usize) * ROM_BANK_SIZE;
			if bank_offset + offset as usize >= rom_size {
				0xFF
			}
			else {
				rom[bank_offset + (offset - 0x4000) as usize]
			}
		}
	}

	fn read_byte_ram(&self, ram: &Box<[u8]>, ram_size: usize, offset: u16) -> u8 {
		if self.ram_timer_enable {
			match self.ram_bank {
				0...3 => {
					let bank_offset = RAM_BANK_SIZE * self.ram_bank as usize;
					if (bank_offset + offset as usize) < ram_size {
						ram[bank_offset + offset as usize]
					}
					else {
						0xFF
					}
				},
				0x8...0xC => {
					if let Some(rtc_cell) = self.rtc.as_ref() {
						let mut rtc = rtc_cell.get();
						rtc.update();
						rtc_cell.set(rtc);
						match self.ram_bank {
							RTC_S => rtc.seconds(),
							RTC_M => rtc.minutes(),
							RTC_H => rtc.hours(),
							RTC_DL => rtc.days_low(),
							RTC_DH => rtc.days_high(),
							_ => 0xFF
						}
					}
					else {
						0xFF //rtc register selected, but no rtc present
					}
				}
				_ => 0xFF //the selected ram bank is invalid (not a valid ram bank or a valid rtc register)
			}
		}
		else {
			0xFF
		}
	}

	fn write_byte_rom(&mut self, address: u16, value: u8) {
		/* Writing to the rom sets internal mbc registers */
		match address {
			0...0x1FFF => self.ram_timer_enable = value == 0x0A,
			0x2000...0x3FFF => self.rom_bank = value & 0x7F,
			0x4000...0x5FFF => {
				/* Select ram bank or a rtc register: 0-3 are ram banks, 8-Ch are rtc registers */
				match value {
					0...3 | 8...0xC => self.ram_bank = value,
					_ => self.ram_bank = 0
				}
			},
			0x6000...0x7FFF => {
				/* latch clock data (write 00h then 01h) */
				if let Some(rtc_cell) = self.rtc.as_mut() {
					let rtc = rtc_cell.get_mut();
					if self.latch_written == Some(0) && value == 1 {
						rtc.latch();
					}
					else if value == 0 {
						rtc.unlatch();
					}
				}
				self.latch_written = Some(value);
			},
			_ => {/* Cartridge memory is only from 0h...7FFFh, this isn't a valid address */}
		}
	}

	fn write_byte_ram(&mut self, ram: &mut Box<[u8]>, ram_size: usize, offset: u16, value: u8) {
		if self.ram_timer_enable {
			match self.ram_bank {
				0...3 => {
					let bank_offset = RAM_BANK_SIZE * self.ram_bank as usize;
					if (bank_offset + offset as usize) < ram_size {
						ram[bank_offset + offset as usize] = value;
					}
				},
				8...0xC => {
					/* Write to rtc registers */
					if let Some(rtc_cell) = self.rtc.as_mut() {
						let mut rtc = rtc_cell.get();
						match self.ram_bank {
							RTC_S => rtc.set_seconds(value),
							RTC_M => rtc.set_minutes(value),
							RTC_H => rtc.set_hours(value),
							RTC_DL => rtc.set_days_low(value),
							RTC_DH => rtc.set_days_high(value),
							_ => {}
						};
						rtc_cell.set(rtc);
					}
				}
				_ => {}
			}
		}
	}

	fn rom_bank(&self) -> usize {
		self.rom_bank as usize
	}

	fn ram_bank(&self) -> usize {
		self.ram_bank as usize
	}
}
