//! Script hash serialization utilities
//!
//! This module provides utilities for serializing and deserializing script hashes.

use primitive_types::H160;
use serde::{Deserialize, Deserializer, Serializer};
use std::str::FromStr;

/// Serializes a script hash as a hex string.
pub fn serialize_script_hash<S>(script_hash: &H160, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let hex_string = format!("0x{}", hex::encode(script_hash.as_bytes()));
    serializer.serialize_str(&hex_string)
}

/// Deserializes a script hash from a hex string.
pub fn deserialize_script_hash<'de, D>(deserializer: D) -> Result<H160, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let s = s.trim_start_matches("0x");
    
    H160::from_str(s).map_err(serde::de::Error::custom)
}

/// Serializes a vector of script hashes as hex strings.
pub fn serialize_vec_script_hash<S>(script_hashes: &[H160], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut hex_strings = Vec::with_capacity(script_hashes.len());
    for script_hash in script_hashes {
        hex_strings.push(format!("0x{}", hex::encode(script_hash.as_bytes())));
    }
    
    serializer.collect_seq(hex_strings)
}

/// Deserializes a vector of script hashes from hex strings.
pub fn deserialize_vec_script_hash<'de, D>(deserializer: D) -> Result<Vec<H160>, D::Error>
where
    D: Deserializer<'de>,
{
    let strings: Vec<String> = Vec::deserialize(deserializer)?;
    let mut result = Vec::with_capacity(strings.len());
    
    for s in strings {
        let s = s.trim_start_matches("0x");
        let script_hash = H160::from_str(s).map_err(serde::de::Error::custom)?;
        result.push(script_hash);
    }
    
    Ok(result)
}
