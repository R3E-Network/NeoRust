#![doc = include_str!("../../README.md")]
#![deny(unsafe_code, rustdoc::broken_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod wallet;
mod wallet_trait;
pub use wallet::*;
pub use wallet_trait::WalletTrait;

/// A wallet instantiated with a locally stored private key
pub type LocalSigner = WalletSigner<Account>;

#[cfg(all(feature = "yubihsm", not(target_arch = "wasm32")))]
/// A wallet instantiated with a YubiHSM
pub type YubiWallet = WalletSigner<yubihsm::ecdsa::Signer<NistP256>>;

#[cfg(all(feature = "ledger", not(target_arch = "wasm32")))]
mod ledger;
#[cfg(all(feature = "ledger", not(target_arch = "wasm32")))]
pub use ledger::{
	app::LedgerNeo as Ledger,
	types::{DerivationType as HDPath, LedgerError},
};

#[cfg(all(feature = "yubihsm", not(target_arch = "wasm32")))]
pub use yubihsm;

#[cfg(all(feature = "yubihsm", not(target_arch = "wasm32")))]
mod yubi;

mod error;
pub use error::*;
mod wallet_signer;

use neo::prelude::{Account, Address, Transaction};
use p256::NistP256;
use std::error::Error;
pub use wallet_signer::WalletSigner;
