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
mod error;
mod hash;
mod hashable;
mod key_pair;
mod keys;
mod utils;
mod wif;

// Re-export all public items
pub use base58_helper::*;
pub use error::*;
// Re-export specific items to avoid ambiguity
pub use hash::HashableForString;
pub use hash::HashableForVec as HashVec;
pub use hashable::HashableForBytes;
pub use hashable::HashableForVec;
pub use hashable::HashableForVecToVec;
pub use key_pair::*;
pub use keys::*;
pub use utils::*;
pub use wif::*;
