use crate::{
	error::TypeError,
	script_hash::{ScriptHash, ScriptHashExtension},
	string::StringExt,
};
use neo_crypto::hash::HashableForVec;
use primitive_types::H160;
use rand::Rng;
use rustc_serialize::hex::FromHex;
use serde_derive::{Deserialize, Serialize};

pub type Address = String;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NameOrAddress {
	Name(String),
	Address(Address),
}

pub trait AddressExtension {
	fn address_to_script_hash(&self) -> Result<ScriptHash, TypeError>;
	fn script_to_script_hash(&self) -> Result<ScriptHash, TypeError>;

	fn hex_to_script_hash(&self) -> Result<ScriptHash, TypeError>;

	fn random() -> Self;
}

impl AddressExtension for String {
	fn address_to_script_hash(&self) -> Result<ScriptHash, TypeError> {
		// Base58-decode the address
		let binding = match bs58::decode(self).into_vec() {
			Ok(data) => ScriptHash::from_script(data.as_slice()),
			Err(_) => return Err(TypeError::InvalidAddress),
		};
		let decoded_data = binding.as_bytes();

		// Extract the data payload
		let data_payload = decoded_data[1..decoded_data.len() - 4].to_vec();

		let script_hash = data_payload.sha256_ripemd160(); //  ripemd160.finalize();

		Ok(H160::from_slice(script_hash.as_slice()))
	}

	fn script_to_script_hash(&self) -> Result<ScriptHash, TypeError> {
		self.from_hex()
			.map(|data| ScriptHash::from_script(data.as_slice()))
			.map_err(|_| TypeError::InvalidScript("Invalid hex string".to_string()))
	}

	fn hex_to_script_hash(&self) -> Result<ScriptHash, TypeError> {
		if self.is_valid_hex() {
			ScriptHash::from_hex(self.as_str()).map_err(|_| TypeError::InvalidFormat)
		} else {
			Err(TypeError::InvalidFormat)
		}
	}

	fn random() -> Self {
		let mut rng = rand::thread_rng();
		let mut bytes = [0u8; 20];
		rng.fill(&mut bytes);
		let script_hash = bytes.sha256_ripemd160();
		let mut data = vec![0x17];
		data.extend_from_slice(&script_hash);
		let mut sha = &data.hash256().hash256();
		data.extend_from_slice(&sha[..4]);
		bs58::encode(data).into_string()
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
		panic!("Not implemented")
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_address_to_script_hash() {
		// Test case 1: Valid N3 address
		let n3_address = "NTGYC16CN5QheM4ZwfhUp9JKq8bMjWtcAp";
		let expected_script_hash_hex = "87c06be672d5600dce4a260e7b2d497112c0ac50";
		let result = n3_address.address_to_script_hash().unwrap();
		assert_eq!(hex::encode(result), expected_script_hash_hex);

		// Test case 3: Invalid N3 address
		let n3_address = "Invalid_Address";
		let result = n3_address.to_string().to_script_hash();
		assert!(result.is_err());
	}
}
