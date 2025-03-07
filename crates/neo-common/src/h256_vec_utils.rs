//! H256 vector serialization utilities
//!
//! This module provides utilities for serializing and deserializing vectors of H256 types.

use primitive_types::H256;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

/// Serializes a vector of H256 as a vector of hex strings.
pub fn serialize_vec_h256<S>(hashes: &[H256], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let hex_strings: Vec<String> = hashes
        .iter()
        .map(|hash| format!("0x{}", hex::encode(hash.as_bytes())))
        .collect();
    
    hex_strings.serialize(serializer)
}

/// Deserializes a vector of H256 from a vector of hex strings.
pub fn deserialize_vec_h256<'de, D>(deserializer: D) -> Result<Vec<H256>, D::Error>
where
    D: Deserializer<'de>,
{
    let strings: Vec<String> = Vec::deserialize(deserializer)?;
    
    strings
        .iter()
        .map(|s| {
            let s = s.trim_start_matches("0x");
            H256::from_str(s).map_err(serde::de::Error::custom)
        })
        .collect()
}
