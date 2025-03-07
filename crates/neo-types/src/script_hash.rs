use byte_slice_cast::AsByteSlice;
use hex::FromHexError;
use primitive_types::H160;
use rustc_serialize::hex::ToHex;

use crate::TypeError;

pub type ScriptHash = H160;

// Re-export the ScriptHashExtension trait from script_hash_extension.rs
pub use crate::script_hash_extension::ScriptHashExtension;

// Implementation moved to script_hash_impl.rs

#[cfg(test)]
mod tests {
	use std::str::FromStr;

	use rustc_serialize::hex::{FromHex, ToHex};

	use neo_builder::InteropService;
	use neo_codec::{Encoder, NeoSerializable};
	use neo_config::TestConstants;
	use crate::op_code::OpCode;

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

	#[test]
	fn test_serialize_and_deserialize() {
		let hex_str = "23ba2703c53263e8d6e522dc32203339dcd8eee9";
		let data = hex_str.from_hex().unwrap();

		let mut buffer = Encoder::new();
		H160::from_hex(hex_str).unwrap().encode(&mut buffer);

		assert_eq!(buffer.to_bytes(), data);
		assert_eq!(H160::from_slice(&data).as_bytes().to_hex(), hex_str);
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
		let hash = H160::from_address("NeE8xcV4ohHi9rjyj4nPdCYTGyXnWZ79UU").unwrap();
		let mut expected = hex::decode(
			"2102208aea0068c429a03316e37be0e3e8e21e6cda5442df4c5914a19b3a9b6de37568747476aa",
		)
		.unwrap()
		.sha256_ripemd160();
		expected.reverse();
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

	#[test]
	fn test_from_public_key_bytes() {
		let public_key = "035fdb1d1f06759547020891ae97c729327853aeb1256b6fe0473bc2e9fa42ff50";
		let script = format!(
			"{}21{}{}{}",
			OpCode::PushData1.to_hex_string(),
			public_key,
			OpCode::Syscall.to_hex_string(),
			InteropService::SystemCryptoCheckSig.hash()
		);

		let hash = H160::from_public_key(&public_key.from_hex().unwrap()).unwrap();
		let mut hash = hash.to_array();
		let mut expected = script.from_hex().unwrap().sha256_ripemd160();
		expected.reverse();
		assert_eq!(hash, expected);
	}

	// #[test]
	// fn test_from_contract_script() {
	// 	let script =
	//         "110c21026aa8fe6b4360a67a530e23c08c6a72525afde34719c5436f9d3ced759f939a3d110b41138defaf";
	// 	let hash = H160::from_script(&script.from_hex().unwrap());

	// 	assert_eq!(hash.to_hex(), "0898ea2197378f623a7670974454448576d0aeaf");
	// }

	#[test]
	fn test_to_address() {
		let mut script_hash = hex::decode(
			"0c2102249425a06b5a1f8e6133fc79afa2c2b8430bf9327297f176761df79e8d8929c50b4195440d78",
		)
		.unwrap()
		.sha256_ripemd160();
		script_hash.reverse();
		let hash = H160::from_hex(&script_hash.to_hex()).unwrap();
		let address = hash.to_address();
		assert_eq!(address, "NLnyLtep7jwyq1qhNPkwXbJpurC4jUT8ke".to_string());
	}
}
