//! Public key serialization utilities
//!
//! This module provides utilities for serializing and deserializing public keys.

use serde::{Deserialize, Deserializer, Serializer};

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

/// Serializes a public key as a hex string.
pub fn serialize_public_key<S, T>(public_key: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: AsRef<[u8]>,
{
    let hex_string = format!("0x{}", hex::encode(public_key.as_ref()));
    serializer.serialize_str(&hex_string)
}

/// Serializes a vector of public keys as a list of hex strings.
pub fn serialize_vec_public_key<S, T>(public_keys: &Vec<T>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: AsRef<[u8]>,
{
    use serde::ser::SerializeSeq;
    let mut seq = serializer.serialize_seq(Some(public_keys.len()))?;
    for key in public_keys {
        seq.serialize_element(&format!("0x{}", hex::encode(key.as_ref())))?;
    }
    seq.end()
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

/// Deserializes a public key from a hex string.
pub fn deserialize_public_key<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let s = s.trim_start_matches("0x");
    hex::decode(s).map_err(serde::de::Error::custom)
}

/// Deserializes a vector of public keys from a list of hex strings.
pub fn deserialize_vec_public_key<'de, D>(deserializer: D) -> Result<Vec<Vec<u8>>, D::Error>
where
    D: Deserializer<'de>,
{
    let strings: Vec<String> = Vec::deserialize(deserializer)?;
    let mut result = Vec::with_capacity(strings.len());
    
    for s in strings {
        let s = s.trim_start_matches("0x");
        result.push(hex::decode(s).map_err(serde::de::Error::custom)?);
    }
    
    Ok(result)
}
