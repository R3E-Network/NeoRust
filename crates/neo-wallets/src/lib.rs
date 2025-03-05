//! # Neo Wallets
//!
//! Wallet management for the NeoRust SDK.
//!
//! This crate provides wallet functionality for the Neo N3 blockchain, including:
//!
//! - Wallet creation and management
//! - NEP-6 wallet standard support
//! - Account management
//! - Key storage and retrieval
//! - Wallet signing capabilities
//! - Hardware wallet support (Yubikey)
//! - Wallet backup and recovery
//!
//! ## Usage
//!
//! ```rust,ignore
//! use neo_wallets::{Wallet, Account, NEP6Account};
//! use neo_types::ScriptHash;
//! use std::str::FromStr;
//!
//! // Create a new wallet
//! let mut wallet = Wallet::new();
//!
//! // Add an account to the wallet
//! let account = Account::from_wif("KwkUAF4y4UQwQGY8RkRtddHX8FgDgpwdH2RYKQcnAi7fFkzYQUV3").unwrap();
//! wallet.add_account(account);
//!
//! // Save the wallet to a file
//! wallet.save_to_file("wallet.json").unwrap();
//!
//! // Load a wallet from a file
//! let loaded_wallet = Wallet::load_from_file("wallet.json").unwrap();
//!
//! // Sign a transaction with the wallet
//! let signature = wallet.sign_with_account(message, account_address).unwrap();
//! ```

#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms, unreachable_pub)]
#![deny(rustdoc::broken_intra_doc_links)]

mod bip39_account;
mod error;
mod ledger;
mod wallet;
mod wallet_signer;
mod wallet_trait;
mod yubi;

// Re-export all public items
pub use bip39_account::*;
pub use error::*;
pub use ledger::*;
pub use wallet::*;
pub use wallet_signer::*;
pub use wallet_trait::*;
pub use yubi::*;

// Type aliases for common wallet types
pub type LocalWallet = WalletSigner<neo_protocol::Account>;
pub type YubiWallet = WalletSigner<yubihsm::ecdsa::Signer<p256::NistP256>>;
