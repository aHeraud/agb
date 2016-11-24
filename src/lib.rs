#![feature(drop_types_in_const)]

mod gameboy;
pub mod debugger;
pub use gameboy::joypad::Key;

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;

///Global gameboy object
static mut GAMEBOY: Option<Box<gameboy::GBC>> = None;

#[no_mangle]
///Create a new gameboy object (and store it as a global variable)
pub fn rustboy_init(rom: Box<[u8]>, ram: Box<[u8]>) {
	unsafe {
		GAMEBOY = Some(Box::new(gameboy::GBC::new(rom, ram)));
	}
}

#[no_mangle]
///Create a new gameboy object from a path to a rom (and store it as a global variable)
pub fn rustboy_init_from_path(rom_path: String, ram_path: String) {
	unsafe {
		GAMEBOY = Some(Box::new(gameboy::GBC::from_path(rom_path, ram_path)));
	}
}

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
pub fn rustboy_keydown(key: Key) {
	unsafe {
		match GAMEBOY {
			Some(ref mut gameboy) => gameboy.keydown(key),
			None => panic!("rustboy not initialized")
		};
	}
}

#[no_mangle]
///Pass a keyup event to the gameboy
pub fn rustboy_keyup(key: Key) {
	unsafe {
		match GAMEBOY {
			Some(ref mut gameboy) => gameboy.keyup(key),
			None => panic!("rustboy not initialized")
		};
	}
}
