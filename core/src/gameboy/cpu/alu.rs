use super::{ZERO_FLAG_MASK, SUBTRACTION_FLAG_MASK, HALF_CARRY_FLAG_MASK, CARRY_FLAG_MASK};
use std::num::Wrapping;

///Adds the values of 2 8-bit registers together, returns the result as a u8.
///The resulting value of the flags register is: Z 0 H C
pub fn add(register: u8, other: u8, flags: &mut u8) -> u8 {
	let result: u16 = (register as u16) + (other as u16);
	*flags = 0;
	*flags |= !(((result & 0x007F) + 0x007F) | result) as u8 & ZERO_FLAG_MASK;
	*flags |= (((register & 0x000F) + (other & 0x000F)) << 1) & HALF_CARRY_FLAG_MASK;
	*flags |= (result >> 4) as u8 & CARRY_FLAG_MASK;
	(result & 0xFF) as u8
}

///Adds the values of 2 8-bit registers together, returns the result as a u8.
///The value of the carry flag is used as a carry in to the lower 4-bit adder.
///The resulting value of the flags register is: Z 0 H C
pub fn adc(register: u8, other: u8, flags: &mut u8) -> u8 {
	let cy: u8 = (*flags & CARRY_FLAG_MASK) >> 4;
	let result: u16 = register as u16 + other as u16 + cy as u16;
	*flags = 0;
	*flags |= !(((result & 0x007F) + 0x007F) | result) as u8 & ZERO_FLAG_MASK;
	*flags |= (((register & 0x000F) + (other & 0x000F) + cy) << 1) & HALF_CARRY_FLAG_MASK;
	*flags |= (result >> 4) as u8 & CARRY_FLAG_MASK;
	(result & 0xFF) as u8
}

///Subtracts the value of the second register from the first register, and returns the result as a u8.
///The resulting value of the flags register is: Z 1 H C
pub fn sub(register: u8, other: u8, flags: &mut u8) -> u8 {
	let result: u32 = (register as u32).wrapping_sub(other as u32);
	*flags = 0;
	if (other & 0x0F) > (register & 0x0F) {
		*flags |= HALF_CARRY_FLAG_MASK;
	}
	*flags |= !(((result & 0x7F) + 0x7F) | result) as u8 & ZERO_FLAG_MASK;
	*flags |= SUBTRACTION_FLAG_MASK;
	*flags |= (result >> 4) as u8 & CARRY_FLAG_MASK;
	(result & 0xFF) as u8
}

///Subtracts the value of the second register from the first register, and returns the result as a u8.
///Also subtracts 1 if the carry flag is set.
///The resulting value of the flags register is: Z 1 H C
pub fn sbc(register: u8, other: u8, flags: &mut u8) -> u8 {
	let cy: u32 = ((*flags & CARRY_FLAG_MASK) >> 4) as u32;
	let result: u32 = (register as u32).wrapping_sub(other as u32).wrapping_sub(cy);
	*flags = SUBTRACTION_FLAG_MASK;
	*flags |= !(((result & 0x007F) + 0x007F) | result) as u8 & ZERO_FLAG_MASK;
	//*flags |= (((register & 0x000F) - (other & 0x000F) - (cy as u8)) << 1) & HALF_CARRY_FLAG_MASK;	//rust no like overflow
	*flags |= (((register & 0x000F).wrapping_sub(other & 0x000F).wrapping_sub(cy as u8)) << 1) & HALF_CARRY_FLAG_MASK;	//TODO: test this
	*flags |= (result >> 4) as u8 & CARRY_FLAG_MASK;
	(result & 0xFF) as u8
}

///Performs a bitwise AND of 2 8-bit registers, and returns the result as a u8.
///The resulting value of the flags register is: Z 0 1 0
pub fn and(register: u8, other: u8, flags: &mut u8) -> u8 {
	let result: u8 = register & other;
	*flags = 0;
	*flags |= HALF_CARRY_FLAG_MASK;
	*flags |= !(((result & 0x7F) + 0x7F) | result) as u8 & ZERO_FLAG_MASK;
	(result & 0xFF) as u8
}

///Performs a bitwise XOR of 2 8-bit registers, and returns the result as a u8.
///The resulting value of the flags register is: Z 0 0 0
pub fn xor(register: u8, other: u8, flags: &mut u8) -> u8 {
	let result: u16 = (register ^ other) as u16;
	*flags = !(((result & 0x007F) + 0x007F) | result) as u8 & ZERO_FLAG_MASK;;
	result as u8
}

///Performs a bitwise OR of 2 8-bit registers, and returns the result as a u8.
///The resulting value of the flags register is: Z 0 0 0
pub fn or(register: u8, other: u8, flags: &mut u8) -> u8 {
	let result: u16 = (register | other) as u16;
	*flags = !(((result & 0x007F) + 0x007F) | result) as u8 & ZERO_FLAG_MASK;
	result as u8
}

///Subtracts the value of other from register, and sets the flags accoring to the definition of the sub operation.
///The result of the subtraction is discarded, and only the value of the flags is kept.
pub fn cp(register: u8, other: u8, flags: &mut u8) {
	sub(register, other, flags);
}

///Increment an 8-bit register by 1. The new value of the register is retured as a u8
///The previous value of the Carry Flag is preserved.
///The resulting value of the flags register is: Z 0 H -
pub fn inc(register: u8, flags: &mut u8) -> u8 {
	let result: u16 = register as u16 + 1;
	//preserve Carry Flag
	*flags &= CARRY_FLAG_MASK;
	*flags |= !(((result & 0x007F) + 0x007F) | result) as u8 & ZERO_FLAG_MASK;
	*flags |= (((register as u8 & 0x0F) + 1) << 1) & HALF_CARRY_FLAG_MASK;
	(result & 0xFF) as u8
}

///Decrements an 8-bit register by 1. The new value of the register is returned as a u8.
///The previous value of the Carry Flag is preserved.
///The resulting value of the flags register is: Z 1 H -
pub fn dec(register: u8, flags: &mut u8) -> u8 {
	let mut temp_flags = 0;
	let result: u8 = sub(register, 1, &mut temp_flags);
	temp_flags &= 0b11100000;
	*flags &= CARRY_FLAG_MASK;
	*flags |= temp_flags;
	result
}

///Performs an 8-bit left rotate on the register. The new value of the register is returned as a u8.
///The resulting value of the flags register is: Z 0 0 C
///The bit that is shifted out from the msb is placed into the carry (as well as moved to the lsb)
pub fn rlc(register: u8, flags: &mut u8) -> u8 {
	let msb: u8 = register & 128;
	let result: u8 = (register << 1) | (msb >> 7);
	*flags = 0;
	*flags |= msb >> 3;
	*flags |= !(((result & 0x7F) + 0x7F) | result) as u8 & ZERO_FLAG_MASK;
	(result & 0xFF) as u8
}

///Performs an 8-bit right rotate on the register. The new value of the register is returned as a u8.
///The bit that is shifted out from the lsb is placed into the carry (as well as moved to the msb)
///The resulting value of the flags register is: Z 0 0 C
pub fn rrc(register: u8, flags: &mut u8) -> u8 {
	let lsb: u16 = (register & 1) as u16;
	let result: u16 = ((register >> 1) as u16) | (lsb << 7);
	*flags = 0;
	*flags |= (lsb as u8) << 4;
	*flags |= !(((result & 0x007F) + 0x007F) | result) as u8 & ZERO_FLAG_MASK;
	(result & 0xFF) as u8
}

///Performs a 9-bit left rotate throught the carry flag on the register.
///The new value of the register is returned as a u8.
///The msb that is rotated out is put in the carry flag, and the previous value of the carry flag
///is rotated into the lsb.
///The resulting value of the flags register is: Z 0 0 C
pub fn rl(register: u8, flags: &mut u8) -> u8 {
	let msb: u8 = register & 128;
	let result: u8 = (register << 1) | ((*flags & CARRY_FLAG_MASK)>> 4);
	*flags = 0;
	*flags |= msb >> 3;	//CY
	*flags |= !(((result & 0x7F) + 0x7F) | result) & ZERO_FLAG_MASK;
	result
}

///Performs a 9-bit right rotate throught the carry flag on the register.
///The new value of the register is returned as a u8.
///The lsb that is rotated out is put in the carry flag, and the previous value of the carry flag
///is rotated into the msb.
///The resulting value of the flags register is: Z 0 0 C
pub fn rr(register: u8, flags: &mut u8) -> u8 {
	let lsb: u8 = register & 1;
	let result: u8 = (register >> 1) | ((*flags & CARRY_FLAG_MASK) << 3);
	*flags = 0;
	*flags |= lsb << 4;	//CY
	*flags |= !(((result & 0x7F) + 0x7F) | result) & ZERO_FLAG_MASK;
	result
}

///Performs a left shift on the register, a 0 is shifted into the lsb.
///The new value of the register is returned as a u8.
///The resulting value of the flags register is: Z 0 0 C
pub fn sla(register: u8, flags: &mut u8) -> u8 {
	let result: u16 = (register as u16) << 1;
	*flags = 0;
	*flags |= ((result & 0x100) >> 4) as u8;
	*flags |= !(((result & 0x007F) + 0x007F) | result) as u8 & ZERO_FLAG_MASK;
	(result & 0xFF) as u8
}

///Performs an arithmetic (signed) right shift (the value of the msb stays the same).
///The new value of the register is returned as a u8.
///The resulting value of the flags register is: Z 0 0 C
pub fn sra(register: u8, flags: &mut u8) -> u8 {
	let msb: u16 = register as u16 & 128;
	let result: u16 = (register >> 1) as u16 | msb;
	*flags = (register & 1) << 4;
	*flags |= !(((result & 0x007F) + 0x007F) | result) as u8 & ZERO_FLAG_MASK;
	(result & 0xFF) as u8
}

///Performs a logical (unsigned) right shift (a 0 is shifted in on the left).
///The new value of the register is returned as a u8.
///The resulting value of the flags register is: Z 0 0 C
pub fn srl(register: u8, flags: &mut u8) -> u8 {
	let result: u16 = (register as u16) >> 1;
	*flags = 0;
	*flags |= (register << 4) & CARRY_FLAG_MASK;
	*flags |= !(((result & 0x007F) + 0x007F) | result) as u8 & ZERO_FLAG_MASK;
	(result & 0xFF) as u8
}

///Spaws the high and low nibble of the register.
///The new value of the register is returned as a u8.
///The resulting value of the flags register is: Z 0 0 0
pub fn swap(register: u8, flags: &mut u8) -> u8 {
	let result: u16 = ((register << 4) | (register >> 4)) as u16;
	*flags = !(((result & 0x007F) + 0x007F) | result) as u8 & ZERO_FLAG_MASK;
	(result & 0xFF) as u8
}

///Tests bit n in the register.
///The Carry Flag is preserved.
///Flags: Z 0 1 -
pub fn bit(register: u8, flags: &mut u8, bit: u8) {
	let bitmask: u8 = 1 << bit;
	let result: u8 = register & bitmask;
	*flags &= CARRY_FLAG_MASK;
	*flags |= HALF_CARRY_FLAG_MASK;
	*flags |= !(((result & 0x7F) + 0x7F) | result) & ZERO_FLAG_MASK;
}

///Set bit n
///Returns the result as a u8.
pub fn set(register: u8, bit: u8) -> u8 {
	register | 1u8 << bit
}

///Reset bit n
///Returns the result as a u8.
pub fn res(register: u8, bit: u8) -> u8 {
	register & !(1 << bit)
}

///Adds a 16 bit value to HL, not to be confused with ADD SP, r8 or LD HL, SP+r8
///The Zero Flag is preserved
///Flags: - 0 H C
pub fn add16(hl: u16, other: u16, flags: &mut u8) -> u16 {
	let result: u32 = (hl as u32) + (other as u32);
	*flags &= ZERO_FLAG_MASK;	//Prezerve zero flag
	if (hl & 0x0FFF) + (other & 0x0FFF) > 0x0FFF {
		//set Half Carry
		*flags |= HALF_CARRY_FLAG_MASK;
	}
	if result > 0xFFFF {
		//set Carry Flag
		*flags |= CARRY_FLAG_MASK;
	}
	result as u16
}

///For 0xE8: ADD SP, r8
///Add a signed byte to sp
///The HC and C flags are set like regular 8-bit addition
///Flags: 0 0 H C
pub fn add_sp_nn(sp: u16, other: u8, flags: &mut u8) -> u16 {
	*flags = 0;
	if (sp & 0x0F) as u8 + (other & 0x0F) > 0x0F {
		*flags |= HALF_CARRY_FLAG_MASK;
	}
	if (sp & 0x00FF) + (other as u16) > 0x00FF {
		*flags |= CARRY_FLAG_MASK;
	}
	(Wrapping(sp as i16) + Wrapping((other as i8) as i16)).0 as u16
}
