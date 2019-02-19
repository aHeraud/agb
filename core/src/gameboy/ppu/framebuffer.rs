use std::fmt;
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::de::{Visitor, SeqAccess};

pub const NUM_BUFFERS: usize = 2;

pub struct FrameBuffer<T: Copy + Sized + Default> {
	buffer: Box<[T]>,
	width: usize,
	height: usize,
	front_buffer_index: usize,
	back_buffer_index: usize,

	/// The number of pixels in the back buffer that have been modified in the current frame.
	///
	/// Pixels are expected to be drawn in order from top left to bottom right of the frame.
	///
	/// During serialization, only the first dirty_pixel_count pixels from the back buffer will be stored, and the rest of the
	/// contents of the buffer will be skipped.
	dirty_pixel_count: usize
}

impl<T: Copy + Sized + Default> FrameBuffer<T> {
	pub fn new(width: usize, height: usize) -> FrameBuffer<T> {
		FrameBuffer {
			buffer: vec![T::default(); width * height * NUM_BUFFERS].into_boxed_slice(),
			width: width,
			height: height,
			front_buffer_index: 0,
			back_buffer_index: 1,
			dirty_pixel_count: 0
		}
	}

	pub fn swap_buffers(&mut self) {
		let temp = self.front_buffer_index;
		self.front_buffer_index = self.back_buffer_index;
		self.back_buffer_index = temp;
		self.dirty_pixel_count = 0;
	}

	pub fn get_front_buffer(&self) -> &[T] {
		let buffer_size: usize = self.width * self.height;
		let buffer_start: usize = buffer_size * self.front_buffer_index;
		let buffer_end = buffer_start + buffer_size;
		&self.buffer[buffer_start .. buffer_end]
	}

	pub fn get_front_buffer_mut(&mut self) -> &mut [T] {
		let buffer_size: usize = self.width * self.height;
		let buffer_start: usize = buffer_size * self.front_buffer_index;
		let buffer_end = buffer_start + buffer_size;
		&mut self.buffer[buffer_start .. buffer_end]
	}

	pub fn set_pixel(&mut self, index: usize, value: T) {
		let back_buffer_start_index = self.back_buffer_index * self.width * self.height;
		self.buffer[back_buffer_start_index + index] = value;
		self.dirty_pixel_count = index;
	}
}

impl<T: Copy + Sized + Default + Serialize> Serialize for FrameBuffer<T> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
		use serde::ser::SerializeStruct;

		let mut state = serializer.serialize_struct("FrameBuffer", 6)?;
		state.serialize_field("width", &self.width)?;
		state.serialize_field("height", &self.height)?;
		state.serialize_field("front_buffer_index", &self.front_buffer_index)?;
		state.serialize_field("back_buffer_index", &self.back_buffer_index)?;
		state.serialize_field("dirty_pixel_count", &self.dirty_pixel_count)?;

		// only the pixels that have been written to the back buffer since the beginning of the current frame actually need to be seriazed,
		// the rest can be ignored
		let back_buffer_start_index = self.width * self.height * self.back_buffer_index;
		let back_buffer_end_index = back_buffer_start_index + self.dirty_pixel_count;
		state.serialize_field("buffer", &self.buffer[back_buffer_start_index..back_buffer_end_index])?;

		state.end()
	}
}

use std::marker::PhantomData;
struct FrameBufferVisitor<T>(PhantomData<T>);
impl<'de, T: Copy + Sized + Default> Visitor<'de> for FrameBufferVisitor<T> where T: Deserialize<'de> {
	type Value = FrameBuffer<T>;

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		formatter.write_str("struct FrameBuffer")
	}

	fn visit_seq<V>(self, mut seq: V) -> Result<FrameBuffer<T>, V::Error> where V: SeqAccess<'de> {
		use serde::de;
		use serde::de::Error;
		use std::ptr::copy_nonoverlapping;

		let width: usize = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(0, &self))?;
		let height: usize = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(1, &self))?;
		let front_buffer_index: usize = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(2, &self))?;
		let back_buffer_index: usize = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(3, &self))?;
		let dirty_pixel_count: usize = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(4, &self))?;
		let back_buf: Vec<T> = seq.next_element()?.ok_or_else(|| de::Error::invalid_length(5, &self))?; //TODO: do this without an allocation? I just want to borrow a slice from the deserializer so i can copy it into a larger slice

		// only the dirty pixels in the back buffer are serialized, we don't really care what the other values are in the buffer
		let mut buffer = vec![T::default(); width * height * NUM_BUFFERS].into_boxed_slice();
		unsafe {
			let back_buffer_start_index: usize = width * height * back_buffer_index;
			let slice = &mut buffer[back_buffer_start_index..(back_buffer_start_index + dirty_pixel_count)];
			if slice.len() != back_buf.len() {
				return Err(V::Error::custom("the serialized framebuffer length and value of dirty_pixel_count did not match"));
			}
			copy_nonoverlapping(back_buf.as_ptr(), slice.as_mut_ptr(), back_buf.len());
		}

		Ok(FrameBuffer {
			buffer: buffer,
			width: width,
			height: height,
			front_buffer_index: front_buffer_index,
			back_buffer_index: back_buffer_index,
			dirty_pixel_count: dirty_pixel_count
		})
	}
}

impl<'de, T: Copy + Sized + Default + Deserialize<'de>> Deserialize<'de> for FrameBuffer<T> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
		const FIELDS: &'static [&'static str] = &["width", "height", "front_buffer_index", "back_buffer_index", "dirty_pixel_count", "buffer"];
		deserializer.deserialize_struct("FrameBuffer", FIELDS, FrameBufferVisitor(PhantomData))
	}
}
