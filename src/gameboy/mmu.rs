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
			_ => panic!("gbc::read_byte_wram - invalid arguments, address must be in the range [0xC000, 0xDFFF]"),
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
			_ => panic!("gbc::read_byte_wram - invalid arguments, address must be in the range [0xC000, 0xDFFF]"),
		};
	}

	fn read_byte_io(&self, address: u16) -> u8 {
		match address {
			0xFF00 => self.joypad.read_joyp(),
			0xFF01...0xFF7F => self.io[(address - 0xFF00) as usize],
			_ => panic!("gbc::read_byte_io - invalid arguments, address must be in the range [0xFF00, 0xFF7F]."),
		}
	}

	//FF4F is the io register that controlls the vram bank on gbc
	fn write_byte_io(&mut self, address: u16, value: u8) {
		match address {
			0xFF00 => self.joypad.write_joyp(value),
			0xFF04 => self.timer.reset_div(),
			0xFF46 => self.start_oam_dma(value),
			0xFF01...0xFF45 | 0xFF47...0xFF7F => self.io[(address - 0xFF00) as usize] = value,
			_ => panic!("gbc::write_byte_io - invalid arguments, address must be in the range [0xFF00, 0xFF7F]."),
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

	fn ram(&self) -> &[u8];
	fn ram_mut(&mut self) -> &mut[u8];

	fn vram(&self) -> &[u8];
	fn vram_mut(&mut self) -> &mut[u8];

	fn wram(&self) -> &[u8];
	fn wram_mut(&mut self) -> &mut[u8];

	fn banked_wram(&self) -> &[u8];
	fn banked_wram_mut(&mut self) -> &mut[u8];

	fn oam(&self) -> &[u8];
	fn oam_mut(&mut self) -> &mut[u8];

	fn io(&self) -> &[u8];
	fn io_mut(&mut self) -> &mut [u8];

	fn hram(&self) -> &[u8];
	fn hram_mut(&mut self) -> &mut[u8];

	fn ier(&self) -> u8;
	fn ier_mut(&mut self) -> &mut u8;
}

impl Mmu for Gameboy {
	fn read_byte(&self, address: u16) -> u8 {
		match address {
			0x0000...0x7FFF => self.cart.read_byte_rom(address),
			0x8000...0x9FFF => self.vram()[(address - 0x8000) as usize],
			0xA000...0xBFFF => self.cart.read_byte_ram(address),
			0xC000...0xCFFF => self.wram()[(address - 0xC000) as usize],
			0xD000...0xDFFF => self.banked_wram()[(address - 0xD000) as usize],
			0xE000...0xF000 => self.banked_wram()[(address - 0xE000) as usize],	//Mirror of wram 0
			0xE000...0xFDFF => self.banked_wram()[(address - 0xD000) as usize],	//Mirror of wram 1
			0xFE00...0xFE9F => self.oam()[(address - 0xFE00) as usize],
			0xFF00 => self.joypad.read_joyp(),
			0xFF01...0xFF7F => self.io()[(address - 0xFF00) as usize],
			0xFF80...0xFFFE => self.hram()[(address - 0xFF80) as usize],
			0xFFFF => self.ier(),
			_ => 0xFF,
		}
	}

	fn write_byte(&mut self, address: u16, value: u8) {
		match address {
			0x0000...0x7FFF => self.cart.write_byte_rom(address, value),
			0x8000...0x9FFF => self.vram_mut()[(address - 0x8000) as usize] = value,
			0xA000...0xBFFF => self.cart.write_byte_ram(address, value),
			0xC000...0xCFFF => self.wram_mut()[(address - 0xC000) as usize] = value,
			0xD000...0xDFFF => self.banked_wram_mut()[(address - 0xD000) as usize] = value,
			0xE000...0xF000 => self.banked_wram_mut()[(address - 0xE000) as usize] = value,	//Mirror of wram 0
			0xE000...0xFDFF => self.banked_wram_mut()[(address - 0xD000) as usize] = value,	//Mirror of wram 1
			0xFE00...0xFE9F => self.oam_mut()[(address - 0xFE00) as usize] = value,
			0xFF00 => self.joypad.write_joyp(value),
			0xFF01...0xFF7F => self.io_mut()[(address - 0xFF00) as usize] = value,
			0xFF80...0xFFFE => self.hram_mut()[(address - 0xFF80) as usize] = value,
			0xFFFF => *self.ier_mut() = value,
			_ => {},
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
			0xFFFF => self.cpu.ier,
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
			0xFFFF => self.cpu.ier = value,
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

	fn ram(&self) -> &[u8] {
		self.cart.ram()
	}
	fn ram_mut(&mut self) -> &mut[u8] {
		self.cart.ram_mut()
	}

	fn vram(&self) -> &[u8] {
		self.ppu.get_vram()
	}

	fn vram_mut(&mut self) -> &mut[u8] {
		self.ppu.get_vram_mut()
	}

	fn wram(&self) -> &[u8] {
		&self.wram[0..0x1000]
	}

	fn wram_mut(&mut self) -> &mut[u8] {
		&mut self.wram[0..0x1000]
	}

	fn banked_wram(&self) -> &[u8] {
		//TODO: wram banks on cgb
		&self.wram[0x1000..0x2000]
	}

	fn banked_wram_mut(&mut self) -> &mut[u8] {
		//TODO: wram banks on cgb
		&mut self.wram[0x1000..0x2000]
	}

	fn oam(&self) -> &[u8] {
		self.ppu.get_oam()
	}

	fn oam_mut(&mut self) -> &mut[u8] {
		self.ppu.get_oam_mut()
	}

	fn io(&self) -> &[u8] {
		&self.io
	}

	fn io_mut(&mut self) -> &mut [u8] {
		&mut self.io
	}

	fn hram(&self) -> &[u8] {
		&self.cpu.hram
	}

	fn hram_mut(&mut self) -> &mut[u8] {
		&mut self.cpu.hram
	}

	fn ier(&self) -> u8 {
		self.cpu.ier
	}

	fn ier_mut(&mut self) -> &mut u8 {
		&mut self.cpu.ier
	}
}
