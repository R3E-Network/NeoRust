//! # neo-primitives
//!
//! Core primitive types for Neo N3 blockchain.
//!
//! This crate is `no_std` compatible and provides the fundamental building blocks
//! for Neo blockchain development:
//!
//! - Addresses and hashes
//! - Cryptographic keys and signatures
//! - Script building
//! - Error types
//!
//! ## Features
//!
//! - `std` (default): Enable standard library support
//!
//! ## Example
//!
//! ```rust
//! use neo_primitives::Address;
//!
//! let address = Address::from_string("NbTiM6h8r99kpRtb428XcsUk1TzKed2gTc").unwrap();
//! println!("Address: {}", address);
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs, rust_2018_idioms)]

extern crate alloc;

pub mod types;
pub mod crypto;
pub mod errors;

// Re-exports
pub use types::{Address, Hash160, Hash256, Script, UInt256};
pub use crypto::{PrivateKey, PublicKey, Signature};
pub use errors::{Error, Result};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::types::*;
    pub use crate::crypto::*;
    pub use crate::errors::*;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitives_basic() {
        // Basic smoke test
        assert!(true);
    }
}
