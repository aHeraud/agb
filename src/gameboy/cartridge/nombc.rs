#[cfg(feature = "no_std")]
use alloc::boxed::Box;

use super::MemoryBankController;

pub struct NoMBC {

}

impl NoMBC {
	pub fn new() -> NoMBC {
		NoMBC {

		}
	}
}

impl MemoryBankController for NoMBC {
	fn read_byte_rom(&self, rom: &Box<[u8]>, rom_size: usize, address: u16) -> u8 {
		let address: usize = address as usize;
		if address < rom_size {
			return rom[address];
		}
		else {
			return 0xFF;
		}
	}

	fn read_byte_ram(&self, ram: &Box<[u8]>, ram_size: usize, address: u16) -> u8 {
		let address: usize = address as usize;
		if address < ram_size {
			return ram[address];
		}
		else {
			return 0xFF;
		}
	}

	#[allow(unused_variables)]
	fn write_byte_rom(&mut self, address: u16, value: u8) {
		//This isn't a real mbc, so this doesn't do anything
	}

	fn write_byte_ram(&self, ram: &mut Box<[u8]>, ram_size: usize, address: u16, value: u8) {
		let address: usize = address as usize;
		if address < ram_size {
			ram[address] = value;
		}
	}

	fn rom_bank(&self) -> usize {
		1 //no mbc, so no bank swapping
	}

	fn ram_bank(&self) -> usize {
		0  //no mbc, so no bank swapping
	}
}
