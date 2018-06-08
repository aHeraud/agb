use gameboy::Gameboy;

use gameboy::{WRAM_BANK_SIZE, WRAM_NUM_BANKS};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MemoryRegion {
	CartridgeRom, CartridgeRam, Vram, Wram, Oam, Unmapped, Io, Hram, Ier
}

impl MemoryRegion {
	/// Map a physical address to a memory region and offset
	pub fn map_address(address: u16) -> (MemoryRegion, u16) {
		match address {
			0x0000...0x7FFF => (MemoryRegion::CartridgeRom, address),
			0x8000...0x9FFF => (MemoryRegion::Vram, address - 0x8000),
			0xA000...0xBFFF => (MemoryRegion::CartridgeRam, address - 0xA000),
			0xC000...0xDFFF => (MemoryRegion::Wram, address - 0xC000),
			0xE000...0xFDFF => (MemoryRegion::Wram, address - 0xE000), //WRAM Mirror
			0xFE00...0xFE9F => (MemoryRegion::Oam, address - 0xFE00),
			0xFEA0...0xFEFF => (MemoryRegion::Unmapped, 0),
			0xFF00...0xFF7F => (MemoryRegion::Io, address - 0xFF00),
			0xFF80...0xFFFE => (MemoryRegion::Hram, address - 0xFF80),
			0xFFFF => (MemoryRegion::Ier, 0),
			_ => panic!("this will never happen")
		}
	}
}

trait MmuHelpers {
	fn read_byte_wram(&self, offset: u16) -> u8;
	fn write_byte_wram(&mut self, offset: u16, value: u8);
	fn read_byte_io(&self, offset: u16) -> u8;
	fn write_byte_io(&mut self, offset: u16, value: u8);
}

impl MmuHelpers for Gameboy {
	fn read_byte_wram(&self, offset: u16) -> u8 {
		let selected_wram_bank = 1;	//TODO: wram banks
		match offset {
			0x0000...0x0FFF => self.wram[offset as usize],
			0x1000...0x1FFF => self.wram[(offset - 0x1000) as usize + (WRAM_BANK_SIZE * selected_wram_bank) as usize],
			_ => panic!("read_byte_wram - invalid offset (must be in the range [0, 0x1FFF]")
		}
	}

	fn write_byte_wram(&mut self, offset: u16, value: u8) {
		let selected_wram_bank = 1;	//TODO: wram banks
		match offset {
			0x0000...0x0FFF => self.wram[offset as usize] = value,
			0x1000...0x1FFF => self.wram[(offset - 0x1000) as usize + (WRAM_BANK_SIZE * selected_wram_bank) as usize] = value,
			_ => panic!("write_byte_wram - invalid offset (must be in the range [0, 0x1FFF]")
		};
	}

	fn read_byte_io(&self, offset: u16) -> u8 {
		use gameboy::ppu::PpuIoRegister;
		use gameboy::timer::TimerRegister;

		assert!(offset <= 0x7F);
		if let Some(register) = PpuIoRegister::map_address(offset + 0xFF00) {
			self.ppu.read_io(register)
		}
		else if let Some(register) = TimerRegister::map_address(offset + 0xFF00) {
			self.timer.read_io(register)
		}
		else {
			match offset {
				0x00 => self.joypad.read_joyp(),
				0x01 => self.serial.read_sb(),
				0x02 => self.serial.read_sc(),
				0x0F => self.cpu.interrupt_flag.read(),
				0x46 => self.oam_dma_state.read_ff46(),
				_ => self.io[offset as usize]
			}
		}
	}

	fn write_byte_io(&mut self, offset: u16, value: u8) {
		use gameboy::ppu::PpuIoRegister;
		use gameboy::timer::TimerRegister;
		use gameboy::oam_dma::OamDmaController;

		assert!(offset <= 0x7F);
		if let Some(register) = PpuIoRegister::map_address(offset + 0xFF00) {
			self.ppu.write_io(register, value);
		}
		else if let Some(register) = TimerRegister::map_address(offset + 0xFF00) {
			self.timer.write_io(register, value);
		}
		else {
			match offset {
				0x00 => self.joypad.write_joyp(value),
				0x01 => self.serial.write_sb(value),
				0x02 => self.serial.write_sc(value),
				0x0F => self.cpu.interrupt_flag.write(value),
				0x46 => self.start_oam_dma(value),
				_ => self.io[offset as usize] = value
			};
		}
	}
}

pub trait Mmu {
	fn read_byte(&self, address: u16) -> u8;
	fn write_byte(&mut self, address: u16, value: u8);

	///similar to read/write, but sometimes a coprocessor has
	///exclusive access to a region of memory and the cpu
	///can't read from / write to it.
	fn read_byte_cpu(&self, address: u16) -> u8;
	fn write_byte_cpu(&mut self, address: u16, value: u8);

	fn rom(&self) -> &[u8];
	fn rom_mut(&mut self) -> &mut[u8];

	fn banked_rom(&self) -> &[u8];
	fn banked_rom_mut(&mut self) -> &mut[u8];
}

impl Mmu for Gameboy {
	fn read_byte(&self, address: u16) -> u8 {
		use self::MemoryRegion::*;
		let (region, offset) = MemoryRegion::map_address(address);
		match region {
			CartridgeRom => self.cart.read_byte_rom(offset),
			Vram => self.ppu.read_byte_vram(offset),
			CartridgeRam => self.cart.read_byte_ram(offset),
			Wram => self.read_byte_wram(offset),
			Oam => self.ppu.read_byte_oam(offset),
			Unmapped => 0xFF,
			Io => self.read_byte_io(offset),
			Hram => self.cpu.read_byte_hram(offset),
			Ier => self.cpu.interrupt_enable.read()
		}
	}

	fn write_byte(&mut self, address: u16, value: u8) {
		use self::MemoryRegion::*;
		let (region, offset) = MemoryRegion::map_address(address);
		match region {
			CartridgeRom => self.cart.write_byte_rom(offset, value),
			Vram => self.ppu.write_byte_vram(offset, value),
			CartridgeRam => self.cart.write_byte_ram(offset, value),
			Wram => self.write_byte_wram(offset, value),
			Oam => self.ppu.write_byte_oam(offset, value),
			Unmapped => {},
			Io => self.write_byte_io(offset, value),
			Hram => self.cpu.write_byte_hram(offset, value),
			Ier => self.cpu.interrupt_enable.write(value)
		};
	}

	///Read a byte at $address
	///Not all memory is readable all of the time,
	///for instance, vram and oam can't be read during certain ppu states.
	///and the cpu can't read anything other than hram and iem during a dma transfer
	fn read_byte_cpu(&self, address: u16) -> u8 {
		use self::MemoryRegion::*;
		if self.oam_dma_state.should_block_cpu_access(address) {
			return 0xFF;
		}
		else {
			let (region, offset) = MemoryRegion::map_address(address);
			match region {
				CartridgeRom => self.cart.read_byte_rom(offset),
				Vram => self.ppu.read_byte_vram(offset),
				CartridgeRam => self.cart.read_byte_ram(offset),
				Wram => self.read_byte_wram(offset),
				Oam => self.ppu.read_byte_oam(offset),
				Unmapped => 0xFF,
				Io => self.read_byte_io(offset),
				Hram => self.cpu.read_byte_hram(offset),
				Ier => self.cpu.interrupt_enable.read()
			}
		}
	}

	fn write_byte_cpu(&mut self, address: u16, value: u8) {
		use self::MemoryRegion::*;
		if self.oam_dma_state.should_block_cpu_access(address) {
			return;
		}
		let (region, offset) = MemoryRegion::map_address(address);
		match region {
			CartridgeRom => self.cart.write_byte_rom(offset, value),
			Vram => self.ppu.write_byte_vram(offset, value),
			CartridgeRam => self.cart.write_byte_ram(offset, value),
			Wram => self.write_byte_wram(offset, value),
			Oam => self.ppu.write_byte_oam(offset, value),
			Unmapped => {},
			Io => self.write_byte_io(offset, value),
			Hram => self.cpu.write_byte_hram(offset, value),
			Ier => self.cpu.interrupt_enable.write(value)
		};
	}

	fn rom(&self) -> &[u8] {
		self.cart.rom()
	}

	fn rom_mut(&mut self) -> &mut[u8] {
		self.cart.rom_mut()
	}

	fn banked_rom(&self) -> &[u8] {
		self.cart.banked_rom()
	}

	fn banked_rom_mut(&mut self) -> &mut[u8] {
		self.cart.banked_rom_mut()
	}
}
