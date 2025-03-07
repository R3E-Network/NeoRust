//! # Neo X
//!
//! Cross-chain and extension functionality for the NeoRust SDK.
//!
//! This crate provides extended functionality for the Neo N3 blockchain, including:
//!
//! - EVM compatibility layer
//! - Cross-chain bridges
//! - Interoperability with other blockchains
//! - Extended protocol support
//! - Advanced transaction types
//! - Custom extensions
//!
//! ## Overview
//!
//! The neo-x crate provides interfaces for interacting with Neo X, an EVM-compatible
//! chain maintained by Neo. It includes:
//!
//! - EVM compatibility layer for interacting with Neo X as an Ethereum-compatible chain
//! - Bridge functionality for transferring tokens between Neo N3 and Neo X
//! - Transaction creation and signing for Neo X
//! - Provider interfaces for connecting to Neo X nodes
//!
//! This module enables seamless integration between Neo N3 and EVM-compatible ecosystems,
//! allowing developers to leverage both blockchain environments.
//!
//! ## Examples
//!
//! ### Connecting to Neo X and getting chain information
//!
//! ```rust,ignore
//! use neo3::prelude::*;
//! use neo_x::evm::NeoXProvider;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Connect to Neo N3
//!     let neo_provider = HttpProvider::new("https://mainnet1.neo.org:443");
//!     let neo_client = RpcClient::new(neo_provider);
//!     
//!     // Initialize the Neo X EVM provider
//!     let neo_x_provider = NeoXProvider::new("https://rpc.neo-x.org", Some(&neo_client));
//!     
//!     // Get the chain ID for Neo X
//!     let chain_id = neo_x_provider.chain_id().await?;
//!     println!("Neo X Chain ID: {}", chain_id);
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### Using the bridge to transfer tokens between Neo N3 and Neo X
//!
//! ```rust,ignore
//! use neo3::prelude::*;
//! use neo_x::bridge::NeoXBridgeContract;
//! use std::str::FromStr;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Connect to Neo N3
//!     let neo_provider = HttpProvider::new("https://mainnet1.neo.org:443");
//!     let neo_client = RpcClient::new(neo_provider);
//!     
//!     // Create an account
//!     let account = Account::from_wif("YOUR_WIF_HERE")?;
//!     
//!     // Initialize the bridge contract
//!     let bridge = NeoXBridgeContract::new(Some(&neo_client))?;
//!     
//!     // Get the GAS token script hash
//!     let gas_token = ScriptHash::from_str("d2a4cff31913016155e38e474a2c06d08be276cf")?;
//!     
//!     // Deposit GAS from Neo N3 to Neo X
//!     let neo_x_address = "0x1234567890123456789012345678901234567890";
//!     let amount = 1_0000_0000; // 1 GAS
//!     
//!     let deposit_tx = bridge.deposit(
//!         &gas_token,
//!         amount,
//!         neo_x_address,
//!         &account,
//!     ).await?;
//!     
//!     Ok(())
//! }
//! ```

#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms, unreachable_pub)]
#![deny(rustdoc::broken_intra_doc_links)]

mod bridge;
mod evm;

// Re-export all public items
pub use bridge::*;
pub use evm::*;
