#![doc = include_str!("../README.md")]
#![deny(unsafe_code, rustdoc::broken_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod wallet;
pub use wallet::*;

/// A wallet instantiated with a locally stored private key
pub type LocalWallet = Wallet<SigningKey>;

#[cfg(all(feature = "yubihsm", not(target_arch = "wasm32")))]
/// A wallet instantiated with a YubiHSM
pub type YubiWallet = Wallet<yubihsm::ecdsa::Signer<NistP256>>;

#[cfg(all(feature = "ledger", not(target_arch = "wasm32")))]
mod ledger;
#[cfg(all(feature = "ledger", not(target_arch = "wasm32")))]
pub use ledger::{
	app::LedgerNeo as Ledger,
	types::{DerivationType as HDPath, LedgerError},
};

#[cfg(all(feature = "yubihsm", not(target_arch = "wasm32")))]
pub use yubihsm;

mod error;

use async_trait::async_trait;
use neo::prelude::{Address, Secp256r1Signature, Transaction, Witness};
use p256::{ecdsa::SigningKey, NistP256};
use std::error::Error;

/// Trait for signing transactions and messages
///
/// Implement this trait to support different signing modes, e.g. Ledger, hosted etc.
#[async_trait]
pub trait Signer: std::fmt::Debug + Send + Sync {
	type Error: Error + Send + Sync;
	/// Signs the hash of the provided message after prefixing it
	async fn sign_message<S: Send + Sync + AsRef<[u8]>>(
		&self,
		message: S,
	) -> Result<Secp256r1Signature, Self::Error>;

	/// Add witness to transaction
	async fn get_witness(&self, message: &Transaction) -> Result<Witness, Self::Error>;

	/// Returns the signer's neo Address
	fn address(&self) -> Address;

	/// Returns the signer's network magic
	fn network_magic(&self) -> u32;

	/// Sets the signer's network magic
	#[must_use]
	fn with_network_magic<T: Into<u32>>(self, network_magic: T) -> Self;
}
