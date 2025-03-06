//! Address utility functions
//!
//! This module provides utility functions for working with addresses and script hashes.

use primitive_types::H160;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Serializes an address or script hash.
pub fn serialize_address_or_script_hash<S>(
    address_or_script_hash: &str,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(address_or_script_hash)
}

/// Deserializes an address or script hash.
pub fn deserialize_address_or_script_hash<'de, D>(
    deserializer: D,
) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer)
}

/// Serializes a public key option.
pub fn serialize_public_key_option<S>(
    public_key_opt: &Option<Vec<u8>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match public_key_opt {
        Some(public_key) => {
            let hex_string = hex::encode(public_key);
            serializer.serialize_str(&hex_string)
        }
        None => serializer.serialize_none(),
    }
}

/// Deserializes a public key option.
pub fn deserialize_public_key_option<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<u8>>, D::Error>
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
            let bytes = hex::decode(s).map_err(serde::de::Error::custom)?;
            Ok(Some(bytes))
        }
        StringOrNull::Null => Ok(None),
    }
}
