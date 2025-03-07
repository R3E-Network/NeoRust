//! H160 serialization utilities
//!
//! This module provides utilities for serializing and deserializing H160 types.

use primitive_types::H160;
use serde::{Deserialize, Deserializer, Serializer};
use std::str::FromStr;

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

/// Serializes an optional H160 as a hex string.
pub fn serialize_h160_option<S>(hash_opt: &Option<H160>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match hash_opt {
        Some(hash) => serialize_h160(hash, serializer),
        None => serializer.serialize_none(),
    }
}

/// Deserializes an optional H160 from a hex string.
pub fn deserialize_h160_option<'de, D>(deserializer: D) -> Result<Option<H160>, D::Error>
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
            Ok(Some(H160::from_str(s).map_err(serde::de::Error::custom)?))
        }
        StringOrNull::Null => Ok(None),
    }
}

/// Serializes a vector of H160 as a list of hex strings.
pub fn serialize_vec_h160<S>(hashes: &Vec<H160>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use serde::ser::SerializeSeq;
    let mut seq = serializer.serialize_seq(Some(hashes.len()))?;
    for hash in hashes {
        seq.serialize_element(&format!("0x{}", hex::encode(hash.as_bytes())))?;
    }
    seq.end()
}

/// Deserializes a vector of H160 from a list of hex strings.
pub fn deserialize_vec_h160<'de, D>(deserializer: D) -> Result<Vec<H160>, D::Error>
where
    D: Deserializer<'de>,
{
    let strings: Vec<String> = Vec::deserialize(deserializer)?;
    let mut result = Vec::with_capacity(strings.len());
    
    for s in strings {
        let s = s.trim_start_matches("0x");
        result.push(H160::from_str(s).map_err(serde::de::Error::custom)?);
    }
    
    Ok(result)
}
