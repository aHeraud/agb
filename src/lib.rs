#![feature(drop_types_in_const)]

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

/* globals */

///Global gameboy object
static mut GAMEBOY: Option<Box<GBC>> = None;

///Global debugger object
static mut DEBUGGER: Option<Box<Debugger>> = None;

///Is there a debugger attached to the emulator
static mut IS_DEBUGGER_ATTACHED: bool = false;


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
			Some(ref mut gameboy) => {
				match IS_DEBUGGER_ATTACHED {
					true => {
						match DEBUGGER {
							Some(ref mut debugger) => { debugger.step_frame(gameboy); },
							None => { panic!("There is a debugger attached, but the debugger doesn't exist (this should never happen.)"); }
						};
					},
					false => { gameboy.step_frame(); },
				};
			},
			None => panic!("rustboy not initialized"),
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
				None => panic!("rustboy not initialized"),
			};
		}
	}
}

#[no_mangle]
///Pass a keyup event to the gameboy
pub fn rustboy_keyup(code: u32) {
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

/********************/
/* Debugger exports */
/********************/

#[no_mangle]
///Attach a debugger
pub fn rustboy_attach_debugger() {
	unsafe {
		match DEBUGGER {
			Some(_) => {},
			None => {
				/* No debugger exists, create debugger */
				DEBUGGER = Some(Box::new(Debugger::new()));
			}
		};
		IS_DEBUGGER_ATTACHED = true;
	}
}

#[no_mangle]
///Detach the debugger
pub fn rustboy_detach_debugger() {
	unsafe {
		IS_DEBUGGER_ATTACHED = false;
	}
}

#[no_mangle]
///Add a breakpoint (currently only an address)
pub fn rustboy_add_breakpoint(address: u16) {
	unsafe {
		match DEBUGGER {
			Some(ref mut debugger) => { debugger.add_breakpoint(address); },
			None => { panic!("You must attach a debugger before you can add breakpoints."); }
		};
	}
}

#[no_mangle]
///Remove a breakpoint
pub fn rustboy_remove_breakpoint(address: u16) {
	unsafe {
		match DEBUGGER {
			Some(ref mut debugger) => { debugger.remove_breakpoint(address); },
			None => { panic!("You must attach a debugger before you can remove breakpoints."); }
		};
	}
}

#[no_mangle]
///Get a list of breakpoints (as a c array)
///Returns the size of the array
pub fn rustboy_get_breakpoints(ptr: &mut *const u16) -> u32 {
	unsafe {
		match DEBUGGER {
			Some(ref mut debugger) => {
				let breakpoints: Vec<u16> = debugger.get_breakpoints().clone();
				let length: u32 = breakpoints.len() as u32;
				*ptr = breakpoints.as_ptr();
				length
			},
			None => {
				//Should this panic?
				*ptr = ptr::null();
				0
			},
		}
	}
}

#[no_mangle]
///Step
pub fn rustboy_step() {
	unsafe {
		match GAMEBOY {
			Some(ref mut gb) => {
				match IS_DEBUGGER_ATTACHED {
					true => {
						match DEBUGGER {
							Some(ref mut debugger) => { debugger.step(gb, true); }
							None => { panic!("There is a debugger attached, but the debugger doesn't exist (this should never happen.)"); }
						};
					},
					false => { panic!("You must attach a debugger before you can use the step function."); },
				};
			},
			None => { panic!("rustboy not initialized"); }
		}
	}
}
