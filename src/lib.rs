#![allow(warnings)]

//! # NeoRust
//!
//! A comprehensive Rust library for interacting with the Neo N3 blockchain.
//!
//! ## Overview
//!
//! NeoRust is a complete SDK for building applications on the Neo N3 blockchain. It provides
//! a type-safe, intuitive interface for all Neo N3 features, including:
//!
//! - RPC client for interacting with Neo nodes
//! - Transaction construction and signing
//! - Smart contract interaction (invocation and deployment)
//! - Wallet management (creation, import, export)
//! - Asset management (NEP-17 tokens)
//! - Cryptographic operations (keys, signatures)
//! - Script building for contract calls
//! - Event monitoring and notifications
//! - Network fee calculation
//! - Neo Name Service (NNS) support
//!
//! ## Quick Start
//!
//! Import all essential types and traits using the `prelude`:
//!
//! ```rust
//! use neo::prelude::*;
//! ```
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
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc(test(no_crate_inject, attr(deny(rust_2018_idioms), allow(dead_code, unused_variables))))]

// For macro expansions only, not public API.
#[doc(hidden)]
#[allow(unused_extern_crates)]
extern crate self as neo;

#[doc(inline)]
pub use neo_builder as builder;
#[doc(inline)]
pub use neo_clients as providers;
#[doc(inline)]
pub use neo_codec as codec;
#[doc(inline)]
pub use neo_config as config;
#[doc(inline)]
pub use neo_contract as contract;
#[doc(inline)]
pub use neo_crypto as crypto;
#[doc(inline)]
pub use neo_protocol as protocol;
#[doc(inline)]
pub use neo_types as types;
#[doc(inline)]
pub use neo_wallets as wallets;
#[doc(inline)]
pub use neo_x as x;

pub mod neo_builder;
pub mod neo_clients;
pub mod neo_codec;
pub mod neo_config;
pub mod neo_contract;
pub mod neo_crypto;
pub mod neo_error;
pub mod neo_protocol;
pub mod neo_types;
pub mod neo_utils;
pub mod neo_wallets;
pub mod neo_x;

#[cfg(feature = "sgx")]
pub mod neo_sgx;

/// Convenient imports for commonly used types and traits.
pub mod prelude {
	pub use super::{
		builder::*, codec::*, config::*, contract::*, crypto::*, neo_error::*, protocol::*,
		providers::*, types::*, wallets::*, x::*,
	};

	#[cfg(feature = "sgx")]
	pub use super::neo_sgx::*;

	pub use super::neo_utils::error::*;
}

#[cfg(test)]
mod tests {
	use super::prelude::*;
	use primitive_types::H160;
	use std::str::FromStr;
	use tokio;
	use url::Url;

	#[tokio::test]
	#[ignore] // Ignoring this test as it requires a live Neo N3 node and real tokens
	async fn test_create_and_send_transaction() -> Result<(), Box<dyn std::error::Error>> {
		init_logger();
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

		// In a real scenario, we would send the transaction and wait for confirmation:
		/*
		// Send the transaction
		let raw_tx = signed_tx.send_tx().await?;
		println!("Transaction sent: {}", raw_tx.hash);

		// Wait for the transaction to be confirmed
		println!("Waiting for confirmation...");
		signed_tx.track_tx(10).await?;
		println!("Transaction confirmed!");

		// Get the application log
		let app_log = signed_tx.get_application_log(&rpc_client).await?;
		println!("Application log: {:?}", app_log);
		*/

		Ok(())
	}
}
