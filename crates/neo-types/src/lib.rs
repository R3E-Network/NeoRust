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

// Re-export TypeError from neo-error
pub use neo_error::type_error::TypeError;

mod address;
mod address_or_scripthash;
mod base64_utils;
mod block;
mod bytes;
mod contract;
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

use ::base64::engine::general_purpose;
use ::base64::Engine;
use serde::{Deserialize, Serialize};
// Re-export all public items with selective imports to avoid ambiguity
pub use address::*;
pub use address_or_scripthash::*;
pub use base64_utils::{decode, encode};
pub use block::*;
pub use bytes::*;
pub use contract::*;
pub use nns::*;
pub use numeric::*;
pub use op_code::*;
pub use path_or_string::*;
pub use plugin_type::*;
pub use script_hash::*;
pub use script_hash_impl::*;
// Re-export serde utility functions
pub use serde_utils::*;
// Import from neo_common instead of serde_with_utils
pub use neo_common::{WitnessScope, HardForks};
pub use stack_item::*;
// Resolve StringExt ambiguity
// pub use string_ext::*; // Commented out to avoid StringExt conflict
pub use syncing::*;
pub use tx_pool::*;
pub use url_session::*;
// Selectively import from util to avoid conflicts
pub use util::*;

// Re-export OpCode from neo-codec
pub use neo_codec::OpCode;

// Add import for neo-common base64 utilities
use neo_common::base64;

pub trait ExternBase64 {
    fn to_base64(&self) -> String;
}

impl ExternBase64 for String {
    fn to_base64(&self) -> String {
        general_purpose::STANDARD.encode(self.as_bytes())
    }
}

// ScryptParams
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ScryptParamsDef {
    #[serde(rename = "n")]
    pub log_n: u8,
    #[serde(rename = "r")]
    pub r: u32,
    #[serde(rename = "p")]
    pub p: u32,
}

impl Default for ScryptParamsDef {
    fn default() -> Self {
        Self { log_n: 14, r: 8, p: 8 }
    }
}

// Extend Vec<u8> with a to_base64 method
pub trait Base64Encode {
    fn to_base64(&self) -> String;
}

impl Base64Encode for Vec<u8> {
    fn to_base64(&self) -> String {
        base64::encode(&self)
    }
}

impl Base64Encode for &[u8] {
    fn to_base64(&self) -> String {
        base64::encode(&self)
    }
}

impl Base64Encode for String {
    fn to_base64(&self) -> String {
        match hex::decode(self) {
            Ok(bytes) => general_purpose::STANDARD.encode(&bytes),
            Err(_) => {
                // If hex decoding fails, return an empty string
                // In a real error handling scenario, we would return a Result
                eprintln!("Failed to decode hex string: {}", self);
                String::new()
            },
        }
    }
}

// pub fn secret_key_to_script_hash(secret_key: &Secp256r1PrivateKey) -> ScriptHash {
// 	let public_key = secret_key.to_public_key();
// 	public_key_to_script_hash(&public_key)
// }

// pub fn public_key_to_script_hash(pubkey: &Secp256r1PublicKey) -> ScriptHash {
// 	raw_public_key_to_script_hash(&pubkey.get_encoded(true)[1..])
// }
//
// pub fn raw_public_key_to_script_hash<T: AsRef<[u8]>>(pubkey: T) -> ScriptHash {
// 	let pubkey = pubkey.as_ref();
// 	let script = format!(
// 		"{}21{}{}{}{}",
// 		OpCode::PushData1.to_string(),
// 		"03",
// 		pubkey.to_hex(),
// 		OpCode::Syscall.to_string(),
// 		InteropService::SystemCryptoCheckSig.hash()
// 	)
// 	.from_hex()
// 	.unwrap();
// 	let mut script = script.sha256_ripemd160();
// 	script.reverse();
// 	ScriptHash::from_slice(&script)
// }

pub fn to_checksum(addr: &ScriptHash, chain_id: Option<u8>) -> String {
    // if !addr.is_valid_address(){
    // 	panic!("invalid address");
    // }
    let prefixed_addr = match chain_id {
        Some(chain_id) => format!("{chain_id}0x{addr:x}"),
        None => format!("{addr:x}"),
    };
    let hash = hex::encode(prefixed_addr);
    let hash = hash.as_bytes();

    let addr_hex = hex::encode(addr.as_bytes());
    let addr_hex = addr_hex.as_bytes();

    addr_hex.iter().zip(hash).fold("0x".to_owned(), |mut encoded, (addr, hash)| {
        encoded.push(if *hash >= 56 {
            addr.to_ascii_uppercase() as char
        } else {
            addr.to_ascii_lowercase() as char
        });
        encoded
    })
}

#[cfg(test)]
mod tests {
    use hex;
    use rustc_serialize::base64::FromBase64;

    use super::*;

    #[test]
    fn test_base64_encode_bytes() {
        let input = hex::decode("150c14242dbf5e2f6ac2568b59b7822278d571b75f17be0c14242dbf5e2f6ac2568b59b7822278d571b75f17be13c00c087472616e736665720c14897720d8cd76f4f00abfa37c0edd889c208fde9b41627d5b5238").unwrap();
        let expected = "FQwUJC2/Xi9qwlaLWbeCInjVcbdfF74MFCQtv14vasJWi1m3giJ41XG3Xxe+E8AMCHRyYW5zZmVyDBSJdyDYzXb08Aq/o3wO3YicII/em0FifVtSOA==";

        let encoded = input.to_base64();

        assert_eq!(encoded, expected);
    }

    #[test]
    fn test_base64_decode() {
        let encoded = "FQwUJC2/Xi9qwlaLWbeCInjVcbdfF74MFCQtv14vasJWi1m3giJ41XG3Xxe+E8AMCHRyYW5zZmVyDBSJdyDYzXb08Aq/o3wO3YicII/em0FifVtSOA==";
        let expected = "150c14242dbf5e2f6ac2568b59b7822278d571b75f17be0c14242dbf5e2f6ac2568b59b7822278d571b75f17be13c00c087472616e736665720c14897720d8cd76f4f00abfa37c0edd889c208fde9b41627d5b5238";

        let decoded = encoded.from_base64().unwrap();
        let decoded_hex = hex::encode(decoded);

        assert_eq!(decoded_hex, expected);
    }
}

// Re-export serialization functions from serde_with_utils
pub use serde_with_utils::{
    deserialize_h160, deserialize_h256, deserialize_script_hash, deserialize_u256, deserialize_u64,
    deserialize_vec_h256, deserialize_vec_u256, deserialize_wildcard, serialize_h160,
    serialize_h256, serialize_script_hash, serialize_u256, serialize_u64, serialize_vec_h256,
    serialize_vec_u256, serialize_wildcard,
};

// Re-export ValueExtension trait
pub use serde_value::ValueExtension;

// Re-export address types
pub use address::NameOrAddress;

// Other re-exports
pub use string::StringExt;
pub use vm_state::VMState;
