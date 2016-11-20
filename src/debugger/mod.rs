use std::io::stdin;
use gameboy::GBC;
mod instructions;

pub struct Debugger {
	breakpoint: Option<u16>,
}

impl Debugger {
	pub fn new()  -> Debugger {
		Debugger {
			breakpoint: None,
		}
	}

	pub fn step(&mut self, gbc: &mut GBC) {
		let pc = gbc.cpu.registers.pc;
		let next: [u8; 3] = [gbc.read_byte(pc), gbc.read_byte(pc + 1), gbc.read_byte(pc + 2)];
		instructions::print_instruction(&next);
		gbc.step();
	}

	pub fn run(&mut self, gbc: &mut GBC) {
		loop {
			let mut line: String = String::new();
			let _ = stdin().read_line(&mut line);
			line = String::from(String::from(line.to_lowercase().trim_left()).trim_right());
			let mut command =  line.split_whitespace();

			match command.next().unwrap() {
				"breakpoint" => {
					match command.next() {
						Some(s) => {
							match u16::from_str_radix(s, 16) {
								Ok(val) => {
									self.breakpoint = Some(val);
									println!("Set a breakpoint at pc = {:#X}", val);
								},
								_ => println!("Invalid arguments"),
							};
						},
						None => {
							println!("Invalid usage of the breakpoint command, no arguments given");
							continue;
						}
					}
				},
				"cart" => {
					println!("{:?}", gbc.cart.get_cart_info());
				}
				"step" => {
					self.step(gbc);
				},
				"read" => {
					match command.next() {
						Some(s) => {
							match u16::from_str_radix(s, 16) {
								Ok(val) => {
									println!("{:#X}", gbc.read_byte(val));
								},
								_ => println!("Invalid arguments"),
							};
						},
						None => {
							println!("Invalid usage of the read command, no arguments given");
							continue;
						}
					}
				},
				"registers" => {
					println!("{:?}", gbc.cpu.registers);
				},
				"run" => {
					//if currently on a breakpoint, ignore it until the next time it is hit
					if self.breakpoint.is_some() && gbc.cpu.registers.pc == self.breakpoint.unwrap() {
						gbc.step();
					}

					loop {
						if self.breakpoint.is_some() && gbc.cpu.registers.pc == self.breakpoint.unwrap() {
							println!("Hit breakpoint at pc = {:#X}", self.breakpoint.unwrap());
							break;
						}
						else {
							gbc.step();
						}
					}
				},
				"exit" => {
					break;
				},

				_ => {
					println!("unrecognized command\nvalid commands are\n\
						\tbreakpoint - set a breakpoint (pass pc as a hex string, ex: breakpoint 0100)\n\
						\tcart       - print out info about the loaded cartridge\n\
						\tstep       - step to the next instruction\n\
						\tread       - print the byte at an address (ex: read 73B)\n\
						\tregisters  - print the current register value\n\
						\trun        - run the emulator\n\
						\texit       - exit");
				},
			};
		}
	}
}
