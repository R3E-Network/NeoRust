//! # Neo Common
//!
//! Common types and utilities shared across NeoRust SDK crates.
//!
//! This crate provides shared functionality to avoid circular dependencies between other crates in the NeoRust SDK.
//!
//! ## Features
//!
//! - Serialization utilities for Neo types
//! - Common error types
//! - Shared enums and constants
//! - Base64 encoding/decoding utilities
//! - Common traits used across multiple crates

#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms, unreachable_pub)]
#![deny(rustdoc::broken_intra_doc_links)]

pub mod address_or_scripthash_utils;
pub mod address_utils;
pub mod base64;
pub mod base64_encode;
pub mod base64_utils;
pub mod block_modes;
pub mod contract_parameter_type;
pub mod crypto_adapter;
pub mod error_adapter;
pub mod error_conversion;
pub mod h160_utils;
pub mod address_conversion;
pub mod h256_utils;
pub mod h256_vec_utils;
pub mod hard_forks;
pub mod hard_forks_utils;
pub mod hashable;
pub mod invocation_script;
pub mod oracle_response_code;
pub mod public_key_utils;
pub mod role;
pub mod rpc_client_trait;
pub mod nep17_provider;
pub mod script_hash_utils;
pub mod serde_utils;
pub mod provider_error;
pub mod rpc_types;
pub mod transaction_attribute;
pub mod transaction_types;
pub mod vec_utils;
pub mod witness_scope;
pub mod verification_script;
pub mod witness;

// Re-export all public items
pub use address_conversion::{PublicKey, public_key_to_address, public_key_to_script_hash, hash256, ripemd160, base58check_encode};
pub use address_or_scripthash_utils::{deserialize_address_or_script_hash, serialize_address_or_script_hash};
pub use base64::*;
pub use base64_encode::Base64Encode;
pub use base64_utils::{Base64Decode};
pub use contract_parameter_type::ContractParameterType;
pub use crypto_adapter::{EncodablePublicKey, external_to_common_public_key, common_to_external_public_key};
pub use error_adapter::*;
pub use error_conversion::{IntoProviderError, FromProviderError};
pub use h160_utils::{deserialize_h160, deserialize_h160_option, serialize_h160, serialize_h160_option};
pub use h256_utils::{deserialize_h256, deserialize_h256_option, serialize_h256, serialize_h256_option};
pub use h256_vec_utils::{deserialize_vec_h256, serialize_vec_h256};
pub use block_modes::{encrypt_aes256_ecb, decrypt_aes256_ecb, Ecb, NoPadding};
pub use hard_forks::HardForks;
pub use hard_forks_utils::deserialize_hardforks;
pub use hashable::HashableForVec;
pub use invocation_script::InvocationScript;
pub use oracle_response_code::OracleResponseCode;
pub use public_key_utils::{deserialize_public_key_option, serialize_public_key_option};
pub use role::Role;
pub use rpc_client_trait::{NeoVersion, ProtocolSettings};
pub use nep17_provider::{Nep17BalanceProvider, Nep17BalancesResponse, Nep17Balance};
pub use script_hash_utils::{deserialize_script_hash, deserialize_vec_script_hash, serialize_script_hash, serialize_vec_script_hash};
pub use serde_utils::*;
pub use provider_error::ProviderError;
pub use rpc_types::{JsonRpcProvider, RpcClient, HttpProvider, WebSocketProvider, IpcProvider};
pub use transaction_attribute::TransactionAttributeType;
pub use transaction_types::{TransactionSigner, Signer, TransactionSendToken, WitnessRule, WitnessAction, WitnessCondition, WitnessScope as WitnessScopeType};
pub use vec_utils::vec_to_array32;
pub use verification_script::VerificationScript;
pub use witness::Witness;
pub use witness_scope::WitnessScope;
