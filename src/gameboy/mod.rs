pub mod apu;
pub mod cpu;
pub mod ppu;
pub mod cartridge;
pub mod instructions;
pub mod timer;
pub mod joypad;
mod util;

//Import Box since we aren't linking with the standard library
use alloc::boxed::Box;

use gameboy::apu::APU;
use gameboy::cpu::{ RegisterPair, CPU };
use gameboy::ppu::PPU;
use gameboy::ppu::dmg_ppu::DmgPpu;
use gameboy::ppu::cgb_ppu::CgbPpu;
use gameboy::timer::Timer;
use gameboy::cartridge::{Cartridge, VirtualCartridge};
use gameboy::joypad::Joypad;
pub use gameboy::joypad::Key;

const IO_SIZE: usize = 128;

const WRAM_BANK_SIZE: usize = 4096;
const WRAM_NUM_BANKS: usize = 8;

#[derive(Debug)]
pub enum Interrupt {
	VBlank, LcdStat, Timer, Serial, Joypad
}

#[derive(Debug)]
pub enum Mode {
	DMG, CGB,
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub enum Register {
	B, C, D, E, H, L, AT_HL, A
}

//Interrupt bit masks
const VBLANK_MASK:  u8 = 1 << 0;
const LCDSTAT_MASK: u8 = 1 << 1;
const TIMER_MASK:   u8 = 1 << 2;
const SERIAL_MASK:  u8 = 1 << 3;
const JOYPAD_MASK:  u8 = 1 << 4;

//Interrupt handler addresses
const VBLANK_ADDR:  u16 = 0x0040;
const LCDSTAT_ADDR: u16 = 0x0048;
const TIMER_ADDR:   u16 = 0x0050;
const SERIAL_ADDR:  u16 = 0x0058;
const JOYPAD_ADDR:  u16 = 0x0060;

pub struct Gameboy {
	pub cpu: CPU,
	pub timer: Timer,
	pub ppu: Box<PPU>,
	pub apu: APU,
	pub joypad: Joypad,
	pub cart: Box<Cartridge>,
	pub io: Box<[u8]>,
	pub wram: Box<[u8]>,
	pub mode: Mode,

	pub oam_dma_active: bool,
	pub oam_dma_start_address: u16,
	pub oam_dma_current_offset: u16,
}

#[allow(dead_code)]
impl Gameboy {
	pub fn new(rom: Box<[u8]>, ram: Option<Box<[u8]>>) -> Result<Gameboy, & 'static str> {
		let cart = Box::new(try!(VirtualCartridge::new(rom, ram)));
		let mode: Mode = match cart.get_cart_info().cgb {
			true => Mode::CGB,
			false => Mode::DMG,
		};
		let ppu: Box<PPU> = match mode {
			_ => Box::new(DmgPpu::new()),
		};

		let gameboy = Gameboy {
			cpu: CPU::new(),
			timer: Timer::new(),
			ppu: ppu,
			apu: APU::new(),
			joypad: Joypad::new(),
			cart: cart,
			io: Box::new([0; IO_SIZE]),
			wram: Box::new([0; WRAM_BANK_SIZE * WRAM_NUM_BANKS]),
			mode: mode,

			oam_dma_active: false,
			oam_dma_start_address: 0,
			oam_dma_current_offset: 0,
		};

		Ok(gameboy)
	}

	pub fn step_frame(&mut self) {
		//A complete frame occurs every ~70224 clock cycles (140448 in gbc double speed mode)
		const FRAME_CLOCKS: usize = 70224;
		let mut counter: usize = 0;

		while counter < FRAME_CLOCKS {
			self.step();
			if self.cpu.double_speed_mode {
				self.step();
			}
			counter += 4;
		}
	}

	fn start_oam_dma(&mut self, value: u8) {
		self.oam_dma_active = true;
		self.oam_dma_start_address = (value as u16) << 8;
		self.oam_dma_current_offset = 0;
	}

	fn service_oam_dma(&mut self) {
		//If oam dma is running, copy some data
		//oam dma supposedly takes 671 cycles
		//Copies 60 bytes
		//TODO: realistic oam transfers

		for i in 0..100 {
			let value: u8 = self.read_byte(self.oam_dma_start_address + i);
			self.ppu.write_byte_oam(&self.io, 0xFE00 + i, value);
		}
		self.oam_dma_active = false;
	}

	pub fn emulate_hardware(&mut self) {
		if self.oam_dma_active {
			self.service_oam_dma();
		}

		self.timer.emulate_hardware(&mut self.io);
		if self.timer.int_requested {
			self.request_interrupt(Interrupt::Timer);
			self.timer.int_requested = false;
		}

		self.ppu.emulate_hardware(&mut self.io);
		if self.ppu.is_vblank_requested() {
			self.request_interrupt(Interrupt::VBlank);
		}
		if self.ppu.is_lcdstat_requested() {
			self.request_interrupt(Interrupt::LcdStat);
		}
		self.ppu.clear_interrupts();

		self.apu.emulate_hardware();
	}

	///Read a byte at $address
	///Not all memory is readable all of the time,
	///for instance, vram and oam can't be read during certain ppu states.
	///and the cpu can't read anything other than hram and iem during a dma transfer
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
			0xFFFF => self.cpu.ier,
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
			0xFFFF => self.cpu.ier = value,
			_ => return,
		};
	}

	pub fn read_byte_wram(&self, address: u16) -> u8 {
		let selected_wram_bank = 1;	//TODO: wram banks

		match address {
			0xC000...0xCFFF => self.wram[(address - 0xC000) as usize],
			0xD000...0xDFFF => self.wram[(address - 0xC000) as usize + (WRAM_BANK_SIZE * selected_wram_bank) as usize],
			_ => panic!("gbc::read_byte_wram - invalid arguments, address must be in the range [0xC000, 0xDFFF]"),
		}
	}

	pub fn write_byte_wram(&mut self, address: u16, value: u8) {
		let selected_wram_bank = 1;	//TODO: wram banks

		match address {
			0xC000...0xCFFF => self.wram[(address - 0xC000) as usize] = value,
			0xD000...0xDFFF => self.wram[(address - 0xC000) as usize + (WRAM_BANK_SIZE * selected_wram_bank) as usize] = value,
			_ => panic!("gbc::read_byte_wram - invalid arguments, address must be in the range [0xC000, 0xDFFF]"),
		};
	}

	pub fn read_byte_io(&self, address: u16) -> u8 {
		match address {
			0xFF00 => self.joypad.read_joyp(),
			0xFF01...0xFF7F => self.io[(address - 0xFF00) as usize],
			_ => panic!("gbc::read_byte_io - invalid arguments, address must be in the range [0xFF00, 0xFF7F]."),
		}
	}

	//FF4F is the io register that controlls the vram bank on gbc
	pub fn write_byte_io(&mut self, address: u16, value: u8) {
		match address {
			0xFF00 => self.joypad.write_joyp(value),
			0xFF46 => self.start_oam_dma(value),
			0xFF01...0xFF45 | 0xFF47...0xFF7F => self.io[(address - 0xFF00) as usize] = value,
			_ => panic!("gbc::write_byte_io - invalid arguments, address must be in the range [0xFF00, 0xFF7F]."),
		};
	}

	fn request_interrupt(&mut self, req_int: Interrupt) {
		let mask = match req_int {
			Interrupt::VBlank => VBLANK_MASK,
			Interrupt::LcdStat => LCDSTAT_MASK,
			Interrupt::Timer =>  TIMER_MASK,
			Interrupt::Serial => SERIAL_MASK,
			Interrupt::Joypad => JOYPAD_MASK,
		};

		//Set the bit corresponding to the requested interrupt in IF (FF0F)
		self.io[0x0F] |= mask;

		//Interrupts wake cpu
		self.cpu.halt = false;
	}

	///Handles interupts
	///If an interrupt is requested in IF (FF0F), and it is enabled in IE (FFFF), and
	///interrupts are enabled by IME (cpu flag),
	///Servicing an interrupt consumes 5 M-Cycles (same as CALL i think)
	///The order than interrupts are serviced is as follows:
	///1. V-Blank
	///2. LCD Stat
	///3. Timer
	///4. Serial
	///5. Joypad
	fn interrupt_service_routine(&mut self) {
		if self.cpu.ime {
			//interrupts are enabled
			let interrupt_flag: u8 = self.io[0x0F];
			let interrupt_enable: u8 = self.cpu.ier;

			//only service requests where it's requested in IF and enabled in IE
			let interrupts = interrupt_flag & interrupt_enable;

			let mut interrupt: Option<Interrupt> = None;

			if (interrupts & VBLANK_MASK) == VBLANK_MASK {
				interrupt = Some(Interrupt::VBlank);
				self.io[0x0F] &= !VBLANK_MASK; //reset the v-blank bit in IF
			}
			else if (interrupts & LCDSTAT_MASK) == LCDSTAT_MASK {
				interrupt = Some(Interrupt::LcdStat);
				self.io[0x0F] &= !LCDSTAT_MASK; //reset the lcd-stat bit in IF
			}
			else if (interrupts & TIMER_MASK) == TIMER_MASK {
				interrupt = Some(Interrupt::Timer);
				self.io[0x0F] &= !TIMER_MASK; //reset the lcd-stat bit in IF
			}
			else if (interrupts & SERIAL_MASK) == SERIAL_MASK {
				interrupt = Some(Interrupt::Serial);
				self.io[0x0F] &= !SERIAL_MASK; //reset the lcd-stat bit in IF
			}
			else if (interrupts & JOYPAD_MASK) == JOYPAD_MASK {
				interrupt = Some(Interrupt::Joypad);
				self.io[0x0F] &= !JOYPAD_MASK; //reset the lcd-stat bit in IF
			}

			if interrupt.is_some() {
				//Nested interrupts are disabled unless the interrupt handler re enables them
				self.cpu.next_ime_state = false;

				//2 cycle delay
				self.emulate_hardware();
				self.emulate_hardware();

				//wake the processor
				self.cpu.halt = false;

				//Service the interrupt
				let new_pc: u16 = match interrupt.unwrap() {
					Interrupt::VBlank => VBLANK_ADDR,
					Interrupt::LcdStat => LCDSTAT_ADDR,
					Interrupt::Timer => TIMER_ADDR,
					Interrupt::Serial => SERIAL_ADDR,
					Interrupt::Joypad => JOYPAD_ADDR,
				};

				let old_pc = self.cpu.registers.pc;
				let sp: u16 = self.cpu.registers.sp;

				//push pc onto stack
				let high: u8 = (old_pc >> 8) as u8;
				self.write_byte(sp - 1, high);
				self.emulate_hardware();

				//push low byte of pc onto stack
				let low: u8 = (old_pc & 0xFF) as u8;
				self.write_byte(sp - 2, low);
				self.emulate_hardware();

				//sub 2 from sp because we pushed a word onto the stack
				self.cpu.registers.sp -= 2;

				//jump to interrupt handler
				self.cpu.registers.pc = new_pc;
				self.emulate_hardware();	//1 cycle delay when setting pc
			}
		}
	}

	/*
	///This is only really here for test roms that output results as text through the
	///serial port (blarggs test roms)
	fn print_serial(&mut self) {
		let sc: u8 = self.io[0x02];	//Serial Control Register
		if (sc & 128) == 128 && (sc & 1) == 1 {
			//if bit 7 (transfer start) and bit 0 (internal clock) are set, start transfer
			let sb = self.io[0x01];	//Serial Transfer Data Register
			print!("{}", sb as char);

			//reset sc bit 7
			self.io[0x02] &= 127;
			self.request_interrupt(Interrupt::Serial);
		}
	}*/

	pub fn keydown(&mut self, key: Key) {
		self.joypad.keydown(key);
		self.request_interrupt(Interrupt::Joypad);
	}

	pub fn keyup(&mut self, key: Key) {
		self.joypad.keyup(key);
	}

	pub fn get_framebuffer(&self) -> &[u32] {
		self.ppu.get_framebuffer()
	}

	pub fn get_framebuffer_mut(&mut self) -> &mut[u32] {
		self.ppu.get_framebuffer_mut()
	}

	pub fn get_oam(&mut self) -> &mut[u8] {
		self.ppu.get_oam()
	}

}
