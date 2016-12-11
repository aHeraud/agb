#![no_std]
#![feature(alloc)]
#![feature(collections)]

extern crate alloc;
extern crate collections;

use alloc::boxed::Box;

pub mod gameboy;
pub mod debugger;
use gameboy::Gameboy;
use gameboy::joypad::Key;
//use debugger::Debugger;

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;

///Initialize the gameboy and load a rom file (and optionally a ram file)
pub fn init(rom: Box<[u8]>, ram: Option<Box<[u8]>>) -> Box<Gameboy> {
	Box::new(Gameboy::new(rom, ram))
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
