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
//! ## Feature Flags
//!
//! This module uses the following feature flags:
//!
//! - **contract**: Core contract functionality (always available when using this module)
//! - **nep17**: Support for NEP-17 fungible tokens
//! - **nep11**: Support for NEP-11 non-fungible tokens
//! - **contract-deploy**: Support for deploying contracts
//! - **contract-invoke**: Support for invoking contract methods
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

use primitive_types::H160;

#[cfg(feature = "http-client")]
pub use crate::neo_clients::rpc::RpcClient;
#[cfg(feature = "http-client")]
pub use crate::neo_clients::rpc::connections::JsonRpcProvider;

// Core contract functionality - always available
pub use contract_error::ContractError;
pub use contract_management::ContractManagement;
pub use iterator::NeoIterator;
pub use neo_uri::NeoURI;
pub use traits::smart_contract::SmartContractTrait;
pub use traits::token::TokenTrait;
pub use traits::fungible_token::FungibleTokenTrait;
pub use contract_parameter::ContractParameter;
pub use contract_manifest::ContractManifest;

// Token standards - conditionally available
#[cfg(feature = "nep17")]
#[cfg_attr(docsrs, doc(cfg(feature = "nep17")))]
pub use fungible_token_contract::*;

#[cfg(feature = "nep17")]
#[cfg_attr(docsrs, doc(cfg(feature = "nep17")))]
pub use gas_token::*;

#[cfg(feature = "nep17")]
#[cfg_attr(docsrs, doc(cfg(feature = "nep17")))]
pub use neo_token::*;

#[cfg(feature = "nep11")]
#[cfg_attr(docsrs, doc(cfg(feature = "nep11")))]
pub use nft_contract::*;

// System contracts - conditionally available
#[cfg(feature = "http-client")]
#[cfg_attr(docsrs, doc(cfg(feature = "http-client")))]
pub use policy_contract::*;

#[cfg(feature = "http-client")]
#[cfg_attr(docsrs, doc(cfg(feature = "http-client")))]
pub use role_management::*;

#[cfg(feature = "http-client")]
#[cfg_attr(docsrs, doc(cfg(feature = "http-client")))]
pub use name_service::*;

// Famous contracts - conditionally available
#[cfg(all(feature = "nep17", feature = "http-client"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "nep17", feature = "http-client"))))]
pub use famous::*;

/// A smart contract on the Neo blockchain.
#[cfg(feature = "http-client")]
pub struct SmartContract<P: JsonRpcProvider> {
    /// The client used to interact with the blockchain.
    client: RpcClient<P>,
    /// The script hash of the contract.
    script_hash: H160,
}

#[cfg(feature = "http-client")]
impl<P: JsonRpcProvider> SmartContract<P> {
    /// Creates a new smart contract.
    pub fn new(client: RpcClient<P>, script_hash: H160) -> Self {
        Self { client, script_hash }
    }

    /// Gets the script hash of the contract.
    pub fn script_hash(&self) -> H160 {
        self.script_hash
    }
}

// Core contract modules - always available
mod contract_error;
mod contract_management;
pub mod contract_parameter;
pub mod contract_manifest;
mod iterator;
mod neo_uri;
mod traits;

// Token standard modules
#[cfg(feature = "nep17")]
mod fungible_token_contract;

#[cfg(feature = "nep17")]
mod gas_token;

#[cfg(feature = "nep17")]
mod neo_token;

#[cfg(feature = "nep11")]
mod nft_contract;

// System contract modules
#[cfg(feature = "http-client")]
mod policy_contract;

#[cfg(feature = "http-client")]
mod role_management;

#[cfg(feature = "http-client")]
mod name_service;

// Famous contracts module
#[cfg(all(feature = "nep17", feature = "http-client"))]
mod famous;

#[cfg(test)]
mod tests;
