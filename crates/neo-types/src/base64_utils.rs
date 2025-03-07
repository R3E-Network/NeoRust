//! Base64 encoding and decoding utilities
//!
//! This module provides utilities for base64 encoding and decoding.

use base64::engine::general_purpose::STANDARD;
use base64::Engine;

/// Extension trait for types that can be converted to base64 strings
pub trait ToBase64 {
    /// Converts the value to a base64 string
    fn to_base64(&self) -> String;
}

/// Extension trait for types that can be converted from base64 strings
pub trait FromBase64 {
    /// Converts a base64 string to the value
    fn from_base64(s: &str) -> Result<Self, base64::DecodeError>
    where
        Self: Sized;
}

impl ToBase64 for [u8] {
    fn to_base64(&self) -> String {
        STANDARD.encode(self)
    }
}

impl ToBase64 for Vec<u8> {
    fn to_base64(&self) -> String {
        STANDARD.encode(self)
    }
}

impl FromBase64 for Vec<u8> {
    fn from_base64(s: &str) -> Result<Self, base64::DecodeError> {
        STANDARD.decode(s)
    }
}

/// Encodes bytes to a base64 string
pub fn encode<T: AsRef<[u8]>>(input: T) -> String {
    STANDARD.encode(input.as_ref())
}

/// Decodes a base64 string to bytes
pub fn decode<T: AsRef<[u8]>>(input: T) -> Result<Vec<u8>, base64::DecodeError> {
    STANDARD.decode(input.as_ref())
}
