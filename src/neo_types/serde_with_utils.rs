//! Serde utilities for Neo types
//!
//! This module contains utilities for serializing and deserializing Neo types.

use std::fmt;

use primitive_types::H160;
use serde::{
    de::{self, Deserialize, Deserializer, MapAccess, Visitor},
    ser::{SerializeMap, SerializeSeq, SerializeStruct, Serializer},
    Serialize,
};

use crate::neo_types::{
    script_hash::ScriptHash,
    Bytes,
};

/// A trait for encoding a value to base64
pub trait Base64Encode {
    /// Encode the value to base64
    fn to_base64(&self) -> String;
}

impl Base64Encode for Vec<u8> {
    fn to_base64(&self) -> String {
        #[cfg(feature = "utils")]
        {
            use base64::Engine;
            use base64::engine::general_purpose::STANDARD;
            STANDARD.encode(self)
        }
        
        #[cfg(not(feature = "utils"))]
        {
            "base64 encoding not available".to_string()
        }
    }
}

impl Base64Encode for &[u8] {
    fn to_base64(&self) -> String {
        #[cfg(feature = "utils")]
        {
            use base64::Engine;
            use base64::engine::general_purpose::STANDARD;
            STANDARD.encode(self)
        }
        
        #[cfg(not(feature = "utils"))]
        {
            "base64 encoding not available".to_string()
        }
    }
}

/// A trait for extending the functionality of Value
pub trait ValueExtension {
    /// Get a value from a path
    fn get_value_at_path(&self, path: &str) -> Option<Self>
    where
        Self: Sized;
}

/// Serialize a map
pub fn serialize_map<S, K, V>(map: &[(K, V)], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    K: Serialize,
    V: Serialize,
{
    let mut map_ser = serializer.serialize_map(Some(map.len()))?;
    for (k, v) in map {
        map_ser.serialize_entry(k, v)?;
    }
    map_ser.end()
}

/// Deserialize a map
pub fn deserialize_map<'de, D, K, V>(deserializer: D) -> Result<Vec<(K, V)>, D::Error>
where
    D: Deserializer<'de>,
    K: Deserialize<'de>,
    V: Deserialize<'de>,
{
    struct MapVisitor<K, V> {
        marker: std::marker::PhantomData<(K, V)>,
    }

    impl<K, V> MapVisitor<K, V> {
        fn new() -> Self {
            Self { marker: std::marker::PhantomData }
        }
    }

    impl<'de, K, V> Visitor<'de> for MapVisitor<K, V>
    where
        K: Deserialize<'de>,
        V: Deserialize<'de>,
    {
        type Value = Vec<(K, V)>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a map")
        }

        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut map = Vec::new();
            while let Some((key, value)) = access.next_entry()? {
                map.push((key, value));
            }
            Ok(map)
        }
    }

    deserializer.deserialize_map(MapVisitor::new())
}

/// Serialize a H160 as a hex string
pub fn serialize_h160<S>(h: &H160, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("0x{:x}", h))
}

/// Deserialize a H160 from a hex string
pub fn deserialize_h160<'de, D>(deserializer: D) -> Result<H160, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let s = s.trim_start_matches("0x");
    let bytes = hex::decode(s).map_err(|e| de::Error::custom(e.to_string()))?;
    Ok(H160::from_slice(&bytes))
}

/// Serialize a script hash as a hex string
pub fn serialize_script_hash<S>(h: &ScriptHash, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("0x{:x}", h))
}

/// Deserialize a script hash from a hex string
pub fn deserialize_script_hash<'de, D>(deserializer: D) -> Result<ScriptHash, D::Error>
where
    D: Deserializer<'de>,
{
    let a: &[u8] = Deserialize::deserialize(deserializer)?;
    Ok(ScriptHash::from_slice(a))
}

/// Serialize a wildcard
pub fn serialize_wildcard<S>(wildcard: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if *wildcard {
        serializer.serialize_str("*")
    } else {
        serializer.serialize_bool(false)
    }
}

/// Deserialize a wildcard
pub fn deserialize_wildcard<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(s == "*")
}

/// Serialize a fixed string
pub fn serialize_fixed_string<S>(
    s: &Option<String>,
    size: usize,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match s {
        Some(s) => {
            if s.len() > size {
                return Err(serde::ser::Error::custom(format!(
                    "String too long: {} > {}",
                    s.len(),
                    size
                )));
            }
            serializer.serialize_str(s)
        }
        None => serializer.serialize_none(),
    }
}

/// Deserialize a fixed string
pub fn deserialize_fixed_string<'de, D>(
    deserializer: D,
    size: usize,
) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Deserialize::deserialize(deserializer)?;
    match s {
        Some(s) => {
            if s.len() > size {
                return Err(de::Error::custom(format!(
                    "String too long: {} > {}",
                    s.len(),
                    size
                )));
            }
            Ok(Some(s))
        }
        None => Ok(None),
    }
}

/// Serialize a bytes as a hex string
pub fn serialize_bytes_hex<S>(bytes: &Bytes, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&bytes.to_hex())
}

/// Deserialize a bytes from a hex string
pub fn deserialize_bytes_hex<'de, D>(deserializer: D) -> Result<Bytes, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let s = s.trim_start_matches("0x");
    let bytes = hex::decode(s).map_err(|e| de::Error::custom(e.to_string()))?;
    Ok(Bytes::from(bytes))
}

/// Serialize a bytes as a base64 string
pub fn serialize_bytes_base64<S>(bytes: &Bytes, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    #[cfg(feature = "utils")]
    {
        use base64::Engine;
        use base64::engine::general_purpose::STANDARD;
        serializer.serialize_str(&STANDARD.encode(&bytes.0))
    }
    
    #[cfg(not(feature = "utils"))]
    {
        serializer.serialize_str("base64 encoding not available")
    }
}

/// Deserialize a bytes from a base64 string
pub fn deserialize_bytes_base64<'de, D>(deserializer: D) -> Result<Bytes, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    
    #[cfg(feature = "utils")]
    {
        use base64::Engine;
        use base64::engine::general_purpose::STANDARD;
        let bytes = STANDARD.decode(&s).map_err(|e| de::Error::custom(e.to_string()))?;
        Ok(Bytes::from(bytes))
    }
    
    #[cfg(not(feature = "utils"))]
    {
        Err(de::Error::custom("base64 decoding not available"))
    }
}

/// Serialize a list of accounts
pub fn serialize_accounts<S, Account>(accounts: &[Account], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    Account: Serialize,
{
    let mut seq = serializer.serialize_seq(Some(accounts.len()))?;
    for account in accounts {
        seq.serialize_element(account)?;
    }
    seq.end()
}

/// Deserialize a list of accounts
pub fn deserialize_accounts<'de, D, Account>(
    deserializer: D,
) -> Result<Vec<Account>, D::Error>
where
    D: Deserializer<'de>,
    Account: Deserialize<'de>,
{
    struct AccountsVisitor<Account> {
        marker: std::marker::PhantomData<Account>,
    }

    impl<Account> AccountsVisitor<Account> {
        fn new() -> Self {
            Self { marker: std::marker::PhantomData }
        }
    }

    impl<'de, Account> Visitor<'de> for AccountsVisitor<Account>
    where
        Account: Deserialize<'de>,
    {
        type Value = Vec<Account>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a list of accounts")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: de::SeqAccess<'de>,
        {
            let mut accounts = Vec::new();
            while let Some(account) = seq.next_element()? {
                accounts.push(account);
            }
            Ok(accounts)
        }
    }

    deserializer.deserialize_seq(AccountsVisitor::new())
}

/// Serialize a public key
pub fn serialize_public_key<S>(key: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&hex::encode(key))
}

/// Deserialize a public key
pub fn deserialize_public_key<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let bytes = hex::decode(s).map_err(|e| de::Error::custom(e.to_string()))?;
    Ok(bytes)
}

/// A placeholder for Secp256r1PublicKey when crypto-standard feature is not enabled
#[cfg(not(feature = "crypto-standard"))]
#[derive(Debug, Clone)]
pub struct Secp256r1PublicKey {
    pub data: Vec<u8>,
}

#[cfg(not(feature = "crypto-standard"))]
impl Secp256r1PublicKey {
    pub fn get_encoded(&self) -> Vec<u8> {
        self.data.clone()
    }
}
