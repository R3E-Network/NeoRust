#![allow(warnings)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! ![Neo Logo](https://neo.org/images/neo-logo/NEO-logo.svg)
//! # NeoRust SDK v0.1.9
//!
//! A comprehensive Rust library for building applications on the Neo N3 blockchain ecosystem.
//!
//! [![Crates.io](https://img.shields.io/crates/v/neo3.svg)](https://crates.io/crates/neo3)
//! [![Documentation](https://docs.rs/neo3/badge.svg)](https://docs.rs/neo3)
//!
//! ## Features
//!
//! This crate provides several feature flags to customize functionality:
//!
//! - **futures**: Enables async/futures support for asynchronous blockchain operations. This is recommended
//!   for most applications that need to interact with the Neo blockchain without blocking.
//!
//! - **ledger**: Enables hardware wallet support via Ledger devices. When enabled, you can use Ledger
//!   hardware wallets for transaction signing and key management. This feature provides an additional
//!   security layer by keeping private keys on dedicated hardware.
//!
//! - **aws**: Enables AWS KMS integration for cloud-based key management. This feature allows you to
//!   store and use keys securely in AWS Key Management Service rather than managing them locally.
//!
//! To enable specific features in your project, modify your `Cargo.toml` as follows:
//!
//! ```toml
//! [dependencies]
//! neo3 = { version = "0.1.9", features = ["futures", "ledger"] }
//! ```
//!
//! You can disable default features with:
//!
//! ```toml
//! neo3 = { version = "0.1.9", default-features = false, features = ["futures"] }
//! ```
//!
//! ## Overview
//!
//! NeoRust is a complete SDK designed to make Neo N3 blockchain development in Rust
//! intuitive, type-safe, and productive. The library provides full support for all
//! Neo N3 features and follows Rust best practices for reliability and performance.
//!
//! ## Core Modules
//!
//! NeoRust is organized into specialized crates, each handling specific aspects of Neo N3:
//!
//! - [**neo-builder**](https://docs.rs/neo-builder): Transaction construction and script building
//! - [**neo-clients**](https://docs.rs/neo-clients): Neo node interaction and RPC client implementations
//! - [**neo-codec**](https://docs.rs/neo-codec): Serialization and deserialization of Neo data structures
//! - [**neo-config**](https://docs.rs/neo-config): Configuration for networks and client settings
//! - [**neo-contract**](https://docs.rs/neo-contract): Smart contract interaction and token standards
//! - [**neo-crypto**](https://docs.rs/neo-crypto): Cryptographic primitives and operations
//! - [**neo-error**](https://docs.rs/neo-error): Unified error handling
//! - [**neo-fs**](https://docs.rs/neo-fs): NeoFS distributed storage system integration
//! - [**neo-protocol**](https://docs.rs/neo-protocol): Core blockchain protocol implementations
//! - [**neo-types**](https://docs.rs/neo-types): Core data types and primitives for Neo N3
//! - [**neo-utils**](https://docs.rs/neo-utils): General utility functions
//! - [**neo-wallets**](https://docs.rs/neo-wallets): Wallet management for Neo N3
//! - [**neo-x**](https://docs.rs/neo-x): Neo X EVM compatibility layer
//!
//! ## Quick Start
//!
//! Import all essential types and traits using the `prelude`:
//!
//! ```rust
//! use neo3::prelude::*;
//! ```
//!
//! ## Complete Example
//!
//! Here's a comprehensive example showcasing common operations with the NeoRust SDK:
//!
//! ```no_run
//! use neo3::prelude::*;
//! use std::str::FromStr;
//!
//! async fn neo_example() -> Result<(), Box<dyn std::error::Error>> {
//!     // Connect to Neo TestNet
//!     let provider = HttpProvider::new("https://testnet1.neo.org:443")?;
//!     let client = RpcClient::new(provider);
//!     
//!     // Get basic blockchain information
//!     let block_height = client.get_block_count().await?;
//!     let best_block_hash = client.get_best_block_hash().await?;
//!     println!("Connected to Neo TestNet at height: {}", block_height);
//!     
//!     // Create a new wallet account
//!     let account = Account::create()?;
//!     println!("New account created:");
//!     println!("  Address:     {}", account.get_address());
//!     println!("  Script Hash: {}", account.get_script_hash());
//!     
//!     // Connect to system token contracts
//!     let neo_token = NeoToken::new(&client);
//!     let gas_token = GasToken::new(&client);
//!     
//!     // Query token information
//!     let neo_symbol = neo_token.symbol().await?;
//!     let neo_total_supply = neo_token.total_supply().await?;
//!     let gas_symbol = gas_token.symbol().await?;
//!     let gas_decimals = gas_token.decimals().await?;
//!     
//!     println!("{} token supply: {}", neo_symbol, neo_total_supply);
//!     println!("{} token decimals: {}", gas_symbol, gas_decimals);
//!     
//!     // Query account balances
//!     // Note: A newly created account will have zero balance
//!     // until it receives tokens from somewhere
//!     let test_address = "NUVPACTpQvd2HHmBgFjJJRWwVXJiR3uAEh";
//!     let test_account = Account::from_address(test_address)?;
//!     let script_hash = test_account.get_script_hash();
//!     
//!     let neo_balance = neo_token.balance_of(&script_hash).await?;
//!     let gas_balance = gas_token.balance_of(&script_hash).await?;
//!     
//!     println!("Account {} balances:", test_address);
//!     println!("  {}: {}", neo_symbol, neo_balance);
//!     println!("  {}: {} (÷ 10^{})", gas_symbol, gas_balance, gas_decimals);
//!     
//!     // Build a transaction to transfer GAS
//!     // (would require the account to have GAS for this to work)
//!     let recipient = ScriptHash::from_str("d2a4cff31913016155e38e474a2c06d08be276cf")?;
//!     let amount = 1_0000_0000; // 1 GAS (with 8 decimals)
//!     
//!     // Create the transfer script
//!     let script = ScriptBuilder::build_contract_call(
//!         &gas_token.script_hash(),
//!         "transfer",
//!         &[
//!             ContractParameter::hash160(&script_hash),
//!             ContractParameter::hash160(&recipient),
//!             ContractParameter::integer(amount),
//!             ContractParameter::any(None),
//!         ],
//!     )?;
//!     
//!     // Build transaction
//!     let transaction = TransactionBuilder::new()
//!         .version(0)
//!         .nonce(rand::random::<u32>())
//!         .valid_until_block(block_height + 100)
//!         .script(script)
//!         .add_signer(Signer::called_by_entry(script_hash.clone()))
//!         .build();
//!     
//!     println!("Transaction built successfully:");
//!     println!("  Size: {} bytes", transaction.get_size());
//!     println!("  Hash: {}", transaction.hash());
//!     
//!     // Note: Signing and sending requires the account to have a private key
//!     // and sufficient GAS for fees. This part is for illustration only.
//!     /*
//!     // Sign transaction (requires account with private key)
//!     let signed_tx = transaction.sign(&client, &test_account).await?;
//!     
//!     // Send transaction to network
//!     let tx_id = client.send_raw_transaction(&signed_tx).await?;
//!     println!("Transaction sent: {}", tx_id);
//!     */
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Usage Examples
//!
//! ### Connecting to a Neo N3 node
//!
//! ```rust
//! use neo3::prelude::*;
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
//! use neo3::prelude::*;
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
//! use neo3::prelude::*;
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
//! use neo3::prelude::*;
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
//! use neo3::prelude::*;
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
//! └── crates
//!     ├── neo-builder        - Transaction and script building utilities
//!     ├── neo-clients        - Neo node interaction clients (RPC and WebSocket)
//!     ├── neo-codec          - Encoding and decoding for Neo-specific data structures
//!     ├── neo-config         - Network and client configuration management
//!     ├── neo-contract       - Smart contract interaction abstractions
//!     ├── neo-crypto         - Neo-specific cryptographic operations
//!     ├── neo-error          - Unified error handling
//!     ├── neo-fs             - NeoFS distributed storage system integration
//!     ├── neo-protocol       - Neo network protocol implementation
//!     ├── neo-types          - Core Neo ecosystem data types
//!     ├── neo-utils          - General utility functions
//!     ├── neo-wallets        - Neo asset and account management
//!     └── neo-x              - Neo X EVM compatibility layer
//! ```
//!
//! ## Module Overview
//!
//! - **neo-builder**: Transaction and script building utilities.
//!   - Transaction construction and signing
//!   - Script building for contract calls
//!   - Network fee calculation
//!
//! - **neo-clients**: Neo node interaction clients.
//!   - HTTP, WebSocket, and IPC providers
//!   - JSON-RPC client implementation
//!   - Event subscription and notification handling
//!
//! - **neo-codec**: Encoding and decoding for Neo-specific data structures.
//!   - Binary serialization and deserialization
//!   - Neo VM script encoding
//!
//! - **neo-config**: Network and client configuration management.
//!   - Network magic numbers
//!   - Client settings
//!
//! - **neo-contract**: Smart contract interaction abstractions.
//!   - Contract invocation and deployment
//!   - NEP-17 token standard implementation
//!   - Native contracts (GAS, NEO, etc.)
//!   - Neo Name Service (NNS) support
//!
//! - **neo-crypto**: Neo-specific cryptographic operations.
//!   - Key generation and management
//!   - Signing and verification
//!   - Hashing functions
//!
//! - **neo-error**: Unified error handling.
//!   - Error types for all SDK components
//!   - Error conversion and propagation
//!
//! - **neo-fs**: NeoFS distributed storage system integration.
//!   - File storage and retrieval
//!   - Container management
//!   - Access control
//!
//! - **neo-protocol**: Neo network protocol implementation.
//!   - Account management
//!   - Address formats and conversions
//!
//! - **neo-types**: Core Neo ecosystem data types.
//!   - Script hashes
//!   - Contract parameters
//!   - Block and transaction types
//!   - NNS name types
//!
//! - **neo-utils**: General utility functions.
//!   - Conversion utilities
//!   - Formatting helpers
//!   - Validation functions
//!
//! - **neo-wallets**: Neo asset and account management.
//!   - Wallet creation and management
//!   - NEP-6 wallet standard support
//!   - Account import/export
//!   - Wallet backup and recovery
//!
//! - **neo-x**: Neo X EVM compatibility layer.
//!   - EVM compatibility layer
//!   - Cross-chain bridges
//!   - Interoperability with other blockchains
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
extern crate self as neo3;

// Re-export all components from their respective crates
pub use neo_builder as builder;
pub use neo_clients as clients;
pub use neo_codec as codec;
pub use neo_common as common;
pub use neo_config as config;
pub use neo_contract as contract;
pub use neo_crypto as crypto;
pub use neo_error as error;
pub use neo_fs as fs;
pub use neo_protocol as protocol;
pub use neo_types as types;
pub use neo_utils as utils;
pub use neo_wallets as wallets;
pub use neo_x as x;

// Constants that may not be crate-specific yet
pub mod constants;

// Export common types at the root level for convenience
pub use neo_types::{
    address::Address,
    block::{Block, BlockHeader},
    contract::ContractParameter,
    hash::ScriptHash,
    transaction::{Transaction, Signer, Witness},
    wallet::Account,
};

pub use neo_clients::{HttpProvider, RpcClient};
pub use neo_contract::{
    nep17::{Nep17Contract, NeoToken, GasToken},
    ContractCall, ContractInvocation,
};
pub use neo_wallets::Wallet;
pub use neo_builder::{TransactionBuilder, ScriptBuilder};

// Include the prelude module
pub mod prelude;

// For backward compatibility, legacy modules will be phased out
// These will be deprecated in future releases
#[deprecated(since = "0.1.9", note = "use neo_error crate instead")]
pub mod neo_error {
    pub use neo_error::*;
}

#[deprecated(since = "0.1.9", note = "use crypto module instead")]
pub mod neo_crypto {
    pub use neo_crypto::*;
}

#[deprecated(since = "0.1.9", note = "use builder module instead")]
pub mod neo_builder {
    pub use neo_builder::*;
}

#[deprecated(since = "0.1.9", note = "use clients module instead")]
pub mod neo_clients {
    pub use neo_clients::*;
}

#[deprecated(since = "0.1.9", note = "use codec module instead")]
pub mod neo_codec {
    pub use neo_codec::*;
}

#[deprecated(since = "0.1.9", note = "use config module instead")]
pub mod neo_config {
    pub use neo_config::*;
}

#[deprecated(since = "0.1.9", note = "use contract module instead")]
pub mod neo_contract {
    pub use neo_contract::*;
}

#[deprecated(since = "0.1.9", note = "use fs module instead")]
pub mod neo_fs {
    pub use neo_fs::*;
}

#[deprecated(since = "0.1.9", note = "use protocol module instead")]
pub mod neo_protocol {
    pub use neo_protocol::*;
}

#[deprecated(since = "0.1.9", note = "use types module instead")]
pub mod neo_types {
    pub use neo_types::*;
}

#[deprecated(since = "0.1.9", note = "use utils module instead")]
pub mod neo_utils {
    pub use neo_utils::*;
}

#[deprecated(since = "0.1.9", note = "use wallets module instead")]
pub mod neo_wallets {
    pub use neo_wallets::*;
}

#[deprecated(since = "0.1.9", note = "use x module instead")]
pub mod neo_x {
    pub use neo_x::*;
}
