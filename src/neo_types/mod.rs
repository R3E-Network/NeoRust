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
use primitive_types::{H160, H256};
use std::collections::HashMap;

// Add dedicated import for wallet-related derives
#[cfg(feature = "wallet")]
use serde_derive::{Deserialize, Serialize};

#[cfg(feature = "serde")]
use serde_derive::{Deserialize, Serialize};

// Core types - always available
pub use address::*;
pub use address_or_scripthash::*;
pub use bytes::*;
pub use error::*;
pub use numeric::*;
pub use script_hash::*;
pub use string::*;
pub use util::*;
pub use constants::*;

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
pub mod serde_with_utils;

// Contract-related types
#[cfg(feature = "contract")]
#[cfg_attr(docsrs, doc(cfg(feature = "contract")))]
pub use contract::*;

#[cfg(feature = "contract")]
pub mod stack_item;

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
pub mod invocation;
mod constants;
pub mod contract;

#[cfg(feature = "http-client")]
mod nns;

// Core modules - always available
pub mod address;
pub mod address_or_scripthash;
pub mod bytes;
pub mod error;
pub mod numeric;
pub mod script_hash;
pub mod string;
pub mod util;

// Conditional modules
#[cfg(any(feature = "transaction", feature = "contract"))]
pub mod op_code;

pub mod vm_state;

#[cfg(feature = "transaction")]
mod block;

#[cfg(feature = "serde")]
mod serde_value;

#[cfg(feature = "serde")]
pub mod serde_with_utils;

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

// Instead of creating a separate module, define ScryptParamsDef directly here
#[cfg(feature = "wallet")]
#[cfg_attr(docsrs, doc(cfg(feature = "wallet")))]
#[derive(Debug, Clone, PartialEq, Eq, serde_derive::Serialize, serde_derive::Deserialize)]
pub struct ScryptParamsDef {
	/// The number of iterations (N value)
	pub n: u32,
	/// The block size factor (r value)
	pub r: u32,
	/// The parallelization factor (p value)
	pub p: u32,
}

#[cfg(feature = "wallet")]
impl Default for ScryptParamsDef {
	fn default() -> Self {
		Self { n: 16384, r: 8, p: 8 }
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

// Add missing Block, Transaction and related types
#[cfg(any(feature = "rest-client", feature = "websocket"))]
pub mod block {
	use crate::neo_types::script_hash::ScriptHash;
	use primitive_types::H256;
	use serde::{Deserialize, Serialize};

	/// Represents a Neo N3 block
	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub struct Block {
		/// Block hash
		pub hash: H256,
		/// Block size in bytes
		pub size: u32,
		/// Block version
		pub version: u32,
		/// Previous block hash
		pub previousblockhash: H256,
		/// Merkle root hash
		pub merkleroot: H256,
		/// Block timestamp
		pub time: u64,
		/// Block index (height)
		pub index: u32,
		/// Primary nonce
		pub nonce: String,
		/// Next consensus address
		pub nextconsensus: String,
		/// List of transaction hashes in this block
		pub tx: Vec<H256>,
		/// Block confirmations
		pub confirmations: u32,
		/// Next block hash
		pub nextblockhash: Option<H256>,
	}
}

#[cfg(any(feature = "rest-client", feature = "websocket"))]
pub mod transaction {
	use crate::neo_types::script_hash::ScriptHash;
	use primitive_types::H256;
	use serde::{Deserialize, Serialize};

	/// Represents a Neo N3 transaction
	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub struct Transaction {
		/// Transaction hash
		pub hash: H256,
		/// Transaction size in bytes
		pub size: u32,
		/// Transaction version
		pub version: u8,
		/// Transaction nonce
		pub nonce: u32,
		/// Sender account
		pub sender: String,
		/// System fee
		pub sysfee: String,
		/// Network fee
		pub netfee: String,
		/// Transaction validity period (block index)
		pub validuntilblock: u32,
		/// List of signers
		pub signers: Vec<Signer>,
		/// List of transaction attributes
		pub attributes: Vec<Attribute>,
		/// Script of the transaction
		pub script: String,
		/// List of witnesses
		pub witnesses: Vec<Witness>,
		/// Block hash this transaction is in
		pub blockhash: Option<H256>,
		/// Block time this transaction is in
		pub blocktime: Option<u64>,
		/// Transaction confirmations
		pub confirmations: u32,
	}

	/// Represents a transaction signer
	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub struct Signer {
		/// Signer account
		pub account: String,
		/// Signer scope
		pub scopes: String,
		/// Allowed contract hashes (when `CustomContracts` scope is used)
		#[serde(skip_serializing_if = "Option::is_none")]
		pub allowedcontracts: Option<Vec<String>>,
		/// Allowed groups (when `CustomGroups` scope is used)
		#[serde(skip_serializing_if = "Option::is_none")]
		pub allowedgroups: Option<Vec<String>>,
	}

	/// Transaction attribute
	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub struct Attribute {
		/// Attribute type
		pub type_: String,
		/// Attribute value
		pub value: String,
	}

	/// Transaction witness
	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub struct Witness {
		/// Witness invocation script
		pub invocation: String,
		/// Witness verification script
		pub verification: String,
	}
}

#[cfg(any(feature = "rest-client", feature = "websocket"))]
pub mod application_log {
	use primitive_types::H256;
	use serde::{Deserialize, Serialize};
	use serde_json::Value;

	/// Represents application execution logs
	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub struct ApplicationLog {
		/// Transaction hash
		pub txid: H256,
		/// Block hash
		pub blockhash: H256,
		/// Execution results
		pub executions: Vec<ExecutionResult>,
	}

	/// Execution result
	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub struct ExecutionResult {
		/// Trigger type
		pub trigger: String,
		/// VM state
		pub vmstate: String,
		/// Exception message if execution failed
		pub exception: Option<String>,
		/// Gas consumed
		pub gasconsumed: String,
		/// Stack items
		pub stack: Vec<Value>,
		/// Notifications
		pub notifications: Vec<Notification>,
	}

	/// Neo N3 notification
	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub struct Notification {
		/// Contract hash
		pub contract: String,
		/// Event name
		pub eventname: String,
		/// Event state
		pub state: Value,
	}
}

#[cfg(any(feature = "rest-client", feature = "websocket"))]
pub mod contract_state {
	use serde::{Deserialize, Serialize};
	
	/// Neo N3 contract state
	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub struct ContractState {
		/// Contract hash
		pub hash: String,
		/// Contract NEF (Neo Executable Format)
		pub nef: Nef,
		/// Contract manifest
		pub manifest: Manifest,
	}

	/// Neo NEF (Neo Executable Format)
	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub struct Nef {
		/// NEF magic number
		pub magic: u32,
		/// NEF compiler
		pub compiler: String,
		/// NEF source
		pub source: String,
		/// Token count
		pub tokens: Vec<String>,
		/// Script hash
		pub script: String,
		/// NEF checksum
		pub checksum: u32,
	}

	/// Neo contract manifest
	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub struct Manifest {
		/// Contract name
		pub name: String,
		/// Contract groups
		pub groups: Vec<Group>,
		/// Contract features
		pub features: serde_json::Value,
		/// Supported standards
		pub supportedstandards: Vec<String>,
		/// ABI (Application Binary Interface)
		pub abi: Abi,
		/// Contract permissions
		pub permissions: Vec<Permission>,
		/// Contract trusts
		pub trusts: Vec<String>,
		/// Additional contract data
		pub extra: Option<serde_json::Value>,
	}

	/// Neo contract group
	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub struct Group {
		/// Public key
		pub pubkey: String,
		/// Signature
		pub signature: String,
	}

	/// Neo contract ABI
	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub struct Abi {
		/// Contract methods
		pub methods: Vec<Method>,
		/// Contract events
		pub events: Vec<Event>,
	}

	/// Neo contract method
	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub struct Method {
		/// Method name
		pub name: String,
		/// Method parameters
		pub parameters: Vec<Parameter>,
		/// Method return type
		pub returntype: String,
		/// Method offset
		pub offset: u32,
		/// Is method safe (read-only)
		pub safe: bool,
	}

	/// Neo contract event
	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub struct Event {
		/// Event name
		pub name: String,
		/// Event parameters
		pub parameters: Vec<Parameter>,
	}

	/// Neo contract parameter
	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub struct Parameter {
		/// Parameter name
		pub name: String,
		/// Parameter type
		pub type_: String,
	}

	/// Neo contract permission
	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub struct Permission {
		/// Contract address
		pub contract: String,
		/// Allowed methods
		pub methods: Vec<String>,
	}
}

// Re-export types conditionally
#[cfg(any(feature = "rest-client", feature = "websocket"))]
pub use block::Block;
#[cfg(any(feature = "rest-client", feature = "websocket"))]
pub use transaction::Transaction;
#[cfg(any(feature = "rest-client", feature = "websocket"))]
pub use application_log::ApplicationLog;
#[cfg(any(feature = "rest-client", feature = "websocket"))]
pub use contract_state::ContractState;
#[cfg(feature = "websocket")]
pub mod notification {
	use serde::{Deserialize, Serialize};
	use serde_json::Value;

	/// Neo N3 notification
	#[derive(Debug, Clone, Serialize, Deserialize)]
	pub struct Notification {
		/// Contract hash
		pub contract: String,
		/// Event name
		pub eventname: String,
		/// Event state
		pub state: Value,
	}
}
#[cfg(feature = "websocket")]
pub use notification::Notification;

// We need to import or forward-declare ContractParameter if it's not imported yet
#[cfg(feature = "contract")]
use crate::neo_contract::contract_parameter::ContractParameter;
#[cfg(not(feature = "contract"))]
/// A forward declaration for ContractParameter when contract feature is not enabled
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ContractParameter;

/// Enum for parameter values used in script building
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParameterValue {
	/// A boolean value
	Bool(bool),
	/// An integer value
	Integer(i64),
	/// A byte array value
	ByteArray(Vec<u8>),
	/// A string value
	String(String),
	/// A H160 (usually script hash) value
	H160(H160),
	/// A H256 (usually transaction hash) value
	H256(H256),
	/// A public key value
	PublicKey(Vec<u8>),
	/// A signature value
	Signature(Vec<u8>),
	/// An array of values
	Array(Vec<ContractParameter>),
	/// A map of key-value pairs
	Map(Box<HashMap<ContractParameter, ContractParameter>>),
}

#[cfg(feature = "transaction")]
mod block;

#[cfg(any(feature = "transaction", feature = "contract"))]
pub use invocation::*;
