//! Crypto utilities for working with public keys and signatures
//!
//! This module provides utilities for working with cryptographic types
//! from the neo-crypto crate, helping to break circular dependencies.

use primitive_types::H160;
use sha2::{Digest, Sha256};
use ripemd::Ripemd160;

/// Convert a public key to a script hash
///
/// This function takes any type that can be converted to a byte slice
/// and converts it to a script hash.
pub fn public_key_to_script_hash(public_key_bytes: &[u8]) -> H160 {
    // Hash the public key with SHA-256
    let mut hasher = Sha256::new();
    hasher.update(public_key_bytes);
    let sha256_result = hasher.finalize();
    
    // Hash the result with RIPEMD-160
    let mut hasher = Ripemd160::new();
    hasher.update(sha256_result);
    let ripemd_result = hasher.finalize();
    
    // Convert to H160
    H160::from_slice(&ripemd_result)
}

/// Wrapper for public key bytes that can be used with functions requiring AsRef<[u8]>
#[derive(Debug, Clone)]
pub struct PublicKeyBytes(pub Vec<u8>);

impl AsRef<[u8]> for PublicKeyBytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<Vec<u8>> for PublicKeyBytes {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

/// Convert a public key to a script hash using the PublicKeyBytes wrapper
pub fn secp256r1_public_key_to_script_hash(public_key_bytes: &[u8]) -> H160 {
    public_key_to_script_hash(public_key_bytes)
}
