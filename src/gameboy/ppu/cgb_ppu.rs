use super::{PPU, VRAM_BANK_SIZE, VRAM_NUM_BANKS_CGB, OAM_SIZE, WIDTH, HEIGHT};

//TODO: VRAM BANKS? HOW DOES IT WORK???

pub struct CgbPpu {
	pub vram: [u8; VRAM_BANK_SIZE * VRAM_NUM_BANKS_CGB],
	pub oam: [u8; OAM_SIZE],
	pub buffer: [u32; WIDTH * HEIGHT],
}

impl CgbPpu {
	pub fn new() -> CgbPpu {
		CgbPpu {
			vram: [0; VRAM_BANK_SIZE * VRAM_NUM_BANKS_CGB],
			oam: [0; OAM_SIZE],
			buffer: [0; WIDTH * HEIGHT],
		}
	}
}

impl PPU for CgbPpu {
	fn emulate_hardware(&mut self, io: &mut [u8]) {

	}

	fn init_io_registers(&mut self, io: &mut [u8]) {

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
		false
	}

	fn is_lcdstat_requested(&self) -> bool {
		false
	}

	fn clear_interrupts(&mut self) {

	}

	fn get_framebuffer(&self) -> &[u32] {
		&self.buffer[0..WIDTH*HEIGHT]
	}

	fn get_framebuffer_mut(&mut self) -> &mut[u32] {
		&mut self.buffer[0..WIDTH*HEIGHT]
	}

	///TODO: VRAM BANKS
	fn get_vram(&self) -> &[u8] {
		&self.vram[0..0x2000]
	}

	///TODO: VRAM BANKS
	fn get_vram_mut(&mut self) -> &mut[u8] {
		&mut self.vram[0..0x2000]
	}

	fn get_oam(&self) -> &[u8] {
		&self.oam
	}

	fn get_oam_mut(&mut self) -> &mut[u8] {
		&mut self.oam
	}
}
