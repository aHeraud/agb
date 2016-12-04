/* C exports */
/* TODO: remove panics and replace with error codes and messages. */

use std::slice;
use std::ptr;

mod gameboy;
mod debugger;
use gameboy::GBC;
use gameboy::joypad::Key;
use debugger::Debugger;

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

#[no_mangle]
///Create a new gameboy object (and store it as a global variable)
pub extern fn rustboy_init(rom_ptr: *const u8, rom_size: u32, ram_ptr: *const u8, ram_size: u32) -> *mut Gameboy {
	unsafe {
		/* Copy into a boxed array */
		let rom_slice: &[u8] = slice::from_raw_parts(rom_ptr, rom_size as usize);
		let ram_slice: &[u8] = slice::from_raw_parts(ram_ptr, ram_size as usize);

		let mut rom: Vec<u8> = Vec::with_capacity(rom_size as usize);
		let mut ram: Vec<u8> = Vec::with_capacity(ram_size as usize);

		rom.extend_from_slice(rom_slice);
		ram.extend_from_slice(ram_slice);

		let gameboy = Box::new(Gameboy::new(rom.into_boxed_slice(), ram.into_boxed_slice()));
		gameboy.into_raw()
	}
}

#[no_mangle]
///Step to the next frame
pub extern fn rustboy_step_frame(gameboy_ptr: *mut Gameboy) {
	if gameboy_ptr.is_null() {
		panic!("gameboy_ptr can not be null.");
	}

	let gameboy = unsafe { *gameboy_ptr };
	gameboy.step_frame();
}

#[no_mangle]
///Get a pointer to the current front framebuffer
pub extern fn rustboy_get_framebuffer(gameboy_ptr: *mut Gameboy) -> *mut u32 {
	if gameboy_ptr.is_null() {
		panic!("gameboy_ptr can not be null.");
	}

	let gameboy = unsafe { *gameboy_ptr };
	gameboy.ppu.get_framebuffer_mut().as_mut_ptr()
}

#[no_mangle]
///Pass a keydown event to the gameboy
pub extern fn rustboy_keydown(gameboy_ptr: *mut Gameboy, code: u32) {
	if gameboy_ptr.is_null() {
		panic!("gameboy_ptr can not be null.");
	}
	let key: Option<Key> = get_key(code);
	if key.is_some() {
		let gameboy = unsafe { *gameboy_ptr };
		gameboy.keydown(key.unwrap());
	}
}

#[no_mangle]
///Pass a keyup event to the gameboy
pub extern fn rustboy_keyup(gameboy_ptr: *mut Gameboy, code: u32) {
	if gameboy_ptr.is_null() {
		panic!("gameboy_ptr can not be null.");
	}
	let key: Option<Key> = get_key(code);
	if key.is_some() {
		let gameboy = unsafe { *gameboy_ptr };
		gameboy.keyup(key.unwrap());
	}
}
