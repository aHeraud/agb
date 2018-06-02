pub mod cpu;
mod mmu;
pub mod ppu;
pub mod cartridge;
pub mod instructions;
pub mod timer;
pub mod joypad;
pub mod debugger;
mod assembly;
mod util;

use gameboy::mmu::Mmu;
use gameboy::cpu::CPU;
use gameboy::cpu::registers::Register;
use gameboy::ppu::PPU;
use gameboy::ppu::dmg_ppu::DmgPpu;
//use gameboy::ppu::cgb_ppu::CgbPpu;
use gameboy::timer::Timer;
use gameboy::cartridge::{Cartridge, VirtualCartridge};
use gameboy::joypad::Joypad;
use gameboy::debugger::{Debugger, DebuggerInterface};
use gameboy::cpu::interrupts::Interrupt;
pub use gameboy::joypad::Key;

const IO_SIZE: usize = 128;

const WRAM_BANK_SIZE: usize = 4096;
const WRAM_NUM_BANKS: usize = 8;

#[derive(Debug, Clone, Copy)]
pub enum Mode {
	DMG, CGB,
}

pub struct Gameboy {
	pub cpu: CPU,
	pub timer: Timer,
	pub ppu: Box<PPU>,
	pub joypad: Joypad,
	pub cart: Box<Cartridge>,
	pub io: Box<[u8]>,
	pub wram: Box<[u8]>,
	pub mode: Mode,
	pub debugger: Debugger,
	pub oam_dma_active: bool,
	pub oam_dma_start_address: u16,
	pub oam_dma_current_cycle: u16,
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

		/* initialize io memory to what it should be at the end of the bootrom if
		   no boot rom is loaded
		   TODO: allow the loading of a boot rom
		 */
		let io: [u8; IO_SIZE] = match mode {
			Mode::DMG => {
				[
					0xCF, 0x00, 0x7E, 0xFF, 0x19, 0x00, 0x00, 0xF8, //ff00
					0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xE1, //ff08
					0x80, 0xBF, 0xF3, 0xFF, 0xBF, 0xFF, 0x3F, 0x00, //ff10
					0xFF, 0xBF, 0x7F, 0xFF, 0x9F, 0xFF, 0xBF, 0xFF, //ff18
					0xFF, 0x00, 0x00, 0xBF, 0x77, 0xF3, 0xF1, 0xFF, //ff20
					0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, //ff28
					0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, //ff30
					0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, //ff38
					0x91, 0x83, 0x00, 0x00, 0x01, 0x00, 0xFF, 0xFC, //ff40
					0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, //ff48
					0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, //ff50
					0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, //ff58
					0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, //ff60
					0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, //ff68
					0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, //ff70
					0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, //ff78
				]
			},
			Mode::CGB => {
				//TODO: cgb bootrom values
				[0; IO_SIZE]
			},
		};

		let gameboy = Gameboy {
			cpu: CPU::new(),
			timer: Timer::new(),
			ppu: ppu,
			joypad: Joypad::new(),
			cart: cart,
			io: Box::new(io),
			wram: Box::new([0; WRAM_BANK_SIZE * WRAM_NUM_BANKS]),
			mode: mode,
			debugger: Debugger::new(),
			oam_dma_active: false,
			oam_dma_start_address: 0,
			oam_dma_current_cycle: 0,
		};
		Ok(gameboy)
	}

	pub fn step_frame(&mut self) {
		//A complete frame occurs every ~70224 clock cycles (140448 in gbc double speed mode)
		const FRAME_CLOCKS: usize = 70224;
		let mut counter: usize = 0;

		if self.debugger.enabled() {
			while counter < FRAME_CLOCKS {
				self.interrupt_service_routine();
				if let Some(breakpoint) = self.breakpoint_lookahead() {
					self.debugger.breakpoint_callback(breakpoint);
					return;
				}
				self.execute();
				match self.cpu.double_speed_mode {
					true => counter += 2,
					false => counter += 4,
				};
			}
		}
		else {
			while counter < FRAME_CLOCKS {
				self.interrupt_service_routine();
				self.execute();
				match self.cpu.double_speed_mode {
					true => counter += 2,
					false => counter += 4,
				};
			}
		}
	}

	fn start_oam_dma(&mut self, value: u8) {
		self.oam_dma_active = true;
		self.oam_dma_start_address = (value as u16) << 8;
		self.oam_dma_current_cycle = 0;
	}

	fn service_oam_dma(&mut self) {
		//according to https://github.com/Gekkio/mooneye-gb/blob/master/docs/accuracy.markdown
		//oam dma is 162 m-cycles, with 2 m-cycles for startup/teardown
		//since oam is 160 bytes (0x100), this means it transfers 1 byte/m-cycle.

		if self.oam_dma_current_cycle > 0 && self.oam_dma_current_cycle < 161 {
			let offset = self.oam_dma_current_cycle - 1;
			let value: u8 = self.read_byte(self.oam_dma_start_address + offset);
			self.ppu.write_byte_oam(0xFE00 + offset, value);
		}
	}

	fn update_oam_dma_status(&mut self) {
		if self.oam_dma_current_cycle < 161 {
			self.oam_dma_current_cycle += 1;
		}
		else {
			self.oam_dma_active = false;
		}
	}

	///Called every 4 cycles
	fn emulate_hardware(&mut self) {
		use gameboy::cpu::interrupts::InterruptLine;

		if self.oam_dma_active {
			self.service_oam_dma();
		}

		let mut interrupt_line = InterruptLine::new(&mut self.cpu.interrupt_flag, &mut self.cpu.halt, &mut self.cpu.stop);
		self.timer.emulate_hardware(&mut self.io, &mut interrupt_line);
		self.ppu.emulate_hardware(&mut interrupt_line);

		if self.oam_dma_active {
			self.update_oam_dma_status();
		}

		self.cpu.cycle_counter += 1;
	}

	fn request_interrupt(&mut self, req_int: Interrupt) {
		self.cpu.interrupt_flag.request_interrupt(req_int);
		self.cpu.stop = false;
		self.cpu.halt = false;
	}

	///Handles interupts
	///If an interrupt is requested in IF (FF0F), and it is enabled in IE (FFFF), and
	///interrupts are enabled by IME (cpu flag),
	///Servicing an interrupt consumes 5 M-Cycles (same as CALL I think)
	///The order than interrupts are serviced is as follows:
	///1. V-Blank
	///2. LCD Stat
	///3. Timer
	///4. Serial
	///5. Joypad
	fn interrupt_service_routine(&mut self) {
		if self.cpu.ime {
			let interrupt_flag: u8 = self.cpu.interrupt_flag.read();
			let interrupt_enable: u8 = self.cpu.interrupt_enable.read();

			//only service requests where it's requested in IF and enabled in IE
			let interrupts = interrupt_flag & interrupt_enable;

			let mut interrupt: Option<Interrupt> = None;

			if (interrupts & (Interrupt::VBlank).mask()) == (Interrupt::VBlank).mask() {
				interrupt = Some(Interrupt::VBlank);
				self.cpu.interrupt_flag.clear_interrupt(Interrupt::VBlank); //reset the v-blank bit in IF
			}
			else if (interrupts & (Interrupt::LcdStat).mask()) == (Interrupt::LcdStat).mask() {
				interrupt = Some(Interrupt::LcdStat);
				self.cpu.interrupt_flag.clear_interrupt(Interrupt::LcdStat); //reset the lcd-stat bit in IF
			}
			else if (interrupts & (Interrupt::Timer).mask()) == (Interrupt::Timer).mask() {
				interrupt = Some(Interrupt::Timer);
				self.cpu.interrupt_flag.clear_interrupt(Interrupt::Timer); //reset the lcd-stat bit in IF
			}
			else if (interrupts & (Interrupt::Serial).mask()) == (Interrupt::Serial).mask() {
				interrupt = Some(Interrupt::Serial);
				self.cpu.interrupt_flag.clear_interrupt(Interrupt::Serial); //reset the lcd-stat bit in IF
			}
			else if (interrupts & (Interrupt::Joypad).mask()) == (Interrupt::Joypad).mask() {
				interrupt = Some(Interrupt::Joypad);
				self.cpu.interrupt_flag.clear_interrupt(Interrupt::Joypad); //reset the lcd-stat bit in IF
			}

			if let Some(interrupt) = interrupt {
				//Interrupts are enabled, so service this interrupt
				//Nested interrupts are disabled unless the interrupt handler re enables them
				self.cpu.next_ime_state = false;

				//2 cycle delay
				self.emulate_hardware();
				self.emulate_hardware();

				//wake the processor
				self.cpu.halt = false;

				//Service the interrupt
				let new_pc = interrupt.address();

				let old_pc = self.cpu.registers.pc;
				let sp: u16 = self.cpu.registers.sp;

				//TODO: if there is an oam dma transfer and sp doesn't point
				//to hram, can this be put on the stack?

				//push pc onto stack
				let high: u8 = (old_pc >> 8) as u8;
				self.write_byte_cpu(sp - 1, high);
				self.emulate_hardware();

				//push low byte of pc onto stack
				let low: u8 = (old_pc & 0xFF) as u8;
				self.write_byte_cpu(sp - 2, low);
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

	/* If for some reason you want to write directly to the framebuffer */
	pub fn get_framebuffer_mut(&mut self) -> &mut[u32] {
		self.ppu.get_framebuffer_mut()
	}
}
