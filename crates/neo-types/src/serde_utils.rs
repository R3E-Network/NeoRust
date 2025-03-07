//! Serialization utilities for Neo types
//!
//! This module provides utilities for serializing and deserializing Neo types.

use primitive_types::H160;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;
use std::collections::HashMap;
use serde::ser::{SerializeMap as SerMap};

/// Serializes a H160 as a hex string.
pub fn serialize_h160<S>(hash: &H160, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let hex_string = format!("0x{}", hex::encode(hash.as_bytes()));
    serializer.serialize_str(&hex_string)
}

/// Deserializes a H160 from a hex string.
pub fn deserialize_h160<'de, D>(deserializer: D) -> Result<H160, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let s = s.trim_start_matches("0x");
    
    H160::from_str(s).map_err(serde::de::Error::custom)
}

/// Serializes a byte array as a base64 string.
pub fn serialize_base64<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use base64::Engine;
    let base64_string = base64::engine::general_purpose::STANDARD.encode(bytes);
    serializer.serialize_str(&base64_string)
}

/// Deserializes a byte array from a base64 string.
pub fn deserialize_base64<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    use base64::Engine;
    let s = String::deserialize(deserializer)?;
    base64::engine::general_purpose::STANDARD.decode(s.as_bytes())
        .map_err(serde::de::Error::custom)
}

/// Serializes a map of contract parameters.
pub fn serialize_map<K, V, S>(map: &HashMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
where
    K: Serialize,
    V: Serialize,
    S: Serializer,
{
    let mut map_ser = serializer.serialize_map(Some(map.len()))?;
    for (k, v) in map {
        map_ser.serialize_entry(k, v)?;
    }
    map_ser.end()
}

/// Deserializes a map of contract parameters.
pub fn deserialize_map<'de, K, V, D>(deserializer: D) -> Result<HashMap<K, V>, D::Error>
where
    K: Deserialize<'de> + std::hash::Hash + Eq,
    V: Deserialize<'de>,
    D: Deserializer<'de>,
{
    HashMap::deserialize(deserializer)
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
