//! # Neo Builder
//!
//! Tools for building Neo N3 transactions and scripts.
//!
//! ## Overview
//!
//! The neo_builder module provides utilities for:
//!
//! - Creating and building transactions
//! - Constructing scripts for smart contract invocation
//! - Managing transaction signers and witnesses
//! - Calculating network fees
//! - Handling transaction attributes
//! - Building verification scripts
//!
//! ## Feature Flags
//!
//! This module supports the following feature flags:
//!
//! - **transaction**: Core transaction functionality (always available when using this module)
//! - **contract-invoke**: Support for contract invocation features 
//! - **contract-deploy**: Support for contract deployment features
//! - **http-client**: Integration with RPC clients for transaction submission
//! - **wallet**: Wallet integration for signing transactions
//!
//! ## Examples
//!
//! ### Building a Script for Contract Invocation
//!
//! ```rust
//! use neo::prelude::*;
//! use std::str::FromStr;
//!
//! // Create a script to invoke a contract method
//! let contract_hash = ScriptHash::from_str("d2a4cff31913016155e38e474a2c06d08be276cf").unwrap();
//! let sender = ScriptHash::from_str("0xd2a4cff31913016155e38e474a2c06d08be276cf").unwrap();
//! let recipient = ScriptHash::from_str("0x5c9c3a340f4c28262e7042b908b5f7e7a4bcd7e7").unwrap();
//! let amount = 1_0000_0000; // 1 GAS (8 decimals)
//!
//! let script = ScriptBuilder::new()
//!     .contract_call(
//!         &contract_hash,
//!         "transfer",
//!         &[
//!             ContractParameter::hash160(&sender),
//!             ContractParameter::hash160(&recipient),
//!             ContractParameter::integer(amount),
//!             ContractParameter::any(None),
//!         ],
//!         None,
//!     )
//!     .unwrap()
//!     .to_bytes();
//! ```
//!
//! ### Building and Signing a Transaction
//!
//! ```rust
//! use neo::prelude::*;
//! use std::str::FromStr;
//!
//! async fn build_transaction(client: &RpcClient<HttpProvider>) -> Result<Transaction, Box<dyn std::error::Error>> {
//!     // Create an account for signing
//!     let account = Account::from_wif("YOUR_WIF_HERE")?;
//!     
//!     // Create a script (simplified for example)
//!     let script = vec![0x01, 0x02, 0x03]; // Placeholder for actual script
//!     
//!     // Create and configure the transaction
//!     let mut tx_builder = TransactionBuilder::with_client(client);
//!     tx_builder
//!         .script(Some(script))
//!         .set_signers(vec![account.clone().into()])
//!         .valid_until_block(client.get_block_count().await? + 5760)?; // Valid for ~1 day
//!
//!     // Sign the transaction
//!     let tx = tx_builder.sign().await?;
//!     
//!     Ok(tx)
//! }
//! ```
//!
//! See the [transaction](transaction/index.html) and [script](script/index.html) modules for more details.

// Core builder error is always available
pub use error::*;

// Script building is always available with transaction feature
pub use script::*;

// Transaction building is always available with transaction feature
pub use transaction::*;

// Utility functions for transaction building
pub use utils::*;

// Core modules - available with transaction feature
mod error;
mod script;
mod transaction;
mod utils;

// Contract invocation specific functionality
#[cfg(feature = "contract-invoke")]
pub mod invoke;

// Contract deployment specific functionality
#[cfg(feature = "contract-deploy")]
pub mod deploy;

// Internal function for testing
#[doc(hidden)]
pub fn add(left: usize, right: usize) -> usize {
	left + right
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn it_works() {
		let result = add(2, 2);
		assert_eq!(result, 4);
	}
}
