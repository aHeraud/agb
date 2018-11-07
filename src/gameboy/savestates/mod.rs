use std::error::Error;

mod header;

pub use self::header::SaveStateHeader;

pub trait SerializeState: Sized {
	type Error: Error;
	fn serialize(&self) -> Vec<u8>;
	fn deserialize(buf: &[u8]) -> Result<Self, Self::Error>;
}
