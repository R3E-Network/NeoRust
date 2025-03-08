//! # Neo Types (v0.1.9)
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
//! // Create a parameter array for a contract invocation
//! let params = vec![string_param, integer_param, bool_param];
//! ```
//!
//! ### Working with stack items
//!
//! ```rust
//! use neo::prelude::*;
//! use serde_json::json;
//!
//! // Create stack items of various types
//! let int_item = StackItem::integer(123);
//! let bool_item = StackItem::bool(true);
//! let bytes_item = StackItem::byte_string(b"Neo");
//!
//! // Convert between stack items and JSON values
//! let json_value = int_item.to_json_value();
//! assert_eq!(json_value, json!(123));
//! ```

use base64::{engine::general_purpose, Engine};
pub use log::*;
use primitive_types::H256;
use serde_derive::{Deserialize, Serialize};

// Re-export everything from these modules
pub use address::*;
pub use address_or_scripthash::*;
pub use block::*;
pub use bytes::*;
pub use contract::*;
pub use error::*;
pub use nns::*;
pub use numeric::*;
pub use op_code::*;
pub use path_or_string::*;
pub use plugin_type::*;
pub mod script_hash;
pub use script_hash::{ScriptHash, ScriptHashExtension};
pub use serde_value::*;
pub use serde_with_utils::*;
pub use stack_item::*;
pub use string::*;
pub use syncing::*;
pub use tx_pool::*;
pub use url_session::*;
pub use util::*;
pub use vm_state::*;

// Make modules public for direct access
pub mod contract;
pub mod error;
pub mod nns;
pub mod serde_value;
pub mod serde_with_utils;

mod address;
mod address_or_scripthash;
mod block;
mod bytes;
mod numeric;
mod op_code;
mod path_or_string;
mod plugin_type;
mod stack_item;
mod string;
mod syncing;
mod tx_pool;
mod url_session;
mod util;
mod vm_state;

pub type Byte = u8;
pub type Bytes = Vec<u8>;
pub type TxHash = H256;
