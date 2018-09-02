extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

extern crate agb_core;

mod manifest {
	use std::convert::Into;
	use std::time::Duration;

	use agb_core::gameboy::{Gameboy, Mode};
	use agb_core::gameboy::debugger::DebuggerInterface;

	#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
	pub enum HardwareType {
		DMG, CGB
	}

	impl Into<Mode> for HardwareType {
		fn into(self) -> Mode {
			match self {
				HardwareType::DMG => Mode::DMG,
				HardwareType::CGB => Mode::CGB
			}
		}
	}

	#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
	pub enum TestDuration {
		Cycles(usize), /* tests aren't guaranteed to end at the exact cycle count, instead the test will end after the last instruction that puts the cycle counter at or greater than the target cycle count. */
		Time(Duration),
		Opcode(u8) /* Run until a specific opcode is executed */
	}

	#[derive(Deserialize, Debug, Clone)]
	pub struct RegisterAssertions {
		a: Option<u8>,
		f: Option<u8>,
		b: Option<u8>,
		c: Option<u8>,
		d: Option<u8>,
		e: Option<u8>,
		h: Option<u8>,
		l: Option<u8>,
		sp: Option<u16>,
		pc: Option<u16>
	}

	impl RegisterAssertions {
		pub fn check(&self, gameboy: &Gameboy) {
			let registers = gameboy.get_registers();
			if let Some(value) = self.a {
				assert_eq!(value, registers.a);
			}
			if let Some(value) = self.f {
				assert_eq!(value, registers.f);
			}
			if let Some(value) = self.b {
				assert_eq!(value, registers.b);
			}
			if let Some(value) = self.c {
				assert_eq!(value, registers.c);
			}
			if let Some(value) = self.d {
				assert_eq!(value, registers.d);
			}
			if let Some(value) = self.e {
				assert_eq!(value, registers.e);
			}
			if let Some(value) = self.h {
				assert_eq!(value, registers.h);
			}
			if let Some(value) = self.l {
				assert_eq!(value, registers.l);
			}
			if let Some(value) = self.sp {
				assert_eq!(value, registers.sp);
			}
			if let Some(value) = self.pc {
				assert_eq!(value, registers.pc);
			}
		}
	}

	#[derive(Deserialize, Debug, Clone)]
	pub struct MemoryAssertion {
		pub address: u16,
		pub value: u8
	}

	#[derive(Deserialize, Debug, Clone)]
	pub struct TestManifest {
		pub rom_path: String,
		pub sram_path: Option<String>,
		pub hardware_versions: Vec<HardwareType>,
		pub duration: TestDuration,
		pub registers: RegisterAssertions,
		pub memory: Vec<MemoryAssertion>
	}
}

pub mod test_runner {
	use std::fs::File;
	use std::io::{Read, Error};
	use std::path::Path;
	use std::vec::Vec;

	use serde_json;

	use agb_core::gameboy::Gameboy;
	use agb_core::gameboy::debugger::DebuggerInterface;

	use ::manifest::*;

	pub fn read_file<P: AsRef<Path>>(path: P) -> Result<Box<[u8]>, Error> {
		let mut file = try!(File::open(path));
		let mut buffer = Vec::new();
		let result = file.read_to_end(&mut buffer);
		match result {
			Ok(_) => Ok(buffer.into_boxed_slice()),
			Err(err) => Err(err),
		}
	}

	pub fn run_test<P: AsRef<Path>>(manifest_path: P) {
		let raw_manifest = {
			let mut file = File::open(manifest_path).expect("failed to open manifest file");
			let mut contents = String::new();
			file.read_to_string(&mut contents).expect("failed to read manifest file");
			contents
		};

		let manifest: TestManifest = serde_json::from_str(&raw_manifest).expect("failed to parse manifest file");

		let rom = read_file(manifest.rom_path).expect("failed to load rom specified in manifest");
		let sram = match manifest.sram_path {
			Some(path) => Some(read_file(path).expect("failed to load sram file specified in manifest")),
			None => None
		};

		let mut gameboy = Gameboy::new(rom, sram).expect("invalid rom file");

		match manifest.duration {
			TestDuration::Time(duration) => gameboy.emulate(duration),
			TestDuration::Cycles(target_cycles) => {
				while gameboy.get_cycle_counter() < target_cycles {
					gameboy.debug_step();
				}
			},
			TestDuration::Opcode(target_opcode) => {
				loop {
					let pc = gameboy.get_registers().pc;
					let next_opcode = gameboy.read_memory(pc);
					gameboy.debug_step();
					if next_opcode == target_opcode {
						break;
					}
				}
			}
		};

		//check test assertions
		manifest.registers.check(&gameboy);
		manifest.memory.into_iter().for_each(|memory_assertion| {
			let expected = memory_assertion.value;
			let actual = gameboy.read_memory(memory_assertion.address);
			assert_eq!(expected, actual);
		});
	}
}

macro_rules! run_tests {
	( $( $name:ident, $path:expr),+ ) => {
		$(
			#[test]
			#[allow(non_snake_case)]
			fn $name() {
				run_test($path)
			}
		)+
	}
}

/* mooneye-gb test roms
 * TODO: boot state test roms
 */
mod mooneye {
	use ::test_runner::run_test;
	run_tests!(
		add_sp_e_timing, "tests/manifests/mooneye-gb/add_sp_e_timing.json",
		call_timing, "tests/manifests/mooneye-gb/call_timing.json",
		call_timing2, "tests/manifests/mooneye-gb/call_timing2.json",
		call_cc_timing, "tests/manifests/mooneye-gb/call_cc_timing.json",
		call_cc_timing2, "tests/manifests/mooneye-gb/call_cc_timing2.json",
		di_timing_GS, "tests/manifests/mooneye-gb/di_timing-GS.json",
		div_timing, "tests/manifests/mooneye-gb/div_timing.json",
		ei_sequence, "tests/manifests/mooneye-gb/ei_sequence.json",
		ei_timing, "tests/manifests/mooneye-gb/ei_timing.json",
		halt_ime0_ei, "tests/manifests/mooneye-gb/halt_ime0_ei.json",
		halt_ime0_nointr_timing, "tests/manifests/mooneye-gb/halt_ime0_nointr_timing.json",
		halt_ime1_timing, "tests/manifests/mooneye-gb/halt_ime1_timing.json",
		halt_ime1_timing2_GS, "tests/manifests/mooneye-gb/halt_ime1_timing2-GS.json",
		if_ie_registers, "tests/manifests/mooneye-gb/if_ie_registers.json",
		intr_timing, "tests/manifests/mooneye-gb/intr_timing.json",
		jp_cc_timing, "tests/manifests/mooneye-gb/jp_cc_timing.json",
		ld_hl_sp_e_timing, "tests/manifests/mooneye-gb/ld_hl_sp_e_timing.json",
		oam_dma_restart, "tests/manifests/mooneye-gb/oam_dma_restart.json",
		oam_dma_timing, "tests/manifests/mooneye-gb/oam_dma_timing.json",
		pop_timing, "tests/manifests/mooneye-gb/pop_timing.json",
		push_timing, "tests/manifests/mooneye-gb/push_timing.json",
		rapid_di_ei, "tests/manifests/mooneye-gb/rapid_di_ei.json",
		ret_cc_timing, "tests/manifests/mooneye-gb/ret_cc_timing.json",
		ret_timing, "tests/manifests/mooneye-gb/ret_timing.json",
		reti_intr_timing, "tests/manifests/mooneye-gb/reti_intr_timing.json",
		reti_timing, "tests/manifests/mooneye-gb/reti_timing.json",
		rst_timing, "tests/manifests/mooneye-gb/rst_timing.json"
	);

	mod timer {
		use ::test_runner::run_test;
		run_tests!(
			div_write, "tests/manifests/mooneye-gb/timer/div_write.json",
			rapid_toggle, "tests/manifests/mooneye-gb/timer/rapid_toggle.json",
			tim00, "tests/manifests/mooneye-gb/timer/tim00.json",
			tim00_div_trigger, "tests/manifests/mooneye-gb/timer/tim00_div_trigger.json",
			tim01, "tests/manifests/mooneye-gb/timer/tim01.json",
			tim01_div_trigger, "tests/manifests/mooneye-gb/timer/tim01_div_trigger.json",
			tim10, "tests/manifests/mooneye-gb/timer/tim10.json",
			tim10_div_trigger, "tests/manifests/mooneye-gb/timer/tim10_div_trigger.json",
			tim11, "tests/manifests/mooneye-gb/timer/tim11.json",
			tim11_div_trigger, "tests/manifests/mooneye-gb/timer/tim11_div_trigger.json",
			tima_reload, "tests/manifests/mooneye-gb/timer/tima_reload.json",
			tima_write_reloading, "tests/manifests/mooneye-gb/timer/tima_write_reloading.json",
			tma_write_reloading, "tests/manifests/mooneye-gb/timer/tma_write_reloading.json"
		);
	}

	mod interrupts {
		use ::test_runner::run_test;
		run_tests!(
			ie_push, "tests/manifests/mooneye-gb/interrupts/ie_push.json"
		);
	}

	mod oam_dma {
		use ::test_runner::run_test;
		run_tests!(
			basic, "tests/manifests/mooneye-gb/oam_dma/basic.json",
			reg_read, "tests/manifests/mooneye-gb/oam_dma/reg_read.json"
		);
	}

	mod ppu {
		use ::test_runner::run_test;
		run_tests!(
			hblank_ly_scx_timing_GS, "tests/manifests/mooneye-gb/ppu/hblank_ly_scx_timing-GS.json",
			intr_1_2_timing_GS, "tests/manifests/mooneye-gb/ppu/intr_1_2_timing-GS.json",
			intr_2_0_timing, "tests/manifests/mooneye-gb/ppu/intr_2_0_timing.json",
			intr_2_mode0_timing_sprites, "tests/manifests/mooneye-gb/ppu/intr_2_mode0_timing_sprites.json",
			intr_2_mode0_timing, "tests/manifests/mooneye-gb/ppu/intr_2_mode0_timing.json",
			intr_2_mode3_timing, "tests/manifests/mooneye-gb/ppu/intr_2_mode3_timing.json",
			intr_2_oam_ok, "tests/manifests/mooneye-gb/ppu/intr_2_oam_ok_timing.json",
			lcdon_write_timing_GS, "tests/manifests/mooneye-gb/ppu/lcdon_write_timing-GS.json",
			stat_irq_blocking, "tests/manifests/mooneye-gb/ppu/stat_irq_blocking.json",
			stat_lyc_onoff, "tests/manifests/mooneye-gb/ppu/stat_lyc_onoff.json",
			vblank_stat_intr_GS, "tests/manifests/mooneye-gb/ppu/vblank_stat_intr-GS.json"
		);
	}
}
