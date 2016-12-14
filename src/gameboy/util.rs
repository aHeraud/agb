//use std::fs::File;
//use std::io::{Read, Error};
//use std::path::Path;

#[cfg(feature = "no_std")]
use core::num::Wrapping;

#[cfg(not(feature = "no_std"))]
use std::num::Wrapping;

pub fn wrapping_add(r1: u16, r2: u16) -> u16 {
	(Wrapping(r1) + Wrapping(r2)).0
}

pub fn wrapping_sub(r1: u16, r2: u16) -> u16 {
	(Wrapping(r1) - Wrapping(r2)).0
}

/*
pub fn read_file<P: AsRef<Path>>(path: P) -> Result<Box<[u8]>, Error> {
	let mut file = try!(File::open(path));
	let mut buffer = Vec::new();
	let result = file.read_to_end(&mut buffer);
	match result {
		Ok(_) => Ok(buffer.into_boxed_slice()),
		Err(err) => Err(err),
	}
}*/
