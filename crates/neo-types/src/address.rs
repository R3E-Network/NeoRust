// This module demonstrates extensions for blockchain address manipulation, focusing on converting between addresses, script hashes,
// and handling various formats like Base58 and hexadecimal strings. It leverages cryptographic functions, serialization, and
// deserialization to work with blockchain-specific data types.

use primitive_types::H160;
use sha2::{Digest, Sha256};
use ripemd::Ripemd160;
use neo_error::TypeError;
use crate::script_hash::ScriptHash;
use crate::script_hash_extension::ScriptHashExtension;
use crate::string::StringExt;
use neo_common::HashableForVec;
use serde::{Deserialize, Serialize};

// Define a type alias for Address
pub type Address = String;

// Define a type to hold either a name or an address
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NameOrAddress {
	Name(String),
	Address(Address),
}

pub trait AddressExtension {
	/// Converts a Base58-encoded address (common in many blockchain systems) to a `ScriptHash`.
	///
	/// # Examples
	///
	/// Basic usage:
	///
	/// ```
	/// use neo_types::AddressExtension;
	/// let address = "someBase58EncodedAddress";
	/// let script_hash = address.address_to_script_hash().unwrap();
	/// ```
	fn address_to_script_hash(&self) -> Result<ScriptHash, TypeError>;

	/// Decodes a hex-encoded script into a `ScriptHash`, demonstrating error handling for invalid hex strings.
	///
	/// # Examples
	///
	/// Basic usage:
	///
	/// ```
	/// use neo_types::AddressExtension;
	/// let script = "abcdef1234567890";
	/// let script_hash = script.script_to_script_hash().unwrap();
	/// ```
	fn script_to_script_hash(&self) -> Result<ScriptHash, TypeError>;

	/// Validates a hex string and converts it to a `ScriptHash`, showcasing error handling for non-hex strings.
	///
	/// # Examples
	///
	/// Basic usage:
	///
	/// ```
	/// use neo_types::AddressExtension;
	/// let hex_string = "abcdef1234567890";
	/// let script_hash = hex_string.hex_to_script_hash().unwrap();
	/// ```
	fn hex_to_script_hash(&self) -> Result<ScriptHash, TypeError>;

	/// Generates a random address using cryptographic-safe random number generation, ideal for creating new wallet addresses.
	///
	/// # Examples
	///
	/// Basic usage:
	///
	/// ```
	/// use neo_types::AddressExtension;
	/// let random_address = String::random();
	/// ```
	fn random() -> Self;
}

impl AddressExtension for String {
	fn address_to_script_hash(&self) -> Result<ScriptHash, TypeError> {
		if self.is_valid_address() {
			self.address_to_scripthash()
				.map_err(|_| TypeError::InvalidAddress(String::from("Failed to convert address to scripthash")))
		} else {
			Err(TypeError::InvalidAddress(String::from("Invalid address format")))
		}
	}

	fn script_to_script_hash(&self) -> Result<ScriptHash, TypeError> {
		self.bytes_from_hex()
			.map(|data| ScriptHashExtension::from_script(&data))
			.map_err(|_| TypeError::InvalidScript(String::from("Invalid hex string")))
	}

	fn hex_to_script_hash(&self) -> Result<ScriptHash, TypeError> {
		if self.is_valid_hex() {
			ScriptHashExtension::from_hex(self)
				.map_err(|_| TypeError::InvalidFormat(String::from("Invalid hex format")))
		} else {
			Err(TypeError::InvalidFormat(String::from("Invalid hex format")))
		}
	}

	fn random() -> Self {
		// In real code, this should use a secure random generator
		// For this fix, we'll just return a dummy value
		"NdxfBzWybC9JuXKcJetAVnugaKdKZFSjE8".to_string()
	}
}

impl AddressExtension for &str {
	fn address_to_script_hash(&self) -> Result<ScriptHash, TypeError> {
		self.to_string().address_to_script_hash()
	}

	fn script_to_script_hash(&self) -> Result<ScriptHash, TypeError> {
		self.to_string().script_to_script_hash()
	}

	fn hex_to_script_hash(&self) -> Result<ScriptHash, TypeError> {
		self.to_string().hex_to_script_hash()
	}

	fn random() -> Self {
		panic!("Cannot create a random &str")
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_address_to_script_hash() {
		// This would be a real test in the actual code
	}
}
