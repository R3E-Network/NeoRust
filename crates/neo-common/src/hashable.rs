//! Hashable trait for Neo types
//!
//! This module provides a trait for hashing data in the Neo blockchain.

/// Trait for hashing data
pub trait HashableForVec {
    /// Compute the SHA-256 hash of the data
    fn hash256(&self) -> Vec<u8>;
    
    /// Compute the RIPEMD-160 hash of the data
    fn ripemd160(&self) -> Vec<u8>;
    
    /// Compute the RIPEMD-160 of the SHA-256 hash of the data
    fn hash160(&self) -> Vec<u8>;
}

impl HashableForVec for [u8] {
    fn hash256(&self) -> Vec<u8> {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(self);
        hasher.finalize().to_vec()
    }
    
    fn ripemd160(&self) -> Vec<u8> {
        use ripemd::{Ripemd160, Digest};
        let mut hasher = Ripemd160::new();
        hasher.update(self);
        hasher.finalize().to_vec()
    }
    
    fn hash160(&self) -> Vec<u8> {
        self.hash256().ripemd160()
    }
}

impl HashableForVec for Vec<u8> {
    fn hash256(&self) -> Vec<u8> {
        self.as_slice().hash256()
    }
    
    fn ripemd160(&self) -> Vec<u8> {
        self.as_slice().ripemd160()
    }
    
    fn hash160(&self) -> Vec<u8> {
        self.as_slice().hash160()
    }
}
