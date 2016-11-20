#[derive(Debug, Clone, Copy)]
pub enum Key {
    Up, Down, Left, Right, A, B, Select, Start
}

fn get_index(key: Key) -> usize {
    match key {
        Key::Up => 0,
        Key::Down => 1,
        Key::Left => 2,
        Key::Right => 3,
        Key::A => 4,
        Key::B => 5,
        Key::Select => 6,
        Key::Start => 7,
    }
}

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
        let index = get_index(key);
        self.keys[index] = true;
    }

    ///Keyup event
    pub fn keyup(&mut self, key: Key) {
        let index = get_index(key);
        self.keys[index] = false;
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
            if self.keys[get_index(Key::Start)] {
                low |= 8;
            }
            if self.keys[get_index(Key::Select)] {
                low |= 4;
            }
            if self.keys[get_index(Key::A)] {
                low |= 2;
            }
            if self.keys[get_index(Key::B)] {
                low |= 1;
            }
        }
        else if self.select_direction_keys {
            if self.keys[get_index(Key::Down)] {
                low |= 8;
            }
            if self.keys[get_index(Key::Up)] {
                low |= 4;
            }
            if self.keys[get_index(Key::Left)] {
                low |= 2;
            }
            if self.keys[get_index(Key::Right)] {
                low |= 1;
            }
        }

        //Convert to active low and return
        !((high & 0xF0) | (low & 0x0F))
    }
}
