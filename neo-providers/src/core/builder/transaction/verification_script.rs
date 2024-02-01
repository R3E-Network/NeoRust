use crate::core::{
	error::BuilderError,
	script::{interop_service::InteropService, script_builder::ScriptBuilder},
};
use getset::{Getters, Setters};
use hex_literal::hex;
use neo_codec::{encode::NeoSerializable, Decoder, Encoder};
use neo_crypto::{
	keys::{PublicKeyExtension, Secp256r1PublicKey, Secp256r1Signature},
	utils::ToArray32,
};
use neo_types::{op_code::OpCode, util::var_size, Bytes};
use num_bigint::BigInt;
use p256::pkcs8::der::Encode;
use primitive_types::H160;
use rustc_serialize::hex::{FromHex, ToHex};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, vec};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Getters, Setters, Serialize, Deserialize)]
pub struct VerificationScript {
	#[getset(get = "pub", set = "pub")]
	script: Bytes,
}

impl VerificationScript {
	pub fn new() -> Self {
		Self { script: Bytes::new() }
	}

	pub fn from(script: Bytes) -> Self {
		Self { script: script.to_vec() }
	}

	pub fn from_public_key(public_key: &Secp256r1PublicKey) -> Self {
		let mut builder = ScriptBuilder::new();
		builder
			.push_data(public_key.get_encoded(true))
			.sys_call(InteropService::SystemCryptoCheckSig);
		Self::from(builder.to_bytes())
	}

	pub fn from_multi_sig(public_keys: &mut [Secp256r1PublicKey], threshold: u8) -> Self {
		// Build multi-sig script
		let mut builder = ScriptBuilder::new();
		builder.push_integer(BigInt::from(threshold));
		public_keys.sort();
		for key in public_keys.iter() {
			builder.push_data(key.get_encoded(true));
		}
		builder
			.push_integer(BigInt::from(public_keys.len()))
			.sys_call(InteropService::SystemCryptoCheckMultiSig);
		Self::from(builder.to_bytes())
	}

	/// Checks if this verification script is from a single signature account.
	///
	/// Returns `true` if this script is from a single signature account, otherwise `false`.
	pub fn is_single_sig(&self) -> bool {
		if self.script.len() != 40 {
			return false
		}

		let interop_service = &self.script[self.script.len() - 4..]; // Get the last 4 bytes
		let interop_service_hex = interop_service.to_hex();

		self.script[0] == OpCode::PushData1.opcode()
			&& self.script[1] == 33
			&& self.script[35] == OpCode::Syscall.opcode()
			&& interop_service_hex == InteropService::SystemCryptoCheckSig.hash() // Assuming `hash` returns a hex string
	}

	pub fn is_multi_sig(&self) -> bool {
		if self.script.len() < 37 {
			return false
		}

		let mut reader = Decoder::new(&self.script);

		let n = reader.by_ref().read_var_int().unwrap();
		if !(1..16).contains(&n) {
			return false
		}

		let mut m = 0;
		while reader.by_ref().read_u8() == OpCode::PushData1 as u8 {
			let len = reader.by_ref().read_u8();
			if len != 33 {
				return false
			}
			let _ = reader.by_ref().skip(33);
			m += 1;
		}

		if !(m >= n && m <= 16) {
			return false
		}

		// additional checks
		let service_bytes = &self.script[self.script.len() - 4..];
		if service_bytes != &InteropService::SystemCryptoCheckMultiSig.hash().into_bytes() {
			return false
		}

		if m != reader.by_ref().read_var_int().unwrap() {
			return false
		}

		if reader.by_ref().read_u8() != OpCode::Syscall as u8 {
			return false
		}

		true
	}

	// other methods
	pub fn hash(&self) -> H160 {
		H160::from_slice(&self.script)
	}

	pub fn get_signatures(&self) -> Vec<Secp256r1Signature> {
		let mut reader = Decoder::new(&self.script);
		let mut signatures = vec![];

		while reader.by_ref().read_u8() == OpCode::PushData1 as u8 {
			let len = reader.by_ref().read_u8();
			let sig =
				Secp256r1Signature::from_bytes(&reader.by_ref().read_bytes(len as usize).unwrap())
					.unwrap();
			signatures.push(sig);
		}

		signatures
	}

	pub fn get_public_keys(&self) -> Result<Vec<Secp256r1PublicKey>, BuilderError> {
		if self.is_single_sig() {
			let mut reader = Decoder::new(&self.script);
			reader.by_ref().read_u8(); // skip pushdata1
			reader.by_ref().read_u8(); // skip length

			let mut point = [0; 33];
			point.copy_from_slice(&reader.by_ref().read_bytes(33).unwrap());

			let key = Secp256r1PublicKey::from_bytes(&point).unwrap();
			return Ok(vec![key])
		}

		if self.is_multi_sig() {
			let mut reader = Decoder::new(&self.script);
			reader.by_ref().read_var_int().unwrap(); // skip threshold

			let mut keys = vec![];
			while reader.by_ref().read_u8() == OpCode::PushData1 as u8 {
				reader.by_ref().read_u8(); // skip length
				let mut point = [0; 33];
				point.copy_from_slice(&reader.by_ref().read_bytes(33).unwrap());
				keys.push(Secp256r1PublicKey::from_bytes(&point).unwrap());
			}

			return Ok(keys)
		}

		Err(BuilderError::InvalidScript("Invalid verification script".to_string()))
	}

	pub fn get_signing_threshold(&self) -> Result<usize, BuilderError> {
		if self.is_single_sig() {
			Ok(1)
		} else if self.is_multi_sig() {
			let reader = &mut Decoder::new(&self.script);
			Ok(reader.by_ref().read_var_int()? as usize)
		} else {
			Err(BuilderError::InvalidScript("Invalid verification script".to_string()))
		}
	}
	pub fn get_nr_of_accounts(&self) -> Result<usize, BuilderError> {
		match self.get_public_keys() {
			Ok(keys) => Ok(keys.len()),
			Err(e) => Err(e),
		}
	}
}

impl NeoSerializable for VerificationScript {
	type Error = BuilderError;

	fn size(&self) -> usize {
		var_size(self.script.len()) + self.script.len()
	}

	fn encode(&self, writer: &mut Encoder) {
		writer.write_var_bytes(&self.script);
	}

	fn decode(reader: &mut Decoder) -> Result<Self, Self::Error> {
		let script = reader.read_var_bytes()?;
		Ok(Self { script })
	}
	fn to_array(&self) -> Vec<u8> {
		let mut writer = Encoder::new();
		self.encode(&mut writer);
		writer.to_bytes()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use rustc_serialize::hex::FromHex;
	#[test]
	fn test_from_public_key() {
		let key =
			hex!("035fdb1d1f06759547020891ae97c729327853aeb1256b6fe0473bc2e9fa42ff50").to_vec();
		let pubkey = Secp256r1PublicKey::from(key.clone());
		let script = VerificationScript::from_public_key(&pubkey);
		let expected = format!(
			"{}21{}{}{}",
			OpCode::PushData1.to_string(),
			key.to_hex(),
			OpCode::Syscall.to_string(),
			InteropService::SystemCryptoCheckSig.hash()
		)
		.from_hex()
		.unwrap();

		assert_eq!(script.script(), &expected);
	}

	#[test]
	fn test_from_public_keys() {
		let key1 =
			hex!("035fdb1d1f06759547020891ae97c729327853aeb1256b6fe0473bc2e9fa42ff50").to_vec();
		let key2 =
			hex!("03eda286d19f7ee0b472afd1163d803d620a961e1581a8f2704b52c0285f6e022d").to_vec();
		let key3 =
			hex!("03ac81ec17f2f15fd6d193182f927c5971559c2a32b9408a06fec9e711fb7ca02e").to_vec();

		let mut pubkeys = vec![
			Secp256r1PublicKey::from(key1.clone()),
			Secp256r1PublicKey::from(key2.clone()),
			Secp256r1PublicKey::from(key3.clone()),
		];

		let script = VerificationScript::from_multi_sig(&mut pubkeys, 2);

		let expected = format!(
			"{}{}21{}{}21{}{}21{}{}{}{}",
			OpCode::Push2.to_string(),
			OpCode::PushData1.to_string(),
			key1.to_hex(),
			OpCode::PushData1.to_string(),
			key3.to_hex(),
			OpCode::PushData1.to_string(),
			key2.to_hex(),
			OpCode::Push3.to_string(),
			OpCode::Syscall.to_string(),
			InteropService::SystemCryptoCheckMultiSig.hash()
		)
		.from_hex()
		.unwrap();

		assert_eq!(script.script(), &expected);
	}

	#[test]
	fn test_serialize_deserialize() {
		let key =
			hex!("035fdb1d1f06759547020891ae97c729327853aeb1256b6fe0473bc2e9fa42ff50").to_vec();
		let pubkey = Secp256r1PublicKey::from(key.clone());

		let script = VerificationScript::from_public_key(&pubkey);

		let mut expected = ScriptBuilder::new();
		expected.push_data(hex!("21").to_vec());
		expected.push_data(key);
		expected.sys_call(InteropService::SystemCryptoCheckSig);

		let serialized = script.to_array();

		// Manually deserialize
		let deserialized = VerificationScript::from(serialized[1..].to_vec());

		// Check deserialized script matches
		assert_eq!(deserialized.script(), &expected.to_bytes());
	}

	#[test]
	fn test_get_signing_threshold() {
		// let key = hex!("...").to_vec();
		//
		// let script = VerificationScript::from(key);
		// assert_eq!(script.get_signing_threshold(), 2);
		//
		// let script = VerificationScript::from(long_script);
		// assert_eq!(script.get_signing_threshold(), 127);
	}

	#[test]
	fn test_invalid_script() {
		let script = VerificationScript::from(hex!("0123456789abcdef").to_vec());

		assert!(script.get_signing_threshold().is_err());
		assert!(script.get_public_keys().is_err());
		assert!(script.get_nr_of_accounts().is_err());
	}

	#[test]
	fn test_size() {
		let data = hex!("147e5f3c929dd830d961626551dbea6b70e4b2837ed2fe9089eed2072ab3a655523ae0fa8711eee4769f1913b180b9b3410bbb2cf770f529c85f6886f22cbaaf").to_vec();
		let script = VerificationScript::from(data);
		assert_eq!(script.size(), 65);
	}

	#[test]
	fn test_is_single_sig_script() {
		let script = format!(
			"{}2102028a99826edc0c97d18e22b6932373d908d323aa7f92656a77ec26e8861699ef{}{}",
			OpCode::PushData1.to_string(),
			OpCode::Syscall.to_string(),
			InteropService::SystemCryptoCheckSig.hash()
		);

		let verification = VerificationScript::from(script.from_hex().unwrap());
		assert!(verification.is_single_sig());
	}
}
