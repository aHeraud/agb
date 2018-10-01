use std::time::Duration;

use glutin;
use glutin::{WindowBuilder, ContextBuilder, EventsLoop};

use glium::{Display, Surface};

use imgui::{ImGui, FrameSize};
use imgui_glium_renderer::Renderer;

use super::{MouseState, Gui};

pub struct AppWindow {
	pub display: Display,
	pub events_loop: EventsLoop,
	pub hidpi_factor: f64,
	pub imgui: ImGui,
	pub imgui_renderer: Renderer
}

impl AppWindow {
	pub fn new(window_builder: WindowBuilder) -> AppWindow {
		let events_loop = EventsLoop::new();
		let context = ContextBuilder::new();
		let display = Display::new(window_builder, context, &events_loop).unwrap();

		let mut imgui = ImGui::init();
		imgui.set_ini_filename(None);
		let hidpi_factor = {
			let window = display.gl_window();
			window.get_hidpi_factor().round()
		};
		imgui.fonts().add_default_font();
		imgui.set_font_global_scale((1.0 / hidpi_factor) as f32);

		let renderer = Renderer::init(&mut imgui, &display).unwrap();

		AppWindow {
			display: display,
			events_loop: events_loop,
			hidpi_factor,
			imgui: imgui,
			imgui_renderer: renderer
		}
	}

	pub fn get_events(&mut self) -> Vec<glutin::WindowEvent> {
		let mut events = Vec::new();
		self.events_loop.poll_events(|event| {
			if let glutin::Event::WindowEvent { event, .. } = event {
				events.push(event);
			}
		});
		events
	}

	pub fn update_mouse(&mut self, mouse_state: &mut MouseState) {
		let (x,y) = mouse_state.position;
		self.imgui.set_mouse_pos(x as f32,y as f32);
		self.imgui.set_mouse_down(mouse_state.pressed);
		self.imgui.set_mouse_wheel(mouse_state.wheel);
		mouse_state.wheel = 0.0_f32;
	}

	pub fn render_ui<'a, G: Gui>(&'a mut self, gui: &mut G, delta: Duration) {
		let window = self.display.gl_window();
		let delta_seconds = delta.as_secs() as f32 + (delta.subsec_nanos() as f32 / 1_000_000_000.0);
		let physical_size = window
			.get_inner_size()
			.unwrap()
			.to_physical(window.get_hidpi_factor());
		let logical_size = physical_size.to_logical(self.hidpi_factor);
		let frame_size = FrameSize {
			logical_size: logical_size.into(),
			hidpi_factor: self.hidpi_factor
		};

		let ui = self.imgui.frame(frame_size, delta_seconds);
		gui.build_ui(&ui);

		let mut target = self.display.draw();
		target.clear_color(0.2f32, 0.2f32, 0.2f32, 1.0f32);

		self.imgui_renderer.render(&mut target, ui).unwrap();
		target.finish().unwrap();
	}
}
