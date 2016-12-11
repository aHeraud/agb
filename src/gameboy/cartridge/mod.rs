mod nombc;
mod mbc1;

use alloc::boxed::Box;
use collections::vec::Vec;
use collections::string::String;

use gameboy::cartridge::nombc::NoMBC;
use gameboy::cartridge::mbc1::MBC1;

#[derive(Debug)]
pub enum MBCType {
	NONE,
	MBC1,
	MBC2,
	MMM01,
	MBC3,
	MBC4,
	MBC5,
	MBC6,
	MBC7,
	CAMERA,
	TAMA5,
	HUC3,
	HUC1
}

#[derive(Debug)]
pub struct CartInfo {
	pub title: String,
	pub sgb: bool,
	pub cgb: bool,
	pub mbc_type: MBCType,
	pub battery: bool,
	pub rom_size: usize,
	pub ram_size: usize,
}

pub trait Cartridge {
	fn read_byte_rom(&self, address: u16) -> u8;
	fn read_byte_ram(&self, address: u16) -> u8;

	fn write_byte_rom(&mut self, address: u16, value: u8);
	fn write_byte_ram(&mut self, address: u16, value: u8);

	fn get_cart_info(&self) -> &CartInfo;
}

pub trait MemoryBankController {
	fn read_byte_rom(&self, rom: &Box<[u8]>, rom_size: usize, address: u16) -> u8;
	fn read_byte_ram(&self, ram: &Box<[u8]>, ram_size: usize, address: u16) -> u8;

	fn write_byte_rom(&mut self, address: u16, value: u8);
	fn write_byte_ram(&self, ram: &mut Box<[u8]>, ram_size: usize, adress: u16, value: u8);
}

impl CartInfo {
	pub fn new(rom: &Box<[u8]>) -> CartInfo {
		CartInfo {
			title: String::from(""),	//TODO
			sgb: rom[0x0146] == 0x03,
			//cgb: rom[0x0143] == 0x80 || rom[0x0143] == 0xC0,
			cgb: rom[0x0143] & 0x80 == 0x80,
			mbc_type: CartInfo::get_type(rom[0x0147]),
			battery: CartInfo::has_battery(rom[0x0147]),
			rom_size: CartInfo::get_rom_size(rom[0x0148]),
			ram_size: CartInfo::get_ram_size(rom[0x0149]),
		}
	}

	fn has_battery(cart_type: u8) -> bool {
		match cart_type {
			0x03 => true,
			0x06 => true,
			0x09 => true,
			0x0D => true,
			0x0F...0x10 => true,
			0x13 => true,
			0x17 => true,
			0x1B => true,
			0x1E => true,
			0x22 => true,
			0xFF => true,
			_ => false,
		}
	}

	fn get_type(cart_type: u8) -> MBCType {
		match cart_type {
			0x00 => MBCType::NONE,
			0x01...0x03 => MBCType::MBC1,
			0x05...0x06 => MBCType::MBC2,
			0x08...0x09 => MBCType::NONE,
			0x0B...0x0D => MBCType::MMM01,
			0x0F...0x13 => MBCType::MBC3,
			0x15...0x17 => MBCType::MBC4,
			0x19...0x1E => MBCType::MBC5,
			0x20 => MBCType::MBC6,
			0x22 => MBCType::MBC7,
			0xFC => MBCType::CAMERA,
			0xFD => MBCType::TAMA5,
			0xFE => MBCType::HUC3,
			0xFF => MBCType::HUC1,
			_ => panic!("Invalid value Cartridge Type in Cartridge Header at index 0x0147!")
		}
	}

	fn get_rom_size(rom_size: u8) -> usize {
		match rom_size {
			0x00...0x07 => 0x8000 << rom_size,
			0x52 => 0x4000 * 72,
			0x53 => 0x4000 * 80,
			0x54 => 0x4000 * 96,
			_ => panic!("Invalid value Rom Size in Cartridge Header at index 0x0148!"),
		}
	}

	fn get_ram_size(ram_size: u8) -> usize {
		match ram_size {
			0x00 => 0,
			0x01 => 2 * 1024, 	//2KB
			0x02 => 8 * 1024,	//8KB
			0x03 => 32 * 1024,	//4 8KB banks
			0x04 => 128 * 1024,	//16 8KB banks
			0x05 => 64 * 1024,	//8 8KB banks
			_ => panic!("Invalid value Ram Size in Cartridge Header at index 0x0149!"),
		}
	}
}

pub struct VirtualCartridge {
	rom: Box<[u8]>,
	ram: Box<[u8]>,
	cart_info: CartInfo,
	mbc: Box<MemoryBankController>,
}

impl VirtualCartridge {
	pub fn new(rom: Box<[u8]>, ram: Option<Box<[u8]>>) -> VirtualCartridge {
		let cart_info: CartInfo = CartInfo::new(&rom);

		//TODO: expand ram if the ram file loaded is too small (and give a warning?)
		let ram = match ram {
			Some(ram) => ram,
			None => {
				//No ram supplied, allocate some.
				let vec: Vec<u8> = Vec::with_capacity(cart_info.ram_size);
				vec.into_boxed_slice()
			}
		};

		VirtualCartridge {
			rom: rom,
			ram: ram,
			mbc: match cart_info.mbc_type {
				MBCType::NONE => Box::new(NoMBC::new()),
				MBCType::MBC1 => Box::new(MBC1::new()),
				_ => panic!("Unimplemented MBC: {:?}", cart_info.mbc_type),
			},
			cart_info: cart_info,
		}
	}
}

impl Cartridge for VirtualCartridge {
	fn read_byte_rom(&self, address: u16) -> u8  {
		self.mbc.read_byte_rom(&self.rom, self.cart_info.rom_size, address)
	}

	fn read_byte_ram(&self, address: u16) -> u8 {
		self.mbc.read_byte_ram(&self.ram, self.cart_info.ram_size, address)
	}

	fn write_byte_rom(&mut self, address: u16, value: u8) {
		self.mbc.write_byte_rom(address, value);
	}

	fn write_byte_ram(&mut self, address: u16, value: u8) {
		self.mbc.write_byte_ram(&mut self.ram, self.cart_info.ram_size, address, value);
	}

	fn get_cart_info(&self) -> &CartInfo {
		&self.cart_info
	}
}
