use std::fmt;
use std::fmt::{Display, Formatter};
use std::error::Error;
use std::convert::TryFrom;

const DMG_MODE: u8 = 0;
const CGB_MODE: u8 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
#[repr(u8)]
pub enum Mode {
	DMG = DMG_MODE, CGB = CGB_MODE
}

#[derive(Debug, Clone, Copy)]
pub struct InvalidModeDiscriminant(u8);

impl Display for InvalidModeDiscriminant {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		write!(f, "Invalid mode value: found {}, expected one of [{}, {}]", self.0, DMG_MODE, CGB_MODE)
	}
}

impl Error for InvalidModeDiscriminant {}

impl TryFrom<u8> for Mode {
	type Error = InvalidModeDiscriminant;

	fn try_from(value: u8) -> Result<Self, Self::Error> {
		match value {
			DMG_MODE => Ok(Mode::DMG),
			CGB_MODE => Ok(Mode::CGB),
			_ => Err(InvalidModeDiscriminant(value))
		}
	}
}
