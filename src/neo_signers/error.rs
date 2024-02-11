use hex::FromHexError;
use neo::prelude::{BuilderError, CryptoError, TypeError, WalletError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SignerError {
	#[error("Invalid passphrase: {0}")]
	InvalidPassphrase(String),
	#[error("Invalid address")]
	InvalidAddress,
	#[error(transparent)]
	BuilderError(#[from] BuilderError),
	#[error(transparent)]
	WalletError(#[from] WalletError),
	#[error(transparent)]
	FromHexError(#[from] FromHexError),
	#[error(transparent)]
	CryptoError(#[from] CryptoError),
	#[error(transparent)]
	RustcFromHexError(#[from] rustc_serialize::hex::FromHexError),
	#[error(transparent)]
	TypeError(#[from] TypeError),
}
