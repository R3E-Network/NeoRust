//! Common utilities and types for the NeoRust SDK.
//!
//! This crate provides common utilities and types that are used across the NeoRust SDK.

#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms, unreachable_pub)]
#![deny(rustdoc::broken_intra_doc_links)]

pub mod address_conversion;
pub mod address_or_scripthash_utils;
pub mod address_utils;
pub mod base64;
pub mod base64_encode;
pub mod base64_utils;
pub mod block_modes;
pub mod contract_parameter_type;
pub mod crypto_adapter;
pub mod crypto_utils;
pub mod error_adapter;
pub mod error_conversion;
pub mod h160_utils;
pub mod h256_utils;
pub mod h256_vec_utils;
pub mod hard_forks;
pub mod hard_forks_utils;
pub mod hashable;
pub mod invocation_script;
pub mod nep17_provider;
pub mod oracle_response_code;
pub mod public_key_utils;
pub mod role;
pub mod rpc_client_trait;
pub mod rpc_types;
pub mod script_hash_utils;
pub mod serde_utils;
pub mod transaction_attribute;
pub mod transaction_signer;
pub mod transaction_types;
pub mod vec_utils;
pub mod verification_script;
pub mod wallet;
pub mod witness;
pub mod witness_rule;
pub mod witness_scope;
// pub mod account_signer;

// Re-export key types from neo-error
pub use neo_error::provider_error::{
    ProviderError, 
    to_provider_error, 
    to_serialization_error, 
    to_network_error, 
    to_rpc_error
};

// Re-export HashableForVec trait from hashable module
pub use hashable::HashableForVec;

// Re-export specific types to avoid ambiguity
pub use address_conversion::{PublicKey, public_key_to_address, public_key_to_script_hash, secp256r1_public_key_to_script_hash};
pub use address_or_scripthash_utils::{deserialize_address_or_script_hash, serialize_address_or_script_hash};
pub use contract_parameter_type::ContractParameterType;
pub use crypto_adapter::{EncodablePublicKey, common_to_external_public_key, external_to_common_public_key};
pub use crypto_utils::{hash160, hash256, ripemd160, sha256};
pub use error_adapter::{
    provider_error_to_string, 
    ErrorAdapter,
};
pub use error_conversion::ProviderErrorConversion;
pub use h256_utils::{deserialize_h256, deserialize_h256_option, serialize_h256, serialize_h256_option};
pub use h256_vec_utils::{deserialize_vec_h256, serialize_vec_h256};
pub use hard_forks::HardForks;
pub use hard_forks_utils::deserialize_hardforks;
pub use nep17_provider::Nep17BalanceProvider;
pub use oracle_response_code::OracleResponseCode;
pub use rpc_client_trait::RpcClient;
pub use rpc_types::{BasicJsonRpcProvider, JsonRpcProvider};
pub use script_hash_utils::{deserialize_script_hash, serialize_script_hash};
pub use transaction_attribute::{TransactionAttribute, TransactionAttributeType};
pub use transaction_signer::{Signer, TransactionSigner};
pub use transaction_types::TransactionType;
pub use vec_utils::vec_to_array32;
pub use verification_script::VerificationScript;
pub use wallet::Wallet;
pub use witness::Witness;
pub use witness_rule::{WitnessAction, WitnessCondition, WitnessRule};
pub use witness_scope::{WitnessScope, WitnessScopeType};
