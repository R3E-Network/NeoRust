//! This module demonstrates the flexibility in handling blockchain addresses and script hashes, leveraging Rust's type system
//! and trait implementations to provide a seamless interface for converting and working with these two fundamental types.

use std::{
    fmt,
    hash::{Hash, Hasher}
};

use primitive_types::H160;
use serde::{Deserialize, Serialize};

use crate::neo_types::{
    address::Address,
    Bytes,
};

#[cfg(feature = "utils")]
use crate::neo_types::script_hash_extension::ScriptHashExtension;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// An enum that can represent either a blockchain  or a ,
/// providing a flexible way to handle both types in APIs and functions.
pub enum AddressOrScriptHash {
    /// A blockchain address (e.g., "NZNos2WqTbu5oCgyfss9kUJgBXJqhuYAaj")
    Address(Address),
    /// A script hash (e.g., 0x5c9c71c55f8bc396714770c232cee8f2a6f2930f)
    ScriptHash(H160),
}

impl Hash for AddressOrScriptHash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            AddressOrScriptHash::Address(a) => a.hash(state),
            AddressOrScriptHash::ScriptHash(s) => s.hash(state),
        }
    }
}

impl fmt::Display for AddressOrScriptHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AddressOrScriptHash::Address(a) => write!(f, "{}", a),
            AddressOrScriptHash::ScriptHash(s) => write!(f, "{:x}", s),
        }
    }
}

impl Default for AddressOrScriptHash {
    fn default() -> Self {
        Self::ScriptHash(H160::zero())
    }
}

impl From<Address> for AddressOrScriptHash {
    fn from(address: Address) -> Self {
        Self::Address(address)
    }
}

impl From<H160> for AddressOrScriptHash {
    fn from(script_hash: H160) -> Self {
        Self::ScriptHash(script_hash)
    }
}

impl From<&str> for AddressOrScriptHash {
    fn from(s: &str) -> Self {
        if s.starts_with("0x") {
            // It's a script hash
            if let Ok(bytes) = hex::decode(&s[2..]) {
                // Create H160 directly from the bytes
                let script_hash = H160::from_slice(&bytes);
                return Self::ScriptHash(script_hash);
            }
            // If we can't parse it as a script hash, treat it as an address
            Self::Address(s.to_string())
        } else {
            // It's an address
            Self::Address(s.to_string())
        }
    }
}

// We don't implement From<String> because it conflicts with From<Address>
// since Address is a type alias for String

impl From<Bytes> for AddressOrScriptHash {
    fn from(bytes: Bytes) -> Self {
        Self::ScriptHash(H160::from_slice(&bytes))
    }
}

impl From<Vec<u8>> for AddressOrScriptHash {
    fn from(bytes: Vec<u8>) -> Self {
        Self::ScriptHash(H160::from_slice(&bytes))
    }
}

impl From<&[u8]> for AddressOrScriptHash {
    fn from(bytes: &[u8]) -> Self {
        Self::ScriptHash(H160::from_slice(bytes))
    }
}

impl AddressOrScriptHash {
    /// Convert to an address
    pub fn to_address(&self) -> Address {
        match self {
            #[cfg(feature = "utils")]
            AddressOrScriptHash::ScriptHash(s) => s.to_address(),
            #[cfg(not(feature = "utils"))]
            AddressOrScriptHash::ScriptHash(_) => "Address conversion not available".to_string(),
            AddressOrScriptHash::Address(a) => a.clone(),
        }
    }

    /// Convert to a script hash
    pub fn to_script_hash(&self) -> H160 {
        match self {
            AddressOrScriptHash::Address(a) => {
                #[cfg(feature = "utils")]
                {
                    // Use the fully qualified path to call the trait method
                    <H160 as ScriptHashExtension>::from_address(a).unwrap_or_default()
                }
                #[cfg(not(feature = "utils"))]
                {
                    H160::zero()
                }
            }
            AddressOrScriptHash::ScriptHash(s) => *s,
        }
    }
}
