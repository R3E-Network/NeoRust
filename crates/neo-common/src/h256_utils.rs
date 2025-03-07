//! H256 serialization utilities
//!
//! This module provides utilities for serializing and deserializing H256 types.

use primitive_types::H256;
use serde::{Deserialize, Deserializer, Serializer};
use std::str::FromStr;

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
