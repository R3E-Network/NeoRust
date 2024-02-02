use crate::error::TypeError;
use hex::FromHexError;
use neo_config::DEFAULT_ADDRESS_VERSION;
use neo_crypto::hash::HashableForVec;
use primitive_types::H160;
use rustc_serialize::hex::ToHex;

pub type ScriptHash = H160;

/// Trait that provides additional methods for types related to `ScriptHash`.
pub trait ScriptHashExtension
where
	Self: Sized,
{
	/// Returns a string representation of the object.
	fn to_string(&self) -> String;

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
	fn from_hex(hex: &str) -> Result<Self, hex::FromHexError>;

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

	/// Converts the object into a byte vector.
	fn to_vec(&self) -> Vec<u8>;

	/// Converts the object into a little-endian byte vector.
	fn to_le_vec(&self) -> Vec<u8>;

	/// Creates an instance from a script byte slice.
	fn from_script(script: &[u8]) -> Self;
}

impl ScriptHashExtension for H160 {
	fn to_string(&self) -> String {
		bs58::encode(self.0).into_string()
	}

	fn from_slice(slice: &[u8]) -> Result<Self, TypeError> {
		if slice.len() != 20 {
			return Err(TypeError::InvalidAddress)
		}

		let mut arr = [0u8; 20];
		arr.copy_from_slice(slice);
		Ok(Self(arr))
	}

	fn from_hex(hex: &str) -> Result<Self, FromHexError> {
		let hex = if hex.starts_with("0x") { &hex[2..] } else { hex };
		let bytes = hex::decode(hex)?;
		Ok(Self::from_slice(&bytes))
	}

	fn from_address(address: &str) -> Result<Self, TypeError> {
		let bytes = match bs58::decode(address).into_vec() {
			Ok(bytes) => bytes,
			Err(_) => return Err(TypeError::InvalidAddress),
		};

		let salt = bytes[0];
		let hash = &bytes[1..21];
		let checksum = &bytes[21..25];
		let mut sha = &bytes[..21].hash256().hash256();
		let check = &sha[..4];
		if checksum != check {
			return Err(TypeError::InvalidAddress)
		}

		let mut rev = [0u8; 20];
		rev.clone_from_slice(hash);
		rev.reverse();
		Ok(Self::from_slice(&rev))
	}

	fn to_address(&self) -> String {
		let mut data = vec![DEFAULT_ADDRESS_VERSION];
		data.extend_from_slice(&self.0);
		let mut sha = &data.hash256().hash256();
		data.extend_from_slice(&sha[..4]);
		bs58::encode(data).into_string()
	}

	fn to_hex(&self) -> String {
		self.0.to_hex()
	}

	fn to_vec(&self) -> Vec<u8> {
		self.0.to_vec()
	}

	fn to_le_vec(&self) -> Vec<u8> {
		let mut vec = self.0.to_vec();
		vec.reverse();
		vec
	}

	fn from_script(script: &[u8]) -> Self {
		let mut hash = script.sha256_ripemd160();
		hash.reverse();
		let mut arr = [0u8; 20];
		arr.copy_from_slice(&hash);
		Self(arr)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use rustc_serialize::hex::ToHex;
	use std::str::FromStr;

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
			"23ba2703c53263e8d6e522dc32203339dcd8eee9".to_string()
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

	#[test]
	fn test_serialize_deserialize() {
		let expected = "23ba2703c53263e8d6e522dc32203339dcd8eee9";
		let expected_bytes: Vec<u8> = hex::decode(expected).unwrap().into_iter().rev().collect();

		// let mut writer = vec![];
		// // H160::from_str(expected).unwrap().serialize(&mut writer).unwrap();
		//
		// assert_eq!(writer, expected_bytes);
		// assert_eq!(H160::deserialize(&expected_bytes).unwrap().to_string(), expected);
	}

	#[test]
	fn test_equals() {
		let hash1 = H160::from_script(&hex::decode("01a402d8").unwrap());
		let hash2 = H160::from_script(&hex::decode("d802a401").unwrap());
		assert_ne!(hash1, hash2);
		assert_eq!(hash1, hash1);
	}

	#[test]
	fn test_from_address() {
		let hash = H160::from_address("NLnyLtep7jwyq1qhNPkwXbJpurC4jUT8ke").unwrap();
		let expected = hex::decode("09a55874c2da4b86e5d49ff530a1b153eb12c7d6").unwrap();
		assert_eq!(hash.to_le_vec(), expected);
	}

	#[test]
	// #[should_panic]
	fn test_from_invalid_address() {
		// assert that this should return Err
		assert_eq!(
			H160::from_address("NLnyLtep7jwyq1qhNPkwXbJpurC4jUT8keas"),
			Err(TypeError::InvalidAddress)
		);
	}
}
