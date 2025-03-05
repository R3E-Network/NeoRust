//! # NEO NEP2 (Neo Extended Protocol 2) Module
//!
//! This module implements the NEP2 standard for encrypting and decrypting NEO blockchain private keys.
//! NEP2 specifies a method for securing private keys with a passphrase, making it safer to store
//! and manage private keys, especially in wallet applications.
//!
//! ## Features
//!
//! - Encrypt private keys using a password to produce a NEP2-formatted string.
//! - Decrypt NEP2 strings back into private keys using the correct password.
//! - Integration with AES encryption and scrypt key derivation for robust security.
//!
//! ## Usage
//!
//! - Encrypt a private key to a NEP2 string:
//!   - Use `NEP2::encrypt` with a password and a `KeyPair` containing the private key.
//!
//! - Decrypt a NEP2 string to obtain the private key:
//!   - Use `NEP2::decrypt` with the password and the NEP2 string.
//!
//! ## Examples
//!
//! ```
//! use p256::elliptic_curve::rand_core::OsRng;
//! use scrypt::Params;
//! use NeoRust::prelude::{KeyPair, NEP2, Secp256r1PrivateKey};
//!
//! // To encrypt a private key:
//! let key_pair = KeyPair::from_secret_key(&Secp256r1PrivateKey::random(&mut OsRng));
//! let encrypted = NEP2::encrypt("your-password", &key_pair, Params::new(14, 8, 8, 32).unwrap()).expect("Encryption failed");
//!
//! // To decrypt a NEP2 string:
//! let decrypted_key_pair = NEP2::decrypt("your-password", &encrypted, Params::new(14, 8, 8, 32).unwrap()).expect("Decryption failed");
//! ```
//!
//! ## Testing
//!
//! The module includes tests to verify the correctness of the encryption and decryption functionalities,
//! ensuring that they comply with the NEP2 standard.
//!
//! ## Error Handling
//!
//! Proper error handling is implemented to deal with common issues like incorrect password, invalid NEP2 format,
//! and other cryptographic errors.

use neo_config::NeoConstants;
use neo_crypto::{base58check_decode, base58check_encode, HashableForVec, KeyPair, Secp256r1PublicKey};
// Temporarily comment out to avoid circular dependency
// use neo_clients::public_key_to_address;
use neo_error::ProviderError;
use neo_types::vec_to_array32;
use block_modes::{BlockMode, Ecb};
use aes::Aes256;
use block_modes::block_padding::NoPadding;
use rustc_serialize::hex::FromHex;
use scrypt::{scrypt, Params};

type Aes256EcbEnc = Ecb<Aes256, NoPadding>;
type Aes256EcbDec = Ecb<Aes256, NoPadding>;

pub struct NEP2;

impl NEP2 {
	const DKLEN: usize = 64;
	const NEP2_PRIVATE_KEY_LENGTH: usize = 39;
	const NEP2_PREFIX_1: u8 = 0x01;
	const NEP2_PREFIX_2: u8 = 0x42;
	const NEP2_FLAGBYTE: u8 = 0xE0;
}

fn encrypt_aes256_ecb(data: &[u8], key: &[u8]) -> Result<Vec<u8>, ProviderError> {
	// Ensure key is the correct length for AES-256
	if key.len() != 32 {
		return Err(ProviderError::CustomError("AES-256 key must be 32 bytes".to_string()));
	}

	let key: [u8; 32] = key.try_into().map_err(|_| {
		ProviderError::CustomError("Failed to convert key to 32-byte array".to_string())
	})?;

	let cipher = Aes256EcbEnc::new_from_slice(&key)
		.map_err(|_| ProviderError::CustomError("Failed to create AES cipher".to_string()))?;
	
	// Create a buffer with the data
	let mut buf = data.to_vec();
	// Ensure the buffer length is a multiple of the block size
	let padding_needed = (16 - (buf.len() % 16)) % 16;
	buf.extend(vec![0u8; padding_needed]);
	
	// Encrypt the data in place
	cipher.encrypt_blocks(&mut buf.chunks_exact_mut(16).collect::<Vec<_>>());

	Ok(buf)
}

fn decrypt_aes256_ecb(encrypted_data: &[u8], key: &[u8]) -> Result<Vec<u8>, ProviderError> {
	// Ensure key is the correct length for AES-256
	if key.len() != 32 {
		return Err(ProviderError::CustomError("AES-256 key must be 32 bytes".to_string()));
	}

	// Ensure encrypted data length is a multiple of the block size
	if encrypted_data.len() % 16 != 0 {
		return Err(ProviderError::CustomError("Encrypted data length must be a multiple of 16 bytes".to_string()));
	}

	let key: [u8; 32] = key.try_into().map_err(|_| {
		ProviderError::CustomError("Failed to convert key to 32-byte array".to_string())
	})?;

	let cipher = Aes256EcbDec::new_from_slice(&key)
		.map_err(|_| ProviderError::CustomError("Failed to create AES cipher".to_string()))?;
	
	// Create a buffer with the encrypted data
	let mut buf = encrypted_data.to_vec();
	
	// Decrypt the data in place
	cipher.decrypt_blocks(&mut buf.chunks_exact_mut(16).collect::<Vec<_>>());

	// Remove padding if needed (in this case, we know the exact size we need)
	Ok(buf)
}

pub fn get_nep2_from_private_key(pri_key: &str, passphrase: &str) -> Result<String, ProviderError> {
	let private_key = pri_key
		.from_hex()
		.map_err(|_| ProviderError::CustomError("Invalid hex in private key".to_string()))?;

	let key_pair =
		KeyPair::from_private_key(&vec_to_array32(private_key.to_vec()).map_err(|_| {
			ProviderError::CustomError("Failed to convert private key to 32-byte array".to_string())
		})?)?;

	let addresshash: [u8; 4] = address_hash_from_pubkey(&key_pair.public_key.get_encoded(true));
	let mut result = vec![0u8; NeoConstants::SCRYPT_DK_LEN];

	let params =
		Params::new(NeoConstants::SCRYPT_LOG_N, NeoConstants::SCRYPT_R, NeoConstants::SCRYPT_P, 32)
			.map_err(|e| {
				ProviderError::CustomError(format!("Failed to create scrypt parameters: {}", e))
			})?;

	scrypt(passphrase.as_bytes(), addresshash.to_vec().as_slice(), &params, &mut result)
		.map_err(|e| ProviderError::CustomError(format!("Scrypt operation failed: {}", e)))?;

	let half_1 = &result[0..32];
	let _half_2 = &result[32..64];
	let mut u8xor = [0u8; 32];

	for i in 0..32 {
		u8xor[i] = &private_key[i] ^ half_1[i];
	}

	let encrypted = encrypt_aes256_ecb(&u8xor.to_vec(), &_half_2)?;

	// # Assemble the final result
	let mut assembled = Vec::new();

	assembled.push(NeoConstants::NEP_HEADER_1);
	assembled.push(NeoConstants::NEP_HEADER_2);
	assembled.push(NeoConstants::NEP_FLAG);
	assembled.extend(addresshash.to_vec());
	assembled.extend(&encrypted[0..32]);

	// # Finally, encode with Base58Check
	Ok(base58check_encode(&assembled))
}

pub fn get_private_key_from_nep2(nep2: &str, passphrase: &str) -> Result<Vec<u8>, ProviderError> {
	if nep2.len() != 58 {
		return Err(ProviderError::CustomError("Invalid NEP2 format: incorrect length".to_string()));
	}

	let decoded_bytes = base58check_decode(nep2).ok_or(ProviderError::CustomError(
		"Invalid NEP2 format: base58check decoding failed".to_string(),
	))?;

	let decoded_key: [u8; 39] = decoded_bytes.try_into().map_err(|_| {
		ProviderError::CustomError("Invalid NEP2 format: incorrect decoded length".to_string())
	})?;

	if decoded_key[0] != 0x01 || decoded_key[1] != 0x42 || decoded_key[2] != 0xe0 {
		return Err(ProviderError::InvalidAddress);
	}

	let address_hash: &[u8] = &decoded_key[3..7];
	let encrypted: &[u8] = &decoded_key[7..39];

	let mut result = vec![0u8; NeoConstants::SCRYPT_DK_LEN];
	let params =
		Params::new(NeoConstants::SCRYPT_LOG_N, NeoConstants::SCRYPT_R, NeoConstants::SCRYPT_P, 32)
			.map_err(|e| {
				ProviderError::CustomError(format!("Failed to create scrypt parameters: {}", e))
			})?;

	scrypt(passphrase.as_bytes(), &address_hash, &params, &mut result)
		.map_err(|e| ProviderError::CustomError(format!("Scrypt operation failed: {}", e)))?;

	let half_1 = &result[0..32];
	let half_2 = &result[32..64];

	let decrypted = decrypt_aes256_ecb(encrypted, half_2)?;

	let mut pri_key = [0u8; 32];

	for i in 0..32 {
		pri_key[i] = decrypted[i] ^ half_1[i];
	}

	let key_pair = KeyPair::from_private_key(&pri_key)?;
	let kp_addresshash: [u8; 4] = address_hash_from_pubkey(&key_pair.public_key.get_encoded(true));

	if kp_addresshash != address_hash {
		return Err(ProviderError::CustomError(
			"Calculated address hash does not match the one in the provided encrypted address. Incorrect password?".to_string()
		));
	}

	Ok(pri_key.to_vec())
}

/// Computes a hash from a public key and extracts the first 4 bytes.
///
/// # Arguments
///
/// * `pubkey` - The public key.
///
/// Returns the first 4 bytes of the hash.
fn address_hash_from_pubkey(pubkey: &[u8]) -> [u8; 4] {
	// This function is used internally and assumes valid public key input
	// In a production environment, we would handle the potential error case
	let public_key = Secp256r1PublicKey::from_bytes(pubkey)
		.expect("Invalid public key format in address_hash_from_pubkey");

	// Temporarily implement address generation directly to avoid circular dependency
	// Original: let addr = public_key_to_address(&public_key);
	let encoded = public_key.get_encoded(true);
	let hash = encoded.hash256();
	let script_hash = hash.ripemd160();
	
	// Convert script hash to address format
	let mut address_bytes = vec![0x35]; // Address version byte for Neo N3
	address_bytes.extend_from_slice(&script_hash);
	
	// Hash the address bytes
	let hash = address_bytes.hash256().hash256();
	let mut result = [0u8; 4];
	result.copy_from_slice(&hash[..4]);
	result
}

#[cfg(test)]
mod tests {
	use super::*;
	use neo_config::TestConstants;

	#[test]
	fn test_decrypt_with_default_scrypt_params() {
		let decrypted_key_pair = match get_private_key_from_nep2(
			TestConstants::DEFAULT_ACCOUNT_ENCRYPTED_PRIVATE_KEY,
			TestConstants::DEFAULT_ACCOUNT_PASSWORD,
		) {
			Ok(key_pair) => key_pair,
			Err(e) => panic!("{}", e),
		};
		assert_eq!(
			decrypted_key_pair,
			hex::decode(TestConstants::DEFAULT_ACCOUNT_PRIVATE_KEY).unwrap()
		);
	}

	#[test]
	fn test_encrypt_with_default_scrypt_params() {
		let encrypted = get_nep2_from_private_key(
			&TestConstants::DEFAULT_ACCOUNT_PRIVATE_KEY,
			TestConstants::DEFAULT_ACCOUNT_PASSWORD,
		)
		.unwrap();
		assert_eq!(encrypted, TestConstants::DEFAULT_ACCOUNT_ENCRYPTED_PRIVATE_KEY);
	}

	#[test]
	fn test_encrypt_decrypt_aes256_ecb() {
		let key = &[0u8; 32];
		let data = b"Hello, World! We want length 32!";

		let encrypted = encrypt_aes256_ecb(data, key).unwrap();
		let decrypted = decrypt_aes256_ecb(&encrypted, key).unwrap();

		assert_eq!(decrypted, data);
	}
}
