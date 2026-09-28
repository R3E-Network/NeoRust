//! Hash types for Neo N3
//!
//! Neo uses two hash types:
//! - [`Hash160`] (20 bytes): script hashes, contract hashes
//! - [`Hash256`] (32 bytes): transaction hashes, block hashes
//!
//! # Byte order
//!
//! Neo is inconsistent about hash byte order, which is a common source of bugs.
//! This module fixes one convention:
//!
//! - **Internal storage** is wire order (little-endian) — the raw output of
//!   `RIPEMD160(SHA256(script))`, and what goes into serialized transactions.
//! - **Display form** ([`Display`](core::fmt::Display), [`Hash160::to_hex`]) is
//!   big-endian with a `0x` prefix — what `neo-cli`, block explorers and RPC
//!   responses show, e.g. the GAS token as `0xd2a4cff3...`.
//!
//! [`Hash160::from_hex`] always parses the display (big-endian) form, whether or
//! not the `0x` prefix is present. Use [`Hash160::from_bytes`] /
//! [`Hash160::from_slice`] for wire-order bytes.

use crate::{
	crypto,
	errors::{Error, Result},
};
use alloc::{format, string::String, vec::Vec};
use core::{fmt, str::FromStr};
use serde::{Deserialize, Serialize};

/// A 160-bit hash (20 bytes): script hashes and contract hashes.
///
/// Stored in wire (little-endian) order; displayed big-endian with `0x`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hash160([u8; 20]);

impl Hash160 {
	/// Size in bytes.
	pub const LEN: usize = 20;

	/// Wraps raw bytes already in wire (little-endian) order.
	pub const fn from_bytes(bytes: [u8; 20]) -> Self {
		Self(bytes)
	}

	/// Wraps a wire-order byte slice; the slice must be exactly 20 bytes.
	pub fn from_slice(slice: &[u8]) -> Result<Self> {
		let bytes: [u8; 20] = slice.try_into().map_err(|_| {
			Error::InvalidHash(format!("expected {} bytes, got {}", Self::LEN, slice.len()))
		})?;
		Ok(Self(bytes))
	}

	/// Parses the big-endian display form, with or without a `0x` prefix.
	pub fn from_hex(s: &str) -> Result<Self> {
		let stripped = s.strip_prefix("0x").unwrap_or(s);
		let mut bytes: [u8; 20] = decode_hex_exact(stripped, Self::LEN)?
			.try_into()
			.map_err(|_| Error::InvalidHash("unexpected length".into()))?;
		bytes.reverse();
		Ok(Self(bytes))
	}

	/// Computes the script hash of `script`: `RIPEMD160(SHA256(script))`.
	pub fn from_script(script: &[u8]) -> Self {
		Self(crypto::hash160(script))
	}

	/// The raw bytes in wire (little-endian) order.
	pub const fn as_bytes(&self) -> &[u8; 20] {
		&self.0
	}

	/// The raw bytes in wire (little-endian) order.
	pub fn to_vec(&self) -> Vec<u8> {
		self.0.to_vec()
	}

	/// The big-endian display form, prefixed with `0x`.
	pub fn to_hex(&self) -> String {
		let mut bytes = self.0;
		bytes.reverse();
		format!("0x{}", hex::encode(bytes))
	}

	/// The all-zero hash.
	pub const fn zero() -> Self {
		Self([0u8; 20])
	}

	/// Whether this is the all-zero hash.
	pub fn is_zero(&self) -> bool {
		self.0 == [0u8; 20]
	}
}

impl Default for Hash160 {
	fn default() -> Self {
		Self::zero()
	}
}

impl fmt::Display for Hash160 {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(&self.to_hex())
	}
}

impl FromStr for Hash160 {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self> {
		Self::from_hex(s)
	}
}

/// A 256-bit hash (32 bytes): transaction hashes and block hashes.
///
/// Stored in wire (little-endian) order; displayed big-endian with `0x`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hash256([u8; 32]);

impl Hash256 {
	/// Size in bytes.
	pub const LEN: usize = 32;

	/// Wraps raw bytes already in wire (little-endian) order.
	pub const fn from_bytes(bytes: [u8; 32]) -> Self {
		Self(bytes)
	}

	/// Wraps a wire-order byte slice; the slice must be exactly 32 bytes.
	pub fn from_slice(slice: &[u8]) -> Result<Self> {
		let bytes: [u8; 32] = slice.try_into().map_err(|_| {
			Error::InvalidHash(format!("expected {} bytes, got {}", Self::LEN, slice.len()))
		})?;
		Ok(Self(bytes))
	}

	/// Parses the big-endian display form, with or without a `0x` prefix.
	pub fn from_hex(s: &str) -> Result<Self> {
		let stripped = s.strip_prefix("0x").unwrap_or(s);
		let mut bytes: [u8; 32] = decode_hex_exact(stripped, Self::LEN)?
			.try_into()
			.map_err(|_| Error::InvalidHash("unexpected length".into()))?;
		bytes.reverse();
		Ok(Self(bytes))
	}

	/// Computes `SHA256(SHA256(data))`, Neo's transaction/block hash.
	pub fn from_data(data: &[u8]) -> Self {
		Self(crypto::hash256(data))
	}

	/// The raw bytes in wire (little-endian) order.
	pub const fn as_bytes(&self) -> &[u8; 32] {
		&self.0
	}

	/// The raw bytes in wire (little-endian) order.
	pub fn to_vec(&self) -> Vec<u8> {
		self.0.to_vec()
	}

	/// The big-endian display form, prefixed with `0x`.
	pub fn to_hex(&self) -> String {
		let mut bytes = self.0;
		bytes.reverse();
		format!("0x{}", hex::encode(bytes))
	}

	/// The all-zero hash.
	pub const fn zero() -> Self {
		Self([0u8; 32])
	}

	/// Whether this is the all-zero hash.
	pub fn is_zero(&self) -> bool {
		self.0 == [0u8; 32]
	}
}

impl Default for Hash256 {
	fn default() -> Self {
		Self::zero()
	}
}

impl fmt::Display for Hash256 {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(&self.to_hex())
	}
}

impl FromStr for Hash256 {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self> {
		Self::from_hex(s)
	}
}

/// Decodes hex, requiring exactly `len` decoded bytes.
fn decode_hex_exact(s: &str, len: usize) -> Result<Vec<u8>> {
	let bytes =
		hex::decode(s).map_err(|e| Error::InvalidHash(format!("invalid hex: {}", e)))?;
	if bytes.len() != len {
		return Err(Error::InvalidHash(format!(
			"expected {} bytes, got {}",
			len,
			bytes.len()
		)));
	}
	Ok(bytes)
}

// Serde: hashes travel over JSON-RPC as the big-endian display string, so that
// is what we serialize. This keeps round-trips through RPC payloads lossless.

impl Serialize for Hash160 {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error> {
		serializer.serialize_str(&self.to_hex())
	}
}

impl<'de> Deserialize<'de> for Hash160 {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> core::result::Result<Self, D::Error> {
		let s = String::deserialize(deserializer)?;
		Self::from_hex(&s).map_err(serde::de::Error::custom)
	}
}

impl Serialize for Hash256 {
	fn serialize<S: serde::Serializer>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error> {
		serializer.serialize_str(&self.to_hex())
	}
}

impl<'de> Deserialize<'de> for Hash256 {
	fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> core::result::Result<Self, D::Error> {
		let s = String::deserialize(deserializer)?;
		Self::from_hex(&s).map_err(serde::de::Error::custom)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	const GAS_TOKEN: &str = "0xd2a4cff31913016155e38e474a2c06d08be276cf";
	const DEFAULT_SCRIPT_HASH: &str = "0x69ecca587293047be4c59159bf8bc399985c160d";

	#[test]
	fn test_hash160_roundtrip() {
		let hash = Hash160::from_hex(GAS_TOKEN).unwrap();
		assert_eq!(hash.to_hex(), GAS_TOKEN);
	}

	#[test]
	fn test_hash160_wire_order_is_reversed_from_display() {
		let hash = Hash160::from_hex(GAS_TOKEN).unwrap();
		// Wire order is the display bytes reversed.
		assert_eq!(hex::encode(hash.as_bytes()), "cf76e28bd0062c4a478ee35561011319f3cfa4d2");

		// And reversing again recovers the display form.
		let mut wire = *hash.as_bytes();
		wire.reverse();
		assert_eq!(format!("0x{}", hex::encode(wire)), GAS_TOKEN);
	}

	#[test]
	fn test_hash160_accepts_hex_without_prefix() {
		let with = Hash160::from_hex(GAS_TOKEN).unwrap();
		let without = Hash160::from_hex(&GAS_TOKEN[2..]).unwrap();
		assert_eq!(with, without);
	}

	#[test]
	fn test_hash160_from_str() {
		let hash: Hash160 = GAS_TOKEN.parse().unwrap();
		assert_eq!(hash.to_hex(), GAS_TOKEN);
	}

	#[test]
	fn test_hash160_rejects_wrong_length() {
		assert!(Hash160::from_hex("0xdead").is_err());
		assert!(Hash160::from_slice(&[0u8; 19]).is_err());
		assert!(Hash160::from_slice(&[0u8; 21]).is_err());
		assert!(Hash160::from_slice(&[0u8; 20]).is_ok());
	}

	#[test]
	fn test_hash160_rejects_invalid_hex() {
		assert!(Hash160::from_hex("0xZZZZ").is_err());
	}

	#[test]
	fn test_hash160_zero() {
		assert_eq!(Hash160::zero(), Hash160::default());
		assert!(Hash160::zero().is_zero());
		assert_eq!(Hash160::zero().as_bytes(), &[0u8; 20]);
	}

	#[test]
	fn test_hash160_from_script() {
		let script = hex::decode(
			"0c21033a4d051b04b7fc0230d2b1aaedfd5a84be279a5361a7358db665ad7857787f1b4156e7b327",
		)
		.unwrap();
		let hash = Hash160::from_script(&script);
		assert_eq!(hash.to_hex(), DEFAULT_SCRIPT_HASH);
	}

	#[test]
	fn test_hash256_roundtrip() {
		let display = "0x0000000000000000000000000000000000000000000000000000000000000001";
		let hash = Hash256::from_hex(display).unwrap();
		assert_eq!(hash.to_hex(), display);
		assert_eq!(hash.as_bytes()[0], 1);
	}

	#[test]
	fn test_hash256_from_data() {
		let data = b"neo";
		let expected = crypto::hash256(data);
		assert_eq!(Hash256::from_data(data).as_bytes(), &expected);
	}

	#[test]
	fn test_hash256_rejects_wrong_length() {
		assert!(Hash256::from_hex("0xdead").is_err());
		assert!(Hash256::from_slice(&[0u8; 31]).is_err());
		assert!(Hash256::from_slice(&[0u8; 32]).is_ok());
	}

	#[test]
	fn test_serde_roundtrip_hash160() {
		let hash = Hash160::from_hex(GAS_TOKEN).unwrap();
		let json = serde_json::to_string(&hash).unwrap();
		assert_eq!(json, format!("\"{}\"", GAS_TOKEN));
		let back: Hash160 = serde_json::from_str(&json).unwrap();
		assert_eq!(hash, back);
	}

	#[test]
	fn test_serde_roundtrip_hash256() {
		let hash = Hash256::from_hex(
			"0x0000000000000000000000000000000000000000000000000000000000000042",
		)
		.unwrap();
		let json = serde_json::to_string(&hash).unwrap();
		let back: Hash256 = serde_json::from_str(&json).unwrap();
		assert_eq!(hash, back);
	}
}
