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
	pub fn new(rom: &Box<[u8]>) -> Result<CartInfo, & 'static str> {
		let mbc_type: MBCType = try!(CartInfo::get_type(rom[0x0147]));
		let rom_size: usize = try!(CartInfo::get_rom_size(rom[0x0148]));
		let ram_size: usize = try!(CartInfo::get_ram_size(rom[0x0149]));

		if rom.len() < 0x150 {
			return Err("Rom is too small to contain a rom header (rom is smaller than 0x150 bytes)");
		}

		let mut info = CartInfo {
			title: String::from(""),	//TODO
			sgb: rom[0x0146] == 0x03,
			cgb: rom[0x0143] & 0x80 == 0x80,
			battery: CartInfo::has_battery(rom[0x0147]),
			mbc_type: mbc_type,
			rom_size: rom_size,
			ram_size: ram_size,
		};

		Ok(info)
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

	fn get_type(cart_type: u8) -> Result<MBCType, & 'static str> {
		match cart_type {
			0x00 => Ok(MBCType::NONE),
			0x01...0x03 => Ok(MBCType::MBC1),
			0x05...0x06 => Ok(MBCType::MBC2),
			0x08...0x09 => Ok(MBCType::NONE),
			0x0B...0x0D => Ok(MBCType::MMM01),
			0x0F...0x13 => Ok(MBCType::MBC3),
			0x15...0x17 => Ok(MBCType::MBC4),
			0x19...0x1E => Ok(MBCType::MBC5),
			0x20 => Ok(MBCType::MBC6),
			0x22 => Ok(MBCType::MBC7),
			0xFC => Ok(MBCType::CAMERA),
			0xFD => Ok(MBCType::TAMA5),
			0xFE => Ok(MBCType::HUC3),
			0xFF => Ok(MBCType::HUC1),
			_ => Err(("Invalid value Cartridge Type in Cartridge Header at index 0x0147!")),
		}
	}

	fn get_rom_size(rom_size: u8) -> Result<usize, & 'static str> {
		match rom_size {
			0x00...0x07 => Ok(0x8000 << rom_size),
			0x52 => Ok(0x4000 * 72),
			0x53 => Ok(0x4000 * 80),
			0x54 => Ok(0x4000 * 96),
			_ => Err(("Invalid value Rom Size in Cartridge Header at index 0x0148!")),
		}
	}

	fn get_ram_size(ram_size: u8) -> Result<usize, & 'static str> {
		match ram_size {
			0x00 => Ok(0),
			0x01 => Ok(2 * 1024), 	//2KB
			0x02 => Ok(8 * 1024),	//8KB
			0x03 => Ok(32 * 1024),	//4 8KB banks
			0x04 => Ok(128 * 1024),	//16 8KB banks
			0x05 => Ok(64 * 1024),	//8 8KB banks
			_ => Err(("Invalid value Ram Size in Cartridge Header at index 0x0149!")),
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
	pub fn new(rom: Box<[u8]>, ram: Option<Box<[u8]>>) -> Result<VirtualCartridge, & 'static str> {
		let cart_info: CartInfo = try!(CartInfo::new(&rom));

		//TODO: expand ram if the ram file loaded is too small (and give a warning?)
		let ram = match ram {
			Some(ram) => ram,
			None => {
				//No ram supplied, allocate some.
				let vec: Vec<u8> = Vec::with_capacity(cart_info.ram_size);
				vec.into_boxed_slice()
			}
		};

		let mbc: Result<Box<MemoryBankController>, & 'static str> = match cart_info.mbc_type {
			MBCType::NONE => Ok(Box::new(NoMBC::new())),
			MBCType::MBC1 => Ok(Box::new(MBC1::new())),
			_ => Err("Unimplemented MBC"),	//TODO: more helpful error message
		};

		let mbc = try!(mbc);

		let cart = VirtualCartridge {
			rom: rom,
			ram: ram,
			mbc: mbc,
			cart_info: cart_info,
		};

		Ok(cart)
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
