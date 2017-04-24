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

#[allow(non_camel_case_types)]
pub enum PpuMode {
	HBLANK, VBLANK, SEARCH_OAM, TRANSFER_TO_LCD
}

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;

pub trait PPU {
	fn init_io_registers(&mut self, io: &mut [u8]);

	///Read a byte from the vram
	///vram is unreadable during certain ppu modes, so 0xFF is returned instead
	///this isn't meant to be used by the ppu itself, because it has direct access to the vram
	fn read_byte_vram(&self, io: &[u8], address: u16) -> u8;

	///Write a byte to the vram
	fn write_byte_vram(&mut self, io: &[u8], address: u16, value: u8);

	///Read a byte from the oam
	fn read_byte_oam(&self, io: &[u8], address: u16) -> u8;

	///Write a byte to the oam
	fn write_byte_oam(&mut self, io: &[u8], address: u16, value: u8);

	///Emulate the ppu for 1 M-Cycle (4 Clocks)
	fn emulate_hardware(&mut self, io: &mut [u8]);

	fn is_vblank_requested(&self) -> bool;
	fn is_lcdstat_requested(&self) -> bool;
	fn clear_interrupts(&mut self);

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
}
