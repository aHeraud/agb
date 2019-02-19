#[macro_use]
extern crate lazy_static;
extern crate wasm_bindgen;
extern crate agb_core;

use std::time::Duration;
use std::sync::{Mutex, mpsc::channel, mpsc::Receiver, mpsc::Sender};
use std::collections::HashMap;

use wasm_bindgen::prelude::*;

use agb_core::gameboy::{Gameboy, Key};

pub const KEY_UP: u32 = 0;
pub const KEY_DOWN: u32 = 1;
pub const KEY_LEFT: u32 = 2;
pub const KEY_RIGHT: u32 = 3;
pub const KEY_B: u32 = 4;
pub const KEY_A: u32 = 5;
pub const KEY_SELECT: u32 = 6;
pub const KEY_START: u32 = 7;

enum FrontendEvent {
	Keydown(Key),
	Keyup(Key)
}

#[wasm_bindgen]
extern {
	#[wasm_bindgen(js_namespace = console)]
	fn log(s: &str);

	#[wasm_bindgen(js_namespace = console)]
	fn error(s: &str);

	#[wasm_bindgen]
	fn alert(s: &str);
}

#[wasm_bindgen(module = "./index")]
extern {
	#[wasm_bindgen]
	fn draw(width: usize, height: usize, pixels: &[u32]);
}

lazy_static! {
	static ref GAMEBOY: Mutex<Option<Gameboy>> = Mutex::default();
	static ref KEYS_LUT: HashMap<u32, Key> = {
		let mut map = HashMap::new();
		map.insert(KEY_UP, Key::Up);
		map.insert(KEY_DOWN, Key::Down);
		map.insert(KEY_LEFT, Key::Left);
		map.insert(KEY_RIGHT, Key::Right);
		map.insert(KEY_B, Key::B);
		map.insert(KEY_A, Key::A);
		map.insert(KEY_SELECT, Key::Select);
		map.insert(KEY_START, Key::Start);
		map
	};
	static ref FRONTEND_EVENT_CHANNELS: (Mutex<Sender<FrontendEvent>>, Mutex<Receiver<FrontendEvent>>) = {
		let (sender, reciever) = channel::<FrontendEvent>();
		(Mutex::from(sender), Mutex::from(reciever))
	};
}

/// Loads a rom + an optional save file.
/// This creates a new Gameboy object.
/// This can fail: if the rom has an invalid header an alert will be displayed  and an error message will be printed to the console
#[wasm_bindgen]
pub fn load_rom(rom: &[u8]) {
	match Gameboy::new(Box::from(rom.clone()), None) {
		Ok(gameboy) => {
			let mut opt_gameboy = GAMEBOY.lock().unwrap();
			*opt_gameboy = Some(gameboy);
			log("agb-web::load_rom - loaded rom");
		}
		Err(e) => {
			error(&format!("{}", e));
			alert("Invalid rom file.");
		}
	}
}

#[wasm_bindgen]
pub fn keydown(keycode: u32) {
	let sender = FRONTEND_EVENT_CHANNELS.0.lock().unwrap();
	if let Some(key) = KEYS_LUT.get(&keycode) {
		sender.send(FrontendEvent::Keydown(*key)).unwrap();
	}
}

#[wasm_bindgen]
pub fn keyup(keycode: u32) {
	let sender = FRONTEND_EVENT_CHANNELS.0.lock().unwrap();
	if let Some(key) = KEYS_LUT.get(&keycode) {
		sender.send(FrontendEvent::Keyup(*key)).unwrap();
	}
}

/// Emulate the gameboy for a specific number of milliseconds
#[wasm_bindgen]
pub fn emulate(ms: u32) {
	let mut opt_gameboy = GAMEBOY.lock().unwrap();
	let event_queue = FRONTEND_EVENT_CHANNELS.1.lock().unwrap();

	while let Ok(event) = event_queue.try_recv() {
		match event {
			FrontendEvent::Keydown(key) => {
				if let Some(ref mut gameboy) = *opt_gameboy {
					gameboy.keydown(key);
				}
			},
			FrontendEvent::Keyup(key) => {
				if let Some(ref mut gameboy) = *opt_gameboy {
					gameboy.keyup(key);
				}
			}
		}
	}

	if let Some(ref mut gameboy) = *opt_gameboy {
		let last_frame_counter = gameboy.get_frame_counter();
		gameboy.emulate(Duration::from_millis(ms as u64));
		if gameboy.get_frame_counter() != last_frame_counter {
			//new frame waiting to be displayed
			draw(agb_core::WIDTH, agb_core::HEIGHT, gameboy.get_framebuffer());
		}
	}
}
