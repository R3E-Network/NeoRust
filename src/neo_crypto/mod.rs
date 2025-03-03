//! # Neo Crypto
//!
//! Cryptographic utilities for the Neo N3 blockchain.
//!
//! ## Overview
//!
//! The neo_crypto module provides cryptographic primitives and utilities for working with
//! the Neo N3 blockchain. It includes:
//!
//! - Key pair generation and management
//! - Cryptographic signing and verification
//! - Hashing functions (SHA256, RIPEMD160, etc.)
//! - Base58 encoding and decoding
//! - WIF (Wallet Import Format) utilities
//! - Secure random number generation
//!
//! This module forms the cryptographic foundation for wallet management, transaction signing,
//! and secure communication within the Neo N3 ecosystem.
//!
//! ## Examples
//!
//! ### Creating a key pair
//!
//! ```rust
//! use neo::prelude::*;
//!
//! // Generate a new random key pair
//! let key_pair = KeyPair::new_random().unwrap();
//! println!("Public key: {}", key_pair.public_key());
//! println!("Private key: {}", key_pair.private_key());
//!
//! // Create a key pair from a private key
//! let private_key = PrivateKey::from_slice(&[/* 32 bytes */]).unwrap();
//! let key_pair = KeyPair::from_private_key(&private_key).unwrap();
//! ```
//!
//! ### Signing and verifying messages
//!
//! ```rust
//! use neo::prelude::*;
//!
//! // Generate a key pair
//! let key_pair = KeyPair::new_random().unwrap();
//!
//! // Sign a message
//! let message = b"Hello, Neo!";
//! let signature = key_pair.sign(message).unwrap();
//!
//! // Verify the signature
//! let is_valid = key_pair.verify(message, &signature).unwrap();
//! assert!(is_valid);
//! ```
//!
//! ### Working with WIF format
//!
//! ```rust
//! use neo::prelude::*;
//!
//! // Convert a private key to WIF format
//! let private_key = PrivateKey::from_slice(&[/* 32 bytes */]).unwrap();
//! let wif = private_key_to_wif(&private_key).unwrap();
//!
//! // Convert WIF back to a private key
//! let recovered_key = wif_to_private_key(&wif).unwrap();
//! assert_eq!(private_key, recovered_key);
//! ```

pub use base58_helper::*;
pub use error::*;
pub use key_pair::*;
pub use keys::*;
pub use utils::*;
pub use wif::*;

mod base58_helper;
mod error;
pub mod hash;
mod key_pair;
mod keys;
mod utils;
mod wif;

// Re-export important types
pub use hash::HashableForVec;
pub use key_pair::KeyPair;
pub use error::CryptoError;
pub use keys::Secp256r1PublicKey;
pub use keys::Secp256r1Signature;

pub(crate) fn add(left: usize, right: usize) -> usize {
	left + right
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn it_works() {
		let result = add(2, 2);
		assert_eq!(result, 4);
	}
}
