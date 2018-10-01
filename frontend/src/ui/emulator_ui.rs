use std::thread::spawn;
use std::sync::mpsc::{Sender, Receiver};
use std::os::raw::c_void;

use imgui::{Ui, ImGuiCond, StyleVar, ImVec2};
use imgui_sys;
use imgui_sys::ImU32;
use nfd;
use agb_core;

use super::Gui;
use ::events::FrontendEvent;

pub struct EmulatorUi {
	size: (f32, f32),
	sender: Sender<FrontendEvent>,
	screen_texture_id: u32
}

impl EmulatorUi {
	pub fn new(window_size: (f32, f32), sender: Sender<FrontendEvent>, screen_texture_id: u32) -> EmulatorUi {
		EmulatorUi {
			size: window_size,
			sender: sender,
			screen_texture_id: screen_texture_id
		}
	}

	pub fn update_size(&mut self, size: (f32, f32)) {
		self.size = size
	}
}

impl Gui for EmulatorUi {
	fn build_ui<'ui>(&mut self, ui: &'ui Ui) {
		ui.with_style_vars(&[StyleVar::WindowRounding(0.0)], || {
			ui.window(im_str!("AGB"))
				.title_bar(false)
				.resizable(false)
				.scrollable(false)
				.horizontal_scrollbar(false)
				.menu_bar(true)
				.size(self.size, ImGuiCond::Always)
				.position((0.0, 0.0), ImGuiCond::Always)
				.build(|| {
					ui.menu_bar(|| {
						ui.menu(im_str!("File")).build(|| {
							if ui.menu_item(im_str!("Load ROM")).build() {
								//open file browser widget (seperate thread as to not block execution of the ui thread)
								let sender = self.sender.clone();
								spawn(move || {
									match nfd::open_file_dialog(None, None) {
										Ok(nfd::Response::Okay(path)) => {
											let _ = sender.send(FrontendEvent::LoadRom(path));
										},
										Ok(nfd::Response::OkayMultiple(paths)) => {
											if let Some(ref path) = paths.first() {
												let _ = sender.send(FrontendEvent::LoadRom(path.to_string()));
											}
										},
										Err(e) => { println!("Failed to get file from file_dialog: {:?}", e) },
										_ => {}
									}
								});
							}
							if ui.menu_item(im_str!("Exit")).build() {
								//exit emulator
								let _ = self.sender.send(FrontendEvent::Exit);
							}
						});
					});
					ui.child_frame(im_str!("emulator"), (0.0, 0.0))
						.build(|| {
							//TODO: draw the emulators screen here
							/*unsafe {
								let draw_list = imgui_sys::igGetWindowDrawList();
								imgui_sys::ImDrawList_AddImage(draw_list, self.screen_texture_id as *mut c_void, ImVec2::new(0.0, 0.0), ImVec2::new(agb_core::WIDTH as f32, agb_core::HEIGHT as f32), ImVec2::new(0.0, 0.0), ImVec2::new(1.1, 1.1), 0xFFFFFFFF as ImU32);
							}*/
						});
				});
		});
	}
}
