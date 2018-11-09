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
