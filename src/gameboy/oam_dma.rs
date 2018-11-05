use ::gameboy::Gameboy;

/// Holds the state of the oam dma controller.
/// When a value is written to $FF46, it begins a dma transfer that copies 0xA0 bytes to the range 0xFE00 - 0xFE9F.
/// The address of the beginning of the source memory is the value written to $FF46 * 0x100.
/// The source address must be in the range 0x0000 - 0xF100.
///
/// The oam dma transfer copies 160 bytes. Including the 2 M-Cycles for startup/teardown, the entire process
/// takes 162 M-Cycles.
///
/// During the oam dma transfer, the cpu can't access the oam, and it also can't access the same region of memory that the oam dma controller is reading from.
/// For now I am operating under the impression that if the cpu or oam dma controller is accessing cartridge memory (rom/cart ram), the other can access vram.
/// TODO: can the cpu access the mmio registers or the interrupt enable register at 0xFFFF?
///
/// There is a 1 M-Cycle delay at the beginning and end of the oam dma transfer when the cpu can freely access memory like the oam dma wasn't running.
/// However, if an oam dma is started while another oam dma transfer is running, the old dma transfer will still be running in the 1 M-Cycle start up period,
/// and the cpu won't be able to access the memory region the dma controller is reading from.
pub struct OamDmaState {
	/// Is there currently an active oam dma transfer
	pub active: bool,

	// If this is set to true, then the currently running oam dma transfer was started while another one was running.
	// This means that in the first M-Cycle of this dma transfer, the other dma transfer may still be active, which could block
	// the cpu from accessing certain regions of memory.
	//pub was_reset: bool,

	/// The beginning of the block of memory to be copied to oam.
	pub start_address: u16,

	/// The offset from the start address to the next byte to be copied.
	pub current_offset: u16,

	/// How many 4MHz cycles the oam dma has been running
	pub current_cycle: u16
}

impl OamDmaState {
	pub fn new() -> OamDmaState {
		OamDmaState {
			active: false,
			start_address: 0,
			current_offset: 0,
			current_cycle: 0
		}
	}

	pub fn reset(&mut self) {
		self.active = false;
		self.start_address = 0;
		self.current_offset = 0;
		self.current_cycle = 0;
	}

	/// The address that the oam dma controller is currently reading from.
	pub fn current_src_address(&self) -> Option<u16> {
		if self.active && self.current_cycle > 4 {
			Some(self.start_address + self.current_offset)
		}
		else {
			None
		}
	}

	pub fn start_oam_dma(&mut self, addr_high: u8) {
		self.active = true;
		self.current_cycle = 0;
		self.current_offset = 0;
		self.start_address = (addr_high as u16) << 8;
	}

	/// Reading from $FF46 (the register you write to to start the oam dma) just returns the last value written.
	pub fn read_ff46(&self) -> u8 {
		(self.start_address >> 8) as u8
	}

	/// The oam dma controller and cpu can't read from certain regions of memory at the same time.
	/// When the cpu tries to read from the region that the oam controller is reading from, it reads $FF.
	pub fn should_block_cpu_access(&self, address: u16) -> bool {
		if let Some(src) = self.current_src_address() {
			match address {
				0...0x7FFF | 0xA000...0xFDFF => (src < 0x8000) || (src >= 0xA000 && src < 0xFE00), // external bus conflict
				0x8000...0x9FFF => src >= 0x8000 && src < 0xA000, //vram conflict
				0xFE00...0xFE9F => true, // oam dma is always writing to oam, so cpu can never access it (except during first M-Cycle of oam dma)
				_ => false
			}
		}
		else {
			false
		}
	}
}

pub trait OamDmaController {
	/// Start an oam dma transfer.
	/// addr_high is the high byte of the source address.
	fn start_oam_dma(&mut self, addr_high: u8);

	/// Service any currently running oam dma transfer.
	/// This should be called every 4MHz clock
	fn service_oam_dma_transfer(&mut self);
}

impl OamDmaController for Gameboy {
	/// TODO: What happens when you write an invalid address to $FF46?
	/// For example, what if you try to do an oam dma transfer where the source is in OAM?
	fn start_oam_dma(&mut self, addr_high: u8) {
		self.oam_dma_state.active = true;
		self.oam_dma_state.current_cycle = 0;
		self.oam_dma_state.current_offset = 0;
		self.oam_dma_state.start_address = (addr_high as u16) << 8;
	}

	fn service_oam_dma_transfer(&mut self) {
		use gameboy::mmu::Mmu;
		if !self.cpu.halt && self.oam_dma_state.active {
			if self.oam_dma_state.current_cycle >= 4 && self.oam_dma_state.current_cycle <= 644 {
				if self.oam_dma_state.current_cycle % 4 == 0 {
					//copy a byte
					let src = self.oam_dma_state.start_address + self.oam_dma_state.current_offset;
					let byte = self.read_byte(src);
					let dest = 0xFE00 + self.oam_dma_state.current_offset;
					self.write_byte(dest, byte);
					self.oam_dma_state.current_offset += 1;
				}
			}

			self.oam_dma_state.current_cycle += 1;
			if self.oam_dma_state.current_cycle >= 648 {
				self.oam_dma_state.active = false;
			}
		}
	}
}
