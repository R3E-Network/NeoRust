//! Base64 encoding and decoding utilities
//!
//! This module provides utilities for encoding and decoding base64 data.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Encodes bytes to a base64 string.
pub fn encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

/// Decodes a base64 string to bytes.
pub fn decode(s: &str) -> Result<Vec<u8>, ::base64::DecodeError> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(s.as_bytes())
}

/// Serializes a byte array as a base64 string.
pub fn serialize_base64<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let base64_string = encode(bytes);
    serializer.serialize_str(&base64_string)
}

/// Deserializes a byte array from a base64 string.
pub fn deserialize_base64<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    decode(&s).map_err(serde::de::Error::custom)
}

/// A wrapper type for base64 encoded data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Base64Encode(pub Vec<u8>);

impl Serialize for Base64Encode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serialize_base64(&self.0, serializer)
    }
}

impl<'de> Deserialize<'de> for Base64Encode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes = deserialize_base64(deserializer)?;
        Ok(Base64Encode(bytes))
    }
}

impl From<Vec<u8>> for Base64Encode {
    fn from(bytes: Vec<u8>) -> Self {
        Base64Encode(bytes)
    }
}

impl From<&[u8]> for Base64Encode {
    fn from(bytes: &[u8]) -> Self {
        Base64Encode(bytes.to_vec())
    }
}

impl From<Base64Encode> for Vec<u8> {
    fn from(base64: Base64Encode) -> Self {
        base64.0
    }
}

/// Trait for converting to base64 encoded string
pub trait ToBase64 {
    /// Convert to base64 encoded string
    fn to_base64(&self) -> String;
}

/// Trait for converting from base64 encoded string
pub trait FromBase64: Sized {
    /// Convert from base64 encoded string
    fn from_base64(s: &str) -> Result<Self, base64::DecodeError>;
}

impl ToBase64 for [u8] {
    fn to_base64(&self) -> String {
        encode(self)
    }
}

impl ToBase64 for Vec<u8> {
    fn to_base64(&self) -> String {
        encode(self)
    }
}

impl FromBase64 for Vec<u8> {
    fn from_base64(s: &str) -> Result<Self, base64::DecodeError> {
        decode(s)
    }
}
