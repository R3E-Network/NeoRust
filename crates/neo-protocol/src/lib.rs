//! # Neo Protocol
//!
//! Neo blockchain protocol implementation for the NeoRust SDK.
//!
//! This crate provides core protocol functionality for interacting with the Neo N3 blockchain, including:
//!
//! - Account management and wallet functionality
//! - Transaction construction and signing
//! - Smart contract interaction
//! - Network protocol implementations
//! - Response types for RPC calls
//! - NEP-2 password-protected key format support
//!
//! ## Usage
//!
//! ```rust,ignore
//! use neo_protocol::{Account, AccountTrait};
//! use neo_types::ScriptHash;
//! use std::str::FromStr;
//!
//! // Create an account from a WIF or address
//! let account = Account::from_wif("KwkUAF4y4UQwQGY8RkRtddHX8FgDgpwdH2RYKQcnAi7fFkzYQUV3").unwrap();
//! let address = account.address();
//!
//! // Get account information
//! let script_hash = account.script_hash();
//! let public_key = account.public_key();
//!
//! // Sign data with the account
//! let signature = account.sign(b"message to sign").unwrap();
//! let is_valid = account.verify(b"message to sign", &signature).unwrap();
//! ```

#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms, unreachable_pub)]
#![deny(rustdoc::broken_intra_doc_links)]

mod account;
mod nep2;
mod protocol_error;
mod responses;
mod role;

// Re-export all public items
pub use account::*;
pub use nep2::*;
pub use protocol_error::*;
pub use responses::*;
// Re-export role module
pub mod role;

// Conditionally re-export client functionality
// Temporarily comment out to avoid circular dependency
// #[cfg(feature = "clients")]
// pub use neo_clients::public_key_to_address;
