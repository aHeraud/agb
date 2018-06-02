use std::str::SplitWhitespace;
use std::fs::File;

use agb_core::gameboy::Gameboy;
use agb_core::gameboy::assembly;
use agb_core::gameboy::debugger::{Breakpoint, DebuggerInterface, AccessType};
use super::{parse_u16, parse_u8, parse_usize};

use image;

pub fn debug(input: String, gameboy: &mut Gameboy, paused: &mut bool) {
	let mut command = input.trim().split_whitespace();
	if let Some(next) = command.next() {
		match next {
			"breakpoint" => breakpoint(&mut command, gameboy),
			"step" => {
				gameboy.debug_step();
			},
			"continue" | "resume" => {
				*paused = false;
				gameboy.debug_step();  //if paused on a breakpoint, need to step over it
			},
			"pause" | "break" => {
				*paused = true;
			},
			"registers" => {
				println!("{:#?}", gameboy.get_registers());
			},
			"memory" => memory(&mut command, gameboy),
			"assembly" => assembly(gameboy),
			"dump_tiles" => dump_tiles(&mut command, gameboy),
			"dump_bg" => dump_bg(&mut command, gameboy),
			"reset" => {
				gameboy.reset();
			},
			"help" => {
				println!("available commands are:\n\
				breakpoint add <type> <address>  - add a breakpoint at <address>, valid types are {{ execute, jump, read, write }}\n\
				breakpoint list           - get a list of breakpoints\n\
				breakpoint remove <index> - remove the breakpoint with index <index> (from list)\n\
				step                      - step forward 1 instruction\n\
				continue | resume         - continue execution\n\
				pause | break             - pause execution\n\
				registers                 - print out the contents of the registers\n\
				memory read <address>     - read the byte at <address>\n\
				memory write <address> <value> - write <value> at <address>\n\
				assembly                  - print out the dissasembly of the current pc\n\
				reset                     - reset the gameboy (keeps breakpoints and any rom/ram patches)\n\
				dump_tiles <filename>     - dumps the tiles in vram as an image named <filename>.png (or tiles.png if no filename is provided)\n\
				dump_bg <filename>        - dumps the background as an image to <filename>.png (or bg.png if no filename is provided)
				quit | exit               - terminate the emulator");
			},
			_ => { println!("invalid command (try typing 'help')"); }
		};
	}
}

pub fn breakpoint(command: &mut SplitWhitespace, gameboy: &mut Gameboy) {
	match command.next() {
		Some(subcommand) => {
			match subcommand {
				"add" => {
					let access_type: Option<AccessType> = match command.next() {
						Some("execute") => Some(AccessType::Execute),
						Some("read") => Some(AccessType::Read),
						Some("write") => Some(AccessType::Write),
						Some("jump") => Some(AccessType::Jump),
						Some(_) => {
							println!("invalid access type");
							None
						},
						None => {
							println!("missing access type");
							None
						}
					};
					let address: Option<u16> = match command.next() {
						Some(address_literal) => {
							match parse_u16(address_literal) {
								//TODO: other access types
								Ok(address) => Some(address),
								Err(_) => {
									println!("invalid address");
									None
								},
							}
						},
						None => {
							println!("missing address");
							None
						},
					};
					if let (Some(access_type), Some(address)) = (access_type, address) {
						let breakpoint = Breakpoint::new(address, access_type);
						gameboy.add_breakpoint(breakpoint);
					}

				},
				"list" => {
					let breakpoints = gameboy.get_breakpoints();
					if breakpoints.len() == 0 {
						println!("no breakpoints");
					}
					for (number, breakpoint) in breakpoints.iter().enumerate() {
						println!("{}: address: 0x{:x}, access_type: {:?}", number, breakpoint.address, breakpoint.access_type);
					}
				},
				"remove" => {
					match command.next() {
						Some(parameter) => {
							match parse_usize(parameter) {
								Ok(index) => {
									match gameboy.remove_breakpoint(index) {
										Ok(breakpoint) => {
											println!("removed breakpoint {{ address: 0x{:x}, access_type: {:?} }}", breakpoint.address, breakpoint.access_type);
										},
										Err(_) => {
											println!("the breakpoint you are trying to remove doesn't exist");
										}
									}
								},
								Err(_) => {
									println!("invalid argument: must be an integer");
								}
							};
						},
						None => {
							println!("invalid usage: specify which interrupt to remove (use the number you got from breakpoint list)");
						}
					}
				}
				_ => { println!("invalid usage: subcommands of breakpoint are {{ add, list, remove }}"); },
			};
		},
		None => println!("invalid usage: missing subcommand"),
	};
}

pub fn memory(command: &mut SplitWhitespace, gameboy: &mut Gameboy) {
	match command.next() {
		Some(subcommand) => {
			match subcommand {
				"read" =>  {
					match command.next() {
						Some(address_literal) => {
							match parse_u16(address_literal) {
								Ok(address) => {
									let value = gameboy.read_memory(address);
									println!("[0x{:x}] = 0x{:x}", address, value);
								}
								Err(_) => {
									println!("invalid address");
								}
							}
						}
						None => {
							println!("you need to specify an address");
						}
					};
				},
				"write" => {
					match command.next() {
						Some(address_literal) => {
							match command.next() {
								Some(value_literal) => {
									let address = parse_u16(address_literal);
									let value = parse_u8(value_literal);
									match (address, value) {
										(Ok(address), Ok(value)) => {
											gameboy.write_memory(address, value);
										},
										(address, value) => {
											if address.is_err() { println!("invalid address"); }
											if value.is_err() { println!("invalid value"); }
										}
									};
								},
								None => {
									println!("invalid usage: specify a value to write")
								}
							}
						}
						None => {
							println!("invalid usage: specify an address");
						}
					};
				},
				_ => {},
			};
		},
		None => {}
	};
}

pub fn dump_tiles(command: &mut SplitWhitespace, gameboy: &mut Gameboy) {
	let path = match command.next() {
		Some(arg) => {
			let mut path = String::from(arg);
			if !path.ends_with(".png") {
				path.push_str(".png");
			}
			path
		},
		None => String::from("tiles.png")
	};
	let raw = gameboy.dump_tiles();
	let file = File::create(path);
	match file {
		Ok(file) => {
			//Convert the u32 pixels into rgba structs for the image library
			let mut buffer: Vec<u8> = Vec::with_capacity(raw.data.len() * 4);
			for val in raw.data.iter() {
				buffer.push((val >> 24) as u8);
				buffer.push((val >> 16) as u8);
				buffer.push((val >> 8) as u8);
				buffer.push((val & 0xFF) as u8);
			}
			let encoder = image::png::PNGEncoder::new(file);
			match encoder.encode(buffer.as_slice(), raw.width as u32, raw.height as u32, image::ColorType::RGBA(8)) {
				Ok(_) => {},
				Err(_) => println!("failed to save tile data to disk")
			};
		},
		Err(e) => println!("{}", e),
	};
}

pub fn dump_bg(command: &mut SplitWhitespace, gameboy: &mut Gameboy) {
	let path = match command.next() {
		Some(arg) => {
			let mut path = String::from(arg);
			if !path.ends_with(".png") {
				path.push_str(".png");
			}
			path
		},
		None => String::from("bg.png"),
	};
	let raw = gameboy.dump_bg();
	let file = File::create(path);
	match file {
		Ok(file) => {
			//Convert the u32 pixels into rgba structs for the image library
			let mut buffer: Vec<u8> = Vec::with_capacity(raw.data.len() * 4);
			for val in raw.data.iter() {
				buffer.push((val >> 24) as u8);
				buffer.push((val >> 16) as u8);
				buffer.push((val >> 8) as u8);
				buffer.push((val & 0xFF) as u8);
			}
			let encoder = image::png::PNGEncoder::new(file);
			match encoder.encode(buffer.as_slice(), raw.width as u32, raw.height as u32, image::ColorType::RGBA(8)) {
				Ok(_) => {},
				Err(_) => println!("failed to save tile data to disk")
			};
		},
		Err(e) => println!("{}", e),
	};
}

pub fn assembly(gameboy: &mut Gameboy) {
	use std::cmp::min;

	let pc = gameboy.get_registers().pc;
	let start:usize = pc as usize;
	let end = min(start + 5, 0xFFFF);
	let data = gameboy.read_range(start as u16, end as u16).unwrap(); //largest opcode is 3 bytes
	let after = gameboy.get_assembly(&data);

	let mut offset: usize = 0;
	for (line,op) in after.iter().enumerate() {
		match line {
			0 => { println!("{:04X}: {} <---", (offset + start) as u16, op); },
			_ => { println!("{:04X}: {}", (offset + start) as u16, op); },
		};
		offset += assembly::INSTRUCTION_LENGTH[data[offset] as usize];
	}
}
