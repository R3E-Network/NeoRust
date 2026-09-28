//! Neo N3 script types

use crate::types::Hash160;
use alloc::vec::Vec;
use core::fmt;
use serde::{Deserialize, Serialize};

/// A Neo N3 script (contract bytecode or verification script).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Script(#[serde(with = "hex_bytes")] Vec<u8>);

impl Script {
	/// Wraps raw script bytes.
	pub fn new(bytes: Vec<u8>) -> Self {
		Self(bytes)
	}

	/// The script bytes.
	pub fn as_bytes(&self) -> &[u8] {
		&self.0
	}

	/// Script length in bytes.
	pub fn len(&self) -> usize {
		self.0.len()
	}

	/// Whether the script is empty.
	pub fn is_empty(&self) -> bool {
		self.0.is_empty()
	}

	/// Compute the script hash: `RIPEMD160(SHA256(script))`.
	///
	/// The result is in wire order; call [`Hash160::to_hex`] for the display form.
	pub fn to_script_hash(&self) -> Hash160 {
		Hash160::from_script(&self.0)
	}

	/// Parse from hex (with or without `0x` prefix).
	pub fn from_hex(s: &str) -> Result<Self, crate::errors::Error> {
		let s = s.strip_prefix("0x").unwrap_or(s);
		let bytes = hex::decode(s).map_err(|e| {
			crate::errors::Error::InvalidScript(alloc::format!("hex decode: {}", e))
		})?;
		Ok(Self(bytes))
	}

	/// Encode as hex with `0x` prefix.
	pub fn to_hex(&self) -> alloc::string::String {
		alloc::format!("0x{}", hex::encode(&self.0))
	}
}

impl From<Vec<u8>> for Script {
	fn from(bytes: Vec<u8>) -> Self {
		Self(bytes)
	}
}

impl AsRef<[u8]> for Script {
	fn as_ref(&self) -> &[u8] {
		&self.0
	}
}

impl fmt::Display for Script {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.to_hex())
	}
}

mod hex_bytes {
	use alloc::{format, string::String, vec::Vec};
	use serde::{de::Error, Deserialize, Deserializer, Serializer};

	pub fn serialize<S: Serializer>(bytes: &Vec<u8>, s: S) -> Result<S::Ok, S::Error> {
		s.serialize_str(&format!("0x{}", hex::encode(bytes)))
	}

	pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
		let s = String::deserialize(d)?;
		let stripped = s.strip_prefix("0x").unwrap_or(&s);
		hex::decode(stripped).map_err(D::Error::custom)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	const VERIFICATION_SCRIPT: &str =
		"0c21033a4d051b04b7fc0230d2b1aaedfd5a84be279a5361a7358db665ad7857787f1b4156e7b327";
	const EXPECTED_HASH: &str = "0x69ecca587293047be4c59159bf8bc399985c160d";

	#[test]
	fn test_script_to_hash() {
		let script = Script::from_hex(VERIFICATION_SCRIPT).unwrap();
		assert_eq!(script.to_script_hash().to_hex(), EXPECTED_HASH);
	}

	#[test]
	fn test_script_from_hex_accepts_prefix() {
		let without = Script::from_hex(VERIFICATION_SCRIPT).unwrap();
		let with = Script::from_hex(&format!("0x{}", VERIFICATION_SCRIPT)).unwrap();
		assert_eq!(with, without);
	}

	#[test]
	fn test_script_roundtrip() {
		let script = Script::from_hex(VERIFICATION_SCRIPT).unwrap();
		let hex = script.to_hex();
		let back = Script::from_hex(&hex).unwrap();
		assert_eq!(script, back);
	}

	#[test]
	fn test_script_len() {
		let script = Script::from_hex(VERIFICATION_SCRIPT).unwrap();
		// PUSHDATA1(1) + len(1) + pubkey(33) + SYSCALL(1) + interop hash(4)
		assert_eq!(script.len(), 40);
		assert!(!script.is_empty());

		let empty = Script::new(Vec::new());
		assert_eq!(empty.len(), 0);
		assert!(empty.is_empty());
	}

	#[test]
	fn test_serde_roundtrip() {
		let script = Script::from_hex(VERIFICATION_SCRIPT).unwrap();
		let json = serde_json::to_string(&script).unwrap();
		assert!(json.contains(VERIFICATION_SCRIPT));
		let back: Script = serde_json::from_str(&json).unwrap();
		assert_eq!(script, back);
	}
}
