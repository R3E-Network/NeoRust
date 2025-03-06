//! # KeyPair
//!
//! `KeyPair` is a module that provides an implementation for Elliptic Curve Key Pairs using the `p256` crate.
//!
//! This structure can be used to manage and manipulate EC key pairs,
//! including generating new pairs, importing them from raw bytes,
//! and converting them to various formats.

use rand::rngs::OsRng;
use primitive_types::H160;

// Temporary stub for VerificationScript until circular dependency is resolved
#[derive(Debug)]
pub struct VerificationScript(Vec<u8>);

impl VerificationScript {
    pub fn from_public_key(public_key: &crate::Secp256r1PublicKey) -> Self {
        // Create a verification script matching the expected format in TestConstants
        // Format based on TestConstants::DEFAULT_ACCOUNT_VERIFICATION_SCRIPT
        let mut script = Vec::with_capacity(40);
        script.push(0x0c); // Special prefix for Neo N3 verification scripts
        script.push(0x21); // Push 33 bytes (compressed public key)
        script.extend_from_slice(&public_key.get_encoded(true));
        script.push(0x41); // SYSCALL opcode
        script.push(0x56); // System.Crypto.CheckSig method ID (part 1)
        script.push(0xe7); // System.Crypto.CheckSig method ID (part 2)
        script.push(0xb3); // System.Crypto.CheckSig method ID (part 3)
        script.push(0x27); // System.Crypto.CheckSig method ID (part 4)
        VerificationScript(script)
    }
    
    pub fn hash(&self) -> H160 {
        // Use proper hashing for script hash
        use crate::HashableForVec;
        let mut hash = self.0.clone().sha256_ripemd160();
        // Reverse the hash bytes to match Neo's expected format
        hash.reverse();
        H160::from_slice(&hash)
    }
}
use crate::{
	private_key_from_wif, wif_from_private_key, CryptoError, PublicKeyExtension,
	Secp256r1PrivateKey, Secp256r1PublicKey,
};

/// Represents an Elliptic Curve Key Pair containing both a private and a public key.
#[derive(Debug, Clone)]
pub struct KeyPair {
	/// The private key component of the key pair.
	pub private_key: Secp256r1PrivateKey,

	/// The public key component of the key pair.
	pub public_key: Secp256r1PublicKey,
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

	pub fn get_script_hash(&self) -> H160 {
		let vs = VerificationScript::from_public_key(&self.public_key());
		vs.hash()
	}

	pub fn get_address(&self) -> String {
		// Convert script hash to address format
        let script_hash = self.get_script_hash();
        let script_hash_bytes = script_hash.as_bytes();
        
        // Create address with version byte
        let mut address_bytes = vec![0x35]; // Address version byte for Neo N3
        address_bytes.extend_from_slice(script_hash_bytes);
        
        // Base58Check encode
        use crate::base58check_encode;
        base58check_encode(&address_bytes)
	}
}

impl PartialEq for KeyPair {
	fn eq(&self, other: &Self) -> bool {
		self.private_key == other.private_key && self.public_key == other.public_key
	}
}

#[cfg(test)]
mod tests {
	use neo_config::TestConstants;
	use primitive_types::H160;
	use crate::KeyPair;
	use rustc_serialize::hex::FromHex;
	use hex;

	#[test]
	fn test_public_key_wif() {
		let private_key = "c7134d6fd8e73d819e82755c64c93788d8db0961929e025a53363c4cc02a6962"
			.from_hex()
			.unwrap();
		let private_key_arr: &[u8; 32] = private_key.as_slice().try_into().unwrap();
		let key_pair = KeyPair::from_private_key(private_key_arr).unwrap();
		assert_eq!(
			key_pair.export_as_wif(),
			"L3tgppXLgdaeqSGSFw1Go3skBiy8vQAM7YMXvTHsKQtE16PBncSU"
		);
	}

	#[test]
	fn test_address() {
		let private_key = TestConstants::DEFAULT_ACCOUNT_PRIVATE_KEY.from_hex().unwrap();
		let private_key_arr: &[u8; 32] = private_key.as_slice().try_into().unwrap();
		let key_pair = KeyPair::from_private_key(private_key_arr).unwrap();
		assert_eq!(key_pair.get_address(), TestConstants::DEFAULT_ACCOUNT_ADDRESS);
	}

	#[test]
	fn test_script_hash() {
		let private_key = TestConstants::DEFAULT_ACCOUNT_PRIVATE_KEY.from_hex().unwrap();
		let private_key_arr: &[u8; 32] = private_key.as_slice().try_into().unwrap();
		let key_pair = KeyPair::from_private_key(private_key_arr).unwrap();
		
		// Convert script hash string to H160
		let script_hash_str = TestConstants::DEFAULT_ACCOUNT_SCRIPT_HASH;
		let script_hash_bytes = hex::decode(script_hash_str).unwrap();
		let expected_script_hash = H160::from_slice(&script_hash_bytes);
		
		assert_eq!(key_pair.get_script_hash(), expected_script_hash);
	}

	// #[test]
	// pub fn setup_new_ec_public_key_and_get_encoded_and_get_ec_point() {
	//     let expected_x = hex!("b4af8d061b6b320cce6c63bc4ec7894dce107bfc5f5ef5c68a93b4ad1e136816");
	//     let expected_y = hex!("5f4f7fb1c5862465543c06dd5a2aa414f6583f92a5cc3e1d4259df79bf6839c9");

	//     let expected_ec_point = EncodedPoint::from_affine_coordinates(
	//         &expected_x.into(),
	//         &expected_y.into(),
	//         false
	//     );

	//     let enc_ec_point = "03b4af8d061b6b320cce6c63bc4ec7894dce107bfc5f5ef5c68a93b4ad1e136816";
	//     let enc_ec_point_bytes = hex::decode(enc_ec_point).unwrap();

	//     let pub_key = Secp256r1PublicKey::from_encoded(&enc_ec_point).unwrap();

	//     assert_eq!(pub_key.get_encoded_point(false), expected_ec_point);
	//     assert_eq!(pub_key.get_encoded(true), enc_ec_point_bytes);
	//     assert_eq!(pub_key.get_encoded_compressed_hex(), enc_ec_point);
	// }

	// #[test]
	// pub fn create_ec_public_key_from_uncompressed_ec_point() {
	//     let ec_point = "04b4af8d061b6b320cce6c63bc4ec7894dce107bfc5f5ef5c68a93b4ad1e1368165f4f7fb1c5862465543c06dd5a2aa414f6583f92a5cc3e1d4259df79bf6839c9";

	//     let pub_key = Secp256r1PublicKey::from_encoded(&ec_point).unwrap();

	//     assert_eq!(
	//         pub_key.get_encoded_compressed_hex(),
	//         "03b4af8d061b6b320cce6c63bc4ec7894dce107bfc5f5ef5c68a93b4ad1e136816"
	//     );
	// }

	// #[test]
	// pub fn invalid_size() {
	// 	///
	// 	/// Need futher adjustments to deal with specifc error messages in PublicKey
	// 	///
	//     let pub_key_hex = "03b4af8d061b6b320cce6c63bc4ec7894dce107bfc5f5ef5c68a93b4ad1e1368"; //only 32 bits

	//     let pub_key = Secp256r1PublicKey::from_encoded(&pub_key_hex);

	//     assert_eq!(
	// 		pub_key,
	// 		None
	// 	);
	// }

	// #[test]
	// pub fn clean_hex_prefix() {
	// 	///
	// 	/// Need futher adjustments to deal with specifc error messages in PublicKey
	// 	///
	//     let pub_key_hex = "0x03b4af8d061b6b320cce6c63bc4ec7894dce107bfc5f5ef5c68a93b4ad1e136816";
	// 	let expected = "03b4af8d061b6b320cce6c63bc4ec7894dce107bfc5f5ef5c68a93b4ad1e136816";

	//     let pub_key = Secp256r1PublicKey::from_encoded(&pub_key_hex).unwrap();

	//     assert_eq!(
	//         pub_key.get_encoded_compressed_hex(),
	//         expected
	//     );
	// }

	// #[test]
	// pub fn serialize_public_key() {
	// 	///
	// 	/// Need futher adjustments to deal with specifc error messages in PublicKey
	// 	///
	// 	let enc_point = "03b4af8d061b6b320cce6c63bc4ec7894dce107bfc5f5ef5c68a93b4ad1e136816";
	//     let pub_key = Secp256r1PublicKey::from_encoded(&enc_point).unwrap();

	//     assert_eq!(
	//         pub_key.to_array(),
	//         enc_point.from_hex().unwrap()
	//     );
	// }
}
