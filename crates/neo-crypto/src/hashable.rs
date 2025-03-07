//! Hashable traits for cryptographic operations
//!
//! This module provides traits for hashing operations commonly used in Neo blockchain.

use sha2::{Digest, Sha256};
use sha3::Keccak256;

/// Extension trait for byte arrays to compute SHA256 hash
pub trait HashableForBytes {
    /// Computes the SHA256 hash of the byte array
    fn hash256(&self) -> [u8; 32];
    
    /// Computes the SHA256 hash twice
    fn hash256_twice(&self) -> [u8; 32];
    
    /// Computes the SHA256 hash followed by RIPEMD160
    fn sha256_ripemd160(&self) -> [u8; 20];
}

/// Extension trait for Vec<u8> to compute SHA256 hash
pub trait HashableForVec {
    /// Computes the SHA256 hash of the byte vector
    fn hash256(&self) -> [u8; 32];
    
    /// Computes the SHA256 hash twice
    fn hash256_twice(&self) -> [u8; 32];
    
    /// Computes the SHA256 hash followed by RIPEMD160
    fn sha256_ripemd160(&self) -> [u8; 20];
}

impl HashableForBytes for [u8] {
    fn hash256(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self);
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
    
    fn hash256_twice(&self) -> [u8; 32] {
        let first_hash = self.hash256();
        first_hash.hash256()
    }
    
    fn sha256_ripemd160(&self) -> [u8; 20] {
        let sha256_hash = self.hash256();
        let mut hasher = ripemd::Ripemd160::new();
        hasher.update(sha256_hash);
        let result = hasher.finalize();
        let mut hash = [0u8; 20];
        hash.copy_from_slice(&result);
        hash
    }
}

impl HashableForBytes for [u8; 20] {
    fn hash256(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self);
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
    
    fn hash256_twice(&self) -> [u8; 32] {
        let first_hash = self.hash256();
        first_hash.hash256()
    }
    
    fn sha256_ripemd160(&self) -> [u8; 20] {
        let sha256_hash = self.hash256();
        let mut hasher = ripemd::Ripemd160::new();
        hasher.update(sha256_hash);
        let result = hasher.finalize();
        let mut hash = [0u8; 20];
        hash.copy_from_slice(&result);
        hash
    }
}

impl HashableForVec for Vec<u8> {
    fn hash256(&self) -> [u8; 32] {
        self.as_slice().hash256()
    }
    
    fn hash256_twice(&self) -> [u8; 32] {
        self.as_slice().hash256_twice()
    }
    
    fn sha256_ripemd160(&self) -> [u8; 20] {
        self.as_slice().sha256_ripemd160()
    }
}

// Add a new trait for Vec<u8> that returns Vec<u8> instead of fixed arrays
pub trait HashableForVecToVec {
    /// Computes the SHA256 hash of the byte vector and returns a Vec<u8>
    fn hash256_vec(&self) -> Vec<u8>;
    
    /// Computes the SHA256 hash twice and returns a Vec<u8>
    fn hash256_twice_vec(&self) -> Vec<u8>;
    
    /// Computes the SHA256 hash followed by RIPEMD160 and returns a Vec<u8>
    fn sha256_ripemd160_vec(&self) -> Vec<u8>;
}

impl HashableForVecToVec for Vec<u8> {
    fn hash256_vec(&self) -> Vec<u8> {
        self.hash256().to_vec()
    }
    
    fn hash256_twice_vec(&self) -> Vec<u8> {
        self.hash256_twice().to_vec()
    }
    
    fn sha256_ripemd160_vec(&self) -> Vec<u8> {
        self.sha256_ripemd160().to_vec()
    }
}
