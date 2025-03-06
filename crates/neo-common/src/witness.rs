//! Witness utilities
//!
//! This module provides utilities for working with witnesses in the Neo blockchain.

/// A witness in the Neo blockchain
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Witness {
    /// The invocation script
    pub invocation_script: Vec<u8>,
    /// The verification script
    pub verification_script: Vec<u8>,
}

impl Witness {
    /// Create a new witness
    pub fn new(invocation_script: Vec<u8>, verification_script: Vec<u8>) -> Self {
        Self {
            invocation_script,
            verification_script,
        }
    }

    /// Get the invocation script
    pub fn invocation_script(&self) -> &[u8] {
        &self.invocation_script
    }

    /// Get the verification script
    pub fn verification_script(&self) -> &[u8] {
        &self.verification_script
    }
}
