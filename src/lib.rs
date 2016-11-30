#![feature(drop_types_in_const)]

use std::io::Error;

mod gameboy;
mod debugger;
use gameboy::GBC;
use gameboy::joypad::Key;
use debugger::Debugger;

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;

/* globals */

///Global gameboy object
static mut GAMEBOY: Option<Box<GBC>> = None;

///Global debugger object
static mut DEBUGGER: Option<Box<Debugger>> = None;

///Is there a debugger attached to the emulator
static mut IS_DEBUGGER_ATTACHED: bool = false;


///Initialize the gameboy and load a rom file (and optionally a ram file)
pub fn init(rom: Box<[u8]>, ram: Option<Box<[u8]>>) {
	unsafe {
		GAMEBOY = Some(Box::new(GBC::new(rom, ram)));
	}
}

pub fn step_frame() -> Result<(), isize> {
	unsafe {
		match GAMEBOY {
			Some(ref mut gameboy) => {
				gameboy.step_frame();
				Ok(())
			},
			None => Err(-1),
		}
	}
}

pub fn get_framebuffer<'a>() -> Result< &'a[u32], isize> {
	unsafe {
		match GAMEBOY {
			Some(ref mut gameboy) => Ok(gameboy.get_framebuffer()),
			None => Err(-1),
		}
	}
}

pub fn keydown(key: Key) -> Result<(), isize> {
	unsafe {
		match GAMEBOY {
			Some(ref mut gameboy) => {
				gameboy.keydown(key);
				Ok(())
			},
			None => Err(-1),
		}
	}
}

pub fn keyup(key: Key) -> Result<(), isize> {
	unsafe {
		match GAMEBOY {
			Some(ref mut gameboy) => {
				gameboy.keyup(key);
				Ok(())
			},
			None => Err(-1),
		}
	}
}
