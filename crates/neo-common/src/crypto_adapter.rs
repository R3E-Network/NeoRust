//! Adapter utilities for crypto types
//!
//! This module provides adapter functions to convert between different crypto types
//! to help break circular dependencies between crates.

use crate::PublicKey;

/// Trait for public keys that can be encoded
pub trait EncodablePublicKey {
    /// Get the encoded representation of the public key
    fn get_encoded(&self, compressed: bool) -> Vec<u8>;
}

/// Convert a public key with get_encoded method to a neo-common PublicKey
pub fn external_to_common_public_key<T>(public_key: &T) -> PublicKey 
where 
    T: EncodablePublicKey,
{
    PublicKey::new(public_key.get_encoded(true))
}

/// Convert a neo-common PublicKey to an external public key type
/// This function should be implemented by the consumer crate
pub fn common_to_external_public_key<T, E>(public_key: &PublicKey, converter: fn(&[u8]) -> Result<T, E>) -> Result<T, E> {
    converter(&public_key.bytes)
}
