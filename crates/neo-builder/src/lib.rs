//! # Neo Builder
//!
//! Transaction and script building utilities for the NeoRust SDK.
//!
//! This crate provides tools for constructing and manipulating Neo N3 blockchain transactions and scripts, including:
//!
//! - Transaction building and signing
//! - Script construction for smart contract invocation
//! - Witness and signer management
//! - Transaction attribute handling
//! - Fee calculation
//!
//! ## Usage
//!
//! ```rust,ignore
//! use neo_builder::{TransactionBuilder, ScriptBuilder};
//! use neo_types::{ContractParameter, Signer, WitnessScope};
//! use neo_protocol::account::Account;
//! use std::str::FromStr;
//!
//! // Create a transaction builder
//! let mut tx_builder = TransactionBuilder::new();
//!
//! // Configure the transaction
//! tx_builder
//!     .version(0)
//!     .nonce(1234)
//!     .valid_until_block(block_height + 100)
//!     .script(script_bytes)
//!     .add_signer(Signer::called_by_entry(sender_script_hash));
//!
//! // Build the transaction
//! let transaction = tx_builder.build();
//!
//! // Create a script for contract invocation
//! let script = ScriptBuilder::new()
//!     .contract_call(
//!         &contract_hash,
//!         "transfer",
//!         &[
//!             ContractParameter::hash160(&sender_script_hash),
//!             ContractParameter::hash160(&recipient_script_hash),
//!             ContractParameter::integer(1_0000_0000), // 1 GAS (8 decimals)
//!             ContractParameter::any(None),
//!         ],
//!         None,
//!     )
//!     .to_bytes();
//! ```

#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms, unreachable_pub)]
#![deny(rustdoc::broken_intra_doc_links)]

mod error;
mod script;
mod transaction;
mod utils;

// Create a builder module to fix import issues
pub mod builder {
    // Re-export transaction types
    pub use crate::transaction::{
        Signer, SignerType, Transaction, TransactionAttribute, TransactionBuilder,
        TransactionError, VerificationScript, Witness, WitnessRule, WitnessAction, WitnessCondition,
    };
    
    // Re-export script types
    pub use crate::script::{ScriptBuilder, InteropService};
}

// Re-export all public items
pub use error::*;
pub use script::*;
pub use transaction::*;
pub use utils::*;

// Re-export builder module items
pub use builder::{
    Signer, SignerType, Transaction, TransactionAttribute, TransactionBuilder,
    TransactionError, VerificationScript, Witness, WitnessRule, WitnessAction, WitnessCondition,
    ScriptBuilder, InteropService,
};
