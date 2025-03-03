#![feature(const_trait_impl)]

//! # Neo Contract Module (v0.1.4)
//!
//! Comprehensive interfaces for interacting with Neo N3 smart contracts and tokens.
//!
//! ## Overview
//!
//! The neo_contract module provides a robust set of interfaces for interacting with
//! various types of smart contracts on the Neo N3 blockchain. This module abstracts
//! away the complexity of contract calls and state management, providing easy-to-use
//! APIs for developers.
//!
//! ## Key Features
//!
//! - **System Contracts**: Built-in interfaces for Neo N3 system contracts:
//!   - NEO Token contract
//!   - GAS Token contract
//!   - Policy contract
//!   - RoleManagement contract
//!   - ContractManagement contract
//!
//! - **Token Standards**:
//!   - NEP-17 fungible token standard (similar to Ethereum's ERC-20)
//!   - NEP-11 non-fungible token standard (similar to Ethereum's ERC-721)
//!
//! - **Advanced Contract Interactions**:
//!   - Neo Name Service (NNS) domain resolution
//!   - Neo URI parsing and validation
//!   - Contract iterator support for handling large result sets
//!
//! - **Famous Contract Integrations**:
//!   - Flamingo Finance DeFi ecosystem
//!   - NeoburgerNeo (bNEO) staking contract
//!   - GrandShare voting and proposals
//!   - NeoCompound yield aggregator
//!
//! - **Developer Tools**:
//!   - Contract deployment helpers
//!   - ABI and manifest handling utilities
//!   - Contract invocation result parsing
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
