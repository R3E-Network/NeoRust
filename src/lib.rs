#![allow(warnings)]

//! # NeoRust SDK v0.1.4
//!
//! A comprehensive Rust library for building applications on the Neo N3 blockchain ecosystem.
//!
//! ## Overview
//!
//! NeoRust is a complete SDK designed to make Neo N3 blockchain development in Rust
//! intuitive, type-safe, and productive. The library provides full support for all
//! Neo N3 features and follows Rust best practices for reliability and performance.
//!
//! ## Key Features
//!
//! - **Complete Blockchain Coverage**: Full support for all Neo N3 features
//! - **Developer Experience**: Intuitive, type-safe APIs with comprehensive documentation
//! - **Modular Architecture**: Well-organized components with clear separation of concerns
//! - **Performance-Optimized**: Efficient implementations with minimal dependencies
//! - **Security-Focused**: Secure by default with audited cryptographic operations
//! - **Cross-Platform**: Works on desktop, server, and WebAssembly environments
//! - **Async-First**: Built for the asynchronous Rust ecosystem
//!
//! ## Core Modules
//!
//! NeoRust is organized into specialized modules, each handling specific aspects of Neo N3:
//!
//! - [**neo_builder**](neo_builder): Transaction construction and script building
//! - [**neo_clients**](neo_clients): Neo node interaction and RPC client implementations
//! - [**neo_codec**](neo_codec): Serialization and deserialization of Neo data structures
//! - [**neo_config**](neo_config): Configuration for networks and client settings
//! - [**neo_contract**](neo_contract): Smart contract interaction and token standards
//! - [**neo_crypto**](neo_crypto): Cryptographic primitives and operations
//! - [**neo_error**](neo_error): Unified error handling
//! - [**neo_fs**](neo_fs): NeoFS distributed storage system integration
//! - [**neo_protocol**](neo_protocol): Core blockchain protocol implementations
//! - [**neo_sgx**](neo_sgx): Intel SGX secure enclave integration
//! - [**neo_types**](neo_types): Fundamental blockchain data types
//! - [**neo_utils**](neo_utils): Utility functions and helpers
//! - [**neo_wallets**](neo_wallets): Wallet management and key security
//! - [**neo_x**](neo_x): Neo X EVM-compatible chain support
//!
//! ## Quick Start
//!
//! Import all essential types and traits using the `prelude`:
//!
//! ```rust
//! use neo::prelude::*;
//! ```
//!
//! ## Getting Started with NeoRust
//!
//! ### Connect to a Neo N3 Node
//!
//! ```rust
//! use neo::prelude::*;
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!    // Connect to a Neo N3 TestNet node
//!    let provider = neo_providers::JsonRpcClient::new("https://testnet1.neo.coz.io:443");
//!    
//!    // Get basic blockchain information
//!    let block_count = provider.get_block_count().await?;
//!    println!("Current block height: {}", block_count);
//!    
//!    // Get the latest block
//!    let latest_block = provider.get_block_by_index(block_count - 1, 1).await?;
//!    println!("Latest block hash: {}", latest_block.hash);
//!    
//!    Ok(())
//! }
//! ```
//!
//! ## Feature Flags
//!
//! NeoRust is designed with a flexible feature system that allows you to include only the 
//! functionality you need. This reduces compile times and binary sizes. The features 
//! are grouped as follows:
//! 
//! ### Core Features (Default)
//! 
//! * `std` - Standard library support
//! * `crypto-standard` - Cryptographic primitives for Neo N3
//! 
//! ### Transport Layer Features
//! 
//! * `http-client` - HTTP JSON-RPC client for Neo N3 nodes
//! * `websocket` - WebSocket client for real-time blockchain events
//! * `rest-client` - RESTful API client for Neo N3 nodes
//! 
//! ### Blockchain Features
//! 
//! * `transaction` - Support for creating and signing transactions
//! * `wallet` - Core wallet management functionality
//! * `wallet-standard` - Enhanced wallet features with standard formats
//! * `wallet-hardware` - Hardware wallet support
//! * `wallet-secure` - Advanced security features for wallets
//! * `contract` - Smart contract interaction tools
//! 
//! ### Integration Features
//! 
//! * `ethereum-compat` - Ethereum compatibility layer
//! * `ledger` - Ledger hardware wallet support
//! * `neofs` - NeoFS distributed storage support
//! * `sgx` - Secure enclave integration
//! 
//! ### Hash Features
//! 
//! * `ripemd160` - RIPEMD-160 hash function
//! * `sha2` - SHA2 hash functions
//! * `digest` - Core digest traits
//! 
//! ### Example Usage
//! 
//! Basic JSON-RPC client:
//! ```toml
//! neo3 = { version = "0.1.3", features = ["http-client"] }
//! ```
//! 
//! Full wallet and transaction support:
//! ```toml
//! neo3 = { version = "0.1.3", features = ["http-client", "wallet", "transaction"] }
//! ```
//! 
//! Hardware wallet support:
//! ```toml
//! neo3 = { version = "0.1.3", features = ["wallet-hardware"] }
//! ```
//! 
//! ## Feature Dependencies
//! 
//! Features have been designed to minimize circular dependencies:
//! 
//! - `contract` requires `transaction`
//! - `wallet-standard` builds on `wallet`
//! - `wallet-hardware` builds on `wallet`
//! - All wallet features require `crypto-standard`
//! - `transaction` requires `crypto-standard`
//!
//! The `wallet` and `transaction` features are now independent, but together they provide 
//! additional functionality like wallet-signed transactions. See the WALLET_FEATURES.md
//! document for detailed information about the wallet feature hierarchy and how to avoid
//! circular dependencies.
//!
//! See the README.md for a complete list of features and usage examples.
//!
//! ## Usage Examples
//!
//! ### Connecting to a Neo N3 node
//!
//! ```rust
//! use neo::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Connect to Neo N3 MainNet
//!     let provider = HttpProvider::new("https://mainnet1.neo.org:443");
//!     let client = RpcClient::new(provider);
//!     
//!     // Get basic blockchain information
//!     let block_count = client.get_block_count().await?;
//!     println!("Current block count: {}", block_count);
//!     
//!     let version = client.get_version().await?;
//!     println!("Node version: {}", version.user_agent);
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### Creating and sending a transaction
//!
//! ```rust
//! use neo::prelude::*;
//! use std::str::FromStr;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize the JSON-RPC provider
//!     let provider = HttpProvider::new("https://testnet1.neo.org:443");
//!     let client = RpcClient::new(provider);
//!
//!     // Create accounts for the sender and recipient
//!     let sender = Account::from_wif("YOUR_SENDER_WIF_HERE")?;
//!     let recipient = ScriptHash::from_address("NbTiM6h8r99kpRtb428XcsUk1TzKed2gTc")?;
//!
//!     // Get the GAS token contract
//!     let gas_token_hash = ScriptHash::from_str("d2a4cff31913016155e38e474a2c06d08be276cf")?;
//!     
//!     // Build the transaction using the ScriptBuilder
//!     let script = ScriptBuilder::new()
//!         .contract_call(
//!             &gas_token_hash,
//!             "transfer",
//!             &[
//!                 ContractParameter::hash160(&sender.get_script_hash()),
//!                 ContractParameter::hash160(&recipient),
//!                 ContractParameter::integer(1_0000_0000), // 1 GAS (8 decimals)
//!                 ContractParameter::any(None),
//!             ],
//!             None,
//!         )?
//!         .to_bytes();
//!     
//!     // Create and configure the transaction
//!     let mut tx_builder = TransactionBuilder::with_client(&client);
//!     tx_builder
//!         .script(Some(script))
//!         .set_signers(vec![sender.clone().into()])
//!         .valid_until_block(client.get_block_count().await? + 5760)?; // Valid for ~1 day
//!
//!     // Sign the transaction
//!     let tx = tx_builder.sign().await?;
//!
//!     // Send the transaction
//!     let result = tx.send_tx().await?;
//!     println!("Transaction sent: {}", result.hash);
//!
//!     // Wait for the transaction to be confirmed
//!     println!("Waiting for confirmation...");
//!     tx.track_tx(10).await?;
//!     println!("Transaction confirmed!");
//!
//!     // Get the application log
//!     let app_log = tx.get_application_log(&client).await?;
//!     println!("Application log: {:?}", app_log);
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Interacting with a smart contract
//!
//! ```rust
//! use neo::prelude::*;
//! use std::str::FromStr;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Connect to Neo N3 TestNet
//!     let provider = HttpProvider::new("https://testnet1.neo.org:443");
//!     let client = RpcClient::new(provider);
//!     
//!     // Define the smart contract to interact with (GAS token)
//!     let contract_hash = ScriptHash::from_str("d2a4cff31913016155e38e474a2c06d08be276cf")?;
//!     let contract = SmartContract::new(contract_hash, Some(&client));
//!     
//!     // Call a read-only method
//!     let result = contract.call_function("symbol", vec![]).await?;
//!     let symbol = result.stack[0].as_string().unwrap_or_default();
//!     
//!     println!("Contract symbol: {}", symbol);
//!     
//!     // Get token decimals
//!     let decimals_result = contract.call_function("decimals", vec![]).await?;
//!     let decimals = decimals_result.stack[0].as_int().unwrap_or_default();
//!     println!("Token decimals: {}", decimals);
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### Working with NEP-17 tokens
//!
//! ```rust
//! use neo::prelude::*;
//! use std::str::FromStr;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Connect to Neo N3 TestNet
//!     let provider = HttpProvider::new("https://testnet1.neo.org:443");
//!     let client = RpcClient::new(provider);
//!     
//!     // Create a reference to the GAS token contract
//!     let gas_hash = ScriptHash::from_str("d2a4cff31913016155e38e474a2c06d08be276cf")?;
//!     let gas_token = FungibleTokenContract::new(gas_hash, Some(&client));
//!     
//!     // Get token information
//!     let symbol = gas_token.symbol().await?;
//!     let decimals = gas_token.decimals().await?;
//!     let total_supply = gas_token.total_supply().await?;
//!     
//!     println!("Token: {}", symbol);
//!     println!("Decimals: {}", decimals);
//!     println!("Total Supply: {}", total_supply);
//!     
//!     // Check an account's balance
//!     let account = Account::from_wif("YOUR_PRIVATE_KEY_WIF_HERE")?;
//!     let balance = gas_token.balance_of(&account.get_script_hash()).await?;
//!     
//!     println!("Account balance: {} {}",
//!              balance as f64 / 10f64.powi(decimals as i32),
//!              symbol);
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### Using the Neo Name Service (NNS)
//!
//! ```rust
//! use neo::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Connect to Neo N3 TestNet
//!     let provider = HttpProvider::new("https://testnet1.neo.org:443");
//!     let client = RpcClient::new(provider);
//!     
//!     // Create a reference to the NNS contract
//!     let nns_service = NeoNameService::new(Some(&client));
//!     
//!     // Check domain availability
//!     let domain_name = "example.neo";
//!     let is_available = nns_service.is_available(domain_name).await?;
//!     
//!     if is_available {
//!         println!("Domain '{}' is available for registration", domain_name);
//!     } else {
//!         println!("Domain '{}' is already registered", domain_name);
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! For more usage examples, refer to the [`examples` directory](https://github.com/R3E-Network/NeoRust/tree/master/examples) in the repository.
//!
//! ## Project Structure
//!
//! ```
//! NeoRust
//! ├── examples
//! │   ├── neo_nodes          - Examples for connecting to Neo nodes
//! │   ├── neo_transactions   - Examples for creating and sending transactions
//! │   ├── neo_smart_contracts - Examples for interacting with smart contracts
//! │   ├── neo_wallets        - Examples for wallet management
//! │   ├── neo_nep17_tokens   - Examples for working with NEP-17 tokens
//! │   └── neo_nns            - Examples for using the Neo Name Service
//! └── src
//!     ├── neo_builder        - Transaction and script building utilities
//!     ├── neo_clients        - Neo node interaction clients (RPC and WebSocket)
//!     ├── neo_codec          - Encoding and decoding for Neo-specific data structures
//!     ├── neo_config         - Network and client configuration management
//!     ├── neo_contract       - Smart contract interaction abstractions
//!     ├── neo_crypto         - Neo-specific cryptographic operations
//!     ├── neo_protocol       - Neo network protocol implementation
//!     ├── neo_types          - Core Neo ecosystem data types
//!     └── neo_wallets        - Neo asset and account management
//! ```
//!
//! ## Module Overview
//!
//! - **neo_builder**: Transaction and script building utilities.
//!   - Transaction construction and signing
//!   - Script building for contract calls
//!   - Network fee calculation
//!
//! - **neo_clients**: Neo node interaction clients.
//!   - HTTP, WebSocket, and IPC providers
//!   - JSON-RPC client implementation
//!   - Event subscription and notification handling
//!
//! - **neo_codec**: Encoding and decoding for Neo-specific data structures.
//!   - Binary serialization and deserialization
//!   - Neo VM script encoding
//!
//! - **neo_config**: Network and client configuration management.
//!   - Network magic numbers
//!   - Client settings
//!
//! - **neo_contract**: Smart contract interaction abstractions.
//!   - Contract invocation and deployment
//!   - NEP-17 token standard implementation
//!   - Native contracts (GAS, NEO, etc.)
//!   - Neo Name Service (NNS) support
//!
//! - **neo_crypto**: Neo-specific cryptographic operations.
//!   - Key generation and management
//!   - Signing and verification
//!   - Hashing functions
//!
//! - **neo_protocol**: Neo network protocol implementation.
//!   - Account management
//!   - Address formats and conversions
//!
//! - **neo_types**: Core Neo ecosystem data types.
//!   - Script hashes
//!   - Contract parameters
//!   - Block and transaction types
//!   - NNS name types
//!
//! - **neo_wallets**: Neo asset and account management.
//!   - Wallet creation and management
//!   - NEP-6 wallet standard support
//!   - Account import/export
//!   - Wallet backup and recovery
//!
//! For detailed information, consult the documentation of each module.

#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms, unreachable_pub)]
#![deny(rustdoc::broken_intra_doc_links)]
// Only enable doc_cfg feature on nightly builds or when explicitly building docs
#![cfg_attr(all(feature = "nightly", docsrs), feature(doc_cfg))]
#![doc(test(no_crate_inject, attr(deny(rust_2018_idioms), allow(dead_code, unused_variables))))]

// For macro expansions only, not public API.
#[doc(hidden)]
#[allow(unused_extern_crates)]
extern crate self as neo;

// Core modules - always available
pub mod neo_error;
pub mod neo_types;
pub mod neo_utils;

// All modules unconditionally available
pub mod neo_crypto;
pub mod neo_builder;
pub mod neo_clients;
pub mod neo_codec;
pub mod neo_config;
pub mod neo_contract;
pub mod neo_protocol;
pub mod neo_wallets;
pub mod neo_x;
pub mod neo_sgx;
pub mod neo_fs;

// Re-exports for convenience
#[doc(inline)]
pub use neo_builder as builder;
#[doc(inline)]
pub use neo_clients as providers;
#[doc(inline)]
pub use neo_codec as codec;
#[doc(inline)]
pub use neo_config as config;
#[doc(inline)]
pub use neo_crypto as crypto;
#[doc(inline)]
pub use neo_protocol as protocol;
#[doc(inline)]
pub use neo_wallets as wallets;
#[doc(inline)]
pub use neo_x as x;
// No need to re-export specialized modules as they're already public with their full names

// Re-export common types directly in lib.rs for easy access
pub use crate::neo_types::{
	Address, AddressOrScriptHash, Bytes, 
	ContractParameter, ContractParameterType, OpCode, OperandSize,
	ScriptHash, ScriptHashExtension, StackItem,
	// Serialization/deserialization helpers
	serialize_script_hash, deserialize_script_hash,
	serialize_url_option, deserialize_url_option,
	serialize_script_hash_option, deserialize_script_hash_option,
	serialize_address_or_script_hash, deserialize_address_or_script_hash,
	serialize_h256, deserialize_h256,
	serialize_h256_option, deserialize_h256_option,
	serialize_hash_map_h160_account, deserialize_hash_map_h160_account,
	vec_to_array32,
	var_size,
	// Additional types
	Base64Encode, StringExt, VMState, TypeError,
	// Contract types
	ContractManifest, ContractState, ContractIdentifiers, 
	InvocationResult, NefFile,
	// NNS types
	NNSName,
	// Additional types
	ScryptParamsDef, ParameterValue,
};

// Add direct re-exports for commonly used serde utils  
pub use crate::neo_types::serde_with_utils::{
	deserialize_boolean_expression, serialize_boolean_expression,
	deserialize_bytes, serialize_bytes,
	deserialize_h160, serialize_h160,
	deserialize_hashmap_address_u256, serialize_hashmap_address_u256,
	deserialize_hashmap_u256_hashset_h256, serialize_hashmap_u256_hashset_h256,
	deserialize_hashmap_u256_hashset_u256, serialize_hashmap_u256_hashset_u256,
	deserialize_hashmap_u256_vec_u256, serialize_hashmap_u256_vec_u256,
	deserialize_hashset_u256, serialize_hashset_u256,
	deserialize_private_key, serialize_private_key,
	deserialize_public_key, serialize_public_key,
	deserialize_public_key_option, serialize_public_key_option,
	deserialize_scopes, serialize_scopes,
	deserialize_vec_script_hash, serialize_vec_script_hash,
	deserialize_vec_script_hash_option, serialize_vec_script_hash_option,
	deserialize_map, serialize_map,
	deserialize_wildcard, serialize_wildcard,
	deserialize_hardforks,
};

// Re-export additional contract types
pub use crate::neo_types::contract::{
	ContractMethodToken, ContractNef, NeoVMStateType, NativeContractState
};

// Re-export value extension trait
pub use crate::neo_types::serde_value::ValueExtension;

/// Convenient imports for commonly used types and traits.
/// 
/// This prelude module provides a single import to access the most commonly used 
/// components of the NeoRust SDK. Import it with:
/// 
/// ```rust
/// use neo::prelude::*;
/// ```
pub mod prelude;

#[cfg(all(test))]
mod tests {
	use super::prelude::*;
	use primitive_types::H160;
	use std::str::FromStr;

	use tokio;

	use url::Url;
	use crate::builder::{AccountSigner, ScriptBuilder, TransactionBuilder};
	use crate::neo_clients::{APITrait, HttpProvider, RpcClient};
	use crate::neo_protocol::{Account, AccountTrait};

	#[cfg(all(test))]
	#[tokio::test]
	#[ignore] // Ignoring this test as it requires a live Neo N3 node and real tokens
	async fn test_create_and_send_transaction() -> Result<(), Box<dyn std::error::Error>> {
		// Initialize the JSON-RPC provider - using TestNet for safer testing
		let http_provider = HttpProvider::new("https://testnet1.neo.org:443")?;
		let rpc_client = RpcClient::new(http_provider);

		// Create accounts for the sender and recipient
		let sender = Account::from_wif("L1WMhxazScMhUrdv34JqQb1HFSQmWeN2Kpc1R9JGKwL7CDNP21uR")?;
		let recipient = Account::from_address("NbTiM6h8r99kpRtb428XcsUk1TzKed2gTc")?;

		// Use the correct GAS token hash for Neo N3 TestNet
		let gas_token_hash = "d2a4cff31913016155e38e474a2c06d08be276cf"; // GAS token on Neo N3

		// Create a new TransactionBuilder
		let mut tx_builder = TransactionBuilder::with_client(&rpc_client);

		// Build the transaction
		tx_builder
			.set_script(Some(
				ScriptBuilder::new()
					.contract_call(
						&H160::from_str(gas_token_hash)?,
						"transfer",
						&[
							ContractParameter::h160(&sender.get_script_hash()),
							ContractParameter::h160(&recipient.get_script_hash()),
							ContractParameter::integer(1_0000_0000), // 1 GAS (8 decimals)
							ContractParameter::any(),
						],
						None,
					)
					.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?
					.to_bytes(),
			))
			.set_signers(vec![AccountSigner::called_by_entry(&sender)?.into()])
			.map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?
			.valid_until_block(rpc_client.get_block_count().await? + 5760)?; // Valid for ~1 day

		// Sign the transaction
		let mut signed_tx = tx_builder.sign().await?;

		// For testing purposes, we'll just verify that we can create and sign the transaction
		// without actually sending it to the network
		println!("Transaction created and signed successfully");
		println!("Transaction size: {} bytes", signed_tx.size());
		println!("System fee: {} GAS", signed_tx.sys_fee as f64 / 100_000_000.0);
		println!("Network fee: {} GAS", signed_tx.net_fee as f64 / 100_000_000.0);

		Ok(())
	}
}

// Adding trait implementations for serde JSON serialization
// These extensions will be used by the http-client feature
pub mod extensions {
	use serde_json::{Result as JsonResult, Value};
	
	pub trait ToValue {
		fn to_value(&self) -> Value;
	}
	
	impl ToValue for String {
		fn to_value(&self) -> Value {
			serde_json::Value::String(self.clone())
		}
	}
	
	impl ToValue for &str {
		fn to_value(&self) -> Value {
			serde_json::Value::String((*self).to_string())
		}
	}
	
	impl ToValue for u32 {
		fn to_value(&self) -> Value {
			serde_json::Value::Number(serde_json::Number::from(*self))
		}
	}
	
	impl ToValue for i32 {
		fn to_value(&self) -> Value {
			serde_json::Value::Number(serde_json::Number::from(*self))
		}
	}
	
	impl ToValue for bool {
		fn to_value(&self) -> Value {
			serde_json::Value::Bool(*self)
		}
	}

	#[cfg(feature = "http-client")]
	impl ToValue for primitive_types::H160 {
		fn to_value(&self) -> Value {
			serde_json::Value::String(format!("0x{}", hex::encode(self.0)))
		}
	}

	#[cfg(feature = "http-client")]
	impl ToValue for primitive_types::H256 {
		fn to_value(&self) -> Value {
			serde_json::Value::String(format!("0x{}", hex::encode(self.0)))
		}
	}

	#[cfg(feature = "http-client")]
	impl ToValue for i64 {
		fn to_value(&self) -> Value {
			serde_json::Value::Number(serde_json::Number::from(*self))
		}
	}

	#[cfg(feature = "http-client")]
	impl ToValue for u64 {
		fn to_value(&self) -> Value {
			serde_json::Value::Number(serde_json::Number::from(*self))
		}
	}

	#[cfg(feature = "http-client")]
	impl<T: ToValue> ToValue for Vec<T> {
		fn to_value(&self) -> Value {
			let values: Vec<Value> = self.iter().map(|item| item.to_value()).collect();
			serde_json::Value::Array(values)
		}
	}

	#[cfg(feature = "http-client")]
	impl<T: ToValue> ToValue for &[T] {
		fn to_value(&self) -> Value {
			let values: Vec<Value> = self.iter().map(|item| item.to_value()).collect();
			serde_json::Value::Array(values)
		}
	}

	#[cfg(feature = "http-client")]
	impl<T: ToValue> ToValue for Option<T> {
		fn to_value(&self) -> Value {
			match self {
				Some(value) => value.to_value(),
				None => serde_json::Value::Null,
			}
		}
	}

	// Convert from std::io::Error to ProviderError
	#[cfg(feature = "http-client")]
	impl From<std::io::Error> for crate::neo_clients::errors::ProviderError {
		fn from(error: std::io::Error) -> Self {
			Self::CustomError(error.to_string())
		}
	}
}
