//! Script hash module for Neo blockchain
//!
//! This module contains the script hash type and related functionality.

use primitive_types::H160;

#[cfg(feature = "utils")]
pub use crate::neo_types::script_hash_extension::ScriptHashExtension;

#[cfg(feature = "crypto-standard")]
use sha2::{Digest, Sha256};
#[cfg(feature = "crypto-standard")]
use ripemd::Ripemd160;

/// Script hash type
pub type ScriptHash = H160;

/// Trait for hashing a vector of bytes
#[cfg(feature = "crypto-standard")]
pub trait HashableForVec {
    /// Hash a vector of bytes with SHA256
    fn sha256(&self) -> Vec<u8>;
    
    /// Hash a vector of bytes with RIPEMD160
    fn ripemd160(&self) -> Vec<u8>;
    
    /// Hash a vector of bytes with SHA256 and then RIPEMD160
    fn sha256_ripemd160(&self) -> Vec<u8>;
    
    /// Hash a vector of bytes with SHA256 twice
    fn hash256(&self) -> Vec<u8>;
}

#[cfg(feature = "crypto-standard")]
impl HashableForVec for Vec<u8> {
    fn sha256(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(self);
        hasher.finalize().to_vec()
    }
    
    fn ripemd160(&self) -> Vec<u8> {
        let mut hasher = Ripemd160::new();
        hasher.update(self);
        hasher.finalize().to_vec()
    }
    
    fn sha256_ripemd160(&self) -> Vec<u8> {
        let sha256 = self.sha256();
        let mut hasher = Ripemd160::new();
        hasher.update(sha256);
        hasher.finalize().to_vec()
    }
    
    fn hash256(&self) -> Vec<u8> {
        let sha256 = self.sha256();
        let mut hasher = Sha256::new();
        hasher.update(sha256);
        hasher.finalize().to_vec()
    }
}

#[cfg(feature = "crypto-standard")]
impl HashableForVec for [u8] {
    fn sha256(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(self);
        hasher.finalize().to_vec()
    }
    
    fn ripemd160(&self) -> Vec<u8> {
        let mut hasher = Ripemd160::new();
        hasher.update(self);
        hasher.finalize().to_vec()
    }
    
    fn sha256_ripemd160(&self) -> Vec<u8> {
        let sha256 = self.sha256();
        let mut hasher = Ripemd160::new();
        hasher.update(sha256);
        hasher.finalize().to_vec()
    }
    
    fn hash256(&self) -> Vec<u8> {
        let sha256 = self.sha256();
        let mut hasher = Sha256::new();
        hasher.update(sha256);
        hasher.finalize().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[cfg(feature = "crypto-standard")]
    #[test]
    fn test_sha256() {
        let data = b"hello";
        let hash = data.sha256();
        assert_eq!(hash.len(), 32);
    }
    
    #[cfg(feature = "crypto-standard")]
    #[test]
    fn test_ripemd160() {
        let data = b"hello";
        let hash = data.ripemd160();
        assert_eq!(hash.len(), 20);
    }
    
    #[cfg(feature = "crypto-standard")]
    #[test]
    fn test_sha256_ripemd160() {
        let data = b"hello";
        let hash = data.sha256_ripemd160();
        assert_eq!(hash.len(), 20);
    }
    
    #[cfg(all(feature = "crypto-standard", feature = "utils"))]
    #[test]
    fn test_to_address() {
        let script_hash = H160::from_low_u64_be(0x1234567890);
        let address = script_hash.to_address();
        assert!(!address.is_empty());
    }
    
    #[cfg(all(feature = "crypto-standard", feature = "utils"))]
    #[test]
    fn test_from_address() {
        let script_hash = H160::from_low_u64_be(0x1234567890);
        let address = script_hash.to_address();
        let recovered = H160::from_address(&address).unwrap();
        assert_eq!(script_hash, recovered);
    }
}
