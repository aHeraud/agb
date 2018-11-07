/* A very simple front end to test agb-core (https://github.com/aHeraud/agb)*/

extern crate agb_core;
extern crate sdl2;
extern crate image;
extern crate clap;

mod debugger;

use std::sync::mpsc::sync_channel;
use std::thread;
use std::time::Duration;
use std::thread::sleep;
use std::collections::HashMap;
use std::fs::File;
use std::io::{stdin, stdout, Read, Write, Error};
use std::path::Path;
use std::num::ParseIntError;
use std::sync::{Arc, Mutex};
use std::ops::DerefMut;
use std::net::{TcpListener, TcpStream, SocketAddr, IpAddr, Ipv4Addr};

use agb_core::gameboy::Gameboy;
use agb_core::gameboy::debugger::DebuggerInterface;

use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use clap::{Arg, App};

const DEFAULT_SCALE: usize = 2;

fn main() {
	/* Create and initialize gameboy */

	//Parse command line arguments
	let matches = App::new("agb")
		.version("0.1")
		.author("Achille Heraud <achille@heraud.xyz>")
		.about("A GameBoy Emulator")
		.arg(Arg::with_name("rom")
			.long("rom")
			.takes_value(true)
			.value_name("FILE")
			.required(true))
		.arg(Arg::with_name("ram")
			.long("ram")
			.takes_value(true)
			.value_name("FILE")
			.required(false))
		.arg(Arg::with_name("paused")
			.long("pause")
			.short("p")
			.required(false))
		.arg(Arg::with_name("print_serial")
			.help("print data written to the serial port")
			.long("print_serial")
			.conflicts_with_all(&["listen", "connect"])
			.required(false))
		.arg(Arg::with_name("listen")
			.help("listen for remote connections (for serial communication over tcp/ip)")
			.long("listen")
			.takes_value(true)
			.value_name("PORT")
			.conflicts_with("connect")
			.required(false))
		.arg(Arg::with_name("connect")
			.help("provide to ip:port to connect to a remote host (for serial communication over tcp/ip")
			.long("connect")
			.takes_value(true)
			.value_name("IP:PORT")
			.required(false))
		.get_matches();

	let rom = read_file(matches.value_of("rom").unwrap()).expect("Could not open rom file.");
	let ram: Option<Box<[u8]>> = if let Some(ram_path) = matches.value_of("ram") {
		Some(read_file(ram_path).expect("failed to read ram file"))
	}
	else {
		None
	};

	let start_paused: bool = matches.occurrences_of("paused") > 0;

	let mut gameboy = Gameboy::new(rom, ram).expect("Failed to initialize gameboy");
	let paused: Arc<Mutex<bool>> = Arc::new(Mutex::new(start_paused));
	gameboy.debugger.enable();
	{
		let paused = paused.clone();
		gameboy.register_breakpoint_callback(move |breakpoint| {
			println!("triggered breakpoint access_type: {:?}, address: 0x{:x}", breakpoint.access_type, breakpoint.address);
			let mut paused = paused.lock().unwrap();
			*paused = true;
		});
	}

	let mut state: Option<Vec<u8>> = None;

	if let Some(ref port_str) = matches.value_of("listen") {
		// set up a tcp socket to accept incoming connections
		if let Ok(port) = u16::from_str_radix(port_str, 10) {
			let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127,0,0,1)), port);
			let listener = TcpListener::bind(addr).unwrap();
			let (local_sender, local_receiver) = gameboy.create_serial_channels();

			println!("waiting for connection...");
			let (mut stream, remote_addr) = listener.accept().unwrap();
			println!("connected to {}", remote_addr);

			stream.set_nonblocking(false).expect("failed to set tcp socket to nonblocking mode");
			stream.set_nodelay(true).expect("failed to set tcp socket nodelay mode");

			//create sender thread
			{
				let mut stream = stream.try_clone().unwrap();
				thread::spawn(move || {
					let mut buffer: [u8; 1] = [0;1];
					loop {
						let data = local_receiver.recv().unwrap();
						buffer[0] = data;
						if stream.write(&buffer[..]).is_err() {
							return;
						}
					}
				});
			}

			//create receiver thread
			thread::spawn(move || {
				let mut buffer: [u8; 1] = [0;1];
				loop {
					if stream.read(&mut buffer[..]).is_err() {
						return;
					}
					local_sender.send(buffer[0]).unwrap();
				}
			});
		}
		else {
			println!("Invalid port: port must be an integer in the range 0 to 65535.");
			return;
		}
	}
	else if let Some(ref addr) = matches.value_of("connect") {
		let (local_sender, local_receiver) = gameboy.create_serial_channels();
		let addr: SocketAddr = addr.parse().unwrap();
		println!("attempting to connect...");
		let mut stream = TcpStream::connect(addr).unwrap();
		println!("connected!");

		stream.set_nonblocking(false).expect("failed to set tcp socket to nonblocking mode");
		stream.set_nodelay(true).expect("failed to set tcp socket nodelay mode");

		//create sender thread
		{
			let mut stream = stream.try_clone().unwrap();
			thread::spawn(move || {
				let mut buffer: [u8; 1] = [0;1];
				loop {
					let data = local_receiver.recv().unwrap();
					buffer[0] = data;
					if stream.write(&buffer[..]).is_err() {
						return;
					}
				}
			});
		}

		//create receiver thread
		thread::spawn(move || {
			let mut buffer: [u8; 1] = [0;1];
			loop {
				if stream.read(&mut buffer[..]).is_err() {
					return;
				}
				local_sender.send(buffer[0]).unwrap();
			}
		});
	}
	else if matches.occurrences_of("print_serial") > 0 {
		let (sender, reciever) = gameboy.create_serial_channels();
		thread::spawn(move || {
			loop {
				let data = reciever.recv().unwrap();
				sender.send(0xFF).unwrap();
				print!("{}", data as char);
				stdout().flush().ok().expect("could not flush stdout");
			}
		});
	}

	//debugger text input
	let (tx, rx) = sync_channel(0);
	let main_handle = thread::current();
	thread::spawn(move || {
		println!("debugger enabled - you can type debugger commands here! Type 'help' for more info.");
		loop {
			let mut buf: String = String::new();
			stdin().read_line(&mut buf).unwrap();
			tx.send(buf).unwrap();
			main_handle.unpark();
		}
	});

	//Keys
	let mut keymap: HashMap<Keycode, agb_core::gameboy::Key> = HashMap::new();
	keymap.insert(Keycode::Up, agb_core::gameboy::Key::Up);
	keymap.insert(Keycode::Down, agb_core::gameboy::Key::Down);
	keymap.insert(Keycode::Left, agb_core::gameboy::Key::Left);
	keymap.insert(Keycode::Right, agb_core::gameboy::Key::Right);
	keymap.insert(Keycode::X, agb_core::gameboy::Key::A);
	keymap.insert(Keycode::Z, agb_core::gameboy::Key::B);
	keymap.insert(Keycode::C, agb_core::gameboy::Key::Select);
	keymap.insert(Keycode::V, agb_core::gameboy::Key::Start);

	/* Setup for sdl */
	let sdl_context = sdl2::init().expect("Failed to initialize sdl2");
	let video_subsystem = sdl_context.video().expect("Failed to initialize sdl2 video subsystem");
	let timer_subsystem = sdl_context.timer().expect("Failed to initialize sdl2 timer subsystem");

	//Set resolution
	let width: u32 = (agb_core::WIDTH * DEFAULT_SCALE) as u32;
	let height: u32 = (agb_core::HEIGHT * DEFAULT_SCALE) as u32;

	let window = video_subsystem.window("agb", width, height)
		.position_centered()
		.opengl()
		.build()
		.expect("Failed to create window");

	let mut renderer = window.renderer().build().unwrap();
	let mut event_pump = sdl_context.event_pump().unwrap();
	let mut draw = |gameboy: &mut Gameboy| {
		renderer.set_draw_color(Color::RGB(80, 120, 120));
		renderer.clear();
		const WIDTH: usize = agb_core::WIDTH;
		const HEIGHT: usize = agb_core::HEIGHT;

		let data: &mut[u8] = unsafe {
			//#[allow(mutable_transmutes)]
			//std::mem::transmute::<&[u32], &mut[u8]>(gbc.get_framebuffer())
			let temp: &mut [u32] = gameboy.get_framebuffer_mut();
			std::mem::transmute::<&mut[u32], &mut[u8]>(temp)
		};

		let mut texture = renderer.create_texture_streaming(PixelFormatEnum::RGBA8888, WIDTH as u32, HEIGHT as u32).unwrap();
		let _ = texture.update(None, data, WIDTH * 4);
		let rect: Rect = renderer.viewport();
		let _ = renderer.copy(&texture, None, Some(rect));
		renderer.present();
	};

	//Get timer frequency
	let frequency: u64 = timer_subsystem.performance_frequency();

	'running: loop {
		//wait for input from the debugger, but don't wait forever since
		//we don't want the block the gui thread forever
		{
			let paused = paused.lock().unwrap();
			if *paused {
				std::thread::park_timeout(Duration::new(0, 100000000));
			}
		}

		let frame_start: u64 = timer_subsystem.performance_counter();

		//handle debugger input
		if let Ok(input) = rx.try_recv() {
			if input.trim() == "quit" || input.trim() == "exit" {
				break 'running;
			}
			else {
				let mut paused = paused.lock().unwrap();
				debugger::debug(input, &mut gameboy, paused.deref_mut(), &mut state);
			}
		}

		for event in event_pump.poll_iter() {
			match event {
				Event::KeyDown { keycode, .. } => {
					if keycode.is_some() {
						let key = keymap.get(&keycode.unwrap());
						if key.is_some() {
							gameboy.keydown(*key.unwrap());
						}
					}
				},
				Event::KeyUp { keycode, .. } => {
					if keycode.is_some() {
						let key = keymap.get(&keycode.unwrap());
						if key.is_some() {
							gameboy.keyup(*key.unwrap());
						}
					}
				},
				Event::Quit {..} => {
					break 'running;
				},
				_ => {},
			};
		}

		let paused = {
				*paused.lock().unwrap()
		};
		if !paused {
			gameboy.emulate(Duration::from_millis(1000 / 60));
			draw(&mut gameboy);

			//60hz
			let frame_end: u64 = timer_subsystem.performance_counter();
			let frame_duration: u64 = frame_end - frame_start;
			let ms: u64 = (frame_duration * 1000) / frequency;
			if ms < 1000/60 {
				let duration = Duration::from_millis((1000/60) - ms);
				sleep(duration);
			}
		}
	}
}

pub fn read_file<P: AsRef<Path>>(path: P) -> Result<Box<[u8]>, Error> {
	let mut file = try!(File::open(path));
	let mut buffer = Vec::new();
	try!(file.read_to_end(&mut buffer));
	Ok(buffer.into_boxed_slice())
}

///accepts prefixes (none for base 10, 0x for hex, 0b for binary)
///from_str_radix isn't part of a trait so it can't be generic
pub fn parse_u8(s: &str) -> Result<u8, ParseIntError> {
	if s.starts_with("0x") {
		u8::from_str_radix(&s[2..], 16)
	}
	else if s.starts_with("0b") {
		u8::from_str_radix(&s[2..], 2)
	}
	else {
		u8::from_str_radix(s, 10)
	}
}

///accepts prefixes (none for base 10, 0x for hex, 0b for binary)
pub fn parse_u16(s: &str) -> Result<u16, ParseIntError> {
	if s.starts_with("0x") {
		u16::from_str_radix(&s[2..], 16)
	}
	else if s.starts_with("0b") {
		u16::from_str_radix(&s[2..], 2)
	}
	else {
		u16::from_str_radix(s, 10)
	}
}

///accepts prefixes (none for base 10, 0x for hex, 0b for binary)
pub fn parse_usize(s: &str) -> Result<usize, ParseIntError> {
	if s.starts_with("0x") {
		usize::from_str_radix(&s[2..], 16)
	}
	else if s.starts_with("0b") {
		usize::from_str_radix(&s[2..], 2)
	}
	else {
		usize::from_str_radix(s, 10)
	}
}
