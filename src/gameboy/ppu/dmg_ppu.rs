#[cfg(feature = "no_std")]
use alloc::boxed::Box;

#[cfg(feature = "no_std")]
use core::num::Wrapping;

#[cfg(not(feature = "no_std"))]
use std::num::Wrapping;

use super::{PPU, VRAM_BANK_SIZE, VRAM_NUM_BANKS_DMG, OAM_SIZE, WIDTH, HEIGHT, PpuMode, Bitmap};
use super::{COINCIDENCE_INTERRUPT_ENABLE_MASK, OAM_INTERUPT_ENABLE_MASK, VBLANK_INTERRUPT_ENABLE_MASK, HBLANK_INTERRUPT_ENABLE_MASK};

/* RGBA shades for dmg */
#[allow(dead_code)]
const DEFAULT_SHADES: [u32; 4] = [ 0xE0F8D0FF, 0x88C070FF, 0x346856FF, 0x081820FF ];

const NUM_BUFFERS: usize = 2;

pub struct DmgPpu {
	pub vram: Box<[u8]>, //[u8; VRAM_BANK_SIZE * VRAM_NUM_BANKS_DMG],
	pub oam: Box<[u8]>, //[u8; OAM_SIZE],
	buffers: Box<[u32]>, //[u32; WIDTH * HEIGHT * NUM_BUFFERS],
	front_buffer_index: usize,
	back_buffer_index: usize,
	pub shades: [u32; 4],
	pub mode: PpuMode,
	pub line: u8,
	pub clock: u32,
	vblank_requested: bool,
	lcdstat_requested: bool,

	//pub lcdc: u8,	//0xFF40
	//pub stat: u8,	//0xFF41
	//pub scy: u8,	//0xFF42
	//pub scx: u8,	//0xFF43
	//pub ly: u8,		//0xFF44
	//pub lyc: u8,	//0xFF45
	//pub dma: u8,	//0xFF46: DMA transfer and start address
	//pub bgp: u8,	//0xFF47
	//pub opb0: u8,	//0xFF48
	//pub opb1: u8,	//0xFF49
	//pub wy: u8,		//0xFF4A: Window Y Position
	//pub wx: u8,		//0xFF4B: Window X Position - 7

	////FF4F: This seems to be some sort of vram bank selector?
	//pub hdma1: u8,	//0xFF51: New DMA Source, high
	//pub hdma2: u8,	//0xFF52: New DMA Source, low
	//pub hdma3: u8, 	//0xFF53: New DMA Destination, high
	//pub hdma4: u8,	//0xFF54: New DMA Destination, low
	//pub hdma5: u8,	//0xFF55: New DMA Length/Mode/Start

	//pub bgpi: u8,	//0xFF68: Background Palette Index
	//pub bgpd: u8,	//0xFF69: Background Palette Data
	//pub obpi: u8,	//0xFF6A: Sprite Pallete Index
	//pub obpd: u8,	//0xFF6B: Sprite Palette Data

}

impl DmgPpu {
	pub fn new() -> DmgPpu {
		DmgPpu {
			vram: Box::new([0; VRAM_BANK_SIZE * VRAM_NUM_BANKS_DMG]),
			oam: Box::new([0; OAM_SIZE]),
			buffers: Box::new([0; WIDTH * HEIGHT * NUM_BUFFERS]),
			front_buffer_index: 1,
			back_buffer_index: 0,
			shades: DEFAULT_SHADES,
			mode: PpuMode::HBLANK,	//TODO: what is the lcd mode at power on?
			line: 0,
			clock: 0,
			vblank_requested: false,
			lcdstat_requested: false,
		}
	}

	fn draw_scanline(&mut self, io: &[u8]) {
		let mut background: [u8; WIDTH] = [0; WIDTH];	//Background/Window
		let mut bg_sprites: [u8; WIDTH] = [0; WIDTH];	//Background sprites (layer 0, behind bg)
		let mut fg_sprites: [u8; WIDTH] = [0; WIDTH];	//Foreground Sprites

		let lcdc = io[0x40];
		let y_scroll = io[0x42];
		let x_scroll = io[0x43];
		let wx = (Wrapping(io[0x4B]) - Wrapping(7)).0;	//Window X Position
		let wy = io[0x4A];	//Window Y Position
		let bgp = io[0x47];

		self.draw_bg(&mut background, lcdc, x_scroll, y_scroll, wx, wy, bgp);
		self.draw_sprites(&mut bg_sprites, &mut fg_sprites, lcdc, io[0x48] /* OBP0 */, io[0x49] /* OBP1 */);

		//combine all 3 layers and draw the entire scanline
		for x in 0..WIDTH {
			let buffer_index: usize = (WIDTH * HEIGHT * self.back_buffer_index) + ((self.line as usize) * WIDTH) + (x as usize);
			//Clear pixel
			self.buffers[buffer_index] = self.shades[0];

			//self.buffers[buffer_index] = self.shades[bg_sprites[x] as usize];
			self.buffers[buffer_index] = self.shades[background[x] as usize];

			if background[x] & 0x0F == 0 {
				//self.buffers[buffer_index] = 0xFF;
				self.buffers[buffer_index] = self.shades[bg_sprites[x] as usize];
			}

			if fg_sprites[x] != 0 {
				self.buffers[buffer_index] = self.shades[fg_sprites[x] as usize];
			}
		}
	}

	///Returns an array of WIDTH u8's representing the shade number of each pixel of the background
	fn draw_bg(&self, background:  &mut[u8], lcdc: u8, x_scroll: u8, y_scroll: u8, wx: u8, wy: u8, bgp: u8) {
		let window_enabled: bool = (lcdc & 32 == 32) && (wy <= self.line);
		let background_enabled: bool = lcdc & 1 == 1;
		let window_tile_map: usize = match lcdc & 64 == 0 {
			true => 0x9800,
			false => 0x9C00,
		};
		let bg_tile_map: usize = match lcdc & 8 == 0 {
			true => 0x9800,
			false => 0x9C00,
		};
		let tile_data: usize = match lcdc & 16 == 0 {
			true => 0x8800,
			false => 0x8000,
		};

		for x in 0..160 {
			let y_pos: u8;
			let x_pos: u8;
			let map_address: usize;

			if window_enabled && x >= wx {
				//Use the window tilemap here
				map_address = window_tile_map + ((((x as usize) - (wx as usize)) >> 3) + ((((self.line as usize) - (wy as usize)) >> 3) << 5));

				//Window doesn't scroll
				x_pos = x;
				y_pos = self.line;
			}
			else if background_enabled {
				y_pos = (Wrapping(self.line) + Wrapping(y_scroll)).0;
				x_pos = (Wrapping(x) + Wrapping(x_scroll)).0;

				//BG is enabled
				map_address = bg_tile_map + (((x_pos as usize) >> 3) + (((y_pos as usize) >> 3) << 5));
			}
			else {
				//Neither the background or window are enabled at this pixel
				//On an actual gameboy color, background_enabled being false means that neither
				//the background or window are shown, however, on the dmg it's possible to disabled
				//the background and still draw the window.
				background[x as usize] = 0;
				continue;
			}

			let mut tile_number: usize = self.vram[map_address - 0x8000] as usize;
			if tile_data == 0x8800 {
				//Convert from signed tile numbers into unsigned offset
				tile_number = tile_number.wrapping_add(128);
			}

			//Read tile data
			let tile_address: usize = tile_data + (tile_number * 16) + (((y_pos as usize) % 8) * 2);
			let tile_2: u8 = self.vram[tile_address - 0x8000];
			let tile_1: u8 = self.vram[tile_address + 1 - 0x8000];

			//Get value for pixel (0..4)
			let value: u8 = ((tile_1 >> (7 - (x_pos % 8)) << 1) & 2) | ((tile_2 >> (7 - (x_pos % 8))) & 1);
			let shade_index: u8 = (bgp >> (value << 1)) & 3;

			background[x as usize] = shade_index;
		}
	}

	#[allow(dead_code)]
	fn draw_sprites(&self, layer0: &mut[u8], layer1: &mut[u8], lcdc: u8, obp0: u8, obp1: u8) {
		//TODO: sprite ordering / overlapping

		if lcdc & 2 == 0 {
			//Sprites are disabled
			return;
		}

		let height: u8 = match lcdc & 4 {
			0 => 8,
			_ => 16,
		};

		//There is an attribute table for 40 sprits in oam,
		//each sprite attribute table entry is 4 bytes long
		for i in 0..40 {
			let offset: usize = (i as usize) * 4;
			let sprite_y: u8 = (Wrapping(self.oam[offset]) - Wrapping(16)).0;
			let sprite_x: u8 = (Wrapping(self.oam[offset + 1]) - Wrapping(8)).0;
			let sprite_tile_number: u8 = self.oam[offset + 2];
			let sprite_flags: u8 = self.oam[offset + 3];

			if sprite_y == 0 || sprite_y >= 160 || sprite_x == 0 || sprite_x >= 168 {
				continue;	//Sprite is completely off screen
			}
			if sprite_y > self.line || sprite_y + height < self.line {
				continue;	//Sprite doens't intersect current scanline
			}

			//BEGIN DRAW_SPRITE
			let x_flip: bool = sprite_flags & 0x20 == 0x20;
			let y_flip: bool = sprite_flags & 0x40 == 0x40;

			let layer: u8 = ((!sprite_flags) & 128) >> 7;

			let mut tile_address: u16 = (sprite_tile_number as u16) * 16;
			let lower_tile_address: u16 = ((sprite_tile_number as u16) | 1) * 16;

			let palette: u8 = match sprite_flags & 0x10 {
				0x10 => obp1,
				_ => obp0,
			};

			let y: u8 = self.line - sprite_y;
			if y >= height {
				continue;	//Sprite not on this line
			}

			if y >= 8 {
				tile_address = lower_tile_address;
			}

			let data0 = match y_flip {
				true => self.vram[(tile_address + 1 + ((((height - y) as u16) % 8) * 2)) as usize],
				false => self.vram[(tile_address + 1 + (((y as u16) % 8) * 2)) as usize],
			};
			let data1: u8 = match y_flip {
				true => self.vram[(tile_address + ((((height - y) as u16) % 8) * 2)) as usize],
				false => self.vram[(tile_address + (((y as u16) % 8) * 2)) as usize],
			};

			for x in 0..8 {
				if x + sprite_x >= 160 {
					continue;	//This pixel is not on the screen
				}

				//Draw sprite
				let value: u8 = match x_flip {
					true => ((data0 >> (x % 8) << 1) & 2) | ((data1 >> (x % 8)) & 1),
					false => ((data0 >> (7 - (x % 8)) << 1) & 2) | ((data1 >> (7 - (x % 8))) & 1),
				};

				let shade = (palette >> (value << 1)) & 3;

				if layer == 0 {
					layer0[(x + sprite_x) as usize] = shade;
				}
				else {
					if value != 0 {
						layer1[(x + sprite_x) as usize] = shade;
					}
				}
			}
			//END DRAW_SPRITE
		}
	}

	///get a raw tile (no coloring, only 2 bit value for each pixel)
	///returns a tuple with the values (width, size, tile).
	fn get_tile_raw(&self, tile_number: usize) -> Bitmap<u8> {
		const TILE_WIDTH: usize = 8;
		const TILE_HEIGHT: usize = 8;
		let tile_address = tile_number * 16;

		let mut tile_data = {
			let mut buf = Vec::with_capacity(TILE_WIDTH * TILE_HEIGHT);
			buf.resize(TILE_WIDTH * TILE_HEIGHT, 0);
			buf.into_boxed_slice()
		};

		for y in 0..TILE_HEIGHT {
			let tile_2: u8 = self.vram[tile_address + (y * 2)];
			let tile_1: u8 = self.vram[tile_address + (y * 2) + 1];
			for x in 0..TILE_WIDTH {
				let value: u8 = ((tile_1 >> (7 - x) << 1) & 2) | ((tile_2 >> (7 - x)) & 1);
				tile_data[(y * TILE_WIDTH) + x] = value;
			}
		}

		Bitmap {
			width: TILE_WIDTH,
			height: TILE_HEIGHT,
			data: tile_data,
		}
	}

	///gets the bitmap of a tile, colored according to the pallete passed in
	fn get_tile(&self, tile_number: usize, bgp: u8) -> Bitmap<u32> {
		let raw = self.get_tile_raw(tile_number);
		let mut data = {
			let mut buf = Vec::with_capacity(raw.width * raw.height);
			buf.resize(raw.width * raw.height, 0);
			buf.into_boxed_slice()
		};

		for (index, value) in raw.data.iter().enumerate() {
			let shade = (bgp >> ((*value as usize) << 1)) & 3;
			data[index] = self.shades[shade as usize];
		}

		Bitmap {
			width: raw.width,
			height: raw.height,
			data: data
		}
	}
}

impl PPU for DmgPpu {
	fn reset(&mut self) {
		self.front_buffer_index = 1;
		self.back_buffer_index = 0;
		self.mode = PpuMode::HBLANK;
		self.line = 0;
		self.clock = 0;
		self.vblank_requested = false;
		self.lcdstat_requested = false;
	}

	fn emulate_hardware(&mut self, io: &mut [u8]) {
		let lcdc: u8 = io[0x40];
		if lcdc & 128 == 0 {
			//Bit 7 of LCDC is zero, so lcd is disabled
			return;
		}

		let mut stat: u8 = io[0x41];
		stat = stat & 0b01111000; //Clear coincidence flag and mode

		self.clock += 4;
		match self.mode {
			PpuMode::HBLANK => {
				if self.clock > 228 {
					self.line += 1;
					self.clock = 0;

					if self.line < 144 {
						self.mode = PpuMode::SEARCH_OAM;

						//Request a lcdstat interrupt if the oam interupt bit is enabled in stat
						if stat & OAM_INTERUPT_ENABLE_MASK == OAM_INTERUPT_ENABLE_MASK {
							//io[0x0F] |= LCDSTAT_INTERRUPT_BIT;
							self.lcdstat_requested = true;
						}
					}

					else {
						//Reached the end of the screen, enter vblank
						self.mode = PpuMode::VBLANK;

						//Request a vlbank interrupt
						//io[0x0F] |= VBLANK_INTERRUPT_BIT;
						self.vblank_requested = true;

						//Additionally, if vblank is enabled in stat, request an lcdstat interrupt
						if stat & VBLANK_INTERRUPT_ENABLE_MASK == VBLANK_INTERRUPT_ENABLE_MASK {
							//io[0x0F] |= LCDSTAT_INTERRUPT_BIT;
							self.lcdstat_requested = true;
						}

						//Swap buffers
						let temp = self.front_buffer_index;
						self.front_buffer_index = self.back_buffer_index;
						self.back_buffer_index = temp;
					}
				}
			},
			PpuMode::VBLANK => {
				if self.clock > 456 { //ly increments 10 times during vblank, for a total of 4560 clocks
					self.line += 1;
					self.clock = 0;
					if self.line >= 153 {
						self.line = 0;
						self.mode = PpuMode::SEARCH_OAM;

						//Request a lcdstat interrupt if the oam interupt bit is enabled in stat
						if stat & OAM_INTERUPT_ENABLE_MASK == OAM_INTERUPT_ENABLE_MASK {
							//io[0x0F] |= LCDSTAT_INTERRUPT_BIT;
							self.lcdstat_requested = true;
						}
					}
				}
			},
			PpuMode::SEARCH_OAM => {
				if self.clock > 76 {
					self.clock = 0;
					self.mode = PpuMode::TRANSFER_TO_LCD;
				}
			},
			PpuMode::TRANSFER_TO_LCD => {
				if self.clock > 152 {
					self.mode = PpuMode::HBLANK;
					self.clock = 0;

					//Request lcd stat interrupt if hblank interrupt is enabled in stat
					if stat & HBLANK_INTERRUPT_ENABLE_MASK == HBLANK_INTERRUPT_ENABLE_MASK {
						//io[0x0F] |= LCDSTAT_INTERRUPT_BIT;
						self.lcdstat_requested = true;
					}

					//draw the scanline
					self.draw_scanline(io);
				}
			},
		};

		//Write back io registers

		//Check for coincidence interrupt
		let lyc: u8 = io[0x45];
		if lyc == self.line {
			//Set coincidence flag, and if coincidence interrupts are enabled, request a lcdstat interrupt
			if stat & COINCIDENCE_INTERRUPT_ENABLE_MASK == COINCIDENCE_INTERRUPT_ENABLE_MASK {
				//io[0x0F] |= LCDSTAT_INTERRUPT_BIT;
				self.lcdstat_requested = true;
			}
			stat |= 4;
		}

		let mode: u8 = match self.mode {
			PpuMode::HBLANK => 0,
			PpuMode::VBLANK => 1,
			PpuMode::SEARCH_OAM => 2,
			PpuMode::TRANSFER_TO_LCD => 3,
		};

		stat |= mode | 0x80;	//High bit of stat always set

		io[0x41] = stat;

		io[0x44] = self.line;
	}

	fn init_io_registers(&mut self, io: &mut [u8]) {
		io[0x40] = 0x91;	//LCDC
		io[0x41] = 0x85;	//STAT
	}

	///Read a byte from the vram as the cpu.
	///When the ppu is in mode 3, the cpu can't access vram, so 0xFF is returned instead
	fn read_byte_vram(&self, io: &[u8], address: u16) -> u8 {
		match address {
			0x8000...0x9FFF => {
				let mode: u8 = io[0x41] & 3;
				if mode == 3 {
					//Ppu is in mode 3 (transferring data to lcd driver)
					//and the cpu can't access vram
					return 0xFF;
				}
				else {
					return self.vram[(address - 0x8000) as usize];
				}
			}
			_ => panic!("ppu::read_byte_vram - invalid arguments, address must be in the range [0x8000, 0x9FFF]."),
		}
	}

	fn write_byte_vram(&mut self, io: &[u8], address: u16, value: u8) {
		match address {
			0x8000...0x9FFF => {
				let mode: u8 = io[0x41] & 3;
				if mode != 3 {
					//Not in mode 3, cpu can write to vram
					self.vram[(address - 0x8000) as usize] = value;
				}
			}
			_ => panic!("ppu::read_byte_vram - invalid arguments, address must be in the range [0x8000, 0x9FFF]."),
		};
	}

	//When the ppu is in mode 2 or 3,
	fn read_byte_oam(&self, io: &[u8], address: u16) -> u8 {
		match address {
			0xFE00...0xFE9F => {
				let mode: u8 = io[0x41] & 3;
				if mode > 1 {
					//ppu is in mode 2 or 3, cpu can't access oam
					return 0xFF;
				}
				else {
					return self.oam[(address - 0xFE00) as usize];
				}
			}
			_ => panic!("ppu::read_byte_oam - invalid arguments, address must be in the range [0xFE00, 0xFE9F]."),
		}
	}

	fn write_byte_oam(&mut self, io: &[u8], address: u16, value: u8) {
		match address {
			0xFE00...0xFE9F => {
				let mode: u8 = io[0x41] & 3;
				if mode < 2 {
					self.oam[(address - 0xFE00) as usize] = value;
				}
			}
			_ => panic!("ppu::read_byte_oam - invalid arguments, address must be in the range [0xFE00, 0xFE9F]."),
		};
	}

	fn is_vblank_requested(&self) -> bool {
		self.vblank_requested
	}

	fn is_lcdstat_requested(&self) -> bool {
		self.lcdstat_requested
	}

	fn clear_interrupts(&mut self) {
		self.vblank_requested = false;
		self.lcdstat_requested = false;
	}

	fn get_framebuffer(&self) -> &[u32] {
		let buffer_size: usize = WIDTH * HEIGHT;
		let buffer_start: usize = buffer_size * self.front_buffer_index;
		let buffer_end = buffer_start + buffer_size;
		&self.buffers[buffer_start .. buffer_end]
	}

	fn get_framebuffer_mut(&mut self) -> &mut[u32] {
		let buffer_size: usize = WIDTH * HEIGHT;
		let buffer_start: usize = buffer_size * self.front_buffer_index;
		let buffer_end = buffer_start + buffer_size;
		&mut self.buffers[buffer_start .. buffer_end]
	}

	fn get_vram(&self) -> &[u8] {
		&self.vram
	}

	fn get_vram_mut(&mut self) -> &mut[u8] {
		&mut self.vram
	}

	fn get_oam(&self) -> &[u8] {
		&self.oam
	}

	fn get_oam_mut(&mut self) -> &mut[u8] {
		&mut self.oam
	}

	///get a bitmap with all of the tiles in vram
	///returns (width, height, bitmap_data)
	fn dump_tiles(&self) -> Bitmap<u32> {
		use std::mem;
		const NUM_TILES: usize = 384;
		const TILE_WIDTH: usize = 8;
		const TILE_HEIGHT: usize = 8;
		const COLS: usize = 16;
		const ROWS: usize = 24;

		let mut tiles: [Bitmap<u8>; NUM_TILES] = unsafe { mem::zeroed() };
		for i in 0..tiles.len() {
			tiles[i] = self.get_tile_raw(i);
		}

		let mut bitmap = {
			let mut buf = Vec::with_capacity(8 * 8 * NUM_TILES);
			buf.resize(8 * 8 * NUM_TILES, 0);
			buf.into_boxed_slice()
		};

		for (index, tile) in tiles.iter().enumerate() {
			let row: usize = index / COLS;
			let col: usize = index % COLS;
			let index: usize = (row * COLS * TILE_WIDTH * TILE_HEIGHT) + (col * TILE_WIDTH);
			for y in 0..tile.height {
				for x in 0..tile.width {
					let offset: usize = (y * TILE_WIDTH * COLS) + x;
					bitmap[index + offset] = self.shades[tile.data[(y * tile.width) + x] as usize];
				}
			}
		}

		Bitmap {
			width: TILE_WIDTH * COLS,
			height: TILE_HEIGHT * ROWS,
			data: bitmap,
		}
	}

	fn dump_bg(&self, io: &[u8]) -> Bitmap<u32> {
		const ROWS: usize = 32;
		const COLS: usize = 32;
		const TILE_WIDTH: usize = 8;
		const TILE_HEIGHT: usize = 8;

		let mut data = {
			let mut buf = Vec::with_capacity(ROWS * COLS * TILE_WIDTH * TILE_HEIGHT);
			buf.resize(ROWS * COLS * TILE_WIDTH * TILE_HEIGHT, 0);
			buf.into_boxed_slice()
		};
		let bgp = io[0x47];
		let lcdc = io[0x40];
		let tile_map_address = match lcdc & 8 {
			0 => 0x9800,
			_ => 0x9c00,
		};
		let tile_data_address = match lcdc & 16 {
			0 => 0x8800,
			_ => 0x8000,
		};

		//draw bg tiles
		for row in 0..ROWS {
			for col in 0..COLS {
				let mut tile_number = self.vram[tile_map_address - 0x8000 + (row * COLS) + col] as usize;
				if tile_data_address == 0x8800 {
					//signed tile numbers, tile # 0 is at 0x9000, -192 is at 0x8800
					//convert to unsigned, where tile 0 is at -x8800
					let signed_tile_number = tile_number as i8;
					tile_number = ((signed_tile_number as isize) + 128) as usize;

					//the tile at address 0x8800 is actually tile 128, not 0
					tile_number += 128;
				}
				let tile = self.get_tile(tile_number, bgp);
				let bitmap_index: usize = (row * TILE_WIDTH * COLS * TILE_HEIGHT) + (col * TILE_WIDTH);
				for y in 0..tile.height {
					for x in 0..tile.width {
						let offset = (y * COLS * TILE_WIDTH) + x;
						data[bitmap_index + offset] = tile.data[(y * tile.width) + x];
					}
				}
			}
		}

		//TODO: draw window on top
		let window_enabled = match lcdc & 32 {
			0 => false,
			_ => true,
		};
		let window_tile_map_address = match lcdc & 64 {
			0 => 0x9800,
			_ => 0x9C00,
		};
		let wx = io[0x4A];
		let wy = io[0x4B] + 7;


		Bitmap {
			width: COLS * TILE_WIDTH,
			height: ROWS * TILE_HEIGHT,
			data: data,
		}

	}
}
