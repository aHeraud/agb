//Interrupt bit masks
const VBLANK_MASK:  u8 = 1 << 0;
const LCDSTAT_MASK: u8 = 1 << 1;
const TIMER_MASK:   u8 = 1 << 2;
const SERIAL_MASK:  u8 = 1 << 3;
const JOYPAD_MASK:  u8 = 1 << 4;

//Interrupt handler addresses
const VBLANK_ADDR:  u16 = 0x0040;
const LCDSTAT_ADDR: u16 = 0x0048;
const TIMER_ADDR:   u16 = 0x0050;
const SERIAL_ADDR:  u16 = 0x0058;
const JOYPAD_ADDR:  u16 = 0x0060;

#[derive(Debug, PartialEq, Eq)]
pub enum Interrupt {
	VBlank, LcdStat, Timer, Serial, Joypad
}

impl Interrupt {
	pub fn mask(&self) -> u8 {
		use self::Interrupt::*;
		match *self {
			VBlank => VBLANK_MASK,
			LcdStat => LCDSTAT_MASK,
			Timer => TIMER_MASK,
			Serial => SERIAL_MASK,
			Joypad => JOYPAD_MASK
		}
	}

	pub fn address(&self) -> u16 {
		use self::Interrupt::*;
		match *self {
			VBlank => VBLANK_ADDR,
			LcdStat => LCDSTAT_ADDR,
			Timer => TIMER_ADDR,
			Serial => SERIAL_ADDR,
			Joypad => JOYPAD_ADDR
		}
	}
}

/// Interrupt Flag Register - $FF0F
#[derive(Clone, Copy)]
pub struct InterruptFlag {
	value: u8,
}

impl InterruptFlag {
	/// Constructor that initializes the value of the register to the post bootrom value
	pub fn new() -> InterruptFlag {
		InterruptFlag { value: 0x01 }
	}

	pub fn reset(&mut self) {
		self.value = 1;
	}

	pub fn request_interrupt(&mut self, int: Interrupt) {
		self.value |= int.mask();
	}

	pub fn clear_interrupt(&mut self, int: Interrupt) {
		self.value &= !int.mask();
	}

	/// Read from the IF register.
	/// High 3 bits always read as 1.
	pub fn read(&self) -> u8 {
		self.value | 0xE0
	}

	pub fn write(&mut self, value: u8) {
		self.value = value & 0x1F;
	}
}

#[derive(Clone, Copy, Default)]
pub struct InterruptEnable {
	value: u8
}

/// Interrupt Enable Register - $FFFF
impl InterruptEnable {
	/// Constructor that initializes the value of the register to the post bootrom value
	pub fn new() -> InterruptEnable {
		InterruptEnable { value: 0 }
	}

	pub fn reset(&mut self) {
		self.value = 0;
	}

	/// Read from the IE register.
	/// High 3 bits always read as 1.
	pub fn read(&self) -> u8 {
		self.value | 0xE0
	}

	pub fn write(&mut self, value: u8) {
		self.value = value & 0x1F;
	}
}

/// Passed to components that need to request interrupts, but don't have access to the global emulator state.
pub struct InterruptLine<'a> {
	interrupt_flag: &'a mut InterruptFlag,
	halt: &'a mut bool,
	stop: &'a mut bool
}

impl<'a> InterruptLine<'a> {
	pub fn new(interrupt_flag: &'a mut InterruptFlag, halt: &'a mut bool, stop: &'a mut bool) -> InterruptLine<'a> {
		InterruptLine {
			interrupt_flag: interrupt_flag,
			halt: halt,
			stop: stop
		}
	}

	/// Request an interrupt
	/// Interrupts will wake the cpu if it is stopped or halted.
	pub fn request_interrupt(&mut self, int: Interrupt) {
		self.interrupt_flag.request_interrupt(int);
		*self.halt = false;
		*self.stop = false;
	}
}
