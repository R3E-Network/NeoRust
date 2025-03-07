//! # Neo Types
//!
//! Core Neo ecosystem data types for the NeoRust SDK.
//!
//! This crate provides the fundamental data types used in the Neo N3 blockchain ecosystem, including:
//!
//! - Script hashes and addresses
//! - Contract parameters and manifests
//! - Block and transaction types
//! - Stack items and VM state
//! - Cryptographic types
//! - Serialization utilities
//! - NNS name types
//!
//! ## Usage
//!
//! ```rust,ignore
//! use neo_types::{ScriptHash, Address, ContractParameter};
//! use std::str::FromStr;
//!
//! // Create a script hash from a string
//! let script_hash = ScriptHash::from_str("0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5").unwrap();
//!
//! // Convert between script hash and address
//! let address = Address::from_script_hash(&script_hash);
//! let script_hash_from_address = address.to_script_hash();
//!
//! // Create contract parameters
//! let param = ContractParameter::integer(42);
//! let string_param = ContractParameter::string("Hello, Neo!");
//! ```

#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms, unreachable_pub)]
#![deny(rustdoc::broken_intra_doc_links)]

mod address;
mod address_or_scripthash;
mod base64_utils;
mod block;
mod bytes;
mod contract;
mod error;
mod nns;
mod numeric;
mod op_code;
mod path_or_string;
mod plugin_type;
mod script_hash;
mod script_hash_extension;
mod script_hash_impl;
pub mod serde_utils;
mod serde_value;
mod serde_with_utils;
mod stack_item;
mod string;
mod string_ext;
mod syncing;
mod tx_pool;
mod url_session;
mod util;
mod vm_state;

// Re-export all public items with selective imports to avoid ambiguity
pub use address::*;
pub use address_or_scripthash::*;
pub use base64_utils::{decode, encode};
pub use block::*;
pub use bytes::*;
pub use contract::*;
pub use error::*;
pub use nns::*;
pub use numeric::*;
pub use op_code::*;
pub use path_or_string::*;
pub use plugin_type::*;
pub use script_hash::*;
pub use script_hash_impl::*;
// Re-export serde utility functions
pub use serde_utils::*;
pub use serde_value::ValueExtension;
// Selectively import from serde_with_utils to avoid conflicts
pub use serde_with_utils::{WitnessScope, HardForks};
pub use stack_item::*;
// Resolve StringExt ambiguity
pub use string::StringExt;
// pub use string_ext::*; // Commented out to avoid StringExt conflict
pub use syncing::*;
pub use tx_pool::*;
pub use url_session::*;
// Selectively import from util to avoid conflicts
pub use util::ToBase64;
pub use base64_utils::FromBase64;
pub use vm_state::*;

// Re-export OpCode from neo-codec
pub use neo_codec::OpCode;
