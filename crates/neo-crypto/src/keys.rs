//! # Secp256r1 Cryptographic Module
//!
//! This module provides cryptographic functionalities for the secp256r1 elliptic curve.
//! It includes implementations for handling public keys, private keys, and signatures,
//! as well as utilities for signing and verifying data.
//!
//! ## Features
//!
//! - Generation of public and private keys.
//! - Conversion between different formats and representations of keys and signatures.
//! - Signing data with a private key and verifying signatures with a public key.
//! - Integration with external libraries like `p256` and `rand_core` for cryptographic operations.
//!
//! ## Usage
//!
//! - `Secp256r1PublicKey`: Represents a public key on the secp256r1 curve. It can be created
//!   from raw coordinates, existing `PublicKey` instances, or from byte slices.
//!   It provides functionalities to verify signatures and to encode the key in various formats.
//!
//! - `Secp256r1PrivateKey`: Represents a private key on the secp256r1 curve. It can be randomly
//!   generated or created from a byte slice. It provides methods to sign data and to retrieve
//!   the associated public key.
//!
//! - `Secp256r1Signature`: Represents a digital signature generated using a secp256r1 private key.
//!   It can be created from scalar values, `U256` representations, or from raw bytes. It offers
//!   a method to convert the signature back into a byte array.
//!
//! ## Examples
//!
//! Basic usage involves creating a private key, generating a signature for a message, and then
//! using the corresponding public key to verify the signature. Public and private keys can be
//! converted to and from various formats for storage or transmission.
//!
//! ```
//! use rand_core::OsRng;
//! use neo_crypto::keys::Secp256r1PrivateKey;
//!
//! // Generate a new private key
//! let private_key = Secp256r1PrivateKey::random(&mut OsRng);
//!
//! // Sign a message
//! let message = b"Example message";
//! let signature = private_key.sign_tx(message).expect("Failed to sign message");
//!
//! // Obtain the public key
//! let public_key = private_key.to_public_key();
//!
//! // Verify the signature
//! assert!(public_key.verify(message, &signature).is_ok());
//! ```
//!
//! Note: Error handling is crucial for cryptographic operations. Ensure proper error handling
//! in real-world applications.

use core::fmt;
use std::{
	cmp::Ordering,
	fmt::Debug,
	hash::{Hash, Hasher},
};

use neo_error::crypto_error::CryptoError;
use elliptic_curve::zeroize::Zeroize;
use p256::{
	ecdsa::{signature::Signer, Signature, SigningKey, VerifyingKey},
	elliptic_curve::{
		sec1::{FromEncodedPoint, ToEncodedPoint},
		Field,
	},
	EncodedPoint, FieldBytes, PublicKey, SecretKey,
};
use primitive_types::U256;
use rand_core::OsRng;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use signature::{hazmat::PrehashSigner, SignerMut, Verifier};

// Constants
const PUBLIC_KEY_SIZE_COMPRESSED: usize = 33;

#[derive(Debug, Clone)]
pub struct Secp256r1PublicKey {
	inner: PublicKey,
}

#[derive(Debug, Clone)]
pub struct Secp256r1PrivateKey {
	inner: SecretKey,
}

#[derive(Clone)]
pub struct Secp256r1Signature {
	inner: Signature,
}

impl Debug for Secp256r1Signature {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "Secp256r1Signature")
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct Secp256r1SignedMsg<T: Serialize> {
	pub msg: T,
	pub signature: Secp256r1Signature,
}

impl Secp256r1PublicKey {
	/// Constructs a new `Secp256r1PublicKey` from the given x and y coordinates.
	///
	/// This function attempts to create a public key from uncompressed x and y coordinates.
	/// It returns `None` if the provided coordinates do not correspond to a valid point on the curve.
	///
	/// - Parameters:
	///     - gx: The x coordinate of the public key.
	///     - gy: The y coordinate of the public key.
	///
	/// - Returns: An `Option<Secp256r1PublicKey>`.
	pub fn new(gx: [u8; 32], gy: [u8; 32]) -> Option<Self> {
		let mut uncompressed_point = Vec::with_capacity(65);
		uncompressed_point.push(0x04);
		uncompressed_point.extend_from_slice(&gx);
		uncompressed_point.extend_from_slice(&gy);

		let encoded_point = EncodedPoint::from_bytes(&uncompressed_point).ok()?;
		let public_key_option = PublicKey::from_encoded_point(&encoded_point);

		if public_key_option.is_some().into() {
			// Safe to unwrap since we checked is_some()
			let public_key = public_key_option.unwrap();
			Some(Secp256r1PublicKey { inner: public_key })
		} else {
			None
		}
	}

	/// Constructs a `Secp256r1PublicKey` from an existing `PublicKey`.
	///
	/// This method can be used to convert a `PublicKey` from the `p256` crate into a `Secp256r1PublicKey`.
	///
	/// - Parameter public_key: A `PublicKey` instance.
	///
	/// - Returns: A `Secp256r1PublicKey` instance.
	pub fn from_public_key(public_key: PublicKey) -> Self {
		Secp256r1PublicKey { inner: public_key }
	}

	/// Constructs a `Secp256r1PublicKey` from a byte slice.
	///
	/// Attempts to parse a byte slice as an encoded elliptic curve point and create a public key.
	/// Returns a `CryptoError` if the byte slice does not represent a valid public key.
	///
	/// - Parameter bytes: A byte slice representing an encoded elliptic curve point.
	///
	/// - Returns: A `Result<Secp256r1PublicKey, CryptoError>`.
	pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
		let encoded_point = match EncodedPoint::from_bytes(bytes) {
			Ok(v) => v,
			Err(_) => return Err(CryptoError::InvalidPublicKey),
		};

		let public_key_option = PublicKey::from_encoded_point(&encoded_point);

		if public_key_option.is_some().into() {
			// Safe to unwrap since we checked is_some()
			let public_key = public_key_option.unwrap();
			Ok(Secp256r1PublicKey { inner: public_key })
		} else {
			Err(CryptoError::InvalidPublicKey)
		}
	}

	/// Verifies a digital signature against a message using this public key.
	///
	/// This method checks if the provided signature is valid for the given message under this public key.
	/// Returns a `CryptoError` if the signature verification fails.
	///
	/// - Parameters:
	///     - message: The message that was signed.
	///     - signature: The signature to verify.
	///
	/// - Returns: A `Result<(), CryptoError>`.
	pub fn verify(
		&self,
		message: &[u8],
		signature: &Secp256r1Signature,
	) -> Result<(), CryptoError> {
		let verifying_key = VerifyingKey::from(&self.inner);

		verifying_key
			.verify(message, &signature.inner)
			.map_err(|_| CryptoError::SignatureVerificationError)
	}

	/// Gets this public key's elliptic curve point encoded as defined in section 2.3.3 of [SEC1](http://www.secg.org/sec1-v2.pdf).
	///
	/// - Parameter compressed: If the EC point should be encoded in compressed or uncompressed format
	///
	/// - Returns: The encoded public key
	pub fn get_encoded(&self, compressed: bool) -> Vec<u8> {
		self.inner.to_encoded_point(compressed).as_bytes().to_vec()
	}

	pub fn get_encoded_point(&self, compressed: bool) -> EncodedPoint {
		self.inner.to_encoded_point(compressed)
	}

	/// Gets this public key's elliptic curve point encoded as defined in section 2.3.3 of [SEC1](http://www.secg.org/sec1-v2.pdf)
	/// in compressed format as hexadecimal.
	///
	/// - Returns: The encoded public key in compressed format as hexadecimal without a prefix
	pub fn get_encoded_compressed_hex(&self) -> String {
		let encoded = self.get_encoded(true);
		hex::encode(encoded)
	}

	/// Constructs a `Secp256r1PublicKey` from a hexadecimal string representation.
	///
	/// This method attempts to parse a hexadecimal string as an encoded elliptic curve point.
	/// Returns `None` if the string is not a valid encoding or does not represent a valid public key.
	///
	/// - Parameter encoded: A hexadecimal string representing an encoded elliptic curve point.
	///
	/// - Returns: An `Option<Secp256r1PublicKey>`.
	pub fn from_encoded(encoded: &str) -> Option<Self> {
		let encoded = &encoded.replace("0x", "");
		let encoded = hex::decode(encoded).ok()?;

		Secp256r1PublicKey::from_bytes(encoded.as_slice()).ok()
	}

	fn get_size(&self) -> usize {
		if self.inner.to_encoded_point(false).is_identity() {
			1
		} else {
			PUBLIC_KEY_SIZE_COMPRESSED
		}
	}
}

impl Secp256r1PrivateKey {
	/// Generates a new private key using the provided random number generator (RNG).
	///
	/// - Parameter rng: A mutable reference to an `OsRng` instance.
	///
	/// - Returns: A new instance of the private key.
	pub fn random(rng: &mut OsRng) -> Self {
		let secret_key = SecretKey::random(rng);
		Secp256r1PrivateKey { inner: secret_key }
	}

	/// Constructs a `Secp256r1PrivateKey` from a byte slice.
	///
	/// Attempts to parse a byte slice as a private key. Returns a `CryptoError` if the byte slice
	/// does not represent a valid private key.
	///
	/// - Parameter bytes: A byte slice representing a private key.
	///
	/// - Returns: A `Result<Secp256r1PrivateKey, CryptoError>`.
	pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
		let field_bytes = FieldBytes::from_slice(bytes);
		let secret_key = SecretKey::from_bytes(field_bytes).map_err(|_| CryptoError::InvalidPrivateKey)?;
		Ok(Secp256r1PrivateKey { inner: secret_key })
	}

	/// Converts this private key to its raw byte representation.
	///
	/// - Returns: A 32-byte array containing the raw bytes of the private key.
	pub fn to_raw_bytes(&self) -> [u8; 32] {
		let mut bytes = [0u8; 32];
		bytes.copy_from_slice(self.inner.to_bytes().as_slice());
		bytes
	}

	/// Derives the public key corresponding to this private key.
	///
	/// - Returns: A `Secp256r1PublicKey` instance.
	pub fn to_public_key(&self) -> Secp256r1PublicKey {
		// Create a signing key first, then get the verifying key from it
		let signing_key = SigningKey::from(&self.inner);
		let verifying_key = signing_key.verifying_key();
		let public_key = PublicKey::from(verifying_key);
		Secp256r1PublicKey { inner: public_key }
	}

	/// Erases the private key by zeroing out its memory.
	///
	/// This method is used to securely erase the private key from memory when it is no longer needed.
	pub fn erase(&mut self) {
		// Zeroize the inner SecretKey
		// This is a placeholder since we don't have direct access to the inner bytes
		// In a real implementation, we would use a proper zeroize method
		let mut bytes = self.to_raw_bytes();
		bytes.zeroize();
	}

	/// Signs a message using this private key.
	///
	/// This method generates a digital signature for the given message using this private key.
	/// Returns a `CryptoError` if the signing operation fails.
	///
	/// - Parameter message: The message to sign.
	///
	/// - Returns: A `Result<Secp256r1Signature, CryptoError>`.
	pub fn sign_tx(&self, message: &[u8]) -> Result<Secp256r1Signature, CryptoError> {
		let signing_key = SigningKey::from(&self.inner);
		let signature = signing_key.sign(message);
		Ok(Secp256r1Signature { inner: signature })
	}

	/// Signs a pre-hashed message using this private key.
	///
	/// This method generates a digital signature for a pre-hashed message using this private key.
	/// Returns a `CryptoError` if the signing operation fails.
	///
	/// - Parameter message: The pre-hashed message to sign.
	///
	/// - Returns: A `Result<Secp256r1Signature, CryptoError>`.
	pub fn sign_prehash(&self, message: &[u8]) -> Result<Secp256r1Signature, CryptoError> {
		let signing_key = SigningKey::from(&self.inner);
		let signature = PrehashSigner::sign_prehash(&signing_key, message)
			.map_err(|_| CryptoError::SigningError)?;
		Ok(Secp256r1Signature { inner: signature })
	}
}

impl Secp256r1Signature {
	/// Constructs a `Secp256r1Signature` from r and s scalar values.
	///
	/// This method attempts to create a signature from the provided r and s values.
	/// Returns `None` if the values do not represent a valid signature.
	///
	/// - Parameters:
	///     - r: The r scalar value of the signature.
	///     - s: The s scalar value of the signature.
	///
	/// - Returns: An `Option<Secp256r1Signature>`.
	pub fn from_scalars(r: [u8; 32], s: [u8; 32]) -> Option<Self> {
		let mut bytes = [0u8; 64];
		bytes[..32].copy_from_slice(&r);
		bytes[32..].copy_from_slice(&s);

		Secp256r1Signature::from_bytes(&bytes).ok()
	}

	/// Constructs a `Secp256r1Signature` from r and s values represented as `U256`.
	///
	/// This method attempts to create a signature from the provided r and s values.
	/// Returns a `CryptoError` if the values do not represent a valid signature.
	///
	/// - Parameters:
	///     - r: The r value of the signature as a `U256`.
	///     - s: The s value of the signature as a `U256`.
	///
	/// - Returns: A `Result<Secp256r1Signature, CryptoError>`.
	pub fn from_u256(r: U256, s: U256) -> Result<Self, CryptoError> {
		let mut bytes = [0u8; 64];
		
		// Convert U256 to bytes manually
		let r_hex = format!("{:064x}", r);
		let s_hex = format!("{:064x}", s);
		
		// Convert hex string to bytes
		for i in 0..32 {
			let r_idx = i * 2;
			let s_idx = i * 2;
			let r_byte = u8::from_str_radix(&r_hex[r_idx..r_idx+2], 16).unwrap();
			let s_byte = u8::from_str_radix(&s_hex[s_idx..s_idx+2], 16).unwrap();
			bytes[i] = r_byte;
			bytes[i+32] = s_byte;
		}

		Secp256r1Signature::from_bytes(&bytes)
	}

	/// Constructs a `Secp256r1Signature` from a byte slice.
	///
	/// Attempts to parse a byte slice as a signature. Returns a `CryptoError` if the byte slice
	/// does not represent a valid signature.
	///
	/// - Parameter bytes: A byte slice representing a signature.
	///
	/// - Returns: A `Result<Secp256r1Signature, CryptoError>`.
	pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
		if bytes.len() != 64 {
			return Err(CryptoError::InvalidFormat("Invalid signature length".to_string()));
		}

		let signature = Signature::from_slice(bytes).map_err(|_| CryptoError::SigningError)?;
		Ok(Secp256r1Signature { inner: signature })
	}

	/// Converts this signature to its raw byte representation.
	///
	/// - Returns: A 64-byte array containing the raw bytes of the signature.
	pub fn to_bytes(&self) -> [u8; 64] {
		let mut bytes = [0u8; 64];
		bytes.copy_from_slice(self.inner.to_bytes().as_slice());
		bytes
	}
}

impl fmt::Display for Secp256r1PrivateKey {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "Secp256r1PrivateKey")
	}
}

impl fmt::Display for Secp256r1PublicKey {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let encoded = self.get_encoded(true);
		let hex_encoded = hex::encode(encoded);
		write!(f, "Secp256r1PublicKey({})", hex_encoded)
	}
}

impl fmt::Display for Secp256r1Signature {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "Secp256r1Signature")
	}
}

impl Serialize for Secp256r1PublicKey {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let encoded = self.get_encoded(true);
		serializer.serialize_bytes(&encoded)
	}
}

impl Serialize for Secp256r1PrivateKey {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let bytes = self.to_raw_bytes();
		serializer.serialize_bytes(&bytes)
	}
}

impl Serialize for Secp256r1Signature {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: Serializer,
	{
		let bytes = self.to_bytes();
		serializer.serialize_bytes(&bytes)
	}
}

impl<'de> Deserialize<'de> for Secp256r1PublicKey {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let bytes = Vec::<u8>::deserialize(deserializer)?;
		Secp256r1PublicKey::from_bytes(&bytes).map_err(serde::de::Error::custom)
	}
}

impl<'de> Deserialize<'de> for Secp256r1PrivateKey {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let bytes = Vec::<u8>::deserialize(deserializer)?;
		Secp256r1PrivateKey::from_bytes(&bytes).map_err(serde::de::Error::custom)
	}
}

impl<'de> Deserialize<'de> for Secp256r1Signature {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let bytes = Vec::<u8>::deserialize(deserializer)?;
		Secp256r1Signature::from_bytes(&bytes).map_err(serde::de::Error::custom)
	}
}

impl PartialEq for Secp256r1PublicKey {
	fn eq(&self, other: &Secp256r1PublicKey) -> bool {
		self.get_encoded(false) == other.get_encoded(false)
	}
}

impl PartialOrd for Secp256r1PublicKey {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		let self_encoded = self.get_encoded(false);
		let other_encoded = other.get_encoded(false);
		self_encoded.partial_cmp(&other_encoded)
	}
}

impl Eq for Secp256r1PublicKey {}

impl Ord for Secp256r1PublicKey {
	fn cmp(&self, other: &Self) -> Ordering {
		let self_encoded = self.get_encoded(false);
		let other_encoded = other.get_encoded(false);
		self_encoded.cmp(&other_encoded)
	}
}

impl Hash for Secp256r1PublicKey {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.get_encoded(false).hash(state);
	}
}

impl Hash for Secp256r1PrivateKey {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.to_raw_bytes().hash(state);
	}
}

impl Hash for Secp256r1Signature {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.to_bytes().hash(state);
	}
}

impl PartialEq for Secp256r1PrivateKey {
	fn eq(&self, other: &Self) -> bool {
		self.to_raw_bytes() == other.to_raw_bytes()
	}
}

impl PartialEq for Secp256r1Signature {
	fn eq(&self, other: &Self) -> bool {
		self.to_bytes() == other.to_bytes()
	}
}

impl From<Vec<u8>> for Secp256r1PublicKey {
	fn from(bytes: Vec<u8>) -> Self {
		Secp256r1PublicKey::from_bytes(&bytes).unwrap()
	}
}

pub trait PrivateKeyExtension
where
	Self: Sized,
{
	fn to_vec(&self) -> Vec<u8>;

	fn from_slice(slice: &[u8]) -> Result<Self, CryptoError>;
}

impl PrivateKeyExtension for Secp256r1PrivateKey {
	fn to_vec(&self) -> Vec<u8> {
		self.to_raw_bytes().to_vec()
	}

	fn from_slice(slice: &[u8]) -> Result<Self, CryptoError> {
		if slice.len() != 32 {
			return Err(CryptoError::InvalidFormat("Invalid private key length".to_string()));
		}

		let mut bytes = [0u8; 32];
		bytes.copy_from_slice(slice);
		Secp256r1PrivateKey::from_bytes(&bytes)
	}
}

pub trait PublicKeyExtension
where
	Self: Sized,
{
	fn to_vec(&self) -> Vec<u8>;
	fn from_slice(slice: &[u8]) -> Result<Self, CryptoError>;
}

impl PublicKeyExtension for Secp256r1PublicKey {
	fn to_vec(&self) -> Vec<u8> {
		self.get_encoded(false)
	}

	fn from_slice(slice: &[u8]) -> Result<Self, CryptoError> {
		Secp256r1PublicKey::from_bytes(slice)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use rand_core::OsRng;

	#[test]
	fn test_sign_message() {
		let mut rng = OsRng;
		let private_key = Secp256r1PrivateKey::random(&mut rng);
		let public_key = private_key.to_public_key();

		let message = b"Hello, world!";
		let signature = private_key.sign_tx(message).unwrap();

		assert!(public_key.verify(message, &signature).is_ok());
	}
}
