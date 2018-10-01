use std::collections::HashMap;

use glutin::VirtualKeyCode;
use imgui::{ImGui, Ui};

mod window;
mod emulator_ui;

pub use self::window::AppWindow;
pub use self::emulator_ui::EmulatorUi;

pub trait Gui {
	fn build_ui<'ui>(&mut self, ui: &Ui<'ui>);
}

#[derive(Clone, Default, Debug)]
pub struct MouseState {
	pub position: (i32, i32),
	pub pressed: [bool; 5],
	pub wheel: f32
}

fn generate_imgui_keyboard_mappings() -> HashMap<VirtualKeyCode, i32> {
	let mut mapping = HashMap::new();
	mapping.insert(VirtualKeyCode::Tab, 0);
	mapping.insert(VirtualKeyCode::Left, 1);
	mapping.insert(VirtualKeyCode::Right, 2);
	mapping.insert(VirtualKeyCode::Up, 3);
	mapping.insert(VirtualKeyCode::Down, 4);
	mapping.insert(VirtualKeyCode::PageUp, 5);
	mapping.insert(VirtualKeyCode::PageDown, 6);
	mapping.insert(VirtualKeyCode::Home, 7);
	mapping.insert(VirtualKeyCode::End, 8);
	mapping.insert(VirtualKeyCode::Delete, 9);
	mapping.insert(VirtualKeyCode::Back, 10);
	mapping.insert(VirtualKeyCode::Return, 11);
	mapping.insert(VirtualKeyCode::Escape, 12);
	mapping.insert(VirtualKeyCode::A, 13);
	mapping.insert(VirtualKeyCode::C, 14);
	mapping.insert(VirtualKeyCode::V, 15);
	mapping.insert(VirtualKeyCode::X, 16);
	mapping.insert(VirtualKeyCode::Y, 17);
	mapping.insert(VirtualKeyCode::Z, 18);
	mapping
}
