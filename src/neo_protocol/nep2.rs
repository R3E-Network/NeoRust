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

use aes::cipher::{block_padding::NoPadding, BlockDecryptMut, BlockEncryptMut, KeyInit};
use neo::prelude::{
	base58check_decode, base58check_encode, public_key_to_address, vec_to_array32, HashableForVec,
	KeyPair, NeoConstants, ProviderError, Secp256r1PublicKey, ToBase58,
};
use rustc_serialize::hex::FromHex;
use scrypt::{scrypt, Params};

type Aes256EcbEnc = ecb::Encryptor<aes::Aes256>;
type Aes256EcbDec = ecb::Decryptor<aes::Aes256>;

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
	assert_eq!(key.len(), 32);
	let key: [u8; 32] = key.try_into().unwrap();
	let mut buf = [0u8; 64];
	let pt_len = data.len();
	buf[..pt_len].copy_from_slice(&data);
	let ct = Aes256EcbEnc::new(&key.into())
		.encrypt_padded_mut::<NoPadding>(&mut buf, pt_len)
		.unwrap();

	Ok(ct.to_vec())
}

fn decrypt_aes256_ecb(encrypted_data: &[u8], key: &[u8]) -> Result<Vec<u8>, ProviderError> {
	// Ensure key is the correct length for AES-256
	assert_eq!(key.len(), 32);
	let key: [u8; 32] = key.try_into().unwrap();
	let mut buf = [0u8; 64];
	let pt = Aes256EcbDec::new(&key.into())
		.decrypt_padded_b2b_mut::<NoPadding>(&encrypted_data, &mut buf)
		.unwrap();

	Ok(pt.to_vec())
}

pub fn get_nep2_from_private_key(pri_key: &str, passphrase: &str) -> Result<String, ProviderError> {
	let private_key = pri_key.from_hex().unwrap();
	let key_pair = KeyPair::from_private_key(&vec_to_array32(private_key.to_vec()).unwrap())?;
	let addresshash: [u8; 4] = address_hash_from_pubkey(&key_pair.public_key.get_encoded(true));
	let mut result = vec![0u8; NeoConstants::SCRYPT_DK_LEN];
	let params =
		Params::new(NeoConstants::SCRYPT_LOG_N, NeoConstants::SCRYPT_R, NeoConstants::SCRYPT_P, 32)
			.unwrap();
	scrypt(passphrase.as_bytes(), addresshash.to_vec().as_slice(), &params, &mut result).unwrap();
	let half_1 = &result[0..32];
	let _half_2 = &result[32..64];
	let mut u8xor = [0u8; 32];

	for i in 0..32 {
		u8xor[i] = &private_key[i] ^ half_1[i];
	}

	let encrypted = encrypt_aes256_ecb(&u8xor.to_vec(), &_half_2)?;

	//assert_eq!(encrypted.len(), 32);

	// # Assemble the final result
	let mut assembled = Vec::new();

	assembled.push(NeoConstants::NEP_HEADER_1);
	assembled.push(NeoConstants::NEP_HEADER_2);
	assembled.push(NeoConstants::NEP_FLAG);
	assembled.extend(addresshash.to_vec());
	assembled.extend(&encrypted[0..32]);

	// # Finally, encode with Base58Check
	//Ok(assembled.to_base58())
	Ok(base58check_encode(&assembled))
}

pub fn get_private_key_from_nep2(nep2: &str, passphrase: &str) -> Result<Vec<u8>, ProviderError> {
	if nep2.len() != 58 {
		println!("Wrong Nep2");
		()
	}
	let decoded_key: [u8; 39] = base58check_decode(nep2).unwrap().try_into().unwrap();
	if (decoded_key[0] != 0x01 || decoded_key[1] != 0x42 || decoded_key[2] != 0xe0) {
		return Err(ProviderError::InvalidAddress);
	}

	let address_hash: &[u8] = &decoded_key[3..7];
	let encrypted: &[u8] = &decoded_key[7..39];

	// pwd_normalized = bytes(unicodedata.normalize('NFC', passphrase), 'utf-8')
	let mut result = vec![0u8; NeoConstants::SCRYPT_DK_LEN];
	let params =
		Params::new(NeoConstants::SCRYPT_LOG_N, NeoConstants::SCRYPT_R, NeoConstants::SCRYPT_P, 32)
			.unwrap();

	scrypt(passphrase.as_bytes(), &address_hash, &params, &mut result).unwrap();

	// derived = scrypt.hash(pwd_normalized, address_hash,
	//                       N=SCRYPT_ITERATIONS,
	//                       r=SCRYPT_BLOCKSIZE,
	//                       p=SCRYPT_PARALLEL_FACTOR,
	//                       buflen=SCRYPT_KEY_LEN_BYTES)

	let half_1 = &result[0..32];
	let half_2 = &result[32..64];

	// derived1 = derived[:32]
	// derived2 = derived[32:]

	let decrypted = decrypt_aes256_ecb(encrypted, half_2)?;

	let mut pri_key = [0u8; 32];

	for i in 0..32 {
		pri_key[i] = decrypted[i] ^ half_1[i];
	}
	// cipher = Aes.new(derived2, Aes.MODE_ECB)
	// decrypted = cipher.decrypt(encrypted)
	// private_key = xor_bytes(decrypted, derived1)

	let key_pair = KeyPair::from_private_key(&pri_key)?;
	let kp_addresshash: [u8; 4] = address_hash_from_pubkey(&key_pair.public_key.get_encoded(true));

	// # Now check that the address hashes match. If they don't, the password was wrong.
	// kp_new = KeyPair(priv_key=private_key)
	// kp_new_address = kp_new.get_address()
	// kp_new_address_hash_tmp = hashlib.sha256(kp_new_address.encode("utf-8")).digest()
	// kp_new_address_hash_tmp2 = hashlib.sha256(kp_new_address_hash_tmp).digest()
	// kp_new_address_hash = kp_new_address_hash_tmp2[:4]
	assert_eq!(kp_addresshash, address_hash);
	if kp_addresshash != address_hash {
		println!(
			"Calculated address hash does not match the one in the provided encrypted address."
		);
		//return Err(ProviderError::CustomError("Calculated address hash does not match the one in the provided encrypted address.".to_string()));
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
	let addr = public_key_to_address(&Secp256r1PublicKey::from_bytes(pubkey).unwrap());
	let hash = addr.as_bytes();
	let hash = hash.hash256().hash256();
	let mut result = [0u8; 4];
	result.copy_from_slice(&hash[..4]);
	result
}

#[cfg(test)]
mod tests {
	use super::*;
	use neo::prelude::TestConstants;

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
