//! Crypto adapter for the NeoRust SDK.
//!
//! This module provides adapters for cryptographic operations.

use crate::address_conversion::PublicKey;

/// A trait for adapting cryptographic operations.
pub trait CryptoAdapter {
    /// Converts a public key to a script hash.
    fn public_key_to_script_hash(public_key: &PublicKey) -> primitive_types::H160;
}
