//! Neo N3 binary serialization
//!
//! Neo uses little-endian encoding with variable-length integers and
//! length-prefixed byte arrays. This module provides the reader/writer
//! primitives every protocol type builds on.

use crate::errors::{Error, Result};
use alloc::{format, string::String, vec::Vec};

/// Writes Neo's binary wire format.
#[derive(Debug, Default, Clone)]
pub struct Writer {
	buffer: Vec<u8>,
}

impl Writer {
	/// Creates an empty writer.
	pub fn new() -> Self {
		Self { buffer: Vec::new() }
	}

	/// Creates a writer with pre-allocated capacity.
	pub fn with_capacity(capacity: usize) -> Self {
		Self { buffer: Vec::with_capacity(capacity) }
	}

	/// Appends a single byte.
	pub fn write_u8(&mut self, value: u8) {
		self.buffer.push(value);
	}

	/// Appends a little-endian `u16`.
	pub fn write_u16(&mut self, value: u16) {
		self.buffer.extend_from_slice(&value.to_le_bytes());
	}

	/// Appends a little-endian `u32`.
	pub fn write_u32(&mut self, value: u32) {
		self.buffer.extend_from_slice(&value.to_le_bytes());
	}

	/// Appends a little-endian `u64`.
	pub fn write_u64(&mut self, value: u64) {
		self.buffer.extend_from_slice(&value.to_le_bytes());
	}

	/// Appends a little-endian `i64`.
	pub fn write_i64(&mut self, value: i64) {
		self.buffer.extend_from_slice(&value.to_le_bytes());
	}

	/// Appends raw bytes without a length prefix.
	pub fn write_bytes(&mut self, bytes: &[u8]) {
		self.buffer.extend_from_slice(bytes);
	}

	/// Appends a variable-length integer in Neo's canonical form.
	///
	/// Values below `0xFD` use one byte; larger values use a `0xFD`/`0xFE`/`0xFF`
	/// marker followed by a little-endian `u16`/`u32`/`u64`.
	pub fn write_var_int(&mut self, value: u64) {
		if value < 0xFD {
			self.write_u8(value as u8);
		} else if value <= u16::MAX as u64 {
			self.write_u8(0xFD);
			self.write_u16(value as u16);
		} else if value <= u32::MAX as u64 {
			self.write_u8(0xFE);
			self.write_u32(value as u32);
		} else {
			self.write_u8(0xFF);
			self.write_u64(value);
		}
	}

	/// Appends a length-prefixed byte array.
	pub fn write_var_bytes(&mut self, bytes: &[u8]) {
		self.write_var_int(bytes.len() as u64);
		self.write_bytes(bytes);
	}

	/// Appends a length-prefixed UTF-8 string.
	pub fn write_var_string(&mut self, s: &str) {
		self.write_var_bytes(s.as_bytes());
	}

	/// Appends a boolean as a single byte.
	pub fn write_bool(&mut self, value: bool) {
		self.write_u8(u8::from(value));
	}

	/// Appends a length-prefixed sequence of serializable items.
	pub fn write_serializable_list<T: Serializable>(&mut self, items: &[T]) {
		self.write_var_int(items.len() as u64);
		for item in items {
			item.serialize(self);
		}
	}

	/// Consumes the writer and returns the encoded bytes.
	pub fn into_bytes(self) -> Vec<u8> {
		self.buffer
	}

	/// Borrows the encoded bytes written so far.
	pub fn as_bytes(&self) -> &[u8] {
		&self.buffer
	}

	/// Number of bytes written so far.
	pub fn len(&self) -> usize {
		self.buffer.len()
	}

	/// Whether nothing has been written yet.
	pub fn is_empty(&self) -> bool {
		self.buffer.is_empty()
	}
}

/// Reads Neo's binary wire format.
#[derive(Debug, Clone)]
pub struct Reader<'a> {
	data: &'a [u8],
	position: usize,
}

impl<'a> Reader<'a> {
	/// Creates a reader over `data`.
	pub fn new(data: &'a [u8]) -> Self {
		Self { data, position: 0 }
	}

	/// Current read offset.
	pub fn position(&self) -> usize {
		self.position
	}

	/// Bytes not yet consumed.
	pub fn remaining(&self) -> usize {
		self.data.len().saturating_sub(self.position)
	}

	/// Whether every byte has been consumed.
	pub fn is_exhausted(&self) -> bool {
		self.remaining() == 0
	}

	fn take(&mut self, len: usize) -> Result<&'a [u8]> {
		if self.remaining() < len {
			return Err(Error::UnexpectedEof { needed: len, available: self.remaining() });
		}
		let slice = &self.data[self.position..self.position + len];
		self.position += len;
		Ok(slice)
	}

	/// Reads a single byte.
	pub fn read_u8(&mut self) -> Result<u8> {
		Ok(self.take(1)?[0])
	}

	/// Reads a little-endian `u16`.
	pub fn read_u16(&mut self) -> Result<u16> {
		let bytes = self.take(2)?;
		Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
	}

	/// Reads a little-endian `u32`.
	pub fn read_u32(&mut self) -> Result<u32> {
		let bytes = self.take(4)?;
		Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
	}

	/// Reads a little-endian `u64`.
	pub fn read_u64(&mut self) -> Result<u64> {
		let bytes = self.take(8)?;
		let mut array = [0u8; 8];
		array.copy_from_slice(bytes);
		Ok(u64::from_le_bytes(array))
	}

	/// Reads a little-endian `i64`.
	pub fn read_i64(&mut self) -> Result<i64> {
		Ok(self.read_u64()? as i64)
	}

	/// Reads a boolean; any non-zero byte is `true`.
	pub fn read_bool(&mut self) -> Result<bool> {
		Ok(self.read_u8()? != 0)
	}

	/// Reads `len` raw bytes.
	pub fn read_bytes(&mut self, len: usize) -> Result<&'a [u8]> {
		self.take(len)
	}

	/// Reads a variable-length integer, rejecting non-canonical encodings.
	///
	/// Neo requires the shortest form for a given value; a longer form is a
	/// consensus-level error, not a value to normalize.
	pub fn read_var_int(&mut self) -> Result<u64> {
		let marker = self.read_u8()?;
		match marker {
			0xFD => {
				let value = self.read_u16()? as u64;
				if value < 0xFD {
					return Err(Error::NonCanonicalVarInt { form: 3, value });
				}
				Ok(value)
			},
			0xFE => {
				let value = self.read_u32()? as u64;
				if value <= u16::MAX as u64 {
					return Err(Error::NonCanonicalVarInt { form: 5, value });
				}
				Ok(value)
			},
			0xFF => {
				let value = self.read_u64()?;
				if value <= u32::MAX as u64 {
					return Err(Error::NonCanonicalVarInt { form: 9, value });
				}
				Ok(value)
			},
			small => Ok(small as u64),
		}
	}

	/// Reads a length-prefixed byte array, rejecting lengths above `max`.
	pub fn read_var_bytes(&mut self, max: usize) -> Result<&'a [u8]> {
		let len = self.read_var_int()?;
		let len =
			usize::try_from(len).map_err(|_| Error::LengthLimitExceeded { len: u64::MAX, max })?;
		if len > max {
			return Err(Error::LengthLimitExceeded { len: len as u64, max });
		}
		self.take(len)
	}

	/// Reads a length-prefixed UTF-8 string, rejecting lengths above `max`.
	pub fn read_var_string(&mut self, max: usize) -> Result<String> {
		let bytes = self.read_var_bytes(max)?;
		String::from_utf8(bytes.to_vec())
			.map_err(|e| Error::InvalidData(format!("invalid UTF-8: {}", e)))
	}

	/// Reads a length-prefixed list, rejecting counts above `max`.
	pub fn read_serializable_list<T: Serializable>(&mut self, max: usize) -> Result<Vec<T>> {
		let count = self.read_var_int()?;
		let count = usize::try_from(count)
			.map_err(|_| Error::LengthLimitExceeded { len: u64::MAX, max })?;
		if count > max {
			return Err(Error::LengthLimitExceeded { len: count as u64, max });
		}

		let mut items = Vec::with_capacity(count.min(64));
		for _ in 0..count {
			items.push(T::deserialize(self)?);
		}
		Ok(items)
	}
}

/// A type with a canonical Neo binary encoding.
pub trait Serializable: Sized {
	/// Writes `self` to `writer`.
	fn serialize(&self, writer: &mut Writer);

	/// Reads a value from `reader`.
	fn deserialize(reader: &mut Reader<'_>) -> Result<Self>;

	/// Encodes `self` to a fresh byte vector.
	fn to_bytes(&self) -> Vec<u8> {
		let mut writer = Writer::new();
		self.serialize(&mut writer);
		writer.into_bytes()
	}

	/// Decodes a value from `bytes`, requiring that all input be consumed.
	fn from_bytes(bytes: &[u8]) -> Result<Self> {
		let mut reader = Reader::new(bytes);
		let value = Self::deserialize(&mut reader)?;
		if !reader.is_exhausted() {
			return Err(Error::TrailingBytes { remaining: reader.remaining() });
		}
		Ok(value)
	}

	/// Size of this value's encoding, in bytes.
	fn size(&self) -> usize {
		let mut writer = Writer::new();
		self.serialize(&mut writer);
		writer.len()
	}
}

/// Byte length of `value` encoded as a variable-length integer.
pub fn var_int_size(value: u64) -> usize {
	if value < 0xFD {
		1
	} else if value <= u16::MAX as u64 {
		3
	} else if value <= u32::MAX as u64 {
		5
	} else {
		9
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_write_read_integers() {
		let mut writer = Writer::new();
		writer.write_u8(0x12);
		writer.write_u16(0x3456);
		writer.write_u32(0x789a_bcde);
		writer.write_u64(0x0123_4567_89ab_cdef);
		let bytes = writer.into_bytes();

		let mut reader = Reader::new(&bytes);
		assert_eq!(reader.read_u8().unwrap(), 0x12);
		assert_eq!(reader.read_u16().unwrap(), 0x3456);
		assert_eq!(reader.read_u32().unwrap(), 0x789a_bcde);
		assert_eq!(reader.read_u64().unwrap(), 0x0123_4567_89ab_cdef);
		assert!(reader.is_exhausted());
	}

	#[test]
	fn test_integers_are_little_endian() {
		let mut writer = Writer::new();
		writer.write_u32(1);
		assert_eq!(writer.as_bytes(), &[0x01, 0x00, 0x00, 0x00]);
	}

	#[test]
	fn test_var_int_uses_shortest_form() {
		let cases: &[(u64, &[u8])] = &[
			(0, &[0x00]),
			(0xFC, &[0xFC]),
			(0xFD, &[0xFD, 0xFD, 0x00]),
			(0xFFFF, &[0xFD, 0xFF, 0xFF]),
			(0x1_0000, &[0xFE, 0x00, 0x00, 0x01, 0x00]),
			(0xFFFF_FFFF, &[0xFE, 0xFF, 0xFF, 0xFF, 0xFF]),
			(0x1_0000_0000, &[0xFF, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00]),
		];

		for (value, expected) in cases {
			let mut writer = Writer::new();
			writer.write_var_int(*value);
			assert_eq!(writer.as_bytes(), *expected, "encoding {:#x}", value);
			assert_eq!(var_int_size(*value), expected.len(), "size of {:#x}", value);

			let mut reader = Reader::new(expected);
			assert_eq!(reader.read_var_int().unwrap(), *value);
		}
	}

	#[test]
	fn test_var_int_rejects_non_canonical_3_byte_form() {
		// 0x01 fits in one byte, so the 3-byte form is invalid.
		let mut reader = Reader::new(&[0xFD, 0x01, 0x00]);
		assert!(matches!(reader.read_var_int(), Err(Error::NonCanonicalVarInt { form: 3, .. })));
	}

	#[test]
	fn test_var_int_rejects_non_canonical_5_byte_form() {
		let mut reader = Reader::new(&[0xFE, 0x01, 0x00, 0x00, 0x00]);
		assert!(matches!(reader.read_var_int(), Err(Error::NonCanonicalVarInt { form: 5, .. })));
	}

	#[test]
	fn test_var_int_rejects_non_canonical_9_byte_form() {
		let mut reader = Reader::new(&[0xFF, 0x01, 0, 0, 0, 0, 0, 0, 0]);
		assert!(matches!(reader.read_var_int(), Err(Error::NonCanonicalVarInt { form: 9, .. })));
	}

	#[test]
	fn test_var_bytes_roundtrip() {
		let payload = b"neo protocol";
		let mut writer = Writer::new();
		writer.write_var_bytes(payload);

		let bytes = writer.into_bytes();
		let mut reader = Reader::new(&bytes);
		assert_eq!(reader.read_var_bytes(1024).unwrap(), payload);
		assert!(reader.is_exhausted());
	}

	#[test]
	fn test_var_bytes_enforces_max_length() {
		let mut writer = Writer::new();
		writer.write_var_bytes(&[0u8; 100]);

		let bytes = writer.into_bytes();
		let mut reader = Reader::new(&bytes);
		assert!(matches!(
			reader.read_var_bytes(50),
			Err(Error::LengthLimitExceeded { len: 100, max: 50 })
		));
	}

	#[test]
	fn test_var_string_roundtrip() {
		let mut writer = Writer::new();
		writer.write_var_string("Neo N3 测试");

		let bytes = writer.into_bytes();
		let mut reader = Reader::new(&bytes);
		assert_eq!(reader.read_var_string(1024).unwrap(), "Neo N3 测试");
	}

	#[test]
	fn test_var_string_rejects_invalid_utf8() {
		let mut writer = Writer::new();
		writer.write_var_bytes(&[0xFF, 0xFE]);

		let bytes = writer.into_bytes();
		let mut reader = Reader::new(&bytes);
		assert!(matches!(reader.read_var_string(1024), Err(Error::InvalidData(_))));
	}

	#[test]
	fn test_reader_reports_eof_with_context() {
		let mut reader = Reader::new(&[0x01, 0x02]);
		assert!(matches!(reader.read_u32(), Err(Error::UnexpectedEof { needed: 4, available: 2 })));
	}

	#[test]
	fn test_bool_roundtrip() {
		let mut writer = Writer::new();
		writer.write_bool(true);
		writer.write_bool(false);

		let bytes = writer.into_bytes();
		assert_eq!(bytes, vec![1, 0]);

		let mut reader = Reader::new(&bytes);
		assert!(reader.read_bool().unwrap());
		assert!(!reader.read_bool().unwrap());
	}
}
