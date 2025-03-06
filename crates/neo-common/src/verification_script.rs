//! Verification script utilities
//!
//! This module provides utilities for working with verification scripts.

use crate::address_conversion::PublicKey;
use crate::hashable::HashableForVec;
use primitive_types::H160;
use rustc_serialize::base64::{ToBase64, STANDARD};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

/// A verification script in the Neo blockchain
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerificationScript {
    /// The script bytes
    pub script: Vec<u8>,
}

impl VerificationScript {
    /// Create a new verification script from bytes
    pub fn new(script: Vec<u8>) -> Self {
        Self { script }
    }

    /// Creates a verification script from a public key
    pub fn from_public_key(public_key: &PublicKey) -> Self {
        // Create a verification script matching the expected format
        let mut script = Vec::with_capacity(40);
        script.push(0x0c); // Special prefix for Neo N3 verification scripts
        script.push(0x21); // Push 33 bytes (compressed public key)
        script.extend_from_slice(&public_key.bytes);
        script.push(0x41); // SYSCALL opcode
        script.push(0x56); // System.Crypto.CheckSig method ID (part 1)
        script.push(0xe7); // System.Crypto.CheckSig method ID (part 2)
        script.push(0xb3); // System.Crypto.CheckSig method ID (part 3)
        script.push(0x27); // System.Crypto.CheckSig method ID (part 4)
        Self { script }
    }

    /// Creates a verification script for a multi-signature account
    pub fn from_multi_sig(public_keys: &[PublicKey], signing_threshold: usize) -> Self {
        // Implementation would create a multi-signature verification script
        let mut script = Vec::new();
        // Add the signing threshold and public keys to the script
        script.push(signing_threshold as u8);
        for key in public_keys {
            let encoded = key.bytes.clone();
            script.extend_from_slice(&encoded);
        }
        Self { script }
    }

    /// Get the script bytes
    pub fn script(&self) -> &[u8] {
        &self.script
    }

    /// Returns the script as a base64 encoded string
    pub fn to_base64(&self) -> String {
        use rustc_serialize::base64::STANDARD;
        self.script.to_base64(STANDARD)
    }

    /// Calculate the script hash
    pub fn hash(&self) -> H160 {
        // Use proper hashing for script hash
        use crate::hash256;
        use crate::ripemd160;
        
        // First SHA-256
        let sha256_hash = hash256(&self.script);
        // Then RIPEMD-160
        let mut hash = ripemd160(&sha256_hash);
        // Reverse the hash bytes to match Neo's expected format
        hash.reverse();
        H160::from_slice(&hash)
    }

    /// Check if the script is a single signature script
    pub fn is_single_sig(&self) -> bool {
        // This is a simplified implementation
        // In a real implementation, this would check the script structure
        !self.script.is_empty() && !self.is_multi_sig()
    }

    /// Check if the script is a multi-signature script
    pub fn is_multi_sig(&self) -> bool {
        // This is a simplified implementation
        // In a real implementation, this would check the script structure
        !self.script.is_empty() && self.script.len() > 1
    }

    /// Get the number of accounts in a multi-signature script
    pub fn get_nr_of_accounts(&self) -> Option<usize> {
        // This is a simplified implementation
        // In a real implementation, this would parse the script
        if self.is_multi_sig() {
            Some(1) // Placeholder
        } else {
            None
        }
    }

    /// Gets the signing threshold for a multi-signature script
    pub fn get_signing_threshold(&self) -> Option<usize> {
        // Implementation would extract the signing threshold from the script
        if self.is_multi_sig() {
            Some(self.script[0] as usize)
        } else {
            None
        }
    }
}

impl Hash for VerificationScript {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.script.hash(state);
    }
}
