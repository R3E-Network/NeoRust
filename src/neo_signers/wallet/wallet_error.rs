use neo::prelude::{BuilderError, CryptoError, TransactionError};
use p256::ecdsa;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WalletError {
	#[error("Account state error: {0}")]
	AccountState(String),
	#[error("No key pair")]
	NoKeyPair,
	/// Error propagated from p256's ECDSA module
	#[error(transparent)]
	EcdsaError(#[from] ecdsa::Error),
	/// Error propagated from the hex crate.
	#[error(transparent)]
	HexError(#[from] hex::FromHexError),
	/// Error propagated by IO operations
	#[error(transparent)]
	IoError(#[from] std::io::Error),
	#[error("No default account")]
	NoDefaultAccount,
	#[error("Invalid key pair")]
	SignHashError,
	#[error(transparent)]
	CryptoError(#[from] CryptoError),
	#[error(transparent)]
	TransactionError(#[from] TransactionError),
	#[error(transparent)]
	BuilderError(#[from] BuilderError),
}
