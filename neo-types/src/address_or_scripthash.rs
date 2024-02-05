use crate::{
	address::{Address, AddressExtension},
	script_hash::ScriptHashExtension,
	Bytes,
};
use primitive_types::H160;
use serde_derive::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// A type that can either be an `Address` or `Bytes`.
pub enum AddressOrScriptHash {
	/// An address type
	Address(Address),
	/// A bytes type
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

impl Default for AddressOrScriptHash {
	fn default() -> Self {
		AddressOrScriptHash::Address(Default::default())
	}
}

impl From<Address> for AddressOrScriptHash {
	fn from(s: Address) -> Self {
		Self::Address(s)
	}
}

impl From<Bytes> for AddressOrScriptHash {
	fn from(s: Bytes) -> Self {
		Self::ScriptHash(H160::from_slice(&s))
	}
}

impl AddressOrScriptHash {
	pub fn address(&self) -> Address {
		match self {
			AddressOrScriptHash::Address(a) => a.clone(),
			AddressOrScriptHash::ScriptHash(s) => s.to_address(),
		}
	}
	pub fn script_hash(&self) -> H160 {
		match self {
			AddressOrScriptHash::Address(a) => a.address_to_script_hash().unwrap(),
			AddressOrScriptHash::ScriptHash(s) => s.clone(),
		}
	}
}
