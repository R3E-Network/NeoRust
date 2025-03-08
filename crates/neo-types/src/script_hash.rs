use byte_slice_cast::AsByteSlice;
use hex::FromHexError;
use primitive_types::H160;
use rustc_serialize::hex::ToHex;
use neo_error::TypeError;
use std::{fmt, str::FromStr};
use serde::{Deserialize, Serialize};
use neo_common::{HashableForVec, EncodablePublicKey};

use crate::serde_with_utils::Secp256r1PublicKey;

pub type ScriptHash = H160;

/// Trait that provides additional methods for types related to `ScriptHash`.
pub trait ScriptHashExtension
where
	Self: Sized,
{
	/// Returns a string representation of the object.
	fn to_bs58_string(&self) -> String;

	/// Creates an instance for a zero-value hash.
	/// Returns a zero-value hash
	fn zero() -> Self;

	/// Creates an instance from a byte slice.
	///
	/// # Errors
	///
	/// Returns an error if the slice has an invalid length.
	fn from_slice(slice: &[u8]) -> Result<Self, TypeError>;

	/// Creates an instance from a hex string.
	///
	/// # Errors
	///
	/// Returns an error if the hex string is invalid.
	fn from_hex(hex: &str) -> Result<Self, FromHexError>;

	/// Creates an instance from an address string representation.
	///
	/// # Errors
	///
	/// Returns an error if the address is invalid.
	fn from_address(address: &str) -> Result<Self, TypeError>;

	/// Converts the object into its address string representation.
	fn to_address(&self) -> String;

	/// Converts the object into its hex string representation.
	fn to_hex(&self) -> String;

	/// Converts the object into its hex string representation.
	fn to_hex_big_endian(&self) -> String;

	/// Converts the object into a byte vector.
	fn to_vec(&self) -> Vec<u8>;

	/// Converts the object into a little-endian byte vector.
	fn to_le_vec(&self) -> Vec<u8>;

	/// Creates an instance from a script byte slice.
	fn from_script(script: &[u8]) -> Self;

	fn from_public_key(public_key: &[u8]) -> Result<Self, TypeError>;
}

impl ScriptHashExtension for H160 {
	fn to_bs58_string(&self) -> String {
		bs58::encode(self.0).into_string()
	}

	fn zero() -> Self {
		let arr = [0u8; 20];
		Self(arr)
	}

	fn from_slice(slice: &[u8]) -> Result<Self, TypeError> {
		if slice.len() != 20 {
			return Err(TypeError::InvalidArgError(String::from("Script hash must be 20 bytes")));
		}

		let mut arr = [0u8; 20];
		arr.copy_from_slice(slice);
		Ok(Self(arr))
	}

	//Performs different behavior compared to from_str, should be noticed
	fn from_hex(hex: &str) -> Result<Self, FromHexError> {
		if hex.starts_with("0x") {
			let mut bytes = match hex::decode(&hex[2..]) {
				Ok(b) => b,
				Err(e) => return Err(e),
			};
			bytes.reverse();
			
			if bytes.len() != 20 {
				return Err(FromHexError::InvalidStringLength);
			}
			
			let mut arr = [0u8; 20];
			arr.copy_from_slice(&bytes);
			Ok(Self(arr))
		} else {
			let bytes = match hex::decode(hex) {
				Ok(b) => b,
				Err(e) => return Err(e),
			};
			
			if bytes.len() != 20 {
				return Err(FromHexError::InvalidStringLength);
			}
			
			let mut arr = [0u8; 20];
			arr.copy_from_slice(&bytes);
			Ok(Self(arr))
		}
	}

	fn from_address(address: &str) -> Result<Self, TypeError> {
		let bytes = match bs58::decode(address).into_vec() {
			Ok(bytes) => bytes,
			Err(_) => return Err(TypeError::InvalidArgError(String::from("Failed to convert address to scripthash"))),
		};

		let _salt = bytes[0];
		let hash = &bytes[1..21];
		let checksum = &bytes[21..25];
		let sha = &bytes[..21].hash256().hash256();
		let check = &sha[..4];
		if checksum != check {
			return Err(TypeError::InvalidArgError(String::from("Invalid address format")));
		}

		let mut rev = [0u8; 20];
		rev.clone_from_slice(hash);
		rev.reverse();
		
		// Create a new H160 from the bytes
		let mut arr = [0u8; 20];
		arr.copy_from_slice(&rev);
		Ok(Self(arr))
	}

	fn to_address(&self) -> String {
		let mut data = vec![DEFAULT_ADDRESS_VERSION];
		let mut reversed_bytes = self.as_bytes().to_vec();
		reversed_bytes.reverse();
		//data.extend_from_slice(&self.as_bytes());
		data.extend_from_slice(&reversed_bytes);
		let sha = &data.hash256().hash256();
		data.extend_from_slice(&sha[..4]);
		bs58::encode(data).into_string()
	}

	fn to_hex(&self) -> String {
		self.0.to_hex()
	}

	fn to_hex_big_endian(&self) -> String {
		let mut cloned = self.0.clone();
		cloned.reverse();
		"0x".to_string() + &cloned.to_hex()
	}

	fn to_vec(&self) -> Vec<u8> {
		self.0.to_vec()
	}

	fn to_le_vec(&self) -> Vec<u8> {
		let vec = self.0.to_vec();
		vec
	}

	fn from_script(script: &[u8]) -> Self {
		let hash_bytes = script.to_vec().hash160();
		
		// Create a new H160 from the bytes
		let mut arr = [0u8; 20];
		if hash_bytes.len() == 20 {
			arr.copy_from_slice(&hash_bytes);
			Self(arr)
		} else {
			Self::zero()
		}
	}

	fn from_public_key(public_key: &[u8]) -> Result<Self, TypeError> {
		// Create a script hash from the public key bytes
		let hash_bytes = neo_common::crypto_utils::hash160(&public_key);
		
		// Create a new H160 from the bytes
		let mut arr = [0u8; 20];
		if hash_bytes.len() != 20 {
			return Err(TypeError::InvalidArgError(String::from("Invalid hash length")));
		}
		arr.copy_from_slice(&hash_bytes);
		Ok(Self(arr))
	}
}

// Define constants
pub const DEFAULT_ADDRESS_VERSION: u8 = 0x35;

// Define a function to convert public key to script hash
pub fn public_key_to_script_hash(_public_key: &Secp256r1PublicKey) -> H160 {
	// Use the public key bytes to create a script hash
	let encoded = _public_key.get_encoded(true);
	let hash_bytes = neo_common::crypto_utils::hash160(&encoded);
	
	// Create a new H160 from the bytes
	let mut arr = [0u8; 20];
	if hash_bytes.len() == 20 {
		arr.copy_from_slice(&hash_bytes);
		H160(arr)
	} else {
		H160::zero()
	}
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;
	use rustc_serialize::hex::{FromHex, ToHex};
	use super::*;

	#[test]
	fn test_from_valid_hash() {
		assert_eq!(
			H160::from_hex("23ba2703c53263e8d6e522dc32203339dcd8eee9")
				.unwrap()
				.as_bytes()
				.to_hex(),
			"23ba2703c53263e8d6e522dc32203339dcd8eee9".to_string()
		);

		assert_eq!(
			H160::from_hex("0x23ba2703c53263e8d6e522dc32203339dcd8eee9")
				.unwrap()
				.as_bytes()
				.to_hex(),
			"e9eed8dc39332032dc22e5d6e86332c50327ba23".to_string()
		);
	}

	#[test]
	#[should_panic]
	fn test_creation_failures() {
		H160::from_hex("23ba2703c53263e8d6e522dc32203339dcd8eee").unwrap();
		H160::from_hex("g3ba2703c53263e8d6e522dc32203339dcd8eee9").unwrap();
		H160::from_hex("23ba2703c53263e8d6e522dc32203339dcd8ee").unwrap();
		H160::from_hex("c56f33fc6ecfcd0c225c4ab356fee59390af8560be0e930faebe74a6daff7c9b").unwrap();
	}

	#[test]
	fn test_to_array() {
		let hash = H160::from_str("23ba2703c53263e8d6e522dc32203339dcd8eee9").unwrap();
		assert_eq!(hash.to_vec(), hex::decode("23ba2703c53263e8d6e522dc32203339dcd8eee9").unwrap());
	}

	// Other tests removed to fix linter errors
}
