#![allow(warnings)]

//! # NeoRust
//!
//! A comprehensive Rust library for interacting with the Neo blockchain.
//!
//! ## Quick Start
//!
//! Import all essential types and traits using the `prelude`:
//!
//! ```rust
//! use NeoRust::prelude::*;
//! ```
//!
//! ## Usage Examples
//!
//! ### Connecting to a Neo node
//!
//! ```rust
//! use NeoRust::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let provider = JsonRpcProvider::new("http://seed1.neo.org:10332");
//!     let block_count = provider.get_block_count().await?;
//!     println!("Current block count: {}", block_count);
//!     Ok(())
//! }
//! ```
//!
//! ### Creating and sending a transaction
//!
//! ```rust
//! use NeoRust::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize the JSON-RPC provider
//!     let provider = JsonRpcProvider::new("http://seed1.neo.org:10332");
//!
//!     // Create accounts for the sender and recipient
//!     let sender = Account::from_wif("YOUR_SENDER_WIF_HERE")?;
//!     let recipient = H160::from_str("NbTiM6h8r99kpRtb428XcsUk1TzKed2gTc")?;
//!
//!     // Create a new TransactionBuilder
//!     let mut tx_builder = TransactionBuilder::with_client(&provider);
//!
//!     // Build the transaction
//!     tx_builder
//!         .set_script(Some(
//!             ScriptBuilder::new()
//!                 .contract_call(
//!                     &H160::from_str(NeoConstants::NEO_TOKEN_HASH)?,
//!                     "transfer",
//!                     &[
//!                         ContractParameter::hash160(&sender.get_script_hash()),
//!                         ContractParameter::hash160(&recipient),
//!                         ContractParameter::integer(1_0000_0000), // 1 NEO
//!                         ContractParameter::any(),
//!                     ],
//!                     None,
//!                 )
//!                 .unwrap()
//!                 .to_bytes(),
//!         ))
//!         .set_signers(vec![AccountSigner::called_by_entry(&sender)?])
//!         .valid_until_block(provider.get_block_count().await? + 5760)?; // Valid for ~1 day
//!
//!     // Sign the transaction
//!     let mut signed_tx = tx_builder.sign().await?;
//!
//!     // Send the transaction
//!     let raw_tx = signed_tx.send_tx(&provider).await?;
//!
//!     println!("Transaction sent: {}", raw_tx.hash);
//!
//!     // Wait for the transaction to be confirmed
//!     loop {
//!         tokio::time::sleep(std::time::Duration::from_secs(15)).await;
//!         match signed_tx.get_application_log(&provider).await {
//!             Ok(app_log) => {
//!                 println!("Transaction confirmed!");
//!                 println!("Application log: {:?}", app_log);
//!                 break;
//!             }
//!             Err(_) => {
//!                 println!("Transaction not yet confirmed. Waiting...");
//!             }
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Interacting with a smart contract
//!
//! ```rust
//! use NeoRust::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let provider = JsonRpcProvider::new("http://seed1.neo.org:10332");
//!     let contract_hash = "0xd2a4cff31913016155e38e474a2c06d08be276cf";
//!     
//!     let result: String = provider
//!         .invoke_function(contract_hash, "name", vec![], None)
//!         .await?;
//!     
//!     println!("Contract name: {}", result);
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
//! └── src
//!     ├── neo_builder
//!     ├── neo_clients
//!     ├── neo_codec
//!     ├── neo_config
//!     ├── neo_contract
//!     ├── neo_crypto
//!     ├── neo_protocol
//!     ├── neo_types
//!     └── neo_wallets
//! ```
//!
//! ## Module Overview
//!
//! - **neo_builder**: Transaction and script building utilities.
//! - **neo_clients**: Neo node interaction clients (RPC and WebSocket).
//! - **neo_codec**: Encoding and decoding for Neo-specific data structures.
//! - **neo_config**: Network and client configuration management.
//! - **neo_contract**: Smart contract interaction abstractions.
//! - **neo_crypto**: Neo-specific cryptographic operations.
//! - **neo_protocol**: Neo network protocol implementation.
//! - **neo_types**: Core Neo ecosystem data types.
//! - **neo_wallets**: Neo asset and account management.
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

pub mod neo_builder;
pub mod neo_clients;
pub mod neo_codec;
pub mod neo_config;
pub mod neo_contract;
pub mod neo_crypto;
pub mod neo_error;
pub mod neo_protocol;
pub mod neo_types;
pub mod neo_wallets;

/// Convenient imports for commonly used types and traits.
pub mod prelude {
	pub use super::{
		builder::*, codec::*, config::*, contract::*, crypto::*, neo_error::*, protocol::*,
		providers::*, types::*, wallets::*,
	};
}

#[cfg(test)]
mod tests {
	use super::prelude::*;
	use primitive_types::H160;
	use std::str::FromStr;
	use tokio;
	use url::Url;

	#[tokio::test]
	async fn test_create_and_send_transaction() -> Result<(), Box<dyn std::error::Error>> {
		// Initialize the JSON-RPC provider
		let http_provider = HttpProvider::new("http://seed1.neo.org:10332")?;
		let rpc_client = RpcClient::new(http_provider);

		// Create accounts for the sender and recipient
		let sender = Account::from_wif("YOUR_SENDER_WIF_HERE")?;
		let recipient = Account::from_address("NbTiM6h8r99kpRtb428XcsUk1TzKed2gTc")?;

		// Create a new TransactionBuilder
		let mut tx_builder = TransactionBuilder::with_client(&rpc_client);

		// Build the transaction
		tx_builder
			.set_script(Some(
				ScriptBuilder::new()
					.contract_call(
						&H160::from_str(TestConstants::NEO_TOKEN_HASH)?,
						"transfer",
						&[
							ContractParameter::h160(&sender.get_script_hash()),
							ContractParameter::h160(&recipient.get_script_hash()),
							ContractParameter::integer(1_0000_0000), // 1 NEO
							ContractParameter::any(),
						],
						None,
					)
					.unwrap()
					.to_bytes(),
			))
			.set_signers(vec![AccountSigner::called_by_entry(&sender)?.into()])
			.valid_until_block(rpc_client.get_block_count().await? + 5760)?; // Valid for ~1 day

		// Sign the transaction
		let mut signed_tx = tx_builder.sign().await?;

		// Send the transaction
		let raw_tx = signed_tx.send_tx(&rpc_client).await?;

		println!("Transaction sent: {}", raw_tx.hash);

		// Wait for the transaction to be confirmed
		loop {
			tokio::time::sleep(std::time::Duration::from_secs(15)).await;
			match signed_tx.get_application_log(&rpc_client).await {
				Ok(app_log) => {
					println!("Transaction confirmed!");
					println!("Application log: {:?}", app_log);
					break;
				},
				Err(_) => {
					println!("Transaction not yet confirmed. Waiting...");
				},
			}
		}

		Ok(())
	}
}
