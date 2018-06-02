use gameboy::Gameboy;

use gameboy::{WRAM_BANK_SIZE, WRAM_NUM_BANKS};

trait MmuHelpers {
	fn read_byte_wram(&self, address: u16) -> u8;
	fn write_byte_wram(&mut self, address: u16, value: u8);
	fn read_byte_io(&self, address: u16) -> u8;
	fn write_byte_io(&mut self, address: u16, value: u8);
}

impl MmuHelpers for Gameboy {
	fn read_byte_wram(&self, address: u16) -> u8 {
		let selected_wram_bank = 1;	//TODO: wram banks
		match address {
			0xC000...0xCFFF => self.wram[(address - 0xC000) as usize],
			0xD000...0xDFFF => self.wram[(address - 0xC000) as usize + (WRAM_BANK_SIZE * selected_wram_bank) as usize],
			_ => panic!("read_byte_wram - invalid arguments, address must be in the range [0xC000, 0xDFFF]"),
		}
	}

	fn write_byte_wram(&mut self, address: u16, value: u8) {
		if self.oam_dma_active {
			return;
		}
		let selected_wram_bank = 1;	//TODO: wram banks
		match address {
			0xC000...0xCFFF => self.wram[(address - 0xC000) as usize] = value,
			0xD000...0xDFFF => self.wram[(address - 0xC000) as usize + (WRAM_BANK_SIZE * selected_wram_bank) as usize] = value,
			_ => panic!("read_byte_wram - invalid arguments, address must be in the range [0xC000, 0xDFFF]"),
		};
	}

	fn read_byte_io(&self, address: u16) -> u8 {
		match address {
			0xFF00 => self.joypad.read_joyp(),
			0xFF0F => self.cpu.interrupt_flag.read(),
			0xFF01...0xFF0E | 0xFF10...0xFF7F => self.io[(address - 0xFF00) as usize],
			_ => panic!("read_byte_io - invalid arguments, address must be in the range [0xFF00, 0xFF7F]."),
		}
	}

	//FF4F is the io register that controlls the vram bank on gbc
	fn write_byte_io(&mut self, address: u16, value: u8) {
		match address {
			0xFF00 => self.joypad.write_joyp(value),
			0xFF04 => self.timer.reset_div(),
			0xFF0F => self.cpu.interrupt_flag.write(value),
			0xFF46 => self.start_oam_dma(value),
			0xFF01...0xFF03 | 0xFF05...0xFF0E | 0xFF10...0xFF45 | 0xFF47...0xFF7F => self.io[(address - 0xFF00) as usize] = value,
			_ => panic!("write_byte_io - invalid arguments, address must be in the range [0xFF00, 0xFF7F]."),
		};
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
		match address {
			0x0000...0x7FFF => self.cart.read_byte_rom(address),
			0x8000...0x9FFF => self.ppu.read_byte_vram(&self.io, address),
			0xA000...0xBFFF => self.cart.read_byte_ram(address),
			0xC000...0xDFFF => self.read_byte_wram(address),
			0xE000...0xFDFF => self.read_byte_wram(address - 0x2000),	//Mirror of wram
			0xFE00...0xFE9F => self.ppu.read_byte_oam(&self.io, address),
			0xFF00...0xFF7F => self.read_byte_io(address),
			0xFF80...0xFFFE => self.cpu.read_byte_hram(address),
			0xFFFF => self.cpu.interrupt_enable.read(),
			_ => 0xFF,
		}
	}

	fn write_byte(&mut self, address: u16, value: u8) {
		match address {
			0x0000...0x7FFF => self.cart.write_byte_rom(address, value),
			0x8000...0x9FFF => self.ppu.write_byte_vram(&self.io, address, value),
			0xA000...0xBFFF => self.cart.write_byte_ram(address, value),
			0xC000...0xDFFF => self.write_byte_wram(address, value),
			0xE000...0xFDFF => self.write_byte_wram(address - 0x2000, value),	//Mirror of wram
			0xFE00...0xFE9F => self.ppu.write_byte_oam(&self.io, address, value),
			0xFF00...0xFF7F => self.write_byte_io(address, value),
			0xFF80...0xFFFE => self.cpu.write_byte_hram(address, value),
			0xFFFF => self.cpu.interrupt_enable.write(value),
			_ => return,
		};
	}

	///Read a byte at $address
	///Not all memory is readable all of the time,
	///for instance, vram and oam can't be read during certain ppu states.
	///and the cpu can't read anything other than hram and iem during a dma transfer
	fn read_byte_cpu(&self, address: u16) -> u8 {
		if self.oam_dma_active && address < 0xFF80 {
			//cpu can't access anything other than hram when oam dma is active
			return 0xFF;
		}
		match address {
			0x0000...0x7FFF => self.cart.read_byte_rom(address),
			0x8000...0x9FFF => self.ppu.read_byte_vram(&self.io, address),
			0xA000...0xBFFF => self.cart.read_byte_ram(address),
			0xC000...0xDFFF => self.read_byte_wram(address),
			0xE000...0xFDFF => self.read_byte_wram(address - 0x2000),	//Mirror of wram
			0xFE00...0xFE9F => self.ppu.read_byte_oam(&self.io, address),
			0xFF00...0xFF7F => self.read_byte_io(address),
			0xFF80...0xFFFE => self.cpu.read_byte_hram(address),
			0xFFFF => self.cpu.interrupt_enable.read(),
			_ => 0xFF,
		}
	}

	fn write_byte_cpu(&mut self, address: u16, value: u8) {
		if self.oam_dma_active && address < 0xFF80 {
			//cpu can't access anything other than hram when oam dma is active
			return;
		}
		match address {
			0x0000...0x7FFF => self.cart.write_byte_rom(address, value),
			0x8000...0x9FFF => self.ppu.write_byte_vram(&self.io, address, value),
			0xA000...0xBFFF => self.cart.write_byte_ram(address, value),
			0xC000...0xDFFF => self.write_byte_wram(address, value),
			0xE000...0xFDFF => self.write_byte_wram(address - 0x2000, value),	//Mirror of wram
			0xFE00...0xFE9F => self.ppu.write_byte_oam(&self.io, address, value),
			0xFF00...0xFF7F => self.write_byte_io(address, value),
			0xFF80...0xFFFE => self.cpu.write_byte_hram(address, value),
			0xFFFF => self.cpu.interrupt_enable.write(value),
			_ => return,
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
