#![feature(inclusive_range_syntax)]
/* */
#![cfg_attr(feature = "no_std", no_std)]
#![cfg_attr(feature = "no_std", feature(alloc))]
#![cfg_attr(feature = "no_std", feature(collections))]

#[cfg(feature = "no_std")]
extern crate alloc;

#[cfg(feature = "no_std")]
extern crate collections;

#[cfg(feature = "no_std")]
use alloc::boxed::Box;

pub mod gameboy;
//pub mod debugger;
use gameboy::Gameboy;
use gameboy::joypad::Key;
//use debugger::Debugger;

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;

///Initialize the gameboy and load a rom file (and optionally a ram file)
pub fn init(rom: Box<[u8]>, ram: Option<Box<[u8]>>) -> Result<Box<Gameboy>, & 'static str> {
	let gameboy = try!(Gameboy::new(rom, ram));
	Ok(Box::new(gameboy))
}

pub fn step_frame(gameboy: &mut Gameboy) {
	gameboy.step_frame();
}

pub fn get_framebuffer(gameboy: & Gameboy) -> &[u32] {
	gameboy.get_framebuffer()
}

pub fn get_framebuffer_mut(gameboy: &mut Gameboy) -> &mut[u32] {
	gameboy.get_framebuffer_mut()
}

pub fn keydown(gameboy: &mut Gameboy, key: Key) {
	gameboy.keydown(key);
}

pub fn keyup(gameboy: &mut Gameboy, key: Key) {
	gameboy.keyup(key);
}
