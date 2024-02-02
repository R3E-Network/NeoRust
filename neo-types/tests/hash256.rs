extern crate core;

#[cfg(test)]
mod tests {
	use neo_codec::{encode::NeoSerializable, Decoder, Encoder};
	use neo_types::script_hash::ScriptHashExtension;
	use primitive_types::H256;
	use rustc_serialize::hex::{FromHex, ToHex};
	use std::{hash::Hash, str::FromStr};

	#[test]
	fn test_from_valid_hash() {
		let hash =
			H256::from_str("b804a98220c69ab4674e97142beeeb00909113d417b9d6a67c12b71a3974a21a")
				.unwrap();

		assert_eq!(
			hash.as_bytes(),
			"b804a98220c69ab4674e97142beeeb00909113d417b9d6a67c12b71a3974a21a"
				.from_hex()
				.unwrap()
		);

		let hash =
			H256::from_str("b804a98220c69ab4674e97142beeeb00909113d417b9d6a67c12b71a3974a21a")
				.unwrap();

		assert_eq!(
			hash.as_bytes(),
			"b804a98220c69ab4674e97142beeeb00909113d417b9d6a67c12b71a3974a21a"
				.from_hex()
				.unwrap()
		);
	}

	#[test]
	fn test_creation_throws() {
		let result =
			H256::from_str("b804a98220c69ab4674e97142beeeb00909113d417b9d6a67c12b71a3974a21ae");
		assert!(result.is_err());

		let result =
			H256::from_str("g804a98220c69ab4674e97142beeeb00909113d417b9d6a67c12b71a3974a21a");
		assert!(result.is_err());

		let result =
			H256::from_str("b804a98220c69ab4674e97142beeeb00909113d417b9d6a67c12b71a3974a2");
		assert!(result.is_err());

		let result =
			H256::from_str("b804a98220c69ab4674e97142beeeb00909113d417b9d6a67c12b71a3974a21a12");
		assert!(result.is_err());
	}

	#[test]
	fn test_from_bytes() {
		let bytes = hex::decode("b804a98220c69ab4674e97142beeeb00909113d417b9d6a67c12b71a3974a21a")
			.unwrap();

		let hash = H256::from_slice(&bytes);

		assert_eq!(
			hash.as_bytes(),
			"b804a98220c69ab4674e97142beeeb00909113d417b9d6a67c12b71a3974a21a"
				.from_hex()
				.unwrap()
		);
	}

	#[test]
	fn test_to_array() {
		let hash =
			H256::from_str("b804a98220c69ab4674e97142beeeb00909113d417b9d6a67c12b71a3974a21a")
				.unwrap();

		let expected =
			hex::decode("b804a98220c69ab4674e97142beeeb00909113d417b9d6a67c12b71a3974a21a")
				.unwrap();

		assert_eq!(hash.to_array(), expected);
	}

	#[test]
	fn test_serialize_and_deserialize() {
		let mut writer = Encoder::new();
		let mut hash_str = "b804a98220c69ab4674e97142beeeb00909113d417b9d6a67c12b71a3974a21a"
			.from_hex()
			.unwrap();

		hash_str.reverse();

		H256::from_slice(&hash_str).encode(&mut writer);

		assert_eq!(writer.to_bytes(), hash_str.clone());

		let mut decoder = Decoder::new(&hash_str);
		let decoded = H256::decode(&mut decoder).unwrap();

		assert_eq!(decoded.as_bytes(), hash_str);
	}

	#[test]
	fn test_equals() {
		let mut bytes1 =
			hex::decode("1aa274391ab7127ca6d6b917d413919000ebee2b14974e67b49ac62082a904b8")
				.unwrap();

		let mut bytes2 =
			hex::decode("b43034ab680d646f8b6ca71647aa6ba167b2eb0b3757e545f6c2715787b13272")
				.unwrap();

		bytes1.reverse();
		bytes2.reverse();

		let hash1 = H256::from_slice(&bytes1);
		let hash2 = H256::from_slice(&bytes2);
		let hash3 =
			H256::from_str("b804a98220c69ab4674e97142beeeb00909113d417b9d6a67c12b71a3974a21a")
				.unwrap();

		assert_ne!(hash1, hash2);
		assert_eq!(hash1, hash1);
		assert_eq!(hash1, hash3);
		assert_eq!(hash1.to_array(), hash3.to_array());
	}

	#[test]
	fn test_compare() {
		let mut bytes1 =
			hex::decode("1aa274391ab7127ca6d6b917d413919000ebee2b14974e67b49ac62082a904b8")
				.unwrap();

		let mut bytes2 =
			hex::decode("b43034ab680d646f8b6ca71647aa6ba167b2eb0b3757e545f6c2715787b13272")
				.unwrap();

		bytes1.reverse();
		bytes2.reverse();

		let hash1 = H256::from_slice(&bytes1);
		let hash2 = H256::from_slice(&bytes2);

		let hash3 =
			H256::from_str("f4609b99e171190c22adcf70c88a7a14b5b530914d2398287bd8bb7ad95a661c")
				.unwrap();

		assert!(hash1 > hash2);
		assert!(hash3 > hash1);
		assert!(hash3 > hash2);
	}

	#[test]
	fn test_size() {
		let hash =
			H256::from_str("b804a98220c69ab4674e97142beeeb00909113d417b9d6a67c12b71a3974a21a")
				.unwrap();
		assert_eq!(hash.to_array().len(), 32);
	}
}
