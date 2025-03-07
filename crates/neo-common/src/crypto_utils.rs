//! Cryptographic utilities for the NeoRust SDK.
//!
//! This module provides cryptographic utilities for the NeoRust SDK.

use sha2::{Digest, Sha256};
use primitive_types::H160;
use ripemd::Ripemd160;

/// Computes the SHA-256 hash of the input data.
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Computes the double SHA-256 hash of the input data.
pub fn hash256(data: &[u8]) -> [u8; 32] {
    let first_hash = sha256(data);
    sha256(&first_hash)
}

/// Computes the RIPEMD-160 hash of the input data.
pub fn ripemd160(data: &[u8]) -> [u8; 20] {
    let mut hasher = Ripemd160::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Computes the RIPEMD-160 hash of the SHA-256 hash of the input data.
pub fn hash160(data: &[u8]) -> [u8; 20] {
    let sha_hash = sha256(data);
    ripemd160(&sha_hash)
}

/// Converts a public key to a script hash.
pub fn public_key_to_script_hash(public_key: &[u8]) -> H160 {
    let script = create_verification_script(public_key);
    let hash = hash160(&script);
    H160::from_slice(&hash)
}

/// Converts a secp256r1 public key to a script hash.
pub fn secp256r1_public_key_to_script_hash(public_key: &[u8]) -> H160 {
    public_key_to_script_hash(public_key)
}

/// Creates a verification script from a public key.
fn create_verification_script(public_key: &[u8]) -> Vec<u8> {
    let mut script = Vec::with_capacity(40);
    script.push(0x0C); // PUSHDATA1
    script.push(0x21); // Length (33 bytes for compressed public key)
    script.extend_from_slice(public_key);
    script.push(0x41); // SYSCALL
    script.extend_from_slice(&[0x56, 0xE7, 0xB3, 0x27]); // System.Crypto.CheckSig
    script
}
