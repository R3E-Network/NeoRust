#![doc = include_str!("../../README.md")]
#![deny(unsafe_code, rustdoc::broken_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "ledger")]
pub use ledger::{HDPath, LedgerWallet};
use p256::NistP256;
#[cfg(all(feature = "yubihsm", not(target_arch = "wasm32")))]
pub use yubihsm;

pub use error::*;
use neo::prelude::Account;
pub use wallet::*;
pub use wallet_signer::WalletSigner;
pub use wallet_trait::WalletTrait;

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
