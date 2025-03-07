//! Address conversion utilities
//!
//! This module provides utilities for converting between different address formats.

use sha2::{Digest, Sha256};
use ripemd::Ripemd160;
use bs58;

/// A simple public key structure
#[derive(Debug, Clone)]
pub struct PublicKey {
    /// The public key bytes
    pub bytes: Vec<u8>,
}

impl PublicKey {
    /// Create a new public key from bytes
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }
    
    /// Get the encoded public key
    pub fn get_encoded(&self, compressed: bool) -> Vec<u8> {
        if compressed {
            // Simple implementation for compressed format
            let mut result = Vec::with_capacity(33);
            // Add prefix byte (02 or 03 depending on y coordinate)
            result.push(if self.bytes.len() > 32 && self.bytes[32] % 2 == 0 { 0x02 } else { 0x03 });
            // Add x coordinate (first 32 bytes)
            result.extend_from_slice(&self.bytes[0..32]);
            result
        } else {
            // Uncompressed format
            let mut result = Vec::with_capacity(65);
            result.push(0x04); // Uncompressed prefix
            result.extend_from_slice(&self.bytes);
            result
        }
    }
}

/// Hash data using SHA-256
pub fn hash256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Hash data using RIPEMD-160
pub fn ripemd160(data: &[u8]) -> Vec<u8> {
    let mut hasher = Ripemd160::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Base58Check encode data
pub fn base58check_encode(data: &[u8]) -> String {
    // Calculate checksum (double SHA-256)
    let hash1 = hash256(data);
    let hash2 = hash256(&hash1);
    
    // Append first 4 bytes of checksum
    let mut with_checksum = data.to_vec();
    with_checksum.extend_from_slice(&hash2[0..4]);
    
    // Encode with Base58
    bs58::encode(with_checksum).into_string()
}

/// Convert a public key to an address
pub fn public_key_to_address(public_key: &PublicKey) -> String {
    // Get the encoded public key
    let encoded = public_key.get_encoded(true);
    
    // Hash the public key to get a script hash
    let hash = hash256(&encoded);
    let script_hash = ripemd160(&hash);
    
    // Convert script hash to address format
    let mut address_bytes = vec![0x35]; // Address version byte for Neo N3
    address_bytes.extend_from_slice(&script_hash);
    
    // Base58Check encode the address
    base58check_encode(&address_bytes)
}

/// Convert a public key to a script hash
pub fn public_key_to_script_hash(public_key: &PublicKey) -> primitive_types::H160 {
    // Get the encoded public key
    let encoded = public_key.get_encoded(true);
    
    // Hash the public key to get a script hash
    let hash = hash256(&encoded);
    let script_hash = ripemd160(&hash);
    
    // Convert to H160
    primitive_types::H160::from_slice(&script_hash)
}

/// Convert a Secp256r1PublicKey to a script hash
pub fn secp256r1_public_key_to_script_hash(public_key_bytes: &[u8]) -> primitive_types::H160 {
    // Create a PublicKey from the bytes
    let pub_key = PublicKey::new(public_key_bytes.to_vec());
    
    // Use the existing function to convert to script hash
    public_key_to_script_hash(&pub_key)
}
