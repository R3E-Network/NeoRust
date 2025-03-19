/// # Neo SDK Prelude (v0.1.8)
///
/// Convenient imports for commonly used types and traits to make working with Neo more ergonomic.
///
/// This prelude module provides a single import to access the most commonly used
/// components of the NeoRust SDK. Import it with:
///
/// ```rust
/// use neo3::prelude::*;
/// ```
///
/// ## Included Categories
///
/// The prelude includes:
///
/// - **Core Types**: Basic blockchain primitives like Address, ScriptHash
/// - **Errors**: The unified NeoError type for error handling
/// - **Contracts**: Types for interacting with Neo smart contracts
/// - **Wallets**: Account and wallet management
/// - **Clients**: RPC and other client interfaces
/// - **Builders**: Transaction construction utilities
/// - **Extensions**: Utility traits and extensions
///
/// ## When to Use
///
/// The prelude is ideal for applications that use multiple Neo features.
/// For more targeted imports, you can import specific modules directly.
// Core error type
pub use crate::neo_error::NeoError;

// === Core Types ===
// Basic blockchain types
pub use crate::neo_types::{
	Address, AddressOrScriptHash, Base64Encode, Bytes, NameOrAddress, ScriptHash,
	ScriptHashExtension, StringExt, ToBase58,
};

// Contract-related types
pub use crate::neo_types::{
	ContractManifest, ContractParameter, ContractParameterType, ContractState, InvocationResult,
	NefFile,
};

// VM and runtime types
pub use crate::neo_types::{OpCode, StackItem, VMState};

// NNS-related types
pub use crate::neo_types::NNSName;

// Common external types
pub use primitive_types::{H160, H256, U256};
pub use serde_json::Value as ParameterValue;
pub use url::Url;

// === Serialization Helpers ===
pub use crate::neo_types::{
	// H160/H256 serialization
	deserialize_h160,
	deserialize_h256,
	// Other serialization helpers
	deserialize_script_hash,
	// U256 serialization
	deserialize_u256,
	deserialize_u64,
	deserialize_vec_h256,
	deserialize_vec_u256,
	deserialize_wildcard,
	serialize_h160,
	serialize_h256,
	serialize_script_hash,
	serialize_u256,
	serialize_u64,

	serialize_vec_h256,

	serialize_vec_u256,
	serialize_wildcard,
};

// === Core Functionality Modules ===
// These are aliased module names for user convenience
pub use crate::{
	neo_builder as builder, neo_clients as providers, neo_codec as codec, neo_config as config,
	neo_crypto as crypto, neo_protocol as protocol, neo_wallets as wallets, neo_x as x,
};

// === Extension modules ===
// These are full modules that provide specialized functionality
pub use crate::neo_fs; // NeoFS distributed storage

// Re-export ValueExtension
pub use crate::neo_types::ValueExtension;
