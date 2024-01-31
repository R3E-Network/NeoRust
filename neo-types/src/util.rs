// #![allow(unused_imports)]
// #![allow(dead_code)]

use blake2::digest::Update;

use rand::{
	seq::{IteratorRandom, SliceRandom},
	SeedableRng,
};

use crate::{error::TypeError, script_hash::ScriptHash};
use futures::AsyncWriteExt;
use neo_crypto::hash::HashableForVec;
use primitive_types::{H160, H256, U256};
use tiny_keccak::{Hasher, Keccak};

pub fn parse_string_u64(u64_str: &str) -> u64 {
	if u64_str.starts_with("0x") {
		u64::from_str_radix(u64_str, 16).unwrap()
	} else {
		u64::from_str_radix(u64_str, 10).unwrap()
	}
}

pub fn parse_string_u256(u256_str: &str) -> U256 {
	if u256_str.starts_with("0x") {
		U256::from_str_radix(u256_str, 16).unwrap()
	} else {
		U256::from_str_radix(u256_str, 10).unwrap()
	}
}

pub fn parse_address(address: &str) -> ScriptHash {
	let bytes = hex::decode(address.trim_start_matches("0x")).unwrap();
	let mut padded_bytes = [0_u8; 20];
	padded_bytes[20 - bytes.len()..].copy_from_slice(&bytes);
	ScriptHash::from_slice(&padded_bytes)
}

pub fn encode_string_h160(h160: &H160) -> String {
	format!("{:?}", h160).to_owned()
}

pub fn parse_string_h256(h256_str: &str) -> H256 {
	let bytes = hex::decode(h256_str.trim_start_matches("0x")).unwrap();
	// pad the bytes to 32bytes
	let mut padded_bytes = [0_u8; 32];
	padded_bytes[32 - bytes.len()..].copy_from_slice(&bytes);

	H256::from_slice(&padded_bytes)
}

pub fn encode_string_h256(h256: &H256) -> String {
	format!("{:?}", h256).to_owned()
}

pub fn encode_string_u256(u256: &U256) -> String {
	format!("0x{:x}", u256).to_owned()
}

pub fn encode_vec_string_vec_u256(item: Vec<U256>) -> Vec<String> {
	item.iter().map(|x| encode_string_u256(&x)).collect()
}

pub fn parse_vec_string_vec_u256(item: Vec<String>) -> Vec<U256> {
	item.iter().map(|x| parse_string_u256(&x)).collect()
}

pub fn h256_to_u256(item: H256) -> U256 {
	U256::from_big_endian(item.as_bytes())
}

pub fn bytes_to_string(mybytes: &[u8]) -> String {
	format!("0x{}", hex::encode(mybytes))
}

pub fn string_to_bytes(mystring: &str) -> Option<Vec<u8>> {
	if mystring.starts_with("0x") {
		let mystring = mystring.trim_start_matches("0x");
		let mybytes = match hex::decode(mystring) {
			Ok(mybytes) => Some(mybytes),
			Err(_) => None,
		};
		mybytes
	} else {
		None
	}
}

pub fn u256_sqrt(input: &U256) -> U256 {
	if *input < 2.into() {
		return input.clone()
	}
	let mut x: U256 = (input + U256::one()) >> 1;
	let mut y = input.clone();
	while x < y {
		y = x;
		x = (input / x + x) >> 1;
	}
	y
}

pub fn u256_min(x: U256, y: U256) -> U256 {
	if x > y {
		y
	} else {
		x
	}
}

pub fn vec_to_array32(vec: Vec<u8>) -> Result<[u8; 32], TypeError> {
	if vec.len() != 32 {
		return Err(TypeError::InvalidData(
			"Vector does not contain exactly 32 elements".to_string(),
		))
	}

	let mut array = [0u8; 32];
	let bytes = &vec[..array.len()]; // Take a slice of the vec
	array.copy_from_slice(bytes); // Copy the slice into the array
	Ok(array)
}

pub fn var_size(value: usize) -> usize {
	let mut v = value;
	let mut bytes = 0;
	while v > 0 {
		v >>= 8;
		bytes += 1;
	}
	if bytes == 0 {
		1
	} else {
		bytes
	}
}

pub trait ToBase58 {
	fn to_base58(&self) -> String;
}

impl ToBase58 for [u8] {
	fn to_base58(&self) -> String {
		bs58::encode(self).into_string()
	}
}

pub trait ToBase64 {
	fn to_base64(&self) -> String;
}

impl ToBase64 for [u8] {
	fn to_base64(&self) -> String {
		base64::encode(self)
	}
}

#[cfg(test)]
mod test {
	use super::*;

	// #[test]
	// pub fn test_blake2var_hash() {
	//     let mut data = [0_u8; 24];
	//     data[0..4].copy_from_slice(b"evm:");
	//     data[4..24].copy_from_slice(&hex::decode("7EF99B0E5bEb8ae42DbF126B40b87410a440a32a").unwrap());
	//     let hash = blake2_hash(&data);
	//     let actual = hex::decode("65f5fbd10250447019bb8b9e06f6918d033b2feb6478470137b1a552656e2911").unwrap();
	//     assert_eq!(&hash, actual.as_slice());
	// }

	#[test]
	pub fn test_bytes_to_string() {
		let mybytes = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
		let bytestring = bytes_to_string(&mybytes);
		let orig_bytestring = "0x0102030405060708090a";
		assert_eq!(&bytestring, orig_bytestring);
		let error_bytestring = "0102030405060708090a";
		let error_mybytes = string_to_bytes(error_bytestring);
		assert_eq!(None, error_mybytes);
		let ok_mybytes = string_to_bytes(orig_bytestring).unwrap();
		assert_eq!(&mybytes[..], &ok_mybytes[..]);
	}
}
