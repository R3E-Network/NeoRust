//! # Neo Crypto
//!
//! Cryptographic utilities for the NeoRust SDK.
//!
//! This crate provides cryptographic functionality for the Neo N3 blockchain, including:
//!
//! - Elliptic curve cryptography (secp256r1)
//! - Key pair generation and management
//! - Digital signatures
//! - Hashing algorithms (SHA256, RIPEMD160, etc.)
//! - Encryption and decryption
//! - Base58 encoding/decoding
//! - Secure random number generation
//!
//! ## Usage
//!
//! ```rust,ignore
//! use neo_crypto::{KeyPair, Secp256r1PrivateKey, HashableForVec};
//! use std::str::FromStr;
//!
//! // Generate a new key pair
//! let private_key = Secp256r1PrivateKey::random();
//! let key_pair = KeyPair::from_private_key(&private_key);
//!
//! // Sign a message
//! let message = b"Hello, Neo!";
//! let signature = key_pair.sign(message);
//!
//! // Verify a signature
//! let is_valid = key_pair.verify(message, &signature);
//! assert!(is_valid);
//!
//! // Hash data
//! let data = b"Data to hash";
//! let hash = data.hash_sha256();
//! ```

#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms, unreachable_pub)]
#![deny(rustdoc::broken_intra_doc_links)]

mod base58_helper;
pub mod crypto_adapter_impl;
mod key_pair;
mod keys;
mod utils;
mod wif;

// Re-export all public items
pub use base58_helper::*;
pub use neo_error::crypto_error::CryptoError;
pub use neo_error::codec_error::CodecError;
pub use neo_error::nep2_error::Nep2Error;
pub use neo_error::sign_error::SignError;
pub use crypto_adapter_impl::*;
pub use key_pair::*;

// Import HashableForVec from neo-common
pub use neo_common::hashable::HashableForVec;

use sha2::Digest;

/// Trait for computing hash values from byte slices
pub trait HashableForBytes {
    /// Computes the SHA256 hash of the data
    fn hash256_bytes(&self) -> [u8; 32];
    
    /// Computes the RIPEMD160 hash of the SHA256 hash of the data
    fn hash160_bytes(&self) -> [u8; 20];
}

impl HashableForBytes for [u8] {
    fn hash256_bytes(&self) -> [u8; 32] {
        let mut hasher = sha2::Sha256::new();
        hasher.update(self);
        let first_hash = hasher.finalize();
        
        let mut hasher = sha2::Sha256::new();
        hasher.update(first_hash);
        let mut result = [0u8; 32];
        result.copy_from_slice(&hasher.finalize());
        result
    }
    
    fn hash160_bytes(&self) -> [u8; 20] {
        let mut hasher = sha2::Sha256::new();
        hasher.update(self);
        let sha256_hash = hasher.finalize();
        
        let mut hasher = ripemd::Ripemd160::new();
        hasher.update(sha256_hash);
        let mut result = [0u8; 20];
        result.copy_from_slice(&hasher.finalize());
        result
    }
}

impl HashableForBytes for [u8; 20] {
    fn hash256_bytes(&self) -> [u8; 32] {
        self.as_slice().hash256_bytes()
    }
    
    fn hash160_bytes(&self) -> [u8; 20] {
        self.as_slice().hash160_bytes()
    }
}

/// Trait for computing hash values from vectors
pub trait HashableForVecToVec {
    /// Computes the SHA256 hash of the data and returns it as a Vec<u8>
    fn hash256_vec(&self) -> Vec<u8>;
    
    /// Computes the RIPEMD160 hash of the SHA256 hash of the data and returns it as a Vec<u8>
    fn hash160_vec(&self) -> Vec<u8>;
}

impl HashableForVecToVec for Vec<u8> {
    fn hash256_vec(&self) -> Vec<u8> {
        self.as_slice().hash256_bytes().to_vec()
    }
    
    fn hash160_vec(&self) -> Vec<u8> {
        self.as_slice().hash160_bytes().to_vec()
    }
}

// Re-export specific types and functions
pub use keys::*;
pub use utils::*;
pub use wif::*;
