//! Neo N3 address encoding

use crate::{
	crypto,
	errors::{Error, Result},
	types::Hash160,
};
use alloc::{format, string::String, vec::Vec};
use core::{fmt, str::FromStr};
use serde::{Deserialize, Serialize};

/// Neo N3 address version byte (produces addresses starting with 'N').
pub const ADDRESS_VERSION: u8 = 0x35;

/// A Neo N3 address (Base58Check encoding of a script hash).
///
/// Addresses are the human-readable form of a 20-byte script hash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Address(String);

impl Address {
	/// Decodes and validates a Neo N3 address string.
	pub fn from_string(s: String) -> Result<Self> {
		let decoded = bs58::decode(&s)
			.into_vec()
			.map_err(|_| Error::InvalidAddress("base58 decode failed".into()))?;

		if decoded.len() != 25 {
			return Err(Error::InvalidAddress(format!(
				"expected 25 bytes, got {}",
				decoded.len()
			)));
		}

		if decoded[0] != ADDRESS_VERSION {
			return Err(Error::InvalidAddress(format!(
				"wrong version: expected {:#x}, got {:#x}",
				ADDRESS_VERSION, decoded[0]
			)));
		}

		let payload = &decoded[..21];
		let checksum = &decoded[21..25];
		let expected = &crypto::hash256(payload)[..4];

		if checksum != expected {
			return Err(Error::InvalidAddress("checksum mismatch".into()));
		}

		Ok(Self(s))
	}

	/// Encodes a script hash as a Neo N3 address.
	pub fn from_script_hash(hash: &Hash160) -> Self {
		let mut payload = Vec::with_capacity(21);
		payload.push(ADDRESS_VERSION);
		payload.extend_from_slice(hash.as_bytes());

		let checksum = &crypto::hash256(&payload)[..4];
		let mut data = payload;
		data.extend_from_slice(checksum);

		Self(bs58::encode(data).into_string())
	}

	/// Decodes this address and returns the underlying script hash.
	pub fn to_script_hash(&self) -> Result<Hash160> {
		let decoded = bs58::decode(&self.0)
			.into_vec()
			.map_err(|_| Error::InvalidAddress("base58 decode failed".into()))?;

		if decoded.len() != 25 {
			return Err(Error::InvalidAddress("wrong length".into()));
		}

		Hash160::from_slice(&decoded[1..21])
	}

	/// The address string.
	pub fn as_str(&self) -> &str {
		&self.0
	}
}

impl fmt::Display for Address {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(&self.0)
	}
}

impl FromStr for Address {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self> {
		Self::from_string(s.to_string())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	const TEST_ADDRESS: &str = "NM7Aky765FG8NhhwtxjXRx7jEL1cnw7PBP";
	const TEST_SCRIPT_HASH: &str = "0x69ecca587293047be4c59159bf8bc399985c160d";

	#[test]
	fn test_address_roundtrip() {
		let addr: Address = TEST_ADDRESS.parse().unwrap();
		assert_eq!(addr.as_str(), TEST_ADDRESS);
	}

	#[test]
	fn test_address_to_script_hash() {
		let addr: Address = TEST_ADDRESS.parse().unwrap();
		let hash = addr.to_script_hash().unwrap();
		assert_eq!(hash.to_hex(), TEST_SCRIPT_HASH);
	}

	#[test]
	fn test_script_hash_to_address() {
		let hash = Hash160::from_hex(TEST_SCRIPT_HASH).unwrap();
		let addr = Address::from_script_hash(&hash);
		assert_eq!(addr.as_str(), TEST_ADDRESS);
	}

	#[test]
	fn test_address_script_hash_roundtrip() {
		let addr1: Address = TEST_ADDRESS.parse().unwrap();
		let hash = addr1.to_script_hash().unwrap();
		let addr2 = Address::from_script_hash(&hash);
		assert_eq!(addr1, addr2);
	}

	#[test]
	fn test_address_rejects_bad_checksum() {
		let mut bad = TEST_ADDRESS.to_string();
		bad.pop();
		bad.push('Z');
		assert!(bad.parse::<Address>().is_err());
	}

	#[test]
	fn test_address_rejects_wrong_version() {
		let btc = "17VZNX1SN5NtKa8UQFxwQbFeFc3iqRYhem";
		assert!(btc.parse::<Address>().is_err());
	}

	#[test]
	fn test_address_rejects_invalid_base58() {
		assert!("000".parse::<Address>().is_err());
		assert!("not-base58!".parse::<Address>().is_err());
	}

	#[test]
	fn test_serde_roundtrip() {
		let addr: Address = TEST_ADDRESS.parse().unwrap();
		let json = serde_json::to_string(&addr).unwrap();
		assert_eq!(json, format!("\"{}\"", TEST_ADDRESS));
		let back: Address = serde_json::from_str(&json).unwrap();
		assert_eq!(addr, back);
	}
}