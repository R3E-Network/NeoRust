//! # Neo SDK Prelude (v0.1.9)
//!
//! Convenient imports for commonly used types and traits to make working with Neo more ergonomic.
//!
//! This prelude module provides a single import to access the most commonly used
//! components of the NeoRust SDK. Import it with:
//!
//! ```rust
//! use neo3::prelude::*;
//! ```
//!
//! ## Included Categories
//!
//! The prelude includes:
//!
//! - **Core Types**: Basic blockchain primitives like Address, ScriptHash
//! - **Errors**: The unified NeoError type for error handling
//! - **Contracts**: Types for interacting with Neo smart contracts
//! - **Wallets**: Account and wallet management
//! - **Clients**: RPC and other client interfaces
//! - **Builders**: Transaction construction utilities
//! - **Extensions**: Utility traits and extensions
//!
//! ## When to Use
//!
//! The prelude is ideal for applications that use multiple Neo features.
//! For more targeted imports, you can import specific modules directly.

// Core error type
pub use neo_error::NeoError;

// === Core Types ===
// Basic blockchain types
pub use neo_types::{
    Address, AddressOrScriptHash, Base64Encode, Bytes, NameOrAddress, ScriptHash,
    ScriptHashExtension, StringExt, ToBase58,
};

// Contract-related types
pub use neo_types::{
    ContractManifest, ContractParameter, ContractParameterType, ContractState, InvocationResult,
    NefFile,
};

// VM and runtime types
pub use neo_types::{OpCode, StackItem, VMState};

// NNS-related types
pub use neo_types::NNSName;

// Common external types
pub use primitive_types::{H160, H256, U256};
pub use serde_json::Value as ParameterValue;
pub use url::Url;

// === Serialization Helpers ===
pub use neo_types::{
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

// === Builder Module ===
pub use neo_builder::{
    AccountSigner, ContractSigner, InvocationScript, ScriptBuilder, Signer, Transaction,
    TransactionAttribute, TransactionBuilder, VerificationScript, Witness, WitnessCondition,
    WitnessRule,
};

// === Clients Module ===
pub use neo_clients::{
    APITrait, HttpProvider, JsonRpcProvider, ProviderError, RpcClient, WebSocketProvider,
};

// === Contract Module ===
pub use neo_contract::{
    ContractError, FungibleTokenContract, GasToken, NeoNameService, NeoToken, NftContract,
    SmartContract, SmartContractTrait,
};

// === Protocol Module ===
pub use neo_protocol::{Account, AccountTrait};

// === Wallets Module ===
pub use neo_wallets::{Wallet, WalletError, WalletSigner, WalletTrait};

// === Neo X Module ===
pub use neo_x::{
    bridge::NeoXBridgeContract,
    evm::{NeoXProvider, NeoXTransaction},
};

// Re-export ValueExtension
pub use neo_types::ValueExtension;
