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

// Core wallet module - always available
mod wallet;
mod wallet_error;

// Standard wallet modules - only with wallet-standard feature
#[cfg(feature = "wallet-standard")]
mod backup;

#[cfg(feature = "wallet-standard")]
mod nep6account;

#[cfg(feature = "wallet-standard")]
mod nep6contract;

#[cfg(feature = "wallet-standard")]
mod nep6wallet;
