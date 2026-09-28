//! Cryptographic key types for Neo N3
//!
//! Neo N3 uses secp256r1 (NIST P-256) for all key operations.

use crate::{
	crypto,
	errors::{Error, Result},
	types::{Address, Hash160, Script},
};
use alloc::{format, string::String, vec::Vec};
use core::fmt;
use p256::{
	ecdsa::{signature::Verifier, Signature as EcdsaSignature, SigningKey, VerifyingKey},
	elliptic_curve::sec1::{FromEncodedPoint, ToEncodedPoint},
	EncodedPoint, PublicKey as P256PublicKey, SecretKey,
};
use serde::{Deserialize, Serialize};

/// A secp256r1 private key (32 bytes).
///
/// `p256::SecretKey` zeroizes its scalar on drop, so key material does not
/// linger in memory after a `PrivateKey` is dropped.
#[derive(Clone)]
pub struct PrivateKey(SecretKey);

impl PrivateKey {
	/// Size in bytes (32).
	pub const LEN: usize = 32;

	/// Creates a private key from raw bytes.
	pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
		if bytes.len() != Self::LEN {
			return Err(Error::InvalidKey(format!(
				"expected {} bytes, got {}",
				Self::LEN,
				bytes.len()
			)));
		}

		let secret = SecretKey::from_slice(bytes)
			.map_err(|e| Error::InvalidKey(format!("invalid scalar: {}", e)))?;
		Ok(Self(secret))
	}

	/// Generates a random private key (requires `std` feature).
	#[cfg(feature = "std")]
	pub fn random() -> Result<Self> {
		use p256::elliptic_curve::rand_core::OsRng;
		let secret = SecretKey::random(&mut OsRng);
		Ok(Self(secret))
	}

	/// Derives the corresponding public key.
	pub fn public_key(&self) -> PublicKey {
		PublicKey(self.0.public_key())
	}

	/// Returns the raw 32-byte scalar.
	pub fn to_bytes(&self) -> [u8; 32] {
		self.0.to_bytes().into()
	}

	/// Encodes as WIF (Wallet Import Format).
	///
	/// Neo N3 WIF format: Base58Check(0x80 || private_key || 0x01)
	/// The trailing 0x01 indicates a compressed public key.
	pub fn to_wif(&self) -> String {
		let mut payload = Vec::with_capacity(34);
		payload.push(0x80); // Bitcoin/Neo private key version
		payload.extend_from_slice(&self.to_bytes());
		payload.push(0x01); // Compressed flag

		let checksum = &crypto::hash256(&payload)[..4];
		payload.extend_from_slice(checksum);
		bs58::encode(payload).into_string()
	}

	/// Decodes from WIF (Wallet Import Format).
	pub fn from_wif(wif: &str) -> Result<Self> {
		let decoded = bs58::decode(wif)
			.into_vec()
			.map_err(|_| Error::InvalidKey("base58 decode failed".into()))?;

		if decoded.len() != 38 {
			return Err(Error::InvalidKey(format!("expected 38 bytes, got {}", decoded.len())));
		}

		if decoded[0] != 0x80 {
			return Err(Error::InvalidKey(format!(
				"wrong version: expected 0x80, got {:#x}",
				decoded[0]
			)));
		}

		if decoded[33] != 0x01 {
			return Err(Error::InvalidKey("missing compressed-key flag (0x01)".into()));
		}

		let payload = &decoded[..34];
		let checksum = &decoded[34..38];
		let expected = &crypto::hash256(payload)[..4];

		if checksum != expected {
			return Err(Error::InvalidKey("checksum mismatch".into()));
		}

		Self::from_bytes(&decoded[1..33])
	}

	/// Signs a message hash with ECDSA.
	///
	/// `message_hash` must be the pre-hashed message (Neo uses SHA256).
	pub fn sign_hash(&self, message_hash: &[u8; 32]) -> Signature {
		use p256::ecdsa::signature::hazmat::PrehashSigner;
		let signing_key = SigningKey::from(&self.0);
		let sig: EcdsaSignature = signing_key
			.sign_prehash(message_hash)
			.expect("signing a 32-byte prehash cannot fail");
		Signature(sig)
	}

	/// Signs a message, hashing it with SHA256 first.
	pub fn sign(&self, message: &[u8]) -> Signature {
		self.sign_hash(&crypto::sha256(message))
	}
}

impl fmt::Debug for PrivateKey {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		// Never print key material.
		f.write_str("PrivateKey(<redacted>)")
	}
}

impl PartialEq for PrivateKey {
	fn eq(&self, other: &Self) -> bool {
		use subtle::ConstantTimeEq;
		self.to_bytes().ct_eq(&other.to_bytes()).into()
	}
}

impl Eq for PrivateKey {}

/// A secp256r1 public key.
///
/// Serialized in SEC1 compressed form (33 bytes) as Neo requires.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PublicKey(P256PublicKey);

impl PublicKey {
	/// Size of the compressed encoding (33 bytes).
	pub const COMPRESSED_LEN: usize = 33;

	/// Parses a public key from its SEC1 encoding.
	///
	/// Accepts compressed (33 bytes) and uncompressed (65 bytes) forms.
	pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
		if bytes.len() != 33 && bytes.len() != 65 {
			return Err(Error::InvalidKey(format!("expected 33 or 65 bytes, got {}", bytes.len())));
		}

		let point = EncodedPoint::from_bytes(bytes)
			.map_err(|e| Error::InvalidKey(format!("invalid SEC1 encoding: {}", e)))?;

		let key = Option::<P256PublicKey>::from(P256PublicKey::from_encoded_point(&point))
			.ok_or_else(|| Error::InvalidKey("point not on curve".into()))?;

		Ok(Self(key))
	}

	/// Parses from a hex string (with or without `0x` prefix).
	pub fn from_hex(s: &str) -> Result<Self> {
		let stripped = s.strip_prefix("0x").unwrap_or(s);
		let bytes =
			hex::decode(stripped).map_err(|e| Error::InvalidKey(format!("invalid hex: {}", e)))?;
		Self::from_bytes(&bytes)
	}

	/// Returns the compressed SEC1 encoding (33 bytes).
	pub fn to_bytes(&self) -> [u8; 33] {
		let point = self.0.to_encoded_point(true);
		let mut out = [0u8; 33];
		out.copy_from_slice(point.as_bytes());
		out
	}

	/// Returns the compressed encoding as lowercase hex (no prefix).
	pub fn to_hex(&self) -> String {
		hex::encode(self.to_bytes())
	}

	/// Builds the single-signature verification script for this key.
	///
	/// Layout: `PUSHDATA1 33 <pubkey> SYSCALL <System.Crypto.CheckSig>`
	pub fn to_verification_script(&self) -> Script {
		let mut script = Vec::with_capacity(40);
		script.push(0x0c); // PUSHDATA1
		script.push(Self::COMPRESSED_LEN as u8);
		script.extend_from_slice(&self.to_bytes());
		script.push(0x41); // SYSCALL
		script.extend_from_slice(&CHECKSIG_INTEROP);
		Script::new(script)
	}

	/// Computes the script hash of this key's verification script.
	pub fn to_script_hash(&self) -> Hash160 {
		self.to_verification_script().to_script_hash()
	}

	/// Computes the Neo N3 address for this key.
	pub fn to_address(&self) -> Address {
		Address::from_script_hash(&self.to_script_hash())
	}

	/// Verifies an ECDSA signature over a pre-hashed message.
	pub fn verify_hash(&self, message_hash: &[u8; 32], signature: &Signature) -> bool {
		use p256::ecdsa::signature::hazmat::PrehashVerifier;
		let verifying_key = VerifyingKey::from(&self.0);
		verifying_key.verify_prehash(message_hash, &signature.0).is_ok()
	}

	/// Verifies an ECDSA signature, hashing the message with SHA256 first.
	pub fn verify(&self, message: &[u8], signature: &Signature) -> bool {
		let verifying_key = VerifyingKey::from(&self.0);
		verifying_key.verify(message, &signature.0).is_ok()
	}
}

/// First 4 bytes of `SHA256("System.Crypto.CheckSig")`, the interop descriptor
/// the Neo VM uses for single-signature verification.
const CHECKSIG_INTEROP: [u8; 4] = [0x56, 0xe7, 0xb3, 0x27];

impl fmt::Debug for PublicKey {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "PublicKey({})", self.to_hex())
	}
}

impl fmt::Display for PublicKey {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(&self.to_hex())
	}
}

impl Serialize for PublicKey {
	fn serialize<S: serde::Serializer>(
		&self,
		serializer: S,
	) -> core::result::Result<S::Ok, S::Error> {
		serializer.serialize_str(&self.to_hex())
	}
}

impl<'de> Deserialize<'de> for PublicKey {
	fn deserialize<D: serde::Deserializer<'de>>(
		deserializer: D,
	) -> core::result::Result<Self, D::Error> {
		let s = String::deserialize(deserializer)?;
		Self::from_hex(&s).map_err(serde::de::Error::custom)
	}
}

/// An ECDSA signature over secp256r1.
///
/// Neo serializes signatures as raw 64-byte `r || s`, not DER.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Signature(EcdsaSignature);

impl Signature {
	/// Size of the raw encoding (64 bytes).
	pub const LEN: usize = 64;

	/// Parses a raw 64-byte `r || s` signature.
	pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
		if bytes.len() != Self::LEN {
			return Err(Error::InvalidSignature(format!(
				"expected {} bytes, got {}",
				Self::LEN,
				bytes.len()
			)));
		}

		let sig = EcdsaSignature::from_slice(bytes)
			.map_err(|e| Error::InvalidSignature(format!("invalid signature: {}", e)))?;
		Ok(Self(sig))
	}

	/// Parses from a hex string (with or without `0x` prefix).
	pub fn from_hex(s: &str) -> Result<Self> {
		let stripped = s.strip_prefix("0x").unwrap_or(s);
		let bytes = hex::decode(stripped)
			.map_err(|e| Error::InvalidSignature(format!("invalid hex: {}", e)))?;
		Self::from_bytes(&bytes)
	}

	/// Returns the raw 64-byte `r || s` encoding.
	pub fn to_bytes(&self) -> [u8; 64] {
		let mut out = [0u8; 64];
		out.copy_from_slice(&self.0.to_bytes());
		out
	}

	/// Returns the raw encoding as lowercase hex (no prefix).
	pub fn to_hex(&self) -> String {
		hex::encode(self.to_bytes())
	}
}

impl fmt::Debug for Signature {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "Signature({})", self.to_hex())
	}
}

impl fmt::Display for Signature {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(&self.to_hex())
	}
}

impl Serialize for Signature {
	fn serialize<S: serde::Serializer>(
		&self,
		serializer: S,
	) -> core::result::Result<S::Ok, S::Error> {
		serializer.serialize_str(&self.to_hex())
	}
}

impl<'de> Deserialize<'de> for Signature {
	fn deserialize<D: serde::Deserializer<'de>>(
		deserializer: D,
	) -> core::result::Result<Self, D::Error> {
		let s = String::deserialize(deserializer)?;
		Self::from_hex(&s).map_err(serde::de::Error::custom)
	}
}

/// A secp256r1 key pair.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyPair {
	private_key: PrivateKey,
	public_key: PublicKey,
}

impl KeyPair {
	/// Builds a key pair from a private key.
	pub fn from_private_key(private_key: PrivateKey) -> Self {
		let public_key = private_key.public_key();
		Self { private_key, public_key }
	}

	/// Builds a key pair from raw private key bytes.
	pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
		Ok(Self::from_private_key(PrivateKey::from_bytes(bytes)?))
	}

	/// Builds a key pair from a WIF-encoded private key.
	pub fn from_wif(wif: &str) -> Result<Self> {
		Ok(Self::from_private_key(PrivateKey::from_wif(wif)?))
	}

	/// Generates a random key pair (requires `std` feature).
	#[cfg(feature = "std")]
	pub fn random() -> Result<Self> {
		Ok(Self::from_private_key(PrivateKey::random()?))
	}

	/// The private key.
	pub fn private_key(&self) -> &PrivateKey {
		&self.private_key
	}

	/// The public key.
	pub fn public_key(&self) -> &PublicKey {
		&self.public_key
	}

	/// The Neo N3 address for this key pair.
	pub fn address(&self) -> Address {
		self.public_key.to_address()
	}

	/// The script hash for this key pair.
	pub fn script_hash(&self) -> Hash160 {
		self.public_key.to_script_hash()
	}

	/// Signs a message, hashing it with SHA256 first.
	pub fn sign(&self, message: &[u8]) -> Signature {
		self.private_key.sign(message)
	}

	/// Signs a pre-hashed message.
	pub fn sign_hash(&self, message_hash: &[u8; 32]) -> Signature {
		self.private_key.sign_hash(message_hash)
	}

	/// Verifies a signature produced by this key pair.
	pub fn verify(&self, message: &[u8], signature: &Signature) -> bool {
		self.public_key.verify(message, signature)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	// Real Neo N3 test vectors (the SDK's default test account).
	const PRIVATE_KEY_HEX: &str =
		"84180ac9d6eb6fba207ea4ef9d2200102d1ebeb4b9c07e2c6a738a42742e27a5";
	const PUBLIC_KEY_HEX: &str =
		"033a4d051b04b7fc0230d2b1aaedfd5a84be279a5361a7358db665ad7857787f1b";
	const WIF: &str = "L1eV34wPoj9weqhGijdDLtVQzUpWGHszXXpdU9dPuh2nRFFzFa7E";
	const ADDRESS: &str = "NM7Aky765FG8NhhwtxjXRx7jEL1cnw7PBP";
	const SCRIPT_HASH: &str = "0x69ecca587293047be4c59159bf8bc399985c160d";
	const VERIFICATION_SCRIPT: &str =
		"0c21033a4d051b04b7fc0230d2b1aaedfd5a84be279a5361a7358db665ad7857787f1b4156e7b327";

	fn test_key() -> PrivateKey {
		PrivateKey::from_bytes(&hex::decode(PRIVATE_KEY_HEX).unwrap()).unwrap()
	}

	#[test]
	fn test_private_key_derives_expected_public_key() {
		assert_eq!(test_key().public_key().to_hex(), PUBLIC_KEY_HEX);
	}

	#[test]
	fn test_private_key_bytes_roundtrip() {
		let key = test_key();
		assert_eq!(hex::encode(key.to_bytes()), PRIVATE_KEY_HEX);
	}

	#[test]
	fn test_private_key_rejects_wrong_length() {
		assert!(PrivateKey::from_bytes(&[0u8; 31]).is_err());
		assert!(PrivateKey::from_bytes(&[0u8; 33]).is_err());
	}

	#[test]
	fn test_private_key_rejects_zero_scalar() {
		// Zero is not a valid secp256r1 scalar.
		assert!(PrivateKey::from_bytes(&[0u8; 32]).is_err());
	}

	#[test]
	fn test_private_key_debug_redacts_material() {
		let debug = format!("{:?}", test_key());
		assert!(!debug.contains(PRIVATE_KEY_HEX));
		assert!(debug.contains("redacted"));
	}

	#[test]
	fn test_wif_encoding() {
		assert_eq!(test_key().to_wif(), WIF);
	}

	#[test]
	fn test_wif_decoding() {
		let key = PrivateKey::from_wif(WIF).unwrap();
		assert_eq!(hex::encode(key.to_bytes()), PRIVATE_KEY_HEX);
	}

	#[test]
	fn test_wif_roundtrip() {
		let key = test_key();
		let decoded = PrivateKey::from_wif(&key.to_wif()).unwrap();
		assert_eq!(key, decoded);
	}

	#[test]
	fn test_wif_rejects_bad_checksum() {
		let mut bad = WIF.to_string();
		bad.pop();
		bad.push('Z');
		assert!(PrivateKey::from_wif(&bad).is_err());
	}

	#[test]
	fn test_wif_rejects_garbage() {
		assert!(PrivateKey::from_wif("not-a-wif").is_err());
		assert!(PrivateKey::from_wif("").is_err());
	}

	#[test]
	fn test_public_key_hex_roundtrip() {
		let key = PublicKey::from_hex(PUBLIC_KEY_HEX).unwrap();
		assert_eq!(key.to_hex(), PUBLIC_KEY_HEX);
	}

	#[test]
	fn test_public_key_accepts_0x_prefix() {
		let with = PublicKey::from_hex(&format!("0x{}", PUBLIC_KEY_HEX)).unwrap();
		let without = PublicKey::from_hex(PUBLIC_KEY_HEX).unwrap();
		assert_eq!(with, without);
	}

	#[test]
	fn test_public_key_uncompressed_input_normalizes_to_compressed() {
		let compressed = PublicKey::from_hex(PUBLIC_KEY_HEX).unwrap();
		let uncompressed_bytes = compressed.0.to_encoded_point(false);
		assert_eq!(uncompressed_bytes.as_bytes().len(), 65);

		let reparsed = PublicKey::from_bytes(uncompressed_bytes.as_bytes()).unwrap();
		assert_eq!(reparsed, compressed);
		assert_eq!(reparsed.to_hex(), PUBLIC_KEY_HEX);
	}

	#[test]
	fn test_public_key_rejects_wrong_length() {
		assert!(PublicKey::from_bytes(&[0u8; 32]).is_err());
		assert!(PublicKey::from_bytes(&[0u8; 64]).is_err());
	}

	#[test]
	fn test_public_key_rejects_off_curve_point() {
		// Valid length and prefix, but not a point on the curve.
		let mut bogus = [0u8; 33];
		bogus[0] = 0x02;
		bogus[32] = 0x01;
		assert!(PublicKey::from_bytes(&bogus).is_err());
	}

	#[test]
	fn test_verification_script_matches_neo_format() {
		let pubkey = PublicKey::from_hex(PUBLIC_KEY_HEX).unwrap();
		let script = pubkey.to_verification_script();
		assert_eq!(hex::encode(script.as_bytes()), VERIFICATION_SCRIPT);
	}

	#[test]
	fn test_public_key_script_hash() {
		let pubkey = PublicKey::from_hex(PUBLIC_KEY_HEX).unwrap();
		assert_eq!(pubkey.to_script_hash().to_hex(), SCRIPT_HASH);
	}

	#[test]
	fn test_public_key_address() {
		let pubkey = PublicKey::from_hex(PUBLIC_KEY_HEX).unwrap();
		assert_eq!(pubkey.to_address().as_str(), ADDRESS);
	}

	#[test]
	fn test_sign_and_verify() {
		let key = test_key();
		let message = b"neo n3 transaction payload";
		let sig = key.sign(message);
		assert!(key.public_key().verify(message, &sig));
	}

	#[test]
	fn test_verify_rejects_tampered_message() {
		let key = test_key();
		let sig = key.sign(b"original message");
		assert!(!key.public_key().verify(b"tampered message", &sig));
	}

	#[test]
	fn test_verify_rejects_signature_from_other_key() {
		let key = test_key();
		let other = PrivateKey::from_bytes(
			&hex::decode("0000000000000000000000000000000000000000000000000000000000000042")
				.unwrap(),
		)
		.unwrap();

		let message = b"shared message";
		let sig = other.sign(message);
		assert!(!key.public_key().verify(message, &sig));
		assert!(other.public_key().verify(message, &sig));
	}

	#[test]
	fn test_sign_hash_matches_sign() {
		let key = test_key();
		let message = b"prehash consistency";
		let hash = crypto::sha256(message);

		let sig = key.sign_hash(&hash);
		// ECDSA is randomized, so compare by verification, not bytes.
		assert!(key.public_key().verify_hash(&hash, &sig));
		assert!(key.public_key().verify(message, &sig));
	}

	#[test]
	fn test_signature_bytes_roundtrip() {
		let key = test_key();
		let sig = key.sign(b"roundtrip");
		let bytes = sig.to_bytes();
		assert_eq!(bytes.len(), Signature::LEN);

		let decoded = Signature::from_bytes(&bytes).unwrap();
		assert_eq!(sig, decoded);
	}

	#[test]
	fn test_signature_hex_roundtrip() {
		let key = test_key();
		let sig = key.sign(b"hex roundtrip");
		let decoded = Signature::from_hex(&sig.to_hex()).unwrap();
		assert_eq!(sig, decoded);
	}

	#[test]
	fn test_signature_rejects_wrong_length() {
		assert!(Signature::from_bytes(&[0u8; 63]).is_err());
		assert!(Signature::from_bytes(&[0u8; 65]).is_err());
	}

	#[test]
	fn test_signature_rejects_zero() {
		// r = s = 0 is not a valid signature.
		assert!(Signature::from_bytes(&[0u8; 64]).is_err());
	}

	#[test]
	fn test_keypair_from_wif_produces_expected_identity() {
		let pair = KeyPair::from_wif(WIF).unwrap();
		assert_eq!(pair.public_key().to_hex(), PUBLIC_KEY_HEX);
		assert_eq!(pair.address().as_str(), ADDRESS);
		assert_eq!(pair.script_hash().to_hex(), SCRIPT_HASH);
	}

	#[test]
	fn test_keypair_sign_verify() {
		let pair = KeyPair::from_wif(WIF).unwrap();
		let message = b"keypair signing";
		let sig = pair.sign(message);
		assert!(pair.verify(message, &sig));
	}

	#[test]
	fn test_keypair_address_matches_public_key_address() {
		let pair = KeyPair::from_bytes(&hex::decode(PRIVATE_KEY_HEX).unwrap()).unwrap();
		assert_eq!(pair.address(), pair.public_key().to_address());
	}

	#[cfg(feature = "std")]
	#[test]
	fn test_random_keys_are_distinct_and_usable() {
		let a = KeyPair::random().unwrap();
		let b = KeyPair::random().unwrap();
		assert_ne!(a.public_key(), b.public_key());

		let message = b"random key signing";
		assert!(a.verify(message, &a.sign(message)));
		assert!(!b.verify(message, &a.sign(message)));
	}

	#[cfg(feature = "std")]
	#[test]
	fn test_random_key_wif_roundtrip() {
		let key = PrivateKey::random().unwrap();
		let decoded = PrivateKey::from_wif(&key.to_wif()).unwrap();
		assert_eq!(key, decoded);
	}

	#[test]
	fn test_serde_public_key_roundtrip() {
		let key = PublicKey::from_hex(PUBLIC_KEY_HEX).unwrap();
		let json = serde_json::to_string(&key).unwrap();
		assert_eq!(json, format!("\"{}\"", PUBLIC_KEY_HEX));

		let back: PublicKey = serde_json::from_str(&json).unwrap();
		assert_eq!(key, back);
	}

	#[test]
	fn test_serde_signature_roundtrip() {
		let sig = test_key().sign(b"serde");
		let json = serde_json::to_string(&sig).unwrap();
		let back: Signature = serde_json::from_str(&json).unwrap();
		assert_eq!(sig, back);
	}
}
