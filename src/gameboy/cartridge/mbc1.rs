use super::MemoryBankController;

#[derive(Debug)]
#[derive(PartialEq)]
enum ModeSelect {
	Rom, Ram
}

pub struct MBC1 {
	ram_bank: u8,
	rom_bank: u8,
	mode: ModeSelect,
	ram_enable: bool,
}

impl MBC1 {
	pub fn new() -> MBC1 {
		MBC1 {
			ram_bank: 0,
			rom_bank: 1,
			mode: ModeSelect::Rom,
			ram_enable: false,
		}
	}
}

impl MemoryBankController for MBC1 {
	fn read_byte_rom(&self, rom: &Box<[u8]>, rom_size: usize, address: u16) -> u8 {
		let address: usize = match address {
			0x0000...0x3FFF => address as usize,
			0x4000...0x7FFF => {
				let mut rom_bank: u8 = self.rom_bank;
				if self.mode == ModeSelect::Rom {
					rom_bank |= self.ram_bank << 5;
				}
				(address - 0x4000) as usize + (0x4000 * rom_bank as usize)
			},
			_ => panic!("Invalid parameters for read_byte_rom: address must be in the range 0x0000...0x7FFF"),
		};
		if address < rom_size {
			return rom[address];
		}
		else {
			return 0xFF;
		}
	}

	fn read_byte_ram(&self, ram: &Box<[u8]>, ram_size: usize, address: u16) -> u8 {
		let mut ram_bank: u8 = 0;
		if self.mode == ModeSelect::Ram {
			ram_bank |= self.ram_bank;
		}
		let address: usize = address as usize + (0x2000 * ram_bank as usize);
		if address < ram_size {
			return ram[address];
		}
		else {
			return 0xFF;
		}
	}

	#[allow(unused_variables)]
	fn write_byte_rom(&mut self, address: u16, value: u8) {
		//0x0000...0x1FFF - RAM enable
		//0x2000...0x3FFF - ROM Bank number (5-bits)
		//0x4000...0x5FFF - RAM Bank number (2-bits)
		//0x6000...0x7FFF - ROM/RAM Mode Select (0=Rom, 1=Ram)
		match address {
			0x0000...0x1FFF => self.ram_enable = (value & 0x0A) == 0x0A,
			0x2000...0x3FFF => self.rom_bank = value & 0x1F,
			0x4000...0x5FFF => self.ram_bank = value & 3,
			0x6000...0x7FFF => {
				if value & 1 == 0 { self.mode = ModeSelect::Rom; }
				else { self.mode = ModeSelect::Ram; }
			},
			_ => return,
		};
	}

	fn write_byte_ram(&mut self, ram: &mut Box<[u8]>, ram_size: usize, address: u16, value: u8) {
		let mut ram_bank: u8 = 0;
		if self.mode == ModeSelect::Ram {
			ram_bank |= self.ram_bank;
		}
		let address: usize = address as usize + (0x2000 * ram_bank as usize);
		if address < ram_size {
			ram[address] = value;
		}
	}

	fn rom_bank(&self) -> usize {
		let mut rom_bank: u8 = self.rom_bank;
		if self.mode == ModeSelect::Rom {
			rom_bank |= self.ram_bank << 5;
		}
		rom_bank as usize
	}

	fn ram_bank(&self) -> usize {
		let mut ram_bank: u8 = 0;
		if self.mode == ModeSelect::Ram {
			ram_bank |= self.ram_bank;
		}
		ram_bank as usize
	}
}
