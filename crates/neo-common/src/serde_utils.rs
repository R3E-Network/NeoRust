//! Serialization utilities for Neo types
//!
//! This module provides utilities for serializing and deserializing Neo types.

use primitive_types::{H160, H256};
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

/// Serializes a H256 as a hex string.
pub fn serialize_h256<S>(hash: &H256, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let hex_string = format!("0x{}", hex::encode(hash.as_bytes()));
    serializer.serialize_str(&hex_string)
}

/// Deserializes a H256 from a hex string.
pub fn deserialize_h256<'de, D>(deserializer: D) -> Result<H256, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let s = s.trim_start_matches("0x");
    
    H256::from_str(s).map_err(serde::de::Error::custom)
}

/// Serializes an optional H256 as a hex string.
pub fn serialize_h256_option<S>(hash_opt: &Option<H256>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match hash_opt {
        Some(hash) => serialize_h256(hash, serializer),
        None => serializer.serialize_none(),
    }
}

/// Deserializes an optional H256 from a hex string.
pub fn deserialize_h256_option<'de, D>(deserializer: D) -> Result<Option<H256>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNull {
        String(String),
        Null,
    }

    match StringOrNull::deserialize(deserializer)? {
        StringOrNull::String(s) => {
            let s = s.trim_start_matches("0x");
            Ok(Some(H256::from_str(s).map_err(serde::de::Error::custom)?))
        }
        StringOrNull::Null => Ok(None),
    }
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

/// Converts a vector to a fixed-size array of 32 bytes.
pub fn vec_to_array32(vec: Vec<u8>) -> [u8; 32] {
    let mut arr = [0u8; 32];
    let len = std::cmp::min(vec.len(), 32);
    arr[..len].copy_from_slice(&vec[..len]);
    arr
}
