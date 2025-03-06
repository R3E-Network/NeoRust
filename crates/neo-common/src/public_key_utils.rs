//! Public key serialization utilities
//!
//! This module provides utilities for serializing and deserializing public keys.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Serializes a public key as a hex string.
pub fn serialize_public_key_option<S>(public_key_opt: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match public_key_opt {
        Some(public_key) => {
            let hex_string = format!("0x{}", hex::encode(public_key));
            serializer.serialize_str(&hex_string)
        }
        None => serializer.serialize_none(),
    }
}

/// Deserializes a public key from a hex string.
pub fn deserialize_public_key_option<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
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
            let bytes = hex::decode(s).map_err(serde::de::Error::custom)?;
            Ok(Some(bytes))
        }
        StringOrNull::Null => Ok(None),
    }
}
