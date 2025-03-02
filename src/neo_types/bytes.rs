//! Bytes type for Neo blockchain
//!
//! This module contains the Bytes type, which is a wrapper around Vec<u8>.

use std::fmt;
use std::ops::Deref;
use serde::{Deserialize, Serialize};

/// Bytes is a wrapper around Vec<u8>
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Bytes(pub Vec<u8>);

impl Bytes {
    /// Create a new Bytes from a Vec<u8>
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Create a new Bytes from a slice
    pub fn from_slice(slice: &[u8]) -> Self {
        Self(slice.to_vec())
    }

    /// Convert to base64 string
    #[cfg(feature = "utils")]
    pub fn to_base64(&self) -> String {
        use base64::Engine;
        use base64::engine::general_purpose::STANDARD;
        STANDARD.encode(&self.0)
    }

    /// Convert to base64 string (fallback implementation)
    #[cfg(not(feature = "utils"))]
    pub fn to_base64(&self) -> String {
        "base64 encoding not available".to_string()
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }
}

impl From<&[u8]> for Bytes {
    fn from(bytes: &[u8]) -> Self {
        Self(bytes.to_vec())
    }
}

impl From<Bytes> for Vec<u8> {
    fn from(bytes: Bytes) -> Self {
        bytes.0
    }
}

impl Deref for Bytes {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", hex::encode(&self.0))
    }
}
