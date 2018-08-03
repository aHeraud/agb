use gameboy::cpu::interrupts::InterruptLine;
use super::{PPU, VRAM_BANK_SIZE, VRAM_NUM_BANKS_CGB, OAM_SIZE, WIDTH, HEIGHT, PpuIoRegister};

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
	fn reset(&mut self) {
		//TODO
	}

	fn get_frame_counter(&self) -> usize {
		//TODO
		0
	}

	fn emulate_hardware(&mut self, _interrupt_line: &mut InterruptLine) {
		//TODO
	}

	fn read_io(&self, _reg: PpuIoRegister) -> u8 {
		panic!("unimplemented");
	}

	fn write_io(&mut self, _reg: PpuIoRegister, _value: u8) {
		panic!("unimplemented");
	}

	///Read a byte from the vram as the cpu.
	///When the ppu is in mode 3, the cpu can't access vram, so 0xFF is returned instead
	fn read_byte_vram(&self, _address: u16) -> u8 {
		unimplemented!();
	}

	fn write_byte_vram(&mut self, _address: u16, _value: u8) {
		unimplemented!();
	}

	//When the ppu is in mode 2 or 3,
	fn read_byte_oam(&self, _address: u16) -> u8 {
		unimplemented!();
	}

	fn write_byte_oam(&mut self, _address: u16, _value: u8) {
		unimplemented!();
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

	///get a bitmap with all of the tiles in vram
	///returns a bitmap of 32-bit rgba pixel values
	///TODO: implement
	fn dump_tiles(&self) -> super::Bitmap<u32> {
		let empty: Vec<u32> = Vec::new();
		super::Bitmap {
			width: 0,
			height: 0,
			data: empty.into_boxed_slice()
		}
	}

	//get a bitmap of the bg
	//TODO: implement
	fn dump_bg(&self) -> super::Bitmap<u32> {
		let empty: Vec<u32> = Vec::new();
		super::Bitmap {
			width: 0,
			height: 0,
			data: empty.into_boxed_slice()
		}
	}
}
