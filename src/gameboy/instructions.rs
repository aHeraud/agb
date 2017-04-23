use super::Gameboy;
use super::Register;
use gameboy::mmu::Mmu;
use gameboy::cpu;
use gameboy::cpu::{ZERO_FLAG, CARRY_FLAG};
use gameboy::cpu::RegisterPair;
use gameboy::util::{wrapping_add, wrapping_sub};


#[derive(Copy, Clone)]
pub enum Conditional {
	Z, NZ, C, NC
}

fn map_register(reg: u8) -> Register {
	match reg {
		0 => Register::B,
		1 => Register::C,
		2 => Register::D,
		3 => Register::E,
		4 => Register::H,
		5 => Register::L,
		6 => Register::AT_HL,
		7 => Register::A,
		_ => panic!("reg must be in the range 0...7"),
	}
}

impl Gameboy {
	pub fn execute(&mut self) {
		//self.interrupt_service_routine();  //called seperately to let debugger see calls to interrupt vectors

		if  self.cpu.halt {
			self.emulate_hardware();
		}

		else {
			//self.print_serial();

			self.cpu.ime = self.cpu.next_ime_state;

			let opcode: u8 = self.read_byte_cpu(self.cpu.registers.pc);
			self.cpu.registers.pc += 1;
			self.emulate_hardware();

			match opcode {
				0x00 => self.nop(),
				0x01 => self.ld_bc_d16(),
				0x02 => self.ld_at_bc_a(),
				0x03 => self.inc_r16(RegisterPair::BC),
				0x04 => self.inc_r8(Register::B),
				0x05 => self.dec_r8(Register::B),
				0x06 => self.ld_r8_d8(Register::B),
				0x07 => self.rlca(),
				0x08 => self.ld_at_a16_sp(),
				0x09 => self.add_hl_r16(RegisterPair::BC),
				0x0A => self.ld_a_at_bc(),
				0x0B => self.dec_r16(RegisterPair::BC),
				0x0C => self.inc_r8(Register::C),
				0x0D => self.dec_r8(Register::C),
				0x0E => self.ld_r8_d8(Register::C),
				0x0F => self.rrca(),
				0x10 => self.stop(),
				0x11 => self.ld_de_d16(),
				0x12 => self.ld_at_de_a(),
				0x13 => self.inc_r16(RegisterPair::DE),
				0x14 => self.inc_r8(Register::D),
				0x15 => self.dec_r8(Register::D),
				0x16 => self.ld_r8_d8(Register::D),
				0x17 => self.rla(),
				0x18 => self.jr_r8(),
				0x19 => self.add_hl_r16(RegisterPair::DE),
				0x1A => self.ld_a_at_de(),
				0x1B => self.dec_r16(RegisterPair::DE),
				0x1C => self.inc_r8(Register::E),
				0x1D => self.dec_r8(Register::E),
				0x1E => self.ld_r8_d8(Register::E),
				0x1F => self.rra(),
				0x20 => self.jr_nz_r8(),
				0x21 => self.ld_hl_d16(),
				0x22 => self.ldi_at_hl_a(),
				0x23 => self.inc_r16(RegisterPair::HL),
				0x24 => self.inc_r8(Register::H),
				0x25 => self.dec_r8(Register::H),
				0x26 => self.ld_r8_d8(Register::H),
				0x27 => self.daa(),
				0x28 => self.jr_z_r8(),
				0x29 => self.add_hl_r16(RegisterPair::HL),
				0x2A => self.ldi_a_at_hl(),
				0x2B => self.dec_r16(RegisterPair::HL),
				0x2C => self.inc_r8(Register::L),
				0x2D => self.dec_r8(Register::L),
				0x2E => self.ld_r8_d8(Register::L),
				0x2F => self.cpl(),
				0x30 => self.jr_nc_r8(),
				0x31 => self.ld_sp_d16(),
				0x32 => self.ldd_at_hl_a(),
				0x33 => self.inc_r16(RegisterPair::SP),
				0x34 => self.inc_r8(Register::AT_HL),
				0x35 => self.dec_r8(Register::AT_HL),
				0x36 => self.ld_r8_d8(Register::AT_HL),
				0x37 => self.scf(),
				0x38 => self.jr_c_r8(),
				0x39 => self.add_hl_r16(RegisterPair::SP),
				0x3A => self.ldd_a_at_hl(),
				0x3B => self.dec_r16(RegisterPair::SP),
				0x3C => self.inc_r8(Register::A),
				0x3D => self.dec_r8(Register::A),
				0x3E => self.ld_r8_d8(Register::A),
				0x3F => self.ccf(),
				0x40...0x47 => self.ld_r_r(Register::B, opcode),
				0x48...0x4F => self.ld_r_r(Register::C, opcode),
				0x50...0x57 => self.ld_r_r(Register::D, opcode),
				0x58...0x5F => self.ld_r_r(Register::E, opcode),
				0x60...0x67 => self.ld_r_r(Register::H, opcode),
				0x68...0x6F => self.ld_r_r(Register::L, opcode),
				//TODO: collapse LD (HL), R into a single fn
				0x70 => self.ld_at_hl_r8(Register::B),
				0x71 => self.ld_at_hl_r8(Register::C),
				0x72 => self.ld_at_hl_r8(Register::D),
				0x73 => self.ld_at_hl_r8(Register::E),
				0x74 => self.ld_at_hl_r8(Register::H),
				0x75 => self.ld_at_hl_r8(Register::L),
				0x76 => self.halt(),
				0x77 => self.ld_at_hl_r8(Register::A),

				//TODO: refactor these to take a register, not an opcode
				0x78...0x7F => self.ld_r_r(Register::A, opcode),
				0x80...0x87 => self.add_a_r8(opcode),
				0x88...0x8F => self.adc_a_r8(opcode),
				0x90...0x97 => self.sub_a_r8(opcode),
				0x98...0x9F => self.sbc_a_r8(opcode),
				0xA0...0xA7 => self.and(opcode),
				0xA8...0xAF => self.xor(opcode),
				0xB0...0xB7 => self.or_r8(opcode),
				0xB8...0xBF => self.cp_r8(opcode),
				0xC0 => self.ret_nz(),
				0xC1 => self.pop_r16(RegisterPair::BC),
				0xC2 => self.jp_conditional(Conditional::NZ),
				0xC3 => self.jp_a16(),
				0xC4 => self.call_conditional(Conditional::NZ),
				0xC5 => self.push_r16(RegisterPair::BC),
				0xC6 => self.add_d8(),
				0xC7 => self.rst(0x00),
				0xC8 => self.ret_z(),
				0xC9 => self.ret(),
				0xCA => self.jp_conditional(Conditional::Z),
				0xCB => self.extended(),
				0xCE => self.adc_a_d8(),
				0xCC => self.call_conditional(Conditional::Z),
				0xCD => self.call_a16(),
				0xCF => self.rst(0x08),
				0xD0 => self.ret_nc(),
				0xD1 => self.pop_r16(RegisterPair::DE),
				0xD2 => self.jp_conditional(Conditional::NC),
				0xD4 => self.call_conditional(Conditional::NC),
				0xD5 => self.push_r16(RegisterPair::DE),
				0xD6 => self.sub_d8(),
				0xD7 => self.rst(0x10),
				0xD8 => self.ret_c(),
				0xD9 => self.reti(),
				0xDA => self.jp_conditional(Conditional::C),
				0xDC => self.call_conditional(Conditional::C),
				0xDE => self.sbc_a_d8(),
				0xDF => self.rst(0x18),
				0xE0 => self.ld_at_ff00_plus_a8_a(),
				0xE1 => self.pop_r16(RegisterPair::HL),
				0xE2 => self.ld_at_ff00_plus_c_a(),
				0xE5 => self.push_r16(RegisterPair::HL),
				0xE6 => self.and_d8(),
				0xE7 => self.rst(0x20),
				0xE8 => self.add_sp_nn(),
				0xE9 => self.jp_hl(),
				0xEA => self.ld_at_a16_a(),
				0xEE => self.xor_d8(),
				0xEF => self.rst(0x28),
				0xF0 => self.ld_a_at_ff00_plus_a8(),
				0xF1 => self.pop_af(),
				0xF2 => self.ld_a_at_ff00_plus_c(),
				0xF3 => self.di(),
				0xF5 => self.push_r16(RegisterPair::AF),
				0xF6 => self.or_d8(),
				0xF7 => self.rst(0x30),
				0xF8 => self.ld_hl_sp_plus_nn(),
				0xF9 => self.ld_sp_hl(),
				0xFA => self.ld_a_at_a16(),
				0xFB => self.ei(),
				0xFE => self.cp_d8(),
				0xFF => self.rst(0x38),
				_ => {
					self.cpu.registers.pc -= 1;
					panic!("\n{:?}\nUnimplemented opcode {:X}", self.cpu.registers ,opcode);
				},
			};
		}
	}

	///Read the u8 at pc, and increment pc
	fn read_next(&mut self) -> u8 {
		let val: u8 = self.read_byte_cpu(self.cpu.registers.pc);
		self.cpu.registers.pc += 1;
		val
	}

	///Push a word onto the stack.
	///There is 2 M-Cycles of memory access, plus 1 M-Cycle of internal delay
	///Because of the 1 M-Cycle delay, push takes 1 more M-Cycle than pop.
	fn push(&mut self, value: u16) {
		//push has an extra internal delay
		self.emulate_hardware();

		//push high byte of pc onto stack
		let sp: u16 = self.cpu.registers.sp;

		let high: u8 = (value >> 8) as u8;
		self.write_byte_cpu(sp - 1, high);
		self.emulate_hardware();

		//push low byte of pc onto stack
		let low: u8 = (value & 0xFF) as u8;
		self.write_byte_cpu(sp - 2, low);
		self.emulate_hardware();

		//sub 2 from sp because we pushed a word onto the stack
		self.cpu.registers.sp -= 2;
	}

	///Pop a byte off of the stack
	///2 M-Cycles of memory access
	fn pop(&mut self) -> u16 {
		let low: u8 = self.read_byte_cpu(self.cpu.registers.sp);
		self.emulate_hardware();

		let high: u8 = self.read_byte_cpu(self.cpu.registers.sp + 1);
		self.emulate_hardware();

		self.cpu.registers.sp += 2;
		((high as u16) << 8) | (low as u16)
	}

	fn _ret(&mut self) {
		//read low byte of return address from stack
		let sp: u16 = self.cpu.registers.sp;
		let addr_low: u8 = self.read_byte_cpu(sp);
		self.emulate_hardware();

		//read high byte of return address from stack
		let addr_high: u8 = self.read_byte_cpu(sp + 1);
		self.emulate_hardware();

		//where does this delay actually go?
		self.emulate_hardware();

		//add 2 to sp, set pc
		self.cpu.registers.sp += 2;
		self.cpu.registers.pc = ((addr_high as u16) << 8) | addr_low as u16;
	}

	fn jp_conditional(&mut self, conditional: Conditional) {
		//1 cycle to read low byte of address
		let addr_low: u8 = self.read_byte_cpu(self.cpu.registers.pc);
		self.cpu.registers.pc += 1;
		self.emulate_hardware();

		//1 cycle to read high byte of address
		let addr_high: u8 = self.read_byte_cpu(self.cpu.registers.pc);
		self.cpu.registers.pc += 1;
		self.emulate_hardware();

		let branch: bool = match conditional {
			Conditional::Z => self.cpu.registers.f & ZERO_FLAG == ZERO_FLAG,
			Conditional::NZ => self.cpu.registers.f & ZERO_FLAG == 0,
			Conditional::C => self.cpu.registers.f & CARRY_FLAG == CARRY_FLAG,
			Conditional::NC => self.cpu.registers.f & CARRY_FLAG == 0,
		};

		if branch {
			//1 cycle of internal delay
			self.emulate_hardware();

			//finally set pc? or does this happen before the last delay
			self.cpu.registers.pc = ((addr_high as u16) << 8) | (addr_low as u16);
		}
	}


	///6 M-Cycles if branch take, else 3 M-Cycles
	///Length: 3 bytes
	fn call_conditional(&mut self, conditional: Conditional) {
		//read low byte of address
		let low: u8 = self.read_next();
		self.emulate_hardware();

		//read high byte of address
		let high: u8 = self.read_next();
		self.emulate_hardware();

		let branch: bool = match conditional {
			Conditional::Z => self.cpu.registers.f & ZERO_FLAG == ZERO_FLAG,
			Conditional::NZ => self.cpu.registers.f & ZERO_FLAG == 0,
			Conditional::C => self.cpu.registers.f & CARRY_FLAG == CARRY_FLAG,
			Conditional::NC => self.cpu.registers.f & CARRY_FLAG == 0,
		};

		if branch {
			//ZF not set, take branch

			//push current value of pc onto the stack
			let pc: u16 = self.cpu.registers.pc;
			self.push(pc);

			//set the new value of pc
			self.cpu.registers.pc = ((high as u16) << 8) | low as u16;
		}
	}

	///RST X
	///4 M-Cycles
	fn rst(&mut self, address: u8) {
		let pc: u16 = self.cpu.registers.pc;
		self.push(pc);
		self.cpu.registers.pc = address as u16;
	}

	fn get_register(&mut self, reg: Register) -> u8 {
		match reg {
			Register::B => self.cpu.registers.b,
			Register::C => self.cpu.registers.c,
			Register::D => self.cpu.registers.d,
			Register::E => self.cpu.registers.e,
			Register::H => self.cpu.registers.h,
			Register::L => self.cpu.registers.l,
			Register::AT_HL => {
				let hl: u16 = self.cpu.registers.get_register_pair(RegisterPair::HL);
				let value = self.read_byte_cpu(hl);
				self.emulate_hardware();
				value
			},
			Register::A => self.cpu.registers.a,
			Register::F => self.cpu.registers.f,
		}
	}

	fn set_register(&mut self, reg: Register, val: u8) {
		match reg {
			Register::B => self.cpu.registers.b = val,
			Register::C => self.cpu.registers.c = val,
			Register::D => self.cpu.registers.d = val,
			Register::E => self.cpu.registers.e = val,
			Register::H => self.cpu.registers.h = val,
			Register::L => self.cpu.registers.l = val,
			Register::AT_HL => {
				let hl: u16 = self.cpu.registers.get_register_pair(RegisterPair::HL);
				self.write_byte_cpu(hl, val);
				self.emulate_hardware();
			},
			Register::A => self.cpu.registers.a = val,
			Register::F => self.cpu.registers.f = val,
		};
	}

	fn jr(&mut self, offset: u8) {
		//add/sub to/from pc
		if offset & 0x80 == 0x80 {
			//offset is a negative 2s compliment integer
			//subtract the 2s compliment of the offset
			self.cpu.registers.pc -= (!offset + 1) as u16;
		}
		else {
			//unsigned
			self.cpu.registers.pc += offset as u16;
		}
	}
}

impl Gameboy {

	///INC r8 (0x04, 0x0C, 0x14, 0x1C, 0x24, 0x2C, 0x34, 0x3C)
	///1 M-Cycle (except 0x34, INC (HL), which takes 3 M-Cycles (read and write to (HL)))
	///Length: 1 byte
	fn inc_r8(&mut self, reg: Register) {
		let mut val: u8 = self.get_register(reg);
		val = cpu::alu::inc(val, &mut self.cpu.registers.f);
		self.set_register(reg, val);
	}

	///DEC r8
	///1 M-Cycle (except 0x35, DEC (HL), which takes 3 M-Cycles (read and write to (HL)))
	///Length: 1 byte
	fn dec_r8(&mut self, reg: Register) {
		let mut val = self.get_register(reg);
		val = cpu::alu::dec(val, &mut self.cpu.registers.f);
		self.set_register(reg, val);
	}

	///INC r16
	///2-M Cycles
	fn inc_r16(&mut self, reg: RegisterPair) {
		//1 cycle extra internal delay
		self.emulate_hardware();

		let mut val: u16 = self.cpu.registers.get_register_pair(reg);
		val = wrapping_add(val, 1);
		self.cpu.registers.set_register_pair(reg, val);
	}

	///DEC r16
	///2 M-Cycles
	fn dec_r16(&mut self, reg: RegisterPair) {
		//1 cycle of internal delay
		self.emulate_hardware();

		let mut val: u16 = self.cpu.registers.get_register_pair(reg);
		val = wrapping_sub(val, 1);
		self.cpu.registers.set_register_pair(reg, val);
	}

	///POP r16
	///3 M-Cycles
	///Length: 1 byte
	fn pop_r16(&mut self, reg: RegisterPair) {
		let r16 = self.pop();
		self.cpu.registers.set_register_pair(reg, r16);
	}

	///PUSH r16
	///4 M-Cycles
	///Length: 1 byte
	fn push_r16(&mut self, reg: RegisterPair) {
		let r16 = self.cpu.registers.get_register_pair(reg);
		self.push(r16);
	}

	///LD r8, d8
	///2 M-Cycles (except for 0x36: LD (HL), d8)
	///Length: 2 bytes
	fn ld_r8_d8(&mut self, reg: Register) {
		let imm: u8 = self.read_next();
		self.set_register(reg, imm);
		self.emulate_hardware();
	}

	///0x09: ADD HL, r16
	///2 M-Cycles
	///Flags: - 0 H C
	fn add_hl_r16(&mut self, reg: RegisterPair) {
		self.emulate_hardware();
		let mut hl: u16 = self.cpu.registers.get_register_pair(RegisterPair::HL);
		let other: u16 = self.cpu.registers.get_register_pair(reg);
		hl = cpu::alu::add16(hl, other, &mut self.cpu.registers.f);
		self.cpu.registers.set_register_pair(RegisterPair::HL, hl);
	}

	///0x00
	///1 M-Cycle
	fn nop(&mut self) {

	}

	///0x01: LD BC, d16
	///3 M-Cycles
	///Length: 3 bytes
	fn ld_bc_d16(&mut self) {
		//1 cycle memory access for low byte
		let low: u8 = self.read_next();
		self.emulate_hardware();

		//1 cycle memory access to read high byte
		let high: u8 = self.read_next();
		self.emulate_hardware();

		//set bc
		self.cpu.registers.b = high;
		self.cpu.registers.c = low;
	}

	///0x02: LD (BC), A
	///2 M-Cycles
	///Length: 1 byte
	fn ld_at_bc_a(&mut self) {
		let bc: u16 = self.cpu.registers.get_register_pair(RegisterPair::BC);
		let a: u8 = self.cpu.registers.a;
		self.write_byte_cpu(bc, a);
		self.emulate_hardware();
	}

	///TODO: what is this really???
	///0x08: LD (a16), SP
	///5 M-Cycles
	///Length: 3 bytes
	fn ld_at_a16_sp(&mut self) {
		let addr_low: u8 = self.read_next();
		self.emulate_hardware();

		let addr_high: u8 = self.read_next();
		self.emulate_hardware();

		let addr: u16 = (addr_high as u16) << 8 | (addr_low as u16);

		let sp: u16 = self.cpu.registers.sp;
		let sp_low: u8 = (sp & 0xFF) as u8;
		let sp_high: u8 = (sp >> 8) as u8;

		self.write_byte_cpu(addr, sp_low);
		self.emulate_hardware();

		self.write_byte_cpu(addr + 1, sp_high);
		self.emulate_hardware();
	}

	///0x07: RLCA
	///1 M-Cycle
	///Length: 1 byte
	///Same as RLC A, except it only consumes 1 m-cycle, and resets the zero flag
	fn rlca(&mut self) {
		self.cpu.registers.a = cpu::alu::rlc(self.cpu.registers.a, &mut self.cpu.registers.f);
		self.cpu.registers.f &= !cpu::ZERO_FLAG;
	}

	///0x0A: LD A, (BC)
	///2 M-Cycles
	///Length: 1 byte
	fn ld_a_at_bc(&mut self) {
		let bc: u16 = self.cpu.registers.get_register_pair(RegisterPair::BC);
		self.cpu.registers.a = self.read_byte_cpu(bc);
		self.emulate_hardware();
	}

	///0x0F: RRCA
	///1 M-Cycle
	///Length: 1 byte
	///A shorter RRC A that only sets the cy flag
	fn rrca(&mut self) {
		self.cpu.registers.a = cpu::alu::rrc(self.cpu.registers.a, &mut self.cpu.registers.f);
		self.cpu.registers.f &= cpu::CARRY_FLAG;
	}

	///0x10: Stop
	///1 M-Cycle
	///Length: 1 byte
	///Not really sure how to handle this
	fn stop(&mut self) {
		self.cpu.stop = true;
	}

	///0x11: LD DE, d16
	///3 M-Cycles
	///Length: 3 bytes
	fn ld_de_d16(&mut self) {
		//1 cycle to load low byte
		self.cpu.registers.e = self.read_next();
		self.emulate_hardware();

		//1 cycle to load high byte
		self.cpu.registers.d = self.read_next();
		self.emulate_hardware();
	}

	///0x12: LD (DE), A
	///2 M-Cycles
	///Length: 1 byte
	fn ld_at_de_a(&mut self) {
		let de: u16 = self.cpu.registers.get_register_pair(RegisterPair::DE);
		let a: u8 = self.cpu.registers.a;
		self.write_byte_cpu(de, a);
		self.emulate_hardware();
	}

	///0x17: RLA
	///1 M-Cycle
	///Length: 1 byte
	///A shorter RL A that only sets the cy flag
	fn rla(&mut self) {
		self.cpu.registers.a = cpu::alu::rl(self.cpu.registers.a, &mut self.cpu.registers.f);
		self.cpu.registers.f &= cpu::CARRY_FLAG;
	}

	///0x18: JR r8
	///3 M-Cycles
	///Length: 2 bytes
	///Timings from https://github.com/Gekkio/mooneye-gb/blob/master/docs/accuracy.markdown
	fn jr_r8(&mut self) {
		//1 cycle to read offset
		let offset: u8 = self.read_next();
		self.emulate_hardware();

		//internal delay
		self.emulate_hardware();

		self.jr(offset);
	}

	///0x1A: LD A, (DE)
	///2 M-Cycles
	///Length: 1 byte
	fn ld_a_at_de(&mut self) {
		let de: u16 = self.cpu.registers.get_register_pair(RegisterPair::DE);
		self.cpu.registers.a = self.read_byte_cpu(de);
		self.emulate_hardware();
	}

	///0x1F: RRA
	///1 M-Cycle
	///Length: 1 byte
	///RRA is basically RR A but it sets all of the flags to 0 except c,
	///and since it's not an extended opcode it takes 1 less M-Cycle
	fn rra(&mut self) {
		let mut a: u8 = self.cpu.registers.a;
		let mut f: u8 = self.cpu.registers.f;
		a = cpu::alu::rr(a, &mut f);
		f &= cpu::CARRY_FLAG;

		self.cpu.registers.a = a;
		self.cpu.registers.f = f;
	}

	///0x20: JR NZ, r8
	///3 M-Cycles if branch taken, 2 M-Cycles if not taken.
	///Length: 2 bytes
	fn jr_nz_r8(&mut self) {
		//1 cycle to read operand
		let offset: u8 = self.read_next();
		self.emulate_hardware();

		//1 cycle if conditional taken
		if self.cpu.registers.f & cpu::ZERO_FLAG == 0 {
			//zero flag not set, add/subtract offset from pc
			self.emulate_hardware();
			self.jr(offset);
		}
	}

	///0x21
	///3 M-cycles
	///Length: 3 bytes
	fn ld_hl_d16(& mut self) {
		//read low byte into l
		let low: u8 = self.read_next();
		self.cpu.registers.l = low;
		self.emulate_hardware();

		//read high byte into h
		let high: u8 = self.read_next();
		self.cpu.registers.h = high;
		self.emulate_hardware();
	}

	///0x22: LD (HL+), A
	///2 M-Cycles
	///Length: 1 byte
	fn ldi_at_hl_a(&mut self) {
		let hl: u16 = self.cpu.registers.get_register_pair(RegisterPair::HL);
		let a: u8 = self.cpu.registers.a;
		self.write_byte_cpu(hl, a);
		//increment hl
		self.cpu.registers.set_register_pair(RegisterPair::HL, wrapping_add(hl, 1));
		self.emulate_hardware();
	}

	///0x27: DAA
	///1 M-Cycle
	///Length: 1 byte
	fn daa(&mut self) {
		//http://forums.nesdev.com/viewtopic.php?t=4728&start=15
		let mut a: u16 = self.cpu.registers.a as u16;
		if self.cpu.registers.f & 0x40 == 0 {
			//Subtraftion flag not set, so last arithmetic operation was not subtraction
			if (self.cpu.registers.f & 0x20) == 0x20 || (a & 0x0F) > 9 {
				a += 0x06;
			}
			if (self.cpu.registers.f & 0x10) == 0x10 || a > 0x9F {
				a += 0x60;
			}
		}
		else {
			//last arithmetic operation was subtraction
			if (self.cpu.registers.f & 0x20) == 0x20 {
				a = (wrapping_sub(a, 6)) & 0xFF;
			}
			if (self.cpu.registers.f & 0x10) == 0x10 {
				a = wrapping_sub(a, 0x60);
			}
		}

		//reset zero flag and half carry flag
		self.cpu.registers.f &= !(cpu::HALF_CARRY_FLAG | cpu::ZERO_FLAG);

		//set zero flag
		self.cpu.registers.f |= !(((a & 0x007F) + 0x007F) | a) as u8 & cpu::ZERO_FLAG;


		//set carry flag
		self.cpu.registers.f |= ((a >> 4) as u8) & cpu::CARRY_FLAG;

		//set a
		a &= 0xFF;
		self.cpu.registers.a = a as u8;

	}

	///0x28: JR Z, r8
	///3 M-Cycles if jump taken, else 2 M-Cycles
	///Length: 2 bytes
	fn jr_z_r8(&mut self) {
		//1 cycle to read offset
		let offset: u8 = self.read_next();
		self.emulate_hardware();

		//1 cycle if branch taken
		if self.cpu.registers.f & cpu::ZERO_FLAG == cpu::ZERO_FLAG {
			self.jr(offset);

			//1 M-Cycle internal delay when branch taken
			self.emulate_hardware();
		}
	}

	///0x2F: CPL
	///A = A xor 0xFF, Flags = - 1 1 -
	///Length: 1 byte
	///TODO: move to cpu?
	fn cpl(&mut self) {
		self.cpu.registers.a = self.cpu.registers.a ^ 0xFF;
		self.cpu.registers.f |= cpu::SUBTRACTION_FLAG | cpu::HALF_CARRY_FLAG;
	}

	///0x30: JR NC, i8
	///3 M-Cycles if branch taken, 2 otherwise
	fn jr_nc_r8(&mut self) {
		//1 cycle to read offset
		let offset: u8 = self.read_next();
		self.emulate_hardware();

		//jump if carry flag is not set
		if (self.cpu.registers.f & cpu::CARRY_FLAG) == 0 {
			//internal delay since branch taken
			self.emulate_hardware();

			self.jr(offset);
		}

	}

	///0x31: LD SP, d16
	///3 M-Cycles
	fn ld_sp_d16(&mut self) {
		//load low byte
		let low: u8 = self.read_next();
		self.emulate_hardware();

		//load high byte
		let high: u8 = self.read_next();
		self.emulate_hardware();

		//set sp
		self.cpu.registers.sp = ((high as u16) << 8) | (low as u16);
	}

	///0x2A: LD A, (HL+)
	///2 M-Cycles
	///1 byte
	///Load the byte at memory address HL into A, and increment HL
	fn ldi_a_at_hl(&mut self) {
		//read memory at (HL)
		let hl: u16 = self.cpu.registers.get_register_pair(RegisterPair::HL);
		let val: u8 = self.read_byte_cpu(hl);
		self.emulate_hardware();

		//Update registers
		self.cpu.registers.a = val;
		self.cpu.registers.set_register_pair(RegisterPair::HL, hl + 1);
	}

	///0x32: LD (HL-), A
	///2 M-Cycle
	///TODO: double check timings for this
	fn ldd_at_hl_a(&mut self) {
		//1 cycle to set memory at address HL
		let hl: u16 = self.cpu.registers.get_register_pair(RegisterPair::HL);
		let a: u8 = self.cpu.registers.a;	//I have to do this because write byte borrows the whole gbc struct as mut
		self.write_byte_cpu(hl, a);
		self.cpu.registers.set_register_pair(RegisterPair::HL, wrapping_sub(hl,1));
		self.emulate_hardware();
	}

	///0x37: SCF (Set Carry Flag)
	///1 M-Cycle
	///Preserves Zero flag, resets Subtraction and Half-carry flags, and sets half-carry
	///TODO: move to cpu?
	fn scf(&mut self) {
		self.cpu.registers.f &= cpu::ZERO_FLAG;
		self.cpu.registers.f |= cpu::CARRY_FLAG;
	}

	///0x38: JR C, r8
	///3 M-Cycles if jump taken, else 2 M-Cycles
	///Length: 2 bytes
	fn jr_c_r8(&mut self) {
		//1 cycle to read offset
		let offset: u8 = self.read_next();
		self.emulate_hardware();

		//1 cycle if branch taken
		if self.cpu.registers.f & cpu::CARRY_FLAG == cpu::CARRY_FLAG {
			self.jr(offset);

			//1 M-Cycle internal delay when branch taken
			self.emulate_hardware();
		}
	}

	///0x3A: LD A, (HL-)
	///2 M-Cycles
	///Length: 1 byte
	fn ldd_a_at_hl(&mut self) {
		let hl: u16 = self.cpu.registers.get_register_pair(RegisterPair::HL);
		self.cpu.registers.a = self.read_byte_cpu(hl);
		self.cpu.registers.set_register_pair(RegisterPair::HL, wrapping_sub(hl, 1));
		self.emulate_hardware();
	}

	///0x3F: CCF
	///1 M-Cycle
	///Length: 1 byte
	///Inverts the carry flag, resets SF and HC, and preserves ZF
	fn ccf(&mut self) {
		let zf = self.cpu.registers.f & cpu::ZERO_FLAG;
		let cf = self.cpu.registers.f & cpu::CARRY_FLAG;
		self.cpu.registers.f = zf | (!cf & cpu::CARRY_FLAG)
	}

	///[0x40...0x75] U [0x77...0x7F]: LD r1, r2
	///1 M-Cycle (except 0x_6 & 0x_E which take 2 M-Cycles)
	///Length: 1 byte
	fn ld_r_r(&mut self, dest: Register, opcode: u8) {
		let src: Register = map_register(opcode & 7);
		let val: u8 = self.get_register(src);
		self.set_register(dest, val);
	}

	///0x70...0x77: LD (HL), r8
	///2 M-Cycles
	fn ld_at_hl_r8(&mut self, reg: Register) {
		let val: u8 = self.get_register(reg);
		let hl: u16 = self.cpu.registers.get_register_pair(RegisterPair::HL);
		self.write_byte_cpu(hl, val);
		self.emulate_hardware();
	}

	///0x76: HALT
	///1 M-Cycle
	///Length: 1 byte
	fn halt(&mut self) {
		self.cpu.halt = true;
	}

	///0x80...0x8F: ADD A, r8
	///1 M-Cycle
	///Length: 1 byte
	fn add_a_r8(&mut self, opcode: u8) {
		let register: u8 = self.get_register(map_register(opcode & 0x7));
		self.cpu.registers.a = cpu::alu::add(self.cpu.registers.a, register, &mut self.cpu.registers.f);
	}

	///0x80...0x8F: ADC A, r8
	///1 M-Cycle
	///Length: 1 byte
	fn adc_a_r8(&mut self, opcode: u8) {
		let register: u8 = self.get_register(map_register(opcode & 0x7));
		self.cpu.registers.a = cpu::alu::adc(self.cpu.registers.a, register, &mut self.cpu.registers.f);
	}

	fn sub_a_r8(&mut self, opcode: u8) {
		let register: u8 = self.get_register(map_register(opcode & 0x7));
		self.cpu.registers.a = cpu::alu::sub(self.cpu.registers.a, register, &mut self.cpu.registers.f);
	}

	fn sbc_a_r8(&mut self, opcode: u8) {
		let register: u8 = self.get_register(map_register(opcode & 0x7));
		self.cpu.registers.a = cpu::alu::sbc(self.cpu.registers.a, register, &mut self.cpu.registers.f);
	}

	///0xA0...0xA7: AND r8
	///1 M-Cycle
	///Length: 1 byte
	fn and(&mut self, opcode: u8) {
		let register: u8 = self.get_register(map_register(opcode & 0x7));
		self.cpu.registers.a = cpu::alu::and(self.cpu.registers.a, register, &mut self.cpu.registers.f);
	}

	///0xA8...0xAF: XOR r8
	///1 M-Cycle (except for 0xAE, XOR (HL), which takes 2)
	///Length: 1 byte
	fn xor(&mut self, opcode: u8) {
		let register: u8 = self.get_register(map_register(opcode & 0x7));
		self.cpu.registers.a = cpu::alu::xor(self.cpu.registers.a, register, &mut self.cpu.registers.f);
	}

	///0xB0...0xB7: OR R8
	///1 M-Cycle, unless the register is (HL), then 2 M-Cycles
	///Length: 1 byte
	fn or_r8(&mut self, opcode: u8) {
		//The register is the low 3 bits of the opcode
		let register: u8 = self.get_register(map_register(opcode & 0x7));
		self.cpu.registers.a = cpu::alu::or(self.cpu.registers.a, register, &mut self.cpu.registers.f);
	}

	///0xB8...0xBF: CP r8
	///1 M-Cycle
	///Length: 1 byte
	fn cp_r8(&mut self, opcode: u8) {
		let register: u8 = self.get_register(map_register(opcode & 0x7));
		cpu::alu::cp(self.cpu.registers.a, register, &mut self.cpu.registers.f);
	}

	///0xC0: RET NZ
	///5 M-Cycles if branch taken, 2 otherwise
	///Length: 1 byte
	///I think that the extra cycle compared to regular ret is to check the conditional
	fn ret_nz(&mut self) {
		//1 cycle to check conditional
		self.emulate_hardware();

		if (self.cpu.registers.f & cpu::ZERO_FLAG) == 0 {
			self._ret();
		}
	}

	///0xC3
	///4-M Cycles
	///Length: 3 bytes
	///Address: u16
	fn jp_a16(&mut self) {
		//1 cycle to read low byte of address
		let addr_low: u8 = self.read_byte_cpu(self.cpu.registers.pc);
		self.cpu.registers.pc += 1;
		self.emulate_hardware();

		//1 cycle to read high byte of address
		let addr_high: u8 = self.read_byte_cpu(self.cpu.registers.pc);
		self.cpu.registers.pc += 1;
		self.emulate_hardware();

		//1 cycle of internal delay
		self.emulate_hardware();

		//finally set pc? or does this happen before the last delay
		self.cpu.registers.pc = ((addr_high as u16) << 8) | (addr_low as u16);
	}

	///0xC6: ADD d8
	///2 M-Cycles
	///Length: 1 byte
	fn add_d8(&mut self) {
		let imm: u8 = self.read_next();
		self.emulate_hardware();

		self.cpu.registers.a = cpu::alu::add(self.cpu.registers.a, imm, &mut self.cpu.registers.f);
	}

	///0xC8: RET Z
	///5 M-Cycles if branch taken, 2 otherwise
	///Length: 1 byte
	///I think that the extra cycle compared to regular ret is to check the conditional
	fn ret_z(&mut self) {
		//1 cycle to check conditional
		self.emulate_hardware();

		if (self.cpu.registers.f & cpu::ZERO_FLAG) == cpu::ZERO_FLAG {
			self._ret();
		}
	}

	///0xC9: RET
	///4 M-Cycles
	///Length: 1 byte
	fn ret(&mut self) {
		self._ret();
	}

	///0xCB: Extended opcodes
	fn extended(&mut self) {
		let opcode: u8 = self.read_next();
		self.emulate_hardware();

		let reg: Register = map_register(opcode & 0x7);
		let val = self.get_register(reg);

		let mut new_val: Option<u8> = None;
		match opcode {
			0x00...0x07 => new_val = Some(cpu::alu::rlc(val, &mut self.cpu.registers.f)),
			0x08...0x0F => new_val = Some(cpu::alu::rrc(val, &mut self.cpu.registers.f)),
			0x10...0x17 => new_val = Some(cpu::alu::rl(val, &mut self.cpu.registers.f)),
			0x18...0x1F => new_val = Some(cpu::alu::rr(val, &mut self.cpu.registers.f)),
			0x20...0x27 => new_val = Some(cpu::alu::sla(val, &mut self.cpu.registers.f)),
			0x28...0x2F => new_val = Some(cpu::alu::sra(val, &mut self.cpu.registers.f)),
			0x30...0x37 => new_val = Some(cpu::alu::swap(val, &mut self.cpu.registers.f)),
			0x38...0x3F => new_val = Some(cpu::alu::srl(val, &mut self.cpu.registers.f)),
			0x40...0x47 => cpu::alu::bit(val, &mut self.cpu.registers.f, 0),
			0x48...0x4F => cpu::alu::bit(val, &mut self.cpu.registers.f, 1),
			0x50...0x57 => cpu::alu::bit(val, &mut self.cpu.registers.f, 2),
			0x58...0x5F => cpu::alu::bit(val, &mut self.cpu.registers.f, 3),
			0x60...0x67 => cpu::alu::bit(val, &mut self.cpu.registers.f, 4),
			0x68...0x6F => cpu::alu::bit(val, &mut self.cpu.registers.f, 5),
			0x70...0x77 => cpu::alu::bit(val, &mut self.cpu.registers.f, 6),
			0x78...0x7F => cpu::alu::bit(val, &mut self.cpu.registers.f, 7),
			0x80...0x87 => new_val = Some(cpu::alu::res(val, 0)),
			0x88...0x8F => new_val = Some(cpu::alu::res(val, 1)),
			0x90...0x97 => new_val = Some(cpu::alu::res(val, 2)),
			0x98...0x9F => new_val = Some(cpu::alu::res(val, 3)),
			0xA0...0xA7 => new_val = Some(cpu::alu::res(val, 4)),
			0xA8...0xAF => new_val = Some(cpu::alu::res(val, 5)),
			0xB0...0xB7 => new_val = Some(cpu::alu::res(val, 6)),
			0xB8...0xBF => new_val = Some(cpu::alu::res(val, 7)),
			0xC0...0xC7 => new_val = Some(cpu::alu::set(val, 0)),
			0xC8...0xCF => new_val = Some(cpu::alu::set(val, 1)),
			0xD0...0xD7 => new_val = Some(cpu::alu::set(val, 2)),
			0xD8...0xDF => new_val = Some(cpu::alu::set(val, 3)),
			0xE0...0xE7 => new_val = Some(cpu::alu::set(val, 4)),
			0xE8...0xEF => new_val = Some(cpu::alu::set(val, 5)),
			0xF0...0xF7 => new_val = Some(cpu::alu::set(val, 6)),
			0xF8...0xFF => new_val = Some(cpu::alu::set(val, 7)),
			_ => {
				self.cpu.registers.pc -= 1;
				panic!("\n{:?}\nUnimlemented extended opcode {:#X}", self.cpu.registers, opcode);
			}
		};

		match new_val {
			Some(v) => self.set_register(reg, v),
			_ => {},
		};
	}

	///0xCD: call a16
	///6-M Cycles
	///Length: 3 bytes
	///Timing from https://github.com/Gekkio/mooneye-gb/blob/master/docs/accuracy.markdown
	fn call_a16(&mut self) {
		//read low byte of address
		let low: u8 = self.read_next();
		self.emulate_hardware();

		//read high byte of address
		let high: u8 = self.read_next();
		self.emulate_hardware();

		//TODO: replace this with self.push
		//push internal delay
		self.emulate_hardware();

		//push high byte of pc onto stack
		let sp: u16 = self.cpu.registers.sp;
		let pc_high: u8 = (self.cpu.registers.pc >> 8) as u8;
		self.write_byte_cpu(sp - 1, pc_high);
		self.emulate_hardware();

		//push low byte of pc onto stack
		let pc_low: u8 = (self.cpu.registers.pc & 0xFF) as u8;
		self.write_byte_cpu(sp - 2, pc_low);
		self.emulate_hardware();

		//sub 2 from sp because we pushed a word onto the stack
		self.cpu.registers.sp -= 2;
		//set the new value of pc
		self.cpu.registers.pc = ((high as u16) << 8) | low as u16;
	}

	///0xCE: ADC A, d8
	///2 M-Cycles
	fn adc_a_d8(&mut self) {
		let imm: u8 = self.read_next();
		self.emulate_hardware();

		self.cpu.registers.a = cpu::alu::adc(self.cpu.registers.a, imm, &mut self.cpu.registers.f);
	}

	///0xD0: RET NC
	///5 M-Cycles if branch taken, 2 otherwise
	///Length: 1 byte
	///I think that the extra cycle compared to regular ret is to check the conditional
	fn ret_nc(&mut self) {
		//1 cycle to check conditional
		self.emulate_hardware();

		if (self.cpu.registers.f & cpu::CARRY_FLAG) == 0 {
			self._ret();
		}
	}

	///0xD6: SUB d8
	///2 M-Cycles
	fn sub_d8(&mut self) {
		let imm = self.read_next();
		self.emulate_hardware();

		self.cpu.registers.a = cpu::alu::sub(self.cpu.registers.a, imm, &mut self.cpu.registers.f);
	}

	///0xD8: RET C
	///5 M-Cycles if branch taken, otherwise 2
	///Length: 1 byte
	fn ret_c(&mut self) {
		//1 cycle to check conditional
		self.emulate_hardware();

		if(self.cpu.registers.f & cpu::CARRY_FLAG) == cpu::CARRY_FLAG  {
			self._ret();
		}
	}

	///0xD9: RETI
	///4 M-Cycles
	fn reti(&mut self) {
		self._ret();
		self.cpu.next_ime_state = true;
	}

	///0xDE: SBC A, d8
	///2 M-Cycles
	///Length: 2 bytes
	fn sbc_a_d8(&mut self) {
		let imm: u8 = self.read_next();
		self.emulate_hardware();

		self.cpu.registers.a = cpu::alu::sbc(self.cpu.registers.a, imm, &mut self.cpu.registers.f);
	}

	///0xE0: LDH (0xFF00 + a8), A
	///3 M-Cycles
	///Length: 2 bytes
	///TODO: rename this
	///TODO: what happens if ff00 + a8 is outside of the memmory mapped io registers space
	fn ld_at_ff00_plus_a8_a(&mut self) {
		//Write to io port
		//1 cycle to read a8
		let a8: u8 = self.read_next();
		self.emulate_hardware();

		//1 cycle to load value of a into ff00 + a8
		let a: u8 = self.cpu.registers.a;
		self.write_byte_cpu(0xFF00 + (a8 as u16), a);
		self.emulate_hardware();
	}

	///0xE2: LD (0xFF00 + C), A
	///2 M-Cycles
	///Length: 1 byte
	fn ld_at_ff00_plus_c_a(&mut self) {
		//1 cycle to write to io port
		let addr: u16 = 0xFF00 + self.cpu.registers.c as u16;
		let val: u8 = self.cpu.registers.a;
		self.write_byte_cpu(addr, val);
		self.emulate_hardware();
	}

	///0xE6: AND d8
	///2 M-Cycles
	///Length: 2 bytes
	fn and_d8(&mut self) {
		let imm: u8 = self.read_next();
		self.emulate_hardware();

		self.cpu.registers.a = cpu::alu::and(self.cpu.registers.a, imm, &mut self.cpu.registers.f);
	}

	///0xE8: ADD SP, nn
	///4 M-Cycles
	///Length: 2 bytes
	fn add_sp_nn(&mut self) {
		let nn: u8 = self.read_next();
		self.emulate_hardware();

		//2 m-cycles delay
		//TODO: check internal delay
		self.emulate_hardware();
		self.emulate_hardware();

		self.cpu.registers.sp = cpu::alu::add_sp_nn(self.cpu.registers.sp, nn, &mut self.cpu.registers.f);
	}

	///0xE9: JP HL (This was incorrectly listed as JP (HL) in the opcode sheet i used)
	///1 M-Cycle
	///Length: 1 byte
	fn jp_hl(&mut self) {
		let hl: u16 = self.cpu.registers.get_register_pair(RegisterPair::HL);
		self.cpu.registers.pc = hl;
	}

	///0xEA: LD (a16), A
	///4 M-Cycles
	fn ld_at_a16_a(&mut self) {
		//load low byte of address
		let low: u8 = self.read_next();
		self.emulate_hardware();

		//load high byte of address
		let high: u8 = self.read_next();
		self.emulate_hardware();

		//move contents of a to (a16)
		let address: u16 = ((high as u16) << 8) | (low as u16);
		let a: u8 = self.cpu.registers.a;
		self.write_byte_cpu(address, a);
		self.emulate_hardware();
	}

	///0xEE: XOR d8
	///2 M-Cycles
	///Length: 2 bytes
	fn xor_d8(&mut self) {
		let imm: u8 = self.read_next();
		self.emulate_hardware();

		self.cpu.registers.a = cpu::alu::xor(self.cpu.registers.a, imm, &mut self.cpu.registers.f);
	}

	///0xF0: LD A, (FF00 + a8)
	///3 M-Cycles
	///Length: 2 bytes
	///TODO: rename this
	///TODO: what happens if ff00 + a8 is outside of the memmory mapped io registers space
	fn ld_a_at_ff00_plus_a8(&mut self) {
		//Read from io register

		//1 cycle to read address
		let a8: u8 = self.read_next();
		self.emulate_hardware();

		//1 cycle to read from io port
		let val: u8 = self.read_byte_cpu(0xFF00 + a8 as u16);
		self.cpu.registers.a = val;
		self.emulate_hardware();
	}

	///0xF1: POP AF
	///3 M-Cycles
	///Length: 1 byte
	///Different from all of the pop_r16 instructions because the low nibble of F can only be 0
	fn pop_af(&mut self) {
		let mut val: u16 = self.pop();
		val &= 0xFFF0;
		self.cpu.registers.set_register_pair(RegisterPair::AF, val);
	}

	///0xF2: LD A, (C)
	///2 M-Cycles
	///Length: 1 byte
	fn ld_a_at_ff00_plus_c(&mut self) {
		//1 cycle to read from io port
		let val: u8 = self.read_byte_cpu(0xFF00 + self.cpu.registers.c as u16);
		self.cpu.registers.a = val;
		self.emulate_hardware();
	}

	///0xF3: DI
	///1 M-Cycle
	///1 Byte
	fn di(&mut self) {
		//Set next ime state to disabled
		self.cpu.next_ime_state = false;
	}

	///0xF6: OR d8
	///2 M-Cycles
	///Length: 2 bytes
	fn or_d8(&mut self) {
		let imm: u8 = self.read_next();
		self.emulate_hardware();

		self.cpu.registers.a = cpu::alu::or(self.cpu.registers.a, imm, &mut self.cpu.registers.f);
	}

	///0xF8: LD HL, SP + nn
	///3 M-Cycles
	///Length: 2 bytes
	fn ld_hl_sp_plus_nn(&mut self) {
		let nn: u8 = self.read_next();
		self.emulate_hardware();

		//1 cycle internal delay
		self.emulate_hardware();

		let hl: u16 = cpu::alu::add_sp_nn(self.cpu.registers.sp, nn, &mut self.cpu.registers.f);
		self.cpu.registers.set_register_pair(RegisterPair::HL, hl);
	}

	///0xF9: LD SP, HL
	///2 M-Cycles
	///Length: 1 byte
	fn ld_sp_hl(&mut self) {
		//internal delay
		self.emulate_hardware();

		let hl: u16 = self.cpu.registers.get_register_pair(RegisterPair::HL);
		self.cpu.registers.sp = hl;
	}

	///0xFA: LD A, (a16)
	///4 M-Cycles
	///Length: 3 bytes
	fn ld_a_at_a16(&mut self) {
		let addr_low = self.read_next();
		self.emulate_hardware();

		let addr_high = self.read_next();
		self.emulate_hardware();

		let address = ((addr_high as u16) << 8) | addr_low as u16;
		self.cpu.registers.a = self.read_byte_cpu(address);
		self.emulate_hardware();
	}

	///0xFB: EI
	///1 M-Cycle
	///Length: 1 byte
	fn ei(&mut self) {
		self.cpu.next_ime_state = true;
	}

	///0xFE: CP d8
	///2 M-Cycles
	///Length: 2 bytes
	fn cp_d8(&mut self) {
		//1 cycle to read imm
		let imm: u8 = self.read_next();
		self.emulate_hardware();

		cpu::alu::cp(self.cpu.registers.a, imm, &mut self.cpu.registers.f);
	}
}
