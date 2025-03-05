//! # Neo Contract
//!
//! Smart contract interaction utilities for the NeoRust SDK.
//!
//! This crate provides utilities for interacting with Neo N3 smart contracts, including:
//!
//! - Standard contract interfaces (NEP-17, NEP-11)
//! - Native contract wrappers (NeoToken, GasToken, PolicyContract, etc.)
//! - Contract management utilities
//! - Name service integration
//! - Famous contract implementations
//!
//! ## Usage
//!
//! ```rust,ignore
//! use neo_contract::{NeoToken, GasToken, PolicyContract};
//! use neo_protocol::account::Account;
//! use std::str::FromStr;
//!
//! // Interact with the Neo native token contract
//! let account = Account::from_str("NZNos2WqTbu5oCgyfss9kUJgBXJqhuYAaj").unwrap();
//! let neo_token = NeoToken::new(provider);
//! let balance = neo_token.balance_of(&account).await?;
//!
//! // Interact with the Gas native token contract
//! let gas_token = GasToken::new(provider);
//! let gas_balance = gas_token.balance_of(&account).await?;
//!
//! // Interact with the Policy contract
//! let policy = PolicyContract::new(provider);
//! let fee_per_byte = policy.get_fee_per_byte().await?;
//! ```

#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms, unreachable_pub)]
#![deny(rustdoc::broken_intra_doc_links)]

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

// Re-export all public items
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
