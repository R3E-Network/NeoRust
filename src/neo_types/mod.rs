//! # Neo Types
//!
//! Core data types for the Neo N3 blockchain.
//!
//! ## Overview
//!
//! The neo_types module provides fundamental data types and structures for working with
//! the Neo N3 blockchain. It includes:
//!
//! - Address and ScriptHash types for identifying accounts and contracts
//! - Contract-related types (parameters, manifests, NEF files)
//! - Neo Name Service (NNS) types
//! - Numeric types with blockchain-specific operations
//! - Serialization and deserialization utilities
//! - Stack item representations for VM operations
//! - Blockchain-specific enumerations (OpCode, VMState)
//!
//! This module forms the type foundation for the entire SDK, providing the core data structures
//! that represent Neo N3 blockchain concepts.
//!
//! ## Feature Flags
//!
//! This module supports several feature flags that control which types are available:
//!
//! - **std**: Core types that don't require specific blockchain features
//! - **transaction**: Transaction-related types
//! - **contract**: Smart contract types
//! - **nep17**: NEP-17 token standard types
//! - **nep11**: NEP-11 token standard types
//! - **http-client**: Types needed for RPC communication
//!
//! ## Examples
//!
//! ### Working with Neo N3 addresses and script hashes
//!
//! ```rust
//! use neo::prelude::*;
//! use std::str::FromStr;
//!
//! // Create a script hash from a string
//! let script_hash = ScriptHash::from_str("0xd2a4cff31913016155e38e474a2c06d08be276cf").unwrap();
//! println!("Script hash: {}", script_hash);
//!
//! // Convert between address and script hash
//! let address = Address::from_script_hash(&script_hash);
//! println!("Address: {}", address);
//!
//! let recovered_hash = ScriptHash::from_address(&address).unwrap();
//! assert_eq!(script_hash, recovered_hash);
//! ```
//!
//! ### Working with contract parameters
//!
//! ```rust
//! use neo::prelude::*;
//! use std::str::FromStr;
//!
//! // Create different types of contract parameters
//! let string_param = ContractParameter::string("Hello, Neo!");
//! let integer_param = ContractParameter::integer(42);
//! let bool_param = ContractParameter::bool(true);
//!
//! // Create a hash160 parameter from a script hash
//! let script_hash = ScriptHash::from_str("0xd2a4cff31913016155e38e474a2c06d08be276cf").unwrap();
//! let hash_param = ContractParameter::hash160(&script_hash);
//!
//! // Create an array parameter
//! let array_param = ContractParameter::array(vec![
//!     ContractParameter::integer(1),
//!     ContractParameter::integer(2),
//!     ContractParameter::integer(3),
//! ]);
//! ```

#[cfg(feature = "serde")]
use base64::{engine::general_purpose, Engine};

pub use log::*;
use primitive_types::H256;

#[cfg(feature = "serde")]
use serde_derive::{Deserialize, Serialize};

// Core types - always available
pub use error::*;
pub use script_hash::*;
pub use address::*;
pub use address_or_scripthash::*;
pub use bytes::*;
pub use numeric::*;
pub use string::*;
pub use util::*;

// OpCode and VM state - needed for transaction and contract features
#[cfg(any(feature = "transaction", feature = "contract"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "transaction", feature = "contract"))))]
pub use op_code::*;

#[cfg(any(feature = "transaction", feature = "contract"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "transaction", feature = "contract"))))]
pub use vm_state::*;

// Transaction related types
#[cfg(feature = "transaction")]
#[cfg_attr(docsrs, doc(cfg(feature = "transaction")))]
pub use block::*;

// Serialization helpers - only available with serde feature
#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
pub use serde_value::*;

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
pub use serde_with_utils::*;

// Contract-related types
#[cfg(feature = "contract")]
#[cfg_attr(docsrs, doc(cfg(feature = "contract")))]
pub use contract::*;

#[cfg(feature = "contract")]
#[cfg_attr(docsrs, doc(cfg(feature = "contract")))]
pub use stack_item::*;

// Neo Name Service types
#[cfg(feature = "http-client")]
#[cfg_attr(docsrs, doc(cfg(feature = "http-client")))]
pub use nns::*;

// RPC-related types
#[cfg(feature = "http-client")]
#[cfg_attr(docsrs, doc(cfg(feature = "http-client")))]
pub use syncing::*;

#[cfg(feature = "http-client")]
#[cfg_attr(docsrs, doc(cfg(feature = "http-client")))]
pub use tx_pool::*;

#[cfg(feature = "http-client")]
#[cfg_attr(docsrs, doc(cfg(feature = "http-client")))]
pub use url_session::*;

// Utility types - available with various features
#[cfg(any(feature = "contract", feature = "transaction"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "contract", feature = "transaction"))))]
pub use path_or_string::*;

#[cfg(feature = "contract")]
#[cfg_attr(docsrs, doc(cfg(feature = "contract")))]
pub use plugin_type::*;

// Module declarations - some are always available, others are conditional
#[cfg(feature = "contract")]
mod contract;

#[cfg(feature = "http-client")]
mod nns;

// Core modules - always available
mod error;
mod script_hash;
mod address;
mod address_or_scripthash;
mod bytes;
mod numeric;
mod string;
mod util;

// Conditional modules
#[cfg(any(feature = "transaction", feature = "contract"))]
mod op_code;

#[cfg(any(feature = "transaction", feature = "contract"))]
mod vm_state;

#[cfg(feature = "transaction")]
mod block;

#[cfg(feature = "serde")]
mod serde_value;

#[cfg(feature = "serde")]
mod serde_with_utils;

#[cfg(feature = "contract")]
mod stack_item;

#[cfg(any(feature = "contract", feature = "transaction"))]
mod path_or_string;

#[cfg(feature = "contract")]
mod plugin_type;

#[cfg(feature = "http-client")]
mod syncing;

#[cfg(feature = "http-client")]
mod tx_pool;

#[cfg(feature = "http-client")]
mod url_session;

// Type aliases
pub type Byte = u8;
pub type Bytes = Vec<u8>;
pub type TxHash = H256;

// Base64 utilities - only available with serde feature
#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
pub trait ExternBase64 {
	fn to_base64(&self) -> String;
}

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl ExternBase64 for String {
	fn to_base64(&self) -> String {
		general_purpose::STANDARD.encode(self.as_bytes())
	}
}

// Scrypt parameters - needed for wallet encryption
#[cfg(feature = "wallet")]
#[cfg_attr(docsrs, doc(cfg(feature = "wallet")))]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScryptParamsDef {
	#[serde(rename = "n")]
	pub log_n: u8,
	#[serde(rename = "r")]
	pub r: u32,
	#[serde(rename = "p")]
	pub p: u32,
}

#[cfg(feature = "wallet")]
#[cfg_attr(docsrs, doc(cfg(feature = "wallet")))]
impl Default for ScryptParamsDef {
	fn default() -> Self {
		Self { log_n: 12, r: 8, p: 8 }
	}
}

// Base64 encoding trait - only available with serde feature
#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
pub trait Base64Encode {
	fn to_base64(&self) -> String;
}

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl Base64Encode for Vec<u8> {
	fn to_base64(&self) -> String {
		general_purpose::STANDARD.encode(self)
	}
}

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl Base64Encode for &[u8] {
	fn to_base64(&self) -> String {
		general_purpose::STANDARD.encode(self)
	}
}

#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl Base64Encode for String {
	fn to_base64(&self) -> String {
		general_purpose::STANDARD.encode(self.as_bytes())
	}
}

// EIP-55 style checksum function - available with crypto-standard
#[cfg(feature = "crypto-standard")]
#[cfg_attr(docsrs, doc(cfg(feature = "crypto-standard")))]
pub fn to_checksum(addr: &ScriptHash, chain_id: Option<u8>) -> String {
	// Implementation details omitted for brevity
	unimplemented!("Implementation details omitted for brevity")
}

#[cfg(test)]
mod tests {
	use super::*;

	#[cfg(feature = "serde")]
	#[test]
	fn test_base64_encode_bytes() {
		let data = vec![1, 2, 3, 4];
		assert_eq!(data.to_base64(), "AQIDBA==");

		let data_slice: &[u8] = &[1, 2, 3, 4];
		assert_eq!(data_slice.to_base64(), "AQIDBA==");
	}

	#[cfg(feature = "serde")]
	#[test]
	fn test_base64_decode() {
		let encoded = "AQIDBA==";
		let decoded = general_purpose::STANDARD.decode(encoded).unwrap();
		assert_eq!(decoded, vec![1, 2, 3, 4]);
	}
}
