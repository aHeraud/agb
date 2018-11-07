use std::fmt;
use std::fmt::{Display, Formatter};
use std::error::Error;
use std::convert::TryFrom;

use ::gameboy::{Mode, InvalidModeDiscriminant};
use super::SerializeState;

const SAVE_STATE_HEADER_SERIALIZED_LENGTH: usize = 38;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaveStateHeader {
	pub version: u8,
	pub mode: ::gameboy::Mode,
	pub cpu_state_offset: u32,
	pub timer_state_offset: u32,
	pub ppu_state_offset: u32,
	pub serial_state_offset: u32,
	pub joypad_state_offset: u32,
	pub cart_state_offset: u32,
	pub io_offset: u32,
	pub wram_offset: u32,
	pub oam_dma_state_offset: u32
}

#[derive(Debug)]
pub enum SaveStateHeaderDeserializationError {
	InvalidBufferLength{length: usize},
	InvalidModeValue(InvalidModeDiscriminant)
}

impl Display for SaveStateHeaderDeserializationError {
	fn fmt(&self, f: &mut Formatter) -> fmt::Result {
		match self {
			SaveStateHeaderDeserializationError::InvalidBufferLength{length} => {
				write!(f, "Error deserializing save state header from buffer, expected buffer length greater or equal to {}, found {}", length, SAVE_STATE_HEADER_SERIALIZED_LENGTH);
				Ok(())
			},
			SaveStateHeaderDeserializationError::InvalidModeValue(_) => {
				write!(f, "Failed to deserialize mode, found illegal value");
				Ok(())
			}
		}
	}
}

impl Error for SaveStateHeaderDeserializationError {
	fn source(&self) -> Option<&(Error + 'static)> {
		match self {
			SaveStateHeaderDeserializationError::InvalidModeValue(e) => Some(e),
			_ => None
		}
	}
}

/// converts a slice of 4 bytes (stored in big endian format) into a u32
/// does not do any bound checking, assumes the slice contains exactly 4 elements
unsafe fn u32_from_be_slice(buf: &[u8]) -> u32 {
	((buf[0] as u32) << 24) | ((buf[1] as u32) << 16) |
		((buf[2] as u32) << 8) | (buf[3] as u32)
}

impl SerializeState for SaveStateHeader {
	type Error = SaveStateHeaderDeserializationError;

	fn serialize(&self) -> Vec<u8> {
		let mut buf: Vec<u8> = Vec::new();
		buf.push(self.version);
		buf.push(self.mode as u8);
		buf.extend_from_slice(&self.cpu_state_offset.to_be_bytes());
		buf.extend_from_slice(&self.timer_state_offset.to_be_bytes());
		buf.extend_from_slice(&self.ppu_state_offset.to_be_bytes());
		buf.extend_from_slice(&self.serial_state_offset.to_be_bytes());
		buf.extend_from_slice(&self.joypad_state_offset.to_be_bytes());
		buf.extend_from_slice(&self.cart_state_offset.to_be_bytes());
		buf.extend_from_slice(&self.io_offset.to_be_bytes());
		buf.extend_from_slice(&self.wram_offset.to_be_bytes());
		buf.extend_from_slice(&self.oam_dma_state_offset.to_be_bytes());
		buf
	}

	fn deserialize(buf: &[u8]) -> Result<Self, Self::Error> {
		if buf.len() != SAVE_STATE_HEADER_SERIALIZED_LENGTH {
			Err(SaveStateHeaderDeserializationError::InvalidBufferLength{ length: buf.len() })
		}
		else {
			let mode = Mode::try_from(buf[1]).map_err(|e| SaveStateHeaderDeserializationError::InvalidModeValue(e))?;
			unsafe {
				Ok(SaveStateHeader {
					version: buf[0],
					mode: mode,
					cpu_state_offset: u32_from_be_slice(&buf[2..6]),
					timer_state_offset: u32_from_be_slice(&buf[6..10]),
					ppu_state_offset: u32_from_be_slice(&buf[10..14]),
					serial_state_offset: u32_from_be_slice(&buf[14..18]),
					joypad_state_offset: u32_from_be_slice(&buf[18..22]),
					cart_state_offset: u32_from_be_slice(&buf[22..26]),
					io_offset: u32_from_be_slice(&buf[26..30]),
					wram_offset: u32_from_be_slice(&buf[30..34]),
					oam_dma_state_offset: u32_from_be_slice(&buf[34..38])
				})
			}
		}
	}
}

#[cfg(test)]
mod test {
	#[test]
	fn serialize_deserialize_header() {
		use super::*;
		use ::gameboy::Mode;

		let header = SaveStateHeader {
			version: 0,
			mode: Mode::CGB,
			cpu_state_offset: 38,
			timer_state_offset: 512,
			ppu_state_offset: 1246,
			serial_state_offset: 12451,
			joypad_state_offset: 91252,
			cart_state_offset: 100000,
			io_offset: 101021,
			wram_offset: 101124,
			oam_dma_state_offset: 101160
		};

		let buffer = header.serialize();

		let deserialized_header = SaveStateHeader::deserialize(&buffer[..]).unwrap();

		assert_eq!(header, deserialized_header);
	}

	#[test]
	fn deserialize_buffer_too_small() {
		use super::*;

		let buffer = vec![0, 0, 0, 0, 0, 0xFF, 0xFF];
		assert!(SaveStateHeader::deserialize(&buffer[..]).is_err())
	}
}
