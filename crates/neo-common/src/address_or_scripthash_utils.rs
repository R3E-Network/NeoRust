//! Address or script hash serialization utilities
//!
//! This module provides utilities for serializing and deserializing address or script hash types.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Serializes an address or script hash.
pub fn serialize_address_or_script_hash<S, T>(address_or_script_hash: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: ToString,
{
    serializer.serialize_str(&address_or_script_hash.to_string())
}

/// Deserializes an address or script hash.
pub fn deserialize_address_or_script_hash<'de, D, T, E>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: std::str::FromStr<Err = E>,
    E: std::fmt::Display,
{
    let s = String::deserialize(deserializer)?;
    T::from_str(&s).map_err(serde::de::Error::custom)
}
