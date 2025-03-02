//! # Neo Wallet Implementation
//!
//! This module contains the core wallet implementation for the Neo N3 blockchain.
//!
//! ## Feature Flags
//!
//! - **wallet**: Core wallet functionality (always enabled)
//! - **wallet-standard**: Standard wallet features including NEP-6 format support
//! - **wallet-secure**: Enhanced security features
//! - **bip39**: BIP-39 mnemonic phrase support

// Core wallet types - always available
pub use wallet::*;
pub use wallet_error::*;
pub use wallet_detailed_error::*;
pub use nep6wallet::*;
pub use backup::*;

// Standard wallet features - only with wallet-standard feature
#[cfg(feature = "wallet-standard")]
#[cfg_attr(docsrs, doc(cfg(feature = "wallet-standard")))]
pub use backup::*;

#[cfg(feature = "wallet-standard")]
#[cfg_attr(docsrs, doc(cfg(feature = "wallet-standard")))]
pub use nep6account::*;

#[cfg(feature = "wallet-standard")]
#[cfg_attr(docsrs, doc(cfg(feature = "wallet-standard")))]
pub use nep6contract::*;

#[cfg(feature = "wallet-standard")]
#[cfg_attr(docsrs, doc(cfg(feature = "wallet-standard")))]
pub use nep6wallet::*;

#[cfg(feature = "wallet-standard")]
#[cfg_attr(docsrs, doc(cfg(feature = "wallet-standard")))]
pub use wallet_standard::*;

// Core wallet module - always available
pub mod wallet;
pub mod wallet_error;
pub mod wallet_detailed_error;
pub mod nep6wallet;
pub mod backup;

// Standard wallet modules - only with wallet-standard feature
#[cfg(feature = "wallet-standard")]
pub mod nep6account;

#[cfg(feature = "wallet-standard")]
pub mod nep6contract;

#[cfg(feature = "wallet-standard")]
pub mod wallet_standard;

// Re-export wallet errors to allow for easy access
pub use wallet_error::WalletError;
pub use wallet_detailed_error::WalletDetailedError;
