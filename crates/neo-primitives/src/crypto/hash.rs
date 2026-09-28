//! Hashing utilities for Neo N3

use ripemd::{Digest as RipemdDigest, Ripemd160};
use sha2::Sha256;

/// Single SHA256 hash
#[inline]
pub fn sha256(data: &[u8]) -> [u8; 32] {
	let mut hasher = Sha256::new();
	hasher.update(data);
	hasher.finalize().into()
}

/// Double SHA256 hash (SHA256(SHA256(data)))
#[inline]
pub fn hash256(data: &[u8]) -> [u8; 32] {
	sha256(&sha256(data))
}

/// RIPEMD160 hash
#[inline]
pub fn ripemd160(data: &[u8]) -> [u8; 20] {
	let mut hasher = Ripemd160::new();
	hasher.update(data);
	hasher.finalize().into()
}

/// Hash160: RIPEMD160(SHA256(data))
/// This is the standard Neo script hash algorithm
#[inline]
pub fn hash160(data: &[u8]) -> [u8; 20] {
	ripemd160(&sha256(data))
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_sha256() {
		let data = b"hello";
		let expected =
			hex::decode("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824")
				.unwrap();
		assert_eq!(sha256(data).as_slice(), expected);
	}

	#[test]
	fn test_hash256() {
		// Test double SHA256
		let data = b"hello";
		let hash1 = sha256(data);
		let hash2 = sha256(&hash1);
		assert_eq!(hash256(data).as_slice(), hash2.as_slice());
	}

	#[test]
	fn test_hash160_verification_script() {
		// Real Neo N3 test vector from test_properties.rs
		let script = hex::decode(
			"0c21033a4d051b04b7fc0230d2b1aaedfd5a84be279a5361a7358db665ad7857787f1b4156e7b327",
		)
		.unwrap();
		let hash = hash160(&script);
		// Raw hash160 output in wire/LE order
		assert_eq!(hex::encode(hash), "0d165c9899c38bbf5991c5e47b04937258caec69");
	}
}
