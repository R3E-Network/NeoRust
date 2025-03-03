#![doc = include_str!("../../README.md")]
#![deny(unsafe_code, rustdoc::broken_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! # Neo Wallets
//!
//! Wallet management for the Neo N3 blockchain.
//!
//! ## Overview
//!
//! The neo_wallets module provides comprehensive wallet management functionality for the Neo N3 blockchain.
//! It includes:
//!
//! - Wallet creation and loading
//! - NEP-6 wallet standard support
//! - BIP-39 mnemonic phrase support
//! - Transaction signing
//! - Key management and derivation
//! - Hardware wallet integration (Ledger)
//! - Secure key storage
//! - Wallet backup and recovery
//!
//! This module enables secure management of private keys and accounts, allowing users to interact
//! with the Neo N3 blockchain in a secure manner.
//!
//! ## Examples
//!
//! ### Creating and using a wallet
//!
//! ```rust
//! use neo::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a new wallet
//!     let wallet = Wallet::new("my_wallet", "password123")?;
//!     
//!     // Create a new account in the wallet
//!     let account = wallet.create_account()?;
//!     println!("New account address: {}", account.address());
//!     
//!     // Save the wallet to a file
//!     wallet.save("my_wallet.json")?;
//!     
//!     // Load a wallet from a file
//!     let loaded_wallet = Wallet::load("my_wallet.json", Some("password123"))?;
//!     
//!     // Get the default account
//!     let default_account = loaded_wallet.default_account()?;
//!     
//!     // Sign a message with the default account
//!     let message = b"Hello, Neo!";
//!     let signature = default_account.sign(message)?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### Using a wallet to sign a transaction
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
//!     // Load a wallet
//!     let wallet = Wallet::load("my_wallet.json", Some("password123"))?;
//!     
//!     // Create a wallet signer
//!     let signer = WalletSigner::from_wallet(wallet, None)?;
//!     
//!     // Create a transaction builder
//!     let mut tx_builder = TransactionBuilder::with_client(&client);
//!     
//!     // Build a transaction to transfer GAS
//!     let gas_token = ScriptHash::from_str("d2a4cff31913016155e38e474a2c06d08be276cf")?;
//!     let recipient = ScriptHash::from_str("5c9c3a340f4c28262e7042b908b5f7e7a4bcd7e7")?;
//!     let amount = 1_0000_0000; // 1 GAS (8 decimals)
//!     
//!     tx_builder
//!         .script(Some(ScriptBuilder::build_contract_call(
//!             &gas_token,
//!             "transfer",
//!             &[
//!                 ContractParameter::hash160(&signer.get_script_hash()),
//!                 ContractParameter::hash160(&recipient),
//!                 ContractParameter::integer(amount),
//!                 ContractParameter::any(None),
//!             ],
//!             None,
//!         )?))
//!         .signers(vec![Signer::calledByEntry(signer.get_script_hash())]);
//!     
//!     // Sign and send the transaction
//!     let tx = tx_builder.sign_with(&signer).await?;
//!     let tx_id = tx.send().await?;
//!     
//!     println!("Transaction sent: {}", tx_id);
//!     
//!     Ok(())
//! }
//! ```

#[cfg(feature = "ledger")]
pub use ledger::{HDPath, LedgerWallet};
use p256::NistP256;
#[cfg(all(feature = "yubihsm", not(target_arch = "wasm32")))]
pub use yubihsm;

pub use error::*;
pub use wallet::*;
pub use wallet_signer::WalletSigner;
pub use wallet_trait::WalletTrait;
use crate::neo_protocol::Account;

#[cfg(feature = "ledger")]
mod ledger;
mod wallet;
mod wallet_trait;

/// A wallet instantiated with a locally stored private key
pub type LocalWallet = WalletSigner<Account>;
// pub type LocalWallet = Wallet<ethers_core::k256::ecdsa::SigningKey>;

/// A wallet instantiated with a YubiHSM
pub type YubiWallet = WalletSigner<yubihsm::ecdsa::Signer<NistP256>>;

// #[cfg(all(feature = "yubihsm", not(target_arch = "wasm32")))]
mod yubi;

mod bip39_account;
mod error;
mod wallet_signer;
