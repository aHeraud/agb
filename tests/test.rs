extern crate rustboy_core;
extern crate image;

use std::fs::{read_dir, create_dir, File};
use std::io::{Read, Error, Write};
use std::path::Path;

use std::vec::Vec;

const TEST_FRAMES: usize = 60 * 15;

pub fn read_file<P: AsRef<Path>>(path: P) -> Result<Box<[u8]>, Error> {
	let mut file = try!(File::open(path));
	let mut buffer = Vec::new();
	let result = file.read_to_end(&mut buffer);
	match result {
		Ok(_) => Ok(buffer.into_boxed_slice()),
		Err(err) => Err(err),
	}
}

///Runs a test rom and saves a sceenshot in tests/results after a specified ammount of cycles
///If there are no errors, it returns a vec of u32's that represent an rgba screenshot
fn run_test_rom(path: String) -> Result<Vec<u32>,String> {
	let rom = read_file(path.clone());
	if let Err(_) = rom {
		return Err(format!("Failed to open file {}.", path));
	}

	let gameboy = rustboy_core::init(rom.unwrap(), None);
	if let Err(ref msg) = gameboy {
		return Err(format!("Test rom {} failed with error {}.", path.clone(), msg));
	}
	let mut gameboy = gameboy.unwrap();

	for frame in 0..TEST_FRAMES {
		gameboy.step_frame();
	}

	let framebuffer = gameboy.get_framebuffer();
	let mut buffer = std::vec::Vec::with_capacity(framebuffer.len() * 4);
	buffer.extend_from_slice(framebuffer);

	//return screenshot
	Ok((buffer))
}

fn save_screenshot(path: String, raw: Vec<u32>) -> Result<(), std::io::Error> {
	let file = try!(File::create(path));

	//Convert the u32 pixels into rgba structs for the image library
	let mut buffer: Vec<u8> = Vec::with_capacity(raw.len() * 4);
	for val in raw {
		buffer.push((val >> 24) as u8);
		buffer.push((val >> 16) as u8);
		buffer.push((val >> 8) as u8);
		buffer.push((val & 0xFF) as u8);
	}
	let encoder = image::png::PNGEncoder::new(file);
	encoder.encode(buffer.as_slice(), rustboy_core::WIDTH as u32, rustboy_core::HEIGHT as u32, image::ColorType::RGBA(8))
}

#[test]
#[allow(unused_must_use)]
fn test_rom_runner() {
	create_dir("tests/results");	//Create a directory for screenshots
	let mut log = File::create("tests/results/test_log.txt").expect("Failed to create log file.");

	let dir = read_dir("tests/test_roms").expect("Test rom directory doesn't exist. Place test roms in tests/test_roms to run them.");

	for item in dir {
		if let Ok(ref entry) = item {
			writeln!(&mut log, "Running rom file {:?}", entry.file_name()).unwrap();

			let file_path = entry.path().into_os_string().into_string().unwrap();
			let gb_result = run_test_rom(file_path);
			if let Ok(screenshot) = gb_result {
				let screenshot_path = format!("tests/results/{}.png", entry.file_name().into_string().unwrap());
				let sc_result = save_screenshot(screenshot_path, screenshot);
				match sc_result {
					Ok(()) => {},
					Err(error) => writeln!(&mut log, "{}", error).unwrap(), /* Error saving screenshot */
				};
			}
			else if let Err(error) = gb_result {
				writeln!(&mut log, "Running rom {:?} failed with error: {}", entry.file_name(), error).unwrap();
			}
		}
	}
}