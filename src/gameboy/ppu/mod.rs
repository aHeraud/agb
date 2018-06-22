use gameboy::cpu::interrupts::InterruptLine;

pub mod dmg_ppu;
pub mod cgb_ppu;

pub const VRAM_BANK_SIZE: usize = 8192;
pub const VRAM_NUM_BANKS_CGB: usize = 2;
pub const VRAM_NUM_BANKS_DMG: usize = 1;
pub const OAM_SIZE: usize = 160;

pub const COINCIDENCE_INTERRUPT_ENABLE_MASK: u8 = 64;
pub const OAM_INTERUPT_ENABLE_MASK: u8 = 32;
pub const VBLANK_INTERRUPT_ENABLE_MASK: u8 = 16;
pub const HBLANK_INTERRUPT_ENABLE_MASK: u8 = 8;

pub const VBLANK_INTERRUPT_BIT: u8 = 1;
pub const LCDSTAT_INTERRUPT_BIT: u8 = 2;

//bitmap\
pub struct Bitmap<T> {
	pub width: usize,
	pub height: usize,
	pub data: Box<[T]>,
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PpuMode {
	HBLANK = 0, VBLANK = 1, SEARCH_OAM = 2, TRANSFER_TO_LCD = 3
}

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;

/// Tile Addressing Modes
/// Selected through LCDC Bit 4: 0 = 0x8800, 1 = 0x8000.
/// Mode 0: Tile 0 is located at 0x8800, and tile numbers are interpreted as signed bytes (tiles 0x80-0xFF are located below 0x8800)
/// Mode 1: Tile 0 is located at 0x8000, and tile numbers are interpreted as unsigned bytes.
enum TileDataAddress {
	TileData8800h, TileData8000h
}

impl TileDataAddress {
	pub fn from_lcdc(lcdc: u8) -> TileDataAddress {
		if lcdc & 16 == 0 {
			TileDataAddress::TileData8800h
		}
		else {
			TileDataAddress::TileData8000h
		}
	}

	pub fn address(&self) -> u16 {
		match self {
			&TileDataAddress::TileData8000h => 0x8000,
			&TileDataAddress::TileData8800h => 0x8800
		}
	}

	pub fn get_tile_address(&self, tile_number: u8) -> u16 {
		match self {
			&TileDataAddress::TileData8000h => {
				self.address() + ((tile_number as u16) * 16)
			},
			&TileDataAddress::TileData8800h => {
				let offset = (tile_number as i8 as i16 + 128) as u16 * 16;
				self.address() + offset
			}
		}
	}
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Palette {
	Bgp, Obp0, Obp1
}

#[repr(packed)]
struct Sprite {
	y: u8, //ypos (minus 16)
	x: u8, //xpos (minus 8)
	tile_number: u8, //unsigned tile nubmer. sprite tiles are located in 0x8000 - 0x8FFF
	attributes: u8
}

impl Sprite {
	pub fn y_pos(&self) -> isize {
		(self.y as isize) - 16
	}

	pub fn x_pos(&self) -> isize {
		(self.x as isize) - 8
	}
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PpuIoRegister {
	Lcdc, Stat, Scy, Scx, Ly, Lyc, Wy, Wx, Bgp, Obp0, Obp1, Bgpi, Bgpd, Obpi, Obpd, Vbk
}

impl PpuIoRegister {
	pub fn address(&self) -> u16 {
		use self::PpuIoRegister::*;
		match *self {
			Lcdc => 0xFF40,
			Stat => 0xFF41,
			Scy => 0xFF42,
			Scx => 0xFF43,
			Ly => 0xFF44,
			Lyc => 0xFF45,
			Bgp => 0xFF47,
			Obp0 => 0xFF48,
			Obp1 => 0xFF49,
			Wy => 0xFF4A,
			Wx => 0xFF4B,
			Vbk => 0xFF4F,
			Bgpi => 0xFF68,
			Bgpd => 0xFF69,
			Obpi => 0xFF6A,
			Obpd => 0xFF6B
		}
	}

	pub fn map_address(address: u16) -> Option<PpuIoRegister> {
		use self::PpuIoRegister::*;
		match address {
			0xFF40 => Some(Lcdc),
			0xFF41 => Some(Stat),
			0xFF42 => Some(Scy),
			0xFF43 => Some(Scx),
			0xFF44 => Some(Ly),
			0xFF45 => Some(Lyc),
			0xFF47 => Some(Bgp),
			0xFF48 => Some(Obp0),
			0xFF49 => Some(Obp1),
			0xFF4A => Some(Wy),
			0xFF4B => Some(Wx),
			0xFF4F => Some(Vbk),
			0xFF68 => Some(Bgpi),
			0xFF69 => Some(Bgpd),
			0xFF6A => Some(Obpi),
			0xFF6B => Some(Obpd),
			_ => None
		}
	}
}

pub trait PPU {
	///Read a byte from the vram
	///vram is unreadable during certain ppu modes, so 0xFF is returned instead
	///this isn't meant to be used by the ppu itself, because it has direct access to the vram
	fn read_byte_vram(&self, offset: u16) -> u8;

	///Write a byte to the vram
	fn write_byte_vram(&mut self, offset: u16, value: u8);

	///Read a byte from the oam
	fn read_byte_oam(&self, offset: u16) -> u8;

	///Write a byte to the oam
	fn write_byte_oam(&mut self, offset: u16, value: u8);

	/// Read one of the ppu's memory mapped io registers
	fn read_io(&self, reg: PpuIoRegister) -> u8;

	/// Write to one of the ppu's memory mapped io registers
	fn write_io(&mut self, reg: PpuIoRegister, value: u8);

	///Emulate the ppu for 1 M-Cycle (4 Clocks)
	fn emulate_hardware(&mut self, interrupt_line: &mut InterruptLine);

	fn reset(&mut self);

	///Gets a pointer to the framebuffer, which is an 160*144 RGBA array of u32's that represents
	///the contents of the gameboys screen
	fn get_framebuffer(&self) -> &[u32];
	fn get_framebuffer_mut(&mut self) -> &mut[u32];

	//Debugger functions
	fn get_vram(&self) -> &[u8];
	fn get_vram_mut(&mut self) -> &mut[u8];
	fn get_oam(&self) -> &[u8];
	fn get_oam_mut(&mut self) -> &mut[u8];
	fn dump_tiles(&self) -> Bitmap<u32>;
	fn dump_bg(&self) -> Bitmap<u32>;
}
