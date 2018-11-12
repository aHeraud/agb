#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub enum Key {
	Up = 0, Down = 1, Left = 2, Right = 3, B = 4, A = 5, Select = 6, Start = 7
}

#[derive(Serialize, Deserialize)]
pub struct Joypad {
	keys: [bool; 8],
	select_button_keys: bool,
	select_direction_keys: bool,
}

impl Joypad {
	pub fn new() -> Joypad {
		Joypad {
			keys: [false; 8],
			select_button_keys: false,
			select_direction_keys: false,
		}
	}

	///Keydown event
	pub fn keydown(&mut self, key: Key) {
		self.keys[key as usize] = true;
	}

	///Keyup event
	pub fn keyup(&mut self, key: Key) {
		self.keys[key as usize] = false;
	}

	///Query the state of a button
	pub fn key_state(&self, key: Key) -> bool {
		self.keys[key as usize]
	}

	///Used to select buttons/dpad
	///only bits 4 and 5 are writeable
	///bit 5: p15 = select button keys (0 = select)
	///bit 4: p14 = select dpad (0 = select)
	//TODO: what happens when they're both selected?
	pub fn write_joyp(&mut self, value: u8) {
		self.select_button_keys = value & 32 == 0;
		self.select_direction_keys = value & 16 == 0;
	}

	pub fn read_joyp(&self) -> u8 {
		let mut high: u8 = 0;
		if self.select_button_keys {
			high |= 32;
		}
		if self.select_direction_keys {
			high |= 16;
		}

		let mut low = 0;
		if self.select_button_keys {
			if self.keys[Key::Start as usize] {
				low |= 8;
			}
			if self.keys[Key::Select as usize] {
				low |= 4;
			}
			if self.keys[Key::B as usize] {
				low |= 2;
			}
			if self.keys[Key::A as usize] {
				low |= 1;
			}
		}
		else if self.select_direction_keys {
			if self.keys[Key::Down as usize] {
				low |= 8;
			}
			if self.keys[Key::Up as usize] {
				low |= 4;
			}
			if self.keys[Key::Left as usize] {
				low |= 2;
			}
			if self.keys[Key::Right as usize] {
				low |= 1;
			}
		}

		//Convert to active low and return
		(!((high & 0xF0) | (low & 0x0F))) & 0xCF
	}
}
