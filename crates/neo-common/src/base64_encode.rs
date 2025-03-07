//! Base64 encoding utilities for Neo types
//!
//! This module provides Base64 encoding functionality for Neo types.

use rustc_serialize::base64::{ToBase64, STANDARD};

/// A wrapper struct for Base64 encoding
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Base64Encode(pub Vec<u8>);

impl Base64Encode {
    /// Create a new Base64Encode from a byte vector
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    /// Convert any byte-like data to Base64
    pub fn to_base64<T: AsRef<[u8]>>(data: T) -> String {
        data.as_ref().to_base64(STANDARD)
    }

    /// Get the inner byte vector
    pub fn into_inner(self) -> Vec<u8> {
        self.0
    }
}

impl From<Vec<u8>> for Base64Encode {
    fn from(v: Vec<u8>) -> Self {
        Self(v)
    }
}

impl From<&[u8]> for Base64Encode {
    fn from(v: &[u8]) -> Self {
        Self(v.to_vec())
    }
}

impl From<String> for Base64Encode {
    fn from(v: String) -> Self {
        Self(v.into_bytes())
    }
}

impl From<&str> for Base64Encode {
    fn from(v: &str) -> Self {
        Self(v.as_bytes().to_vec())
    }
}

impl AsRef<[u8]> for Base64Encode {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl ToBase64 for Base64Encode {
    fn to_base64(&self, config: rustc_serialize::base64::Config) -> String {
        self.0.to_base64(config)
    }
}
