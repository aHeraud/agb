#![feature(nll)]
#![feature(int_to_from_bytes)]
#![feature(try_from)]

extern crate time;

pub mod gameboy;

pub use gameboy::Gameboy;
pub use gameboy::joypad::Key;

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;
pub const FPS: f64 = 59.7_f64;
