//! Neo primitives - Core types for Neo N3 blockchain
//!
//! This crate provides low-level, `no_std`-compatible primitives:
//! - Hash types (Hash160, Hash256)
//! - Address encoding/decoding
//! - Script types
//! - Cryptographic utilities
//!
//! # Features
//! - `std` (default): Enable standard library support and random key generation
//!
//! # Examples
//!
//! ```
//! use neo_primitives::{Address, Hash160};
//! use core::str::FromStr;
//!
//! let addr = Address::from_str("NM7Aky765FG8NhhwtxjXRx7jEL1cnw7PBP").unwrap();
//! let hash = addr.to_script_hash().unwrap();
//! println!("Script hash: {}", hash);
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

pub mod crypto;
pub mod errors;
pub mod types;

pub use crypto::{KeyPair, PrivateKey, PublicKey, Signature};
pub use errors::{Error, Result};
pub use types::{Address, Hash160, Hash256, Script};
