use std::fmt;
use std::str::FromStr;
use primitive_types::H160;
use serde::{Deserialize, Serialize};

use crate::{Address, AddressOrScriptHash, ScriptHash, ScriptHashExtension};

impl fmt::Display for AddressOrScriptHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AddressOrScriptHash::Address(addr) => write!(f, "{}", addr),
            AddressOrScriptHash::ScriptHash(script_hash) => {
                write!(f, "{}", script_hash.to_address())
            }
        }
    }
}

impl FromStr for AddressOrScriptHash {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("0x") || s.len() == 40 {
            // Assume it's a script hash
            let script_hash = ScriptHash::from_str(s)
                .map_err(|e| format!("Invalid script hash format: {}", e))?;
            Ok(AddressOrScriptHash::ScriptHash(script_hash))
        } else if s.starts_with('N') {
            // Assume it's an address
            Ok(AddressOrScriptHash::Address(s.to_string()))
        } else {
            Err(format!("Invalid address or script hash format: {}", s))
        }
    }
}

impl ToString for AddressOrScriptHash {
    fn to_string(&self) -> String {
        match self {
            AddressOrScriptHash::Address(addr) => addr.clone(),
            AddressOrScriptHash::ScriptHash(script_hash) => script_hash.to_address(),
        }
    }
}
