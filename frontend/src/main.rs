extern crate glutin;
extern crate glium;

extern crate nfd;

#[macro_use]
extern crate imgui;
extern crate imgui_sys;
extern crate imgui_glium_renderer;

extern crate agb_core;

use std::time::Instant;
use std::sync::mpsc::channel;
use std::path::Path;
use std::fs::File;

use glutin::{WindowEvent, WindowBuilder};
use glium::{GlObject};
use glium::texture::{UncompressedFloatFormat, MipmapsOption};
use glium::texture::Texture2d;

use agb_core::gameboy::Gameboy;

pub mod events;
mod ui;

use events::FrontendEvent;
use ui::{AppWindow, EmulatorUi};

const DEFAULT_SCALE: f64 = 3.0_f64;

fn main() {
	let window_builder = WindowBuilder::new()
		.with_title("AGB")
		.with_dimensions(glutin::dpi::LogicalSize::new(agb_core::WIDTH as f64 * DEFAULT_SCALE, agb_core::HEIGHT as f64 * DEFAULT_SCALE));
	let mut window = AppWindow::new(window_builder);

	//set up OpenGL stuff to draw emulator screen to the window
	//see https://github.com/ocornut/imgui/issues/497
	//and https://github.com/Gekkio/imgui-rs/pull/111
	let mut texture = Texture2d::empty_with_format(&window.display, UncompressedFloatFormat::U8U8U8U8, MipmapsOption::NoMipmap, agb_core::WIDTH as u32, agb_core::HEIGHT as u32).unwrap();

	let (frontend_sender, frontend_receiver) = channel::<FrontendEvent>();
	let mut emulator_gui = {
		let window = window.display.gl_window();
		let logical_size = window.get_inner_size().unwrap();
		let physical_size = logical_size.to_physical(window.get_hidpi_factor());
		EmulatorUi::new((physical_size.width as f32, physical_size.height as f32), frontend_sender, texture.get_id())
	};
	let mut emulator_opt: Option<Gameboy> = None;

	let mut mouse_state = ui::MouseState:: default();

	let mut last_frame = Instant::now();
	let mut quit = false;
	loop {
		let events = window.get_events();
		events.into_iter().for_each(|event| {
			use glutin::ElementState;
			use glutin::MouseButton;

			match event {
				WindowEvent::Resized(size) => {
					let (width, height): (f64, f64) = size.to_physical(window.display.gl_window().get_hidpi_factor()).into();
					emulator_gui.update_size((width as f32, height as f32));
				},
				WindowEvent::CloseRequested => quit = true,
				WindowEvent::CursorMoved { position, .. } => {
					mouse_state.position = position
						.to_physical(window.display.gl_window().get_hidpi_factor())
						.to_logical(window.hidpi_factor)
						.into();
				},
				WindowEvent::MouseInput { state, button, .. } => {
					match button {
						MouseButton::Left => mouse_state.pressed[0] = state == ElementState::Pressed,
						MouseButton::Right => mouse_state.pressed[0] = state == ElementState::Pressed,
						MouseButton::Middle => mouse_state.pressed[0] = state == ElementState::Pressed,
						_ => {}
					}
				}
				_ => {}
			};
		});

		frontend_receiver.try_iter().for_each(|event| {
			match event {
				FrontendEvent::LoadRom(path) => {
					match read_file(&path) {
						Ok(buffer) => {
							//TODO: load save
							match Gameboy::new(buffer, None) {
								Ok(gameboy) => emulator_opt = Some(gameboy),
								Err(e) => {
									//TODO: display this in a message box
									println!("Failed to initialize emulator: {:?}", e);
								}
							}
						},
						Err(e) => {
							//TODO: display this in a message box
							println!("Failed to read {:?}: {:?}", path, e);
						}
					}
				},
				FrontendEvent::Exit => quit = true
			}
		});

		let now = Instant::now();
		let delta = now - last_frame;
		last_frame = now;

		if let Some(ref mut gameboy) = emulator_opt {
			let last = gameboy.get_frame_counter();
			gameboy.emulate(delta);
			if last != gameboy.get_frame_counter() {
				//upload new frame to texture
				{
					//let buffer = gameboy.get_framebuffer().clone().to_vec();
					//let image = glium::texture::RawImage2d::from_raw_rgba(buffer, (agb_core::WIDTH as u32, agb_core::HEIGHT as u32));
					//texture.write(Rect{ left: 0, bottom: 0, width: agb_core::WIDTH as u32, height: agb_core::HEIGHT as u32}, image);
				}
			}
		}

		window.update_mouse(&mut mouse_state);
		window.render_ui(&mut emulator_gui, delta);

		if quit {
			break;
		}
	}
}

pub fn read_file<P: AsRef<Path>>(path: &P) -> Result<Box<[u8]>, std::io::Error> {
	use std::io::Read;

	let mut file = try!(File::open(path));
	let mut buffer = Vec::new();
	try!(file.read_to_end(&mut buffer));
	Ok(buffer.into_boxed_slice())
}
