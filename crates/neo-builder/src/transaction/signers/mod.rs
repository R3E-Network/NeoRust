//! This module contains implementations for different types of signers in the NEO blockchain.
//!
//! It includes:
//! - `AccountSigner`: Represents an account-based signer.
//! - `ContractSigner`: Represents a contract-based signer.
//! - `TransactionSigner`: Represents a transaction-specific signer.
//! - `Signer`: An enum that can be any of the above signer types.
//!
//! This module also provides traits and utilities for working with signers,
//! including serialization, deserialization, and common signer operations.
//!
//! # Usage
//!
//! To use the signers in your NEO blockchain transactions:
//!
//! 1. Import the necessary types:
//!    ```rust
//!    use NeoRust::neo_builder::transaction::signers::{AccountSigner, ContractSigner, TransactionSigner, Signer};
//!    ```
//!
//! 2. Create a signer based on your needs:
//!    ```rust
//!    // For an account-based signer
//!    let account = Account::from_wif("your_wif_here").unwrap();
//!    let account_signer = AccountSigner::called_by_entry(&account).unwrap();
//!
//!    // For a contract-based signer
//!    let contract_hash = H160::from_str("your_contract_hash_here").unwrap();
//!    let contract_signer = ContractSigner::called_by_entry(contract_hash, &[]);
//!
//!    // For a transaction-specific signer
//!    let transaction_signer = TransactionSigner::new(account.get_script_hash(), vec![WitnessScope::CalledByEntry]);
//!    ```
//!
//! 3. Use the signer in your transaction:
//!    ```rust
//!    let mut tx_builder = TransactionBuilder::new();
//!    tx_builder.add_signer(account_signer);
//!    // ... add other transaction details ...
//!    let tx = tx_builder.build().unwrap();
//!    ```
//!
//! 4. You can also convert between signer types using the `Signer` enum:
//!    ```rust
//!    let generic_signer: Signer = account_signer.into();
//!    ```
//!
//! Remember to handle errors and manage scopes, allowed contracts, and other signer properties as needed for your specific use case.

pub use account_signer::*;
pub use contract_signer::*;
pub use signer::*;
pub use transaction_signer::*;

mod account_signer;
mod contract_signer;
mod signer;
mod transaction_signer;
