use thiserror::Error;

/// NEP-2 related errors
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum Nep2Error {
	/// Base58 decoding error.
	#[error("Base58 decode error: {0}")]
	Base58Decode(String),
	
	/// Invalid passphrase.
	#[error("Invalid passphrase: {0}")]
	InvalidPassphrase(String),
	
	/// Invalid format.
	#[error("Invalid format: {0}")]
	InvalidFormat(String),
} 