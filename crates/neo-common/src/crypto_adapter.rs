//! Crypto adapter for the NeoRust SDK.
//!
//! This module provides adapters for cryptographic operations.

use crate::address_conversion::PublicKey;

/// A trait for adapting cryptographic operations.
pub trait CryptoAdapter {
    /// Converts a public key to a script hash.
    fn public_key_to_script_hash(public_key: &PublicKey) -> primitive_types::H160;
}

/// A trait for public keys that can be encoded
pub trait EncodablePublicKey {
    /// Get the encoded public key
    fn get_encoded(&self, compressed: bool) -> Vec<u8>;
}

/// Convert an external public key type to a common PublicKey
pub fn external_to_common_public_key<T: EncodablePublicKey>(public_key: &T) -> PublicKey {
    let bytes = public_key.get_encoded(true);
    PublicKey::new(bytes)
}

/// Convert a common PublicKey to an external public key type
pub fn common_to_external_public_key<T, E, F>(
    public_key: &PublicKey,
    converter: F,
) -> Result<T, E>
where
    F: FnOnce(&[u8]) -> Result<T, E>,
{
    converter(&public_key.bytes)
}
