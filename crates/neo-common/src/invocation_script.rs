//! Invocation script utilities
//!
//! This module provides utilities for working with invocation scripts.

/// An invocation script in the Neo blockchain
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvocationScript {
    /// The script bytes
    pub script: Vec<u8>,
}

impl InvocationScript {
    /// Create a new invocation script from bytes
    pub fn new(script: Vec<u8>) -> Self {
        Self { script }
    }

    /// Get the script bytes
    pub fn script(&self) -> &[u8] {
        &self.script
    }
}
