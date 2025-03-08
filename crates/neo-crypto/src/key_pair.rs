//! # KeyPair
//!
//! `KeyPair` is a module that provides an implementation for Elliptic Curve Key Pairs using the `p256` crate.
//!
//! This structure can be used to manage and manipulate EC key pairs,
//! including generating new pairs, importing them from raw bytes,
//! and converting them to various formats.

use rand::rngs::OsRng;
use neo_error::crypto_error::CryptoError;

use crate::keys::{Secp256r1PrivateKey, Secp256r1PublicKey, PublicKeyExtension};
use crate::wif::{private_key_from_wif, wif_from_private_key};

/// Represents an Elliptic Curve Key Pair containing both a private and a public key.
#[derive(Debug, Clone)]
pub struct KeyPair {
	/// The private key component of the key pair.
	pub private_key: Secp256r1PrivateKey,

	/// The public key component of the key pair.
	pub public_key: Secp256r1PublicKey,
}

/// A script hash type
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptHash(pub [u8; 20]);

impl ScriptHash {
	/// Convert the script hash to an address
	pub fn to_address(&self) -> String {
		// This is a placeholder implementation
		// In a real implementation, this would convert the script hash to a Neo address
		format!("N{}", hex::encode(&self.0))
	}

	/// Create a script hash from a hex string
	pub fn from_hex(hex_str: &str) -> Result<Self, CryptoError> {
		let bytes = hex::decode(hex_str)
			.map_err(|e| CryptoError::InvalidFormat(format!("Invalid hex: {}", e)))?;
		
		if bytes.len() != 20 {
			return Err(CryptoError::InvalidFormat("Script hash must be 20 bytes".to_string()));
		}
		
		let mut hash = [0u8; 20];
		hash.copy_from_slice(&bytes);
		Ok(ScriptHash(hash))
	}
}

/// A verification script
#[derive(Debug, Clone)]
pub struct VerificationScript(pub Vec<u8>);

impl VerificationScript {
	/// Create a verification script from a public key
	pub fn from_public_key(public_key: &Secp256r1PublicKey) -> Self {
		// This is a placeholder implementation
		// In a real implementation, this would create a proper verification script
		let mut script = Vec::new();
		script.extend_from_slice(&[33]); // Push the public key length
		script.extend_from_slice(&public_key.get_encoded(true)); // Push the compressed public key
		script.extend_from_slice(&[0xac]); // CHECKSIG opcode
		VerificationScript(script)
	}

	/// Hash the verification script to get a script hash
	pub fn hash(&self) -> ScriptHash {
		// This is a placeholder implementation
		// In a real implementation, this would hash the script properly
		let mut hash = [0u8; 20];
		// Just use the first 20 bytes of the script as a placeholder
		let len = std::cmp::min(self.0.len(), 20);
		hash[..len].copy_from_slice(&self.0[..len]);
		ScriptHash(hash)
	}
}

impl KeyPair {
	/// Creates a new `KeyPair` instance given a private key and its corresponding public key.
	///
	/// # Arguments
	///
	/// * `private_key` - A `Secp256r1PrivateKey` representing the private key.
	/// * `public_key` - A `Secp256r1PublicKey` representing the public key.
	pub fn new(private_key: Secp256r1PrivateKey, public_key: Secp256r1PublicKey) -> Self {
		Self { private_key, public_key }
	}

	pub fn private_key(&self) -> Secp256r1PrivateKey {
		self.private_key.clone()
	}

	pub fn public_key(&self) -> Secp256r1PublicKey {
		self.public_key.clone()
	}

	/// Derives a new `KeyPair` instance from just a private key.
	/// The public key is derived from the given private key.
	///
	/// # Arguments
	///
	/// * `private_key` - A `Secp256r1PrivateKey` representing the private key.
	pub fn from_secret_key(private_key: &Secp256r1PrivateKey) -> Self {
		let public_key = private_key.clone().to_public_key();
		Self::new(private_key.clone(), public_key)
	}

	/// Returns the 32-byte representation of the private key.
	pub fn private_key_bytes(&self) -> [u8; 32] {
		self.private_key.to_raw_bytes()
	}

	/// Returns the 64-byte uncompressed representation of the public key.
	pub fn public_key_bytes(&self) -> [u8; 64] {
		let mut buf = [0u8; 64];
		// Convert the Secp256r1PublicKey to its byte representation
		let vec_bytes: Vec<u8> = self.public_key.to_vec(); // uncompressed form
		buf.copy_from_slice(&vec_bytes[0..64]);

		buf
	}
}

impl KeyPair {
	/// Generates a new random `KeyPair`.
	pub fn new_random() -> Self {
		let mut rng = OsRng; // A cryptographically secure random number generator
		let secret_key = Secp256r1PrivateKey::random(&mut rng);
		Self::from_secret_key(&secret_key)
	}

	/// Creates an `KeyPair` from a given 32-byte private key.
	///
	/// # Arguments
	///
	/// * `private_key` - A 32-byte slice representing the private key.
	pub fn from_private_key(private_key: &[u8; 32]) -> Result<Self, CryptoError> {
		let secret_key = Secp256r1PrivateKey::from_bytes(private_key)?;
		Ok(Self::from_secret_key(&secret_key))
	}

	/// Creates an `KeyPair` from a given Wallet Import Format (WIF) string.
	/// This will use the private key encoded in the WIF to generate the key pair.
	///
	///  # Arguments
	///
	/// * `wif` - A Wallet Import Format (WIF) string.
	///
	/// The WIF string should be in the format `Kx...` or `Lx...`.
	/// The key pair will be generated from the private key encoded in the WIF.
	/// The public key will be derived from the private key.
	pub fn from_wif(wif: &str) -> Result<Self, CryptoError> {
		let private_key = private_key_from_wif(wif)?;
		Ok(Self::from_secret_key(&private_key))
	}

	/// Creates an `KeyPair` from a given 65-byte public key.
	/// This will use a dummy private key internally.
	///
	/// # Arguments
	///
	/// * `public_key` - A 65-byte slice representing the uncompressed public key.
	pub fn from_public_key(public_key: &[u8; 64]) -> Result<Self, CryptoError> {
		let public_key = Secp256r1PublicKey::from_slice(public_key)?;
		let secret_key = Secp256r1PrivateKey::from_bytes(&[0u8; 32]).unwrap(); // dummy private key
		Ok(Self::new(secret_key, public_key))
	}

	/// Exports the key pair as a Wallet Import Format (WIF) string
	///
	/// Returns: The WIF encoding of this key pair
	pub fn export_as_wif(&self) -> String {
		wif_from_private_key(&self.private_key())
	}

	pub fn get_script_hash(&self) -> ScriptHash {
		let vs = VerificationScript::from_public_key(&self.public_key());
		vs.hash()
	}

	pub fn get_address(&self) -> String {
		self.get_script_hash().to_address()
	}
}

impl PartialEq for KeyPair {
	fn eq(&self, other: &Self) -> bool {
		self.private_key == other.private_key && self.public_key == other.public_key
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_public_key_wif() {
		let private_key = hex::decode("c7134d6fd8e73d819e82755c64c93788d8db0961929e025a53363c4cc02a6962")
			.unwrap();
		let private_key_arr: &[u8; 32] = private_key.as_slice().try_into().unwrap();
		let key_pair = KeyPair::from_private_key(private_key_arr).unwrap();
		assert_eq!(
			key_pair.export_as_wif(),
			"L3tgppXLgdaeqSGSFw1Go3skBiy8vQAM7YMXvTHsKQtE16PBncSU"
		);
	}
}
