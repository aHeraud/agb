#![feature(drop_types_in_const)]

use std::slice;

mod gameboy;
mod debugger;
use gameboy::joypad::Key;

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;

pub const KEY_UP: u32 = 0;
pub const KEY_DOWN: u32 = 1;
pub const KEY_LEFT: u32 = 2;
pub const KEY_RIGHT: u32 = 3;
pub const KEY_B: u32 = 4;
pub const KEY_A: u32 = 5;
pub const KEY_SELECT: u32 = 6;
pub const KEY_START: u32 = 7;

fn get_key(code: u32) -> Option<Key> {
	match code {
		KEY_UP => Some(Key::Up),
		KEY_DOWN => Some(Key::Down),
		KEY_LEFT => Some(Key::Left),
		KEY_RIGHT => Some(Key::Right),
		KEY_A => Some(Key::A),
		KEY_B => Some(Key::B),
		KEY_SELECT => Some(Key::Select),
		KEY_START => Some(Key::Start),
		_ => None,
	}
}

///Global gameboy object
static mut GAMEBOY: Option<Box<gameboy::GBC>> = None;

#[no_mangle]
///Create a new gameboy object (and store it as a global variable)
pub fn rustboy_init(rom_ptr: *const u8, rom_size: u32, ram_ptr: *const u8, ram_size: u32) {
	unsafe {
		/* Copy into a boxed array */
		let rom_slice: &[u8] = slice::from_raw_parts(rom_ptr, rom_size as usize);
		let ram_slice: &[u8] = slice::from_raw_parts(ram_ptr, ram_size as usize);

		let mut rom: Vec<u8> = Vec::with_capacity(rom_size as usize);
		let mut ram: Vec<u8> = Vec::with_capacity(ram_size as usize);

		rom.extend_from_slice(rom_slice);
		ram.extend_from_slice(ram_slice);

		GAMEBOY = Some(Box::new(gameboy::GBC::new(rom.into_boxed_slice(), ram.into_boxed_slice())));
	}
}

#[no_mangle]
//Create a new gameboy object from a path to a rom (and store it as a global variable)
//pub fn rustboy_init_from_path(rom_path: String, ram_path: String) {
//	unsafe {
//		GAMEBOY = Some(Box::new(gameboy::GBC::from_path(rom_path, ram_path)));
//	}
//}

#[no_mangle]
///Step to the next frame
pub fn rustboy_step_frame() {
	unsafe {
		match GAMEBOY {
			Some(ref mut gameboy) => gameboy.step_frame(),
			None => panic!("rustboy not initialized")
		}
	}
}

#[no_mangle]
///Get a pointer to the current front framebuffer
pub fn rustboy_get_framebuffer() -> *mut u32 {
	unsafe {
		match GAMEBOY {
			Some(ref mut gameboy) => gameboy.ppu.get_framebuffer_mut().as_mut_ptr(),
			None => panic!("rustboy not initialized"),	//return null?
		}
	}
}

#[no_mangle]
///Pass a keydown event to the gameboy
pub fn rustboy_keydown(code: u32) {
	let key: Option<Key> = get_key(code);
	if key.is_some() {
		unsafe {
			match GAMEBOY {
				Some(ref mut gameboy) => gameboy.keydown(key.unwrap()),
				None => panic!("rustboy not initialized")
			};
		}
	}
}

#[no_mangle]
///Pass a keyup event to the gameboy
pub fn rustboy_keyup(key: u32) {
	let key: Option<Key> = get_key(code);
	if key.is_some() {
		unsafe {
			match GAMEBOY {
				Some(ref mut gameboy) => gameboy.keyup(key.unwrap()),
				None => panic!("rustboy not initialized")
			};
		}
	}
}
