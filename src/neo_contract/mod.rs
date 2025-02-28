#![feature(const_trait_impl)]

//! # Neo Contract
//!
//! Interfaces for interacting with Neo N3 smart contracts.
//!
//! ## Overview
//!
//! The neo_contract module provides a comprehensive set of interfaces for interacting with
//! various types of smart contracts on the Neo N3 blockchain. It includes:
//!
//! - Standard Neo N3 contracts (NEO, GAS, Policy, RoleManagement)
//! - NEP-17 fungible token contracts
//! - NEP-11 non-fungible token (NFT) contracts
//! - Neo Name Service (NNS) contracts
//! - Famous Neo N3 contracts (Flamingo, NeoburgerNeo, GrandShare, NeoCompound)
//! - Contract management utilities
//! - Smart contract traits and interfaces
//!
//! This module makes it easy to interact with both system contracts and custom contracts
//! on the Neo N3 blockchain.
//!
//! ## Examples
//!
//! ### Interacting with the NEO token contract
//!
//! ```rust
//! use neo::prelude::*;
//! use std::str::FromStr;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a provider and client
//!     let provider = HttpProvider::new("https://testnet1.neo.org:443");
//!     let client = RpcClient::new(provider);
//!     
//!     // Create a NEO token contract instance
//!     let neo_token = NeoToken::new(client);
//!     
//!     // Get the NEO token symbol
//!     let symbol = neo_token.symbol().await?;
//!     println!("Token symbol: {}", symbol);
//!     
//!     // Get the balance of an account
//!     let account = ScriptHash::from_str("0xd2a4cff31913016155e38e474a2c06d08be276cf")?;
//!     let balance = neo_token.balance_of(&account).await?;
//!     println!("Account balance: {} NEO", balance);
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### Interacting with a custom NEP-17 token
//!
//! ```rust
//! use neo::prelude::*;
//! use std::str::FromStr;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a provider and client
//!     let provider = HttpProvider::new("https://testnet1.neo.org:443");
//!     let client = RpcClient::new(provider);
//!     
//!     // Create a script hash for the token contract
//!     let token_hash = ScriptHash::from_str("0x1578103c13e39df15d0d29826d957e85d770d8c9")?;
//!     
//!     // Create a NEP-17 token contract instance
//!     let token = FungibleTokenContract::new(client, token_hash);
//!     
//!     // Get token information
//!     let symbol = token.symbol().await?;
//!     let decimals = token.decimals().await?;
//!     let total_supply = token.total_supply().await?;
//!     
//!     println!("Token: {} (Decimals: {})", symbol, decimals);
//!     println!("Total Supply: {}", total_supply);
//!     
//!     Ok(())
//! }
//! ```

pub use contract_error::*;
pub use contract_management::*;
pub use famous::*;
pub use fungible_token_contract::*;
pub use gas_token::*;
pub use iterator::*;
pub use name_service::*;
pub use neo_token::*;
pub use neo_uri::*;
pub use nft_contract::*;
pub use policy_contract::*;
pub use role_management::*;
pub use traits::*;

mod contract_error;
mod contract_management;
mod famous;
mod fungible_token_contract;
mod gas_token;
mod iterator;
mod name_service;
mod neo_token;
mod neo_uri;
mod nft_contract;
mod policy_contract;
mod role_management;
mod traits;

#[cfg(test)]
mod tests;
