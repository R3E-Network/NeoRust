// This module demonstrates the flexibility in handling blockchain addresses and script hashes, leveraging Rust's type system
// and trait implementations to provide a seamless interface for converting and working with these two fundamental types.

use std::hash::{Hash, Hasher};
use std::fmt;
use std::str::FromStr;

use primitive_types::H160;
use serde_derive::{Deserialize, Serialize};

use crate::{Address, ScriptHashExtension};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// An enum that can represent either a blockchain `Address` or a `ScriptHash`,
/// offering flexibility for APIs that can work with either.
pub enum AddressOrScriptHash {
	/// An address type
	Address(Address),
	/// A bytes type
	ScriptHash(H160),
}

impl Hash for AddressOrScriptHash {
	/// Implements the `Hash` trait to allow `AddressOrScriptHash`
	/// instances to be used as keys in hash maps or elements in hash sets.
	///
	/// # Examples
	///
	/// ```
	/// use std::collections::HashSet;
	/// use NeoRust::prelude::AddressOrScriptHash;
	/// let mut set = HashSet::new();
	/// set.insert(AddressOrScriptHash::Address("myAddress".into()));
	/// ```
	fn hash<H: Hasher>(&self, state: &mut H) {
		match self {
			AddressOrScriptHash::Address(a) => a.hash(state),
			AddressOrScriptHash::ScriptHash(s) => s.hash(state),
		}
	}
}

impl Default for AddressOrScriptHash {
	fn default() -> Self {
		AddressOrScriptHash::Address(Default::default())
	}
}

impl From<Address> for AddressOrScriptHash {
	/// Allows creating an `AddressOrScriptHash` directly from an `Address`.
	///
	/// # Examples
	///
	/// ```
	/// use NeoRust::prelude::AddressOrScriptHash;
	/// let from_address = AddressOrScriptHash::from("myAddress".into());
	/// assert!(matches!(from_address, AddressOrScriptHash::Address(_)));
	/// ```
	fn from(s: Address) -> Self {
		Self::Address(s)
	}
}

impl From<Vec<u8>> for AddressOrScriptHash {
	/// Allows creating an `AddressOrScriptHash` from a byte vector, automatically converting it into a `ScriptHash`.
	///
	/// # Examples
	///
	/// ```
	/// use NeoRust::prelude::{AddressOrScriptHash};
	/// let bytes: Vec<u8> = vec![0xdeu8, 0xadu8, 0xbeu8, 0xefu8];
	/// let from_bytes = AddressOrScriptHash::from(bytes);
	/// assert!(matches!(from_bytes, AddressOrScriptHash::ScriptHash(_)));
	/// ```
	fn from(s: Vec<u8>) -> Self {
		Self::ScriptHash(H160::from_slice(&s))
	}
}

impl AddressOrScriptHash {
	/// Retrieves the `Address` representation. If the instance is a `ScriptHash`, converts it to an `Address`.
	///
	/// # Examples
	///
	/// ```
	/// use primitive_types::H160;
	/// use NeoRust::prelude::AddressOrScriptHash;
	/// let script_hash = AddressOrScriptHash::ScriptHash(H160::repeat_byte(0x01));
	/// let address = script_hash.address();
	/// assert_eq!(address, "convertedAddressFromScriptHash");
	/// ```
	pub fn address(&self) -> Address {
		match self {
			AddressOrScriptHash::Address(a) => a.clone(),
			AddressOrScriptHash::ScriptHash(s) => s.to_address(),
		}
	}

	/// Retrieves the `ScriptHash` representation. If the instance is an `Address`, converts it to a `ScriptHash`.
	///
	/// # Examples
	///
	/// ```
	/// use primitive_types::H160;
	/// use NeoRust::prelude::AddressOrScriptHash;
	/// let address = AddressOrScriptHash::Address("myAddress".into());
	/// let script_hash = address.script_hash();
	/// assert_eq!(script_hash, H160::repeat_byte(0x02)); // Assuming `to_address` converts an address into a specific script hash
	/// ```
	pub fn script_hash(&self) -> H160 {
		match self {
			AddressOrScriptHash::Address(a) => H160::from_address(&a).unwrap(), //a.address_to_script_hash().unwrap(),
			AddressOrScriptHash::ScriptHash(s) => s.clone(),
		}
	}
}

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
            let script_hash = H160::from_str(s)
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

// ToString is automatically implemented for types that implement Display
