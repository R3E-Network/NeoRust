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
//! ## Feature Flags
//!
//! This module supports the following feature flags:
//!
//! - **wallet**: Core wallet functionality (always available when using this module)
//! - **wallet-standard**: Enhanced wallet features with standard file formats
//! - **wallet-hardware**: Support for hardware wallets like Ledger
//! - **wallet-secure**: Advanced security features for wallets
//! - **bip39**: Support for BIP-39 mnemonic phrases
//! - **yubikey**: Support for YubiHSM hardware security modules
//!
//! ## Module Structure
//!
//! To avoid circular dependencies, this module uses a layered approach:
//! 
//! 1. Core wallet types and basic functionality are in the base wallet feature
//! 2. Additional features like hardware support are in separate feature flags
//! 3. Error types use a two-way conversion system to prevent circular references
//!    between the core library error types and wallet-specific error types
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
//!     
//!     // Set a default account
//!     wallet.set_default_account(&account.address)?;
//!     
//!     // Get the wallet's default account
//!     let default_account = wallet.get_default_account()?;
//!     println!("Default account address: {}", default_account.address);
//!     
//!     // Save the wallet to disk
//!     wallet.save("my_wallet.json", Some("password123"))?;
//!     
//!     // Load the wallet from disk
//!     let loaded_wallet = Wallet::load("my_wallet.json", Some("password123"))?;
//!     
//!     // Sign a message with the default account
//!     let message = b"Hello, Neo!";
//!     let signature = loaded_wallet.sign_message(message)?;
//!     
//!     // Verify the signature
//!     assert!(loaded_wallet.verify_signature(message, &signature, &default_account.address)?);
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
//!     let tx_hash = client.send_raw_transaction(&tx).await?;
//!     
//!     println!("Transaction sent: {}", tx_hash);
//!     
//!     Ok(())
//! }
//! ```

// Core wallet types
pub use error::*;
pub use wallet_signer::*;
pub use wallet_trait::*;
pub use account_trait::*;

// Re-export the wallet module contents
pub use wallet::wallet::*;
pub use wallet::wallet_error::*;
pub use wallet::wallet_detailed_error::*;
pub use wallet::backup::*;
pub use wallet::nep6wallet::*;

#[cfg(feature = "wallet-standard")]
pub use wallet::nep6account::*;

#[cfg(feature = "wallet-standard")]
pub use wallet::nep6contract::*;

// BIP-39 support
#[cfg(feature = "bip39")]
#[cfg_attr(docsrs, doc(cfg(feature = "bip39")))]
pub use bip39_account::*;

// Hardware wallet support
#[cfg(feature = "wallet-hardware")]
#[cfg_attr(docsrs, doc(cfg(feature = "wallet-hardware")))]
pub use ledger::*;

// YubiHSM support
#[cfg(feature = "yubikey")]
#[cfg_attr(docsrs, doc(cfg(feature = "yubikey")))]
pub use yubi::*;

// Core wallet modules
mod error;
pub mod wallet;
mod wallet_signer;
mod wallet_trait;
mod account_trait;

// BIP-39 support module
#[cfg(feature = "bip39")]
mod bip39_account;

// Hardware wallet support module
#[cfg(feature = "wallet-hardware")]
mod ledger;

// YubiHSM support module
#[cfg(feature = "yubikey")]
mod yubi;

// Type aliases for wallet implementations
pub type LocalWallet = WalletSigner<wallet::wallet::Account>;

// Type aliases for hardware wallet implementations
#[cfg(feature = "yubikey")]
#[cfg_attr(docsrs, doc(cfg(feature = "yubikey")))]
pub type YubiWallet = WalletSigner<yubihsm::ecdsa::Signer<yubihsm::ecdsa::NistP256>>;

// Tests
#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_wallet_creation() {
		let wallet = Wallet::new().unwrap();
		assert!(wallet.accounts().is_empty());
	}
}
