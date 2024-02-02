extern crate core;

#[cfg(test)]
mod tests {
	use neo_codec::{encode::NeoSerializable, Decoder, Encoder};
	use neo_types::script_hash::ScriptHashExtension;
	use primitive_types::H160;
	use rustc_serialize::hex::ToHex;

	#[test]
	fn test_from_valid_hash() {
		let hash = H160::from_hex("23ba2703c53263e8d6e522dc32203339dcd8eee9").unwrap();
		assert_eq!(hash.to_hex(), "23ba2703c53263e8d6e522dc32203339dcd8eee9");

		let hash = H160::from_hex("23ba2703c53263e8d6e522dc32203339dcd8eee9").unwrap();
		assert_eq!(hash.to_hex(), "23ba2703c53263e8d6e522dc32203339dcd8eee9");
	}

	#[test]
	#[should_panic]
	fn test_creation_throws() {
		let result = H160::from_hex("23ba2703c53263e8d6e522dc32203339dcd8eee");
		assert!(result.is_err());

		let result = H160::from_hex("g3ba2703c53263e8d6e522dc32203339dcd8eee9");
		assert!(result.is_err());

		let result = H160::from_hex("23ba2703c53263e8d6e522dc32203339dcd8ee");
		assert!(result.is_err());

		let result =
			H160::from_hex("c56f33fc6ecfcd0c225c4ab356fee59390af8560be0e930faebe74a6daff7c9b");
		assert!(result.is_err());
	}

	#[test]
	fn test_to_array() {
		let hash = H160::from_hex("23ba2703c53263e8d6e522dc32203339dcd8eee9").unwrap().to_le_vec();
		let mut expected = hex::decode("23ba2703c53263e8d6e522dc32203339dcd8eee9").unwrap();
		expected.reverse();
		assert_eq!(hash, expected);
	}

	#[test]
	fn test_serialize_and_deserialize() {
		let mut writer = Encoder::new();
		let hash_str = "23ba2703c53263e8d6e522dc32203339dcd8eee9";
		let data = hex::decode(hash_str).unwrap();

		H160::from_hex(hash_str).unwrap().encode(&mut writer);

		assert_eq!(writer.to_bytes(), data);

		let decoded = H160::from_slice(&data);
		assert_eq!(decoded.to_hex(), hash_str);
	}

	#[test]
	fn test_equals() {
		let hash1 = H160::from_script(hex::decode("01a402d8").unwrap().as_slice());
		let hash2 = H160::from_script(hex::decode("d802a401").unwrap().as_slice());

		assert_ne!(hash1, hash2);
		assert_eq!(hash1, hash1);
	}

	#[test]
	fn test_from_valid_address() {
		let mut hash = H160::from_address("NLnyLtep7jwyq1qhNPkwXbJpurC4jUT8ke").unwrap().to_array();
		let expected = hex::decode("09a55874c2da4b86e5d49ff530a1b153eb12c7d6").unwrap();
		hash.reverse();
		assert_eq!(hash, expected);
	}

	#[test]
	fn test_from_invalid_address() {
		let result = H160::from_address("NLnyLtep7jwyq1qhNPkwXbJpurC4jUT8keas");
		assert!(result.is_err());
	}

	#[test]
	fn test_from_contract_script() {
		let script = hex::decode("110c21026aa8fe6b4360a67a530e23c08c6a72525afde34719c5436f9d3ced759f939a3d110b41138defaf").unwrap();

		let hash = H160::from_script(&script);

		assert_eq!(hash.to_hex(), "afaed076854454449770763a628f379721ea9808");
		assert_eq!(hash.to_le_vec().to_hex(), "0898ea2197378f623a7670974454448576d0aeaf");
	}

	#[test]
	fn test_compare_to() {
		let hash1 = H160::from_script(&hex::decode("01a402d8").unwrap());
		let hash2 = H160::from_script(&hex::decode("d802a401").unwrap());
		let hash3 = H160::from_script(&hex::decode("a7b3a191").unwrap());

		assert!(hash2 > hash1);
		assert!(hash3 > hash1);
		assert!(hash2 > hash3);
	}

	#[test]
	fn test_size() {
		let hash = H160::from_hex("23ba2703c53263e8d6e522dc32203339dcd8eee9").unwrap();
		assert_eq!(hash.size(), 20);
	}
}
