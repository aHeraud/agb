mod nombc;
mod mbc1;
mod mbc3;

use gameboy::cartridge::nombc::NoMBC;
use gameboy::cartridge::mbc1::MBC1;
use gameboy::cartridge::mbc3::MBC3;

pub const ROM_BANK_SIZE: usize = 0x4000;
pub const RAM_BANK_SIZE: usize = 0x2000;

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
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
#[derive(Serialize, Deserialize)]
pub struct CartInfo {
	pub title: String,
	pub sgb: bool,
	pub cgb: bool,
	pub mbc_type: MBCType,
	pub battery: bool,
	pub rtc: bool,
	pub rom_size: usize,
	pub ram_size: usize,
}

pub trait Cartridge {
	fn read_byte_rom(&self, offset: u16) -> u8;
	fn read_byte_ram(&self, offset: u16) -> u8;

	fn write_byte_rom(&mut self, offset: u16, value: u8);
	fn write_byte_ram(&mut self, offset: u16, value: u8);

	fn get_cart_info(&self) -> &CartInfo;

	fn rom(&self) -> &[u8];
	fn rom_mut(&mut self) -> &mut[u8];

	fn banked_rom(&self) -> &[u8];
	fn banked_rom_mut(&mut self) -> &mut[u8];

	fn ram(&self) -> &[u8];
	fn ram_mut(&mut self) -> &mut[u8];
}

pub trait MemoryBankController: Send {
	fn read_byte_rom(&self, rom: &Box<[u8]>, rom_size: usize, offset: u16) -> u8;
	fn read_byte_ram(&self, ram: &Box<[u8]>, ram_size: usize, offset: u16) -> u8;

	fn write_byte_rom(&mut self, offset: u16, value: u8);
	fn write_byte_ram(&mut self, ram: &mut Box<[u8]>, ram_size: usize, offset: u16, value: u8);

	fn rom_bank(&self) -> usize;
	fn ram_bank(&self) -> usize;
}

impl CartInfo {
	pub fn new(rom: &Box<[u8]>) -> Result<CartInfo, & 'static str> {
		if rom.len() < 0x150 {
			return Err("Rom is too small to contain a rom header (rom is smaller than 0x150 bytes)");
		}

		let mbc_type: MBCType = try!(CartInfo::get_type(rom[0x0147]));
		let rom_size: usize = try!(CartInfo::get_rom_size(rom[0x0148]));
		let ram_size: usize = try!(CartInfo::get_ram_size(rom[0x0149]));

		let info = CartInfo {
			title: String::from(""),	//TODO: Cart title
			sgb: rom[0x0146] == 0x03,
			cgb: rom[0x0143] & 0x80 == 0x80,
			battery: CartInfo::has_battery(rom[0x0147]),
			rtc: CartInfo::has_rtc(rom[0x0147]),
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

	fn has_rtc(cart_type: u8) -> bool {
		match cart_type {
			0x0F | 0x10 => true,
			_ => false
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
			_ => Err("Invalid value Cartridge Type in Cartridge Header at index 0x0147!"),
		}
	}

	fn get_rom_size(rom_size: u8) -> Result<usize, & 'static str> {
		match rom_size {
			0x00...0x07 => Ok(0x8000 << rom_size),
			0x52 => Ok(0x4000 * 72),
			0x53 => Ok(0x4000 * 80),
			0x54 => Ok(0x4000 * 96),
			_ => Err("Invalid value Rom Size in Cartridge Header at index 0x0148!"),
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
			_ => Err("Invalid value Ram Size in Cartridge Header at index 0x0149!"),
		}
	}
}

/// An enum containing a variant for each possible MBC implementation.
/// This exists so we can easily serialize/deserialize the mbc state, which is much more
/// difficult when the mbc exists as a trait object.
/// TODO: get rid of this when it becomes possible to easily serialize + deserialize trait objects with serde.
#[derive(Serialize, Deserialize)]
enum MBC {
	NoMBC(Box<NoMBC>),
	Mbc1(Box<MBC1>),
	Mbc3(Box<MBC3>)
}

impl MemoryBankController for MBC {
	fn read_byte_rom(&self, rom: &Box<[u8]>, rom_size: usize, offset: u16) -> u8 {
		match self {
			MBC::NoMBC(mbc) => mbc.read_byte_rom(rom, rom_size, offset),
			MBC::Mbc1(mbc) => mbc.read_byte_rom(rom, rom_size, offset),
			MBC::Mbc3(mbc) => mbc.read_byte_rom(rom, rom_size, offset)
		}
	}

	fn read_byte_ram(&self, ram: &Box<[u8]>, ram_size: usize, offset: u16) -> u8 {
		match self {
			MBC::NoMBC(mbc) => mbc.read_byte_ram(ram, ram_size, offset),
			MBC::Mbc1(mbc) => mbc.read_byte_ram(ram, ram_size, offset),
			MBC::Mbc3(mbc) => mbc.read_byte_ram(ram, ram_size, offset)
		}
	}

	fn write_byte_rom(&mut self, offset: u16, value: u8) {
		match self {
			MBC::NoMBC(mbc) => mbc.write_byte_rom(offset, value),
			MBC::Mbc1(mbc) => mbc.write_byte_rom(offset, value),
			MBC::Mbc3(mbc) => mbc.write_byte_rom(offset, value)
		}
	}

	fn write_byte_ram(&mut self, ram: &mut Box<[u8]>, ram_size: usize, offset: u16, value: u8) {
		match self {
			MBC::NoMBC(mbc) => mbc.write_byte_ram(ram, ram_size, offset, value),
			MBC::Mbc1(mbc) => mbc.write_byte_ram(ram, ram_size, offset, value),
			MBC::Mbc3(mbc) => mbc.write_byte_ram(ram, ram_size, offset, value)
		}
	}

	fn rom_bank(&self) -> usize {
		match self {
			MBC::NoMBC(mbc) => mbc.rom_bank(),
			MBC::Mbc1(mbc) => mbc.rom_bank(),
			MBC::Mbc3(mbc) => mbc.rom_bank()
		}
	}

	fn ram_bank(&self) -> usize {
		match self {
			MBC::NoMBC(mbc) => mbc.ram_bank(),
			MBC::Mbc1(mbc) => mbc.ram_bank(),
			MBC::Mbc3(mbc) => mbc.ram_bank()
		}
	}
}

#[derive(Serialize, Deserialize)]
pub struct VirtualCartridge {
	#[serde(skip)] // don't serialize the rom, force the rom to already be loaded when the save state is loaded
	pub rom: Box<[u8]>, // this needs to be public so it can be swapped to the new cart struct when a state is loaded
	ram: Box<[u8]>,
	cart_info: CartInfo,
	mbc: MBC,
}

impl VirtualCartridge {
	pub fn new(rom: Box<[u8]>, ram: Option<Box<[u8]>>) -> Result<VirtualCartridge, & 'static str> {
		let cart_info: CartInfo = try!(CartInfo::new(&rom));

		let ram = match ram {
			Some(ram) => {
				if ram.len() < cart_info.ram_size {
					// ram too small, expand it to the proper size
					// TODO: warning message?
					let mut vec = Vec::from(ram);
					vec.resize(cart_info.ram_size, 0);
					vec.into_boxed_slice()
				}
				else {
					ram
				}
			},
			None => {
				//No ram supplied, allocate some.
				let mut vec: Vec<u8> = Vec::with_capacity(cart_info.ram_size);
				vec.resize(cart_info.ram_size, 0);
				vec.into_boxed_slice()
			}
		};

		let mbc: Result<MBC, & 'static str> = match cart_info.mbc_type {
			MBCType::NONE => Ok(MBC::NoMBC(Box::new(NoMBC::new()))),
			MBCType::MBC1 => Ok(MBC::Mbc1(Box::new(MBC1::new()))),
			MBCType::MBC3 => Ok(MBC::Mbc3(Box::new(MBC3::new(cart_info.rtc)))),
			_ => {
				Err("Unimplemented MBC")	//TODO: more helpful error message
			},
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
	fn read_byte_rom(&self, offset: u16) -> u8 {
		self.mbc.read_byte_rom(&self.rom, self.cart_info.rom_size, offset)
	}

	fn read_byte_ram(&self, offset: u16) -> u8 {
		self.mbc.read_byte_ram(&self.ram, self.cart_info.ram_size, offset)
	}

	fn write_byte_rom(&mut self, offset: u16, value: u8) {
		self.mbc.write_byte_rom(offset, value);
	}

	fn write_byte_ram(&mut self, offset: u16, value: u8) {
		self.mbc.write_byte_ram(&mut self.ram, self.cart_info.ram_size, offset, value);
	}

	fn get_cart_info(&self) -> &CartInfo {
		&self.cart_info
	}

	fn rom(&self) -> &[u8] {
		&self.rom[0..0x4000]
	}

	fn rom_mut(&mut self) -> &mut[u8] {
		&mut self.rom[0..0x4000]
	}

	fn banked_rom(&self) -> &[u8] {
		let rom_bank = self.mbc.rom_bank();
		let base_address = rom_bank * ROM_BANK_SIZE;
		if base_address >= self.rom.len() {
			&self.rom[0..0]  //this doesn't exist so return an empty slice
		}
		else {
			let end_address = base_address + ROM_BANK_SIZE;
			if end_address >= self.rom.len() {
				&self.rom[base_address..self.rom.len()]
			}
			else {
				&self.rom[base_address..end_address]
			}
		}
	}

	fn banked_rom_mut(&mut self) -> &mut[u8] {
		let rom_bank = self.mbc.rom_bank();
		let rom_length = self.rom.len();
		let base_address = rom_bank * ROM_BANK_SIZE;
		if base_address >= rom_length {
			&mut self.rom[0..0]  //this doesn't exist so return an empty slice
		}
		else {
			let end_address = base_address + ROM_BANK_SIZE;
			if end_address >= rom_length {
				&mut self.rom[base_address..rom_length]
			}
			else {
				&mut self.rom[base_address..end_address]
			}
		}
	}

	fn ram(&self) -> &[u8] {
		let ram_bank = self.mbc.ram_bank();
		let base_address = ram_bank * RAM_BANK_SIZE;
		if base_address >= self.ram.len() {
			&self.ram[0..0]
		}
		else {
			let end_address = base_address + RAM_BANK_SIZE;
			if end_address >= self.ram.len() {
				&self.ram[base_address..self.ram.len()]
			}
			else {
				&self.ram[base_address..end_address]
			}
		}
	}

	fn ram_mut(&mut self) -> &mut[u8] {
		let ram_bank = self.mbc.ram_bank();
		let ram_length = self.ram.len();
		let base_address = ram_bank * RAM_BANK_SIZE;
		if base_address >= ram_length {
			&mut self.ram[0..0]
		}
		else {
			let end_address = base_address + RAM_BANK_SIZE;
			if end_address >= ram_length {
				&mut self.ram[base_address..ram_length]
			}
			else {
				&mut self.ram[base_address..end_address]
			}
		}
	}
}
