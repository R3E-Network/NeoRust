//! Base64 encoding/decoding utilities
//!
//! This module provides utilities for base64 encoding and decoding.

use base64::{engine::general_purpose, Engine};

/// Trait for base64 encoding
pub trait Base64Encode {
    /// Convert to base64 string
    fn to_base64(&self) -> String;
}

/// Trait for base64 decoding
pub trait Base64Decode {
    /// Convert from base64 string
    fn from_base64(base64_str: &str) -> Result<Self, base64::DecodeError>
    where
        Self: Sized;
}

impl Base64Encode for [u8] {
    fn to_base64(&self) -> String {
        general_purpose::STANDARD.encode(self)
    }
}

impl Base64Encode for Vec<u8> {
    fn to_base64(&self) -> String {
        general_purpose::STANDARD.encode(self)
    }
}

impl Base64Decode for Vec<u8> {
    fn from_base64(base64_str: &str) -> Result<Self, base64::DecodeError> {
        general_purpose::STANDARD.decode(base64_str)
    }
}
