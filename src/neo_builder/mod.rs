//! # Neo Builder Module (v0.1.4)
//!
//! Advanced tooling for constructing Neo N3 transactions and smart contract scripts.
//!
//! ## Overview
//!
//! The neo_builder module provides a comprehensive set of utilities for constructing
//! and manipulating Neo N3 transactions and scripts. It offers a flexible API for
//! building various types of transactions, from simple transfers to complex
//! multi-signature contract invocations.
//!
//! ## Key Components
//!
//! ### Transaction Building
//!
//! - **Transaction Builder**: Fluent API for creating and configuring transactions
//! - **Fee Calculation**: Automatic network and system fee calculation
//! - **Signer Management**: Support for multiple transaction signers with different scopes
//! - **Witness Configuration**: Tools for creating and managing transaction witnesses
//! - **Attribute Handling**: Support for transaction attributes
//!
//! ### Script Construction
//!
//! - **Script Builder**: Create VM scripts for contract invocation
//! - **Opcode Support**: Full support for Neo VM opcodes
//! - **Parameter Handling**: Type-safe handling of contract parameters
//! - **Verification Scripts**: Utilities for building signature verification scripts
//!
//! ### Advanced Features
//!
//! - **Multi-signature Support**: Create and work with multi-signature accounts
//! - **Helper Methods**: Convenience methods for common operations
//! - **Serialization**: Serialization utilities for network transmission
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

pub use error::*;
pub use script::*;
pub use transaction::*;
pub use utils::*;

mod error;
mod script;
mod transaction;
mod utils;

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
