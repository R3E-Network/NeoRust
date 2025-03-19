//! # Neo Error Types (v0.1.8)
//!
//! This module provides a unified error handling system for the NeoRust SDK.
//!
//! ## Overview
//!
//! The `NeoError` enum acts as the central error type for the entire SDK, providing
//! specific error categories and conversion from various error types used throughout
//! the codebase. This approach simplifies error handling by allowing all operations
//! to return a consistent error type.
//!
//! ## Usage
//!
//! ```rust
//! use neo::prelude::*;
//! use std::str::FromStr;
//!
//! fn example() -> Result<(), NeoError> {
//!     // NeoError can be created directly
//!     if some_condition {
//!         return Err(NeoError::InvalidAddress);
//!     }
//!     
//!     // Or it can be created with a message
//!     let error_msg = "Invalid parameter value";
//!     return Err(NeoError::IllegalArgument(error_msg.to_string()));
//!     
//!     // Standard library errors are also converted automatically
//!     let io_result = std::fs::File::open("non_existent_file");
//!     let file = io_result?; // This will return NeoError::IoError if the file doesn't exist
//!     
//!     Ok(())
//! }
//! ```

use thiserror::Error;

/// Comprehensive error type for the NeoRust SDK.
///
/// This enum provides a unified error handling system for all operations in the SDK.
/// It includes specific error types for different categories of errors and implements
/// conversions from various error types used throughout the codebase.
#[derive(Error, Debug)]
pub enum NeoError {
	/// An error for illegal or invalid argument values.
	#[error("Illegal argument: {0}")]
	IllegalArgument(String),

	/// An error that occurs during deserialization.
	#[error("Deserialization error: {0}")]
	Deserialization(String),

	/// An error for illegal state conditions.
	#[error("Illegal state: {0}")]
	IllegalState(String),

	/// An error for index out of bounds conditions.
	#[error("Index out of bounds: {0}")]
	IndexOutOfBounds(String),

	/// An error for invalid configuration settings.
	#[error("Invalid configuration: {0}")]
	InvalidConfiguration(String),

	/// A general runtime error.
	#[error("Runtime error: {0}")]
	Runtime(String),

	/// An error for invalid data.
	#[error("Invalid data: {0}")]
	InvalidData(String),

	/// An error for unsupported operations.
	#[error("Unsupported operation: {0}")]
	UnsupportedOperation(String),

	/// A general transaction error.
	#[error("Transaction error: {0}")]
	Transaction(String),

	/// An error for invalid scripts.
	#[error("Invalid script: {0}")]
	InvalidScript(String),

	/// An error for invalid format.
	#[error("Invalid format")]
	InvalidFormat,

	/// An error indicating that NeoRust has not been initialized.
	#[error("neo-rs not initialized")]
	NeoNotInitialized,

	/// An error related to smart contracts.
	#[error("Contract error: {0}")]
	ContractError(#[from] ContractError),

	/// An error for wallet-related issues.
	#[error("Wallet error: {0}")]
	WalletError(#[from] WalletError),

	/// An error for signing-related issues.
	#[error("Sign error: {0}")]
	SignError(#[from] SignError),

	/// A general transaction error.
	#[error("Transaction error: {0}")]
	TransactionError(#[from] TransactionError),

	/// An error for unexpected returned types.
	#[error("Unexpected returned type")]
	UnexpectedReturnType,

	/// An error for invalid private keys.
	#[error("Invalid private key")]
	InvalidPrivateKey,

	/// An error for invalid public keys.
	#[error("Invalid public key")]
	InvalidPublicKey,

	/// An error for invalid addresses.
	#[error("Invalid address")]
	InvalidAddress,

	/// An error for invalid signatures.
	#[error("Invalid signature")]
	InvalidSignature,

	/// An error for invalid encoding.
	#[error("Invalid encoding {0}")]
	InvalidEncoding(String),

	/// An error for invalid op codes.
	#[error("Invalid op code")]
	InvalidOpCode,

	/// An error for numeric overflow.
	#[error("Numeric overflow")]
	NumericOverflow,

	/// An error for WIF (Wallet Import Format) issues.
	#[error("Wif error {0}")]
	WifError(String),

	/// An error for provider-related issues.
	#[error("Provider error: {0}")]
	ProviderError(#[from] ProviderError),

	/// An error for codec-related issues.
	#[error("Codec error: {0}")]
	CodecError(#[from] CodecError),

	/// An error for type-related issues.
	#[error("Type error: {0}")]
	TypeError(#[from] TypeError),

	/// An error for protocol-related issues.
	#[error("Protocol error: {0}")]
	ProtocolError(#[from] ProtocolError),

	/// An error for JSON RPC-related issues.
	#[error("JSON RPC error: {0}")]
	JsonRpcError(String),

	/// An error for IO-related issues.
	#[error("IO error: {0}")]
	IoError(#[from] std::io::Error),

	/// An error for serialization-related issues.
	#[error("Serialization error: {0}")]
	SerializationError(String),
}

impl Into<TransactionError> for NeoError {
	fn into(self) -> TransactionError {
		TransactionError::TransactionConfiguration(self.to_string())
	}
}

impl From<serde_json::Error> for NeoError {
	fn from(err: serde_json::Error) -> Self {
		NeoError::SerializationError(err.to_string())
	}
}

impl From<String> for NeoError {
	fn from(err: String) -> Self {
		NeoError::IllegalState(err)
	}
}

impl From<&str> for NeoError {
	fn from(err: &str) -> Self {
		NeoError::IllegalState(err.to_string())
	}
}

use crate::{
	builder::{BuilderError, TransactionError},
	codec::CodecError,
	crypto::{CryptoError, Nep2Error, SignError},
	neo_clients::ProviderError,
	neo_contract::ContractError,
	neo_protocol::ProtocolError,
	neo_wallets::WalletError,
	TypeError,
};

impl From<BuilderError> for NeoError {
	fn from(err: BuilderError) -> Self {
		match err {
			BuilderError::InvalidScript(msg) => NeoError::InvalidScript(msg),
			BuilderError::InvalidOperation =>
				NeoError::UnsupportedOperation("Invalid operation".to_string()),
			BuilderError::InvalidArgument =>
				NeoError::IllegalArgument("Invalid argument".to_string()),
			BuilderError::InvalidState => NeoError::IllegalState("Invalid state".to_string()),
			BuilderError::InvalidInvocation =>
				NeoError::IllegalState("Invalid invocation".to_string()),
			BuilderError::StackOverflow => NeoError::Runtime("Stack overflow".to_string()),
			BuilderError::OutOfGas => NeoError::Runtime("Out of gas".to_string()),
			BuilderError::OutOfMemory => NeoError::Runtime("Out of memory".to_string()),
			BuilderError::OutOfCycles => NeoError::Runtime("Out of cycles".to_string()),
			BuilderError::UnknownError => NeoError::Runtime("Unknown error".to_string()),
			BuilderError::UnsupportedOperation(msg) => NeoError::UnsupportedOperation(msg),
			BuilderError::SignerConfiguration(msg) =>
				NeoError::IllegalState(format!("Signer configuration error: {}", msg)),
			BuilderError::TransactionConfiguration(msg) => NeoError::Transaction(msg),
			BuilderError::InvalidConfiguration(msg) => NeoError::InvalidConfiguration(msg),
			BuilderError::TooManySigners(msg) =>
				NeoError::IllegalState(format!("Too many signers: {}", msg)),
			BuilderError::IllegalState(msg) => NeoError::IllegalState(msg),
			BuilderError::IllegalArgument(msg) => NeoError::IllegalArgument(msg),
			BuilderError::CodecError(err) => NeoError::CodecError(err),
			BuilderError::CryptoError(err) => NeoError::from(err),
			BuilderError::ProviderError(err) => NeoError::ProviderError(err),
			BuilderError::TransactionError(err) => NeoError::TransactionError(*err),
		}
	}
}

impl From<CryptoError> for NeoError {
	fn from(err: CryptoError) -> Self {
		match err {
			CryptoError::InvalidPassphrase(msg) =>
				NeoError::IllegalArgument(format!("Invalid passphrase: {}", msg)),
			CryptoError::InvalidFormat(msg) => NeoError::InvalidFormat,
			CryptoError::HeaderOutOfRange(byte) =>
				NeoError::IllegalArgument(format!("Header byte out of range: {}", byte)),
			CryptoError::RecoverFailed =>
				NeoError::IllegalState("Could not recover public key from signature".to_string()),
			CryptoError::InvalidPublicKey => NeoError::InvalidPublicKey,
			CryptoError::InvalidPrivateKey => NeoError::InvalidPrivateKey,
			CryptoError::P256Error(err) => NeoError::IllegalState(format!("P256 error: {}", err)),
			CryptoError::SigningError => NeoError::SignError(SignError::RecoverFailed),
			CryptoError::SignatureVerificationError => NeoError::InvalidSignature,
			CryptoError::FromHexError(err) =>
				NeoError::InvalidEncoding(format!("Hex error: {}", err)),
			CryptoError::DecryptionError(msg) =>
				NeoError::IllegalState(format!("Decryption error: {}", msg)),
			CryptoError::KeyError(msg) => NeoError::IllegalState(format!("Key error: {}", msg)),
		}
	}
}

impl From<Nep2Error> for NeoError {
	fn from(err: Nep2Error) -> Self {
		match err {
			Nep2Error::InvalidPassphrase(msg) =>
				NeoError::IllegalArgument(format!("Invalid passphrase: {}", msg)),
			Nep2Error::InvalidFormat(msg) =>
				NeoError::InvalidEncoding(format!("Invalid NEP-2 format: {}", msg)),
			Nep2Error::InvalidPrivateKey(msg) =>
				NeoError::InvalidPrivateKey,
			Nep2Error::EncryptionError(msg) =>
				NeoError::IllegalState(format!("NEP-2 encryption error: {}", msg)),
			Nep2Error::DecryptionError(msg) =>
				NeoError::IllegalState(format!("NEP-2 decryption error: {}", msg)),
			Nep2Error::VerificationFailed(msg) =>
				NeoError::InvalidSignature,
			Nep2Error::ScryptError(msg) =>
				NeoError::IllegalState(format!("NEP-2 scrypt error: {}", msg)),
			Nep2Error::Base58Error(msg) =>
				NeoError::InvalidEncoding(format!("NEP-2 Base58 error: {}", msg)),
		}
	}
}

// Implement From for reqwest::Error to handle HTTP errors
impl From<reqwest::Error> for NeoError {
	fn from(err: reqwest::Error) -> Self {
		NeoError::IoError(std::io::Error::new(
			std::io::ErrorKind::Other,
			format!("HTTP error: {}", err),
		))
	}
}

// Implement From for hex::FromHexError to handle hex decoding errors
impl From<hex::FromHexError> for NeoError {
	fn from(err: hex::FromHexError) -> Self {
		NeoError::InvalidEncoding(format!("Hex error: {}", err))
	}
}

// Implement From for std::num::ParseIntError to handle integer parsing errors
impl From<std::num::ParseIntError> for NeoError {
	fn from(err: std::num::ParseIntError) -> Self {
		NeoError::IllegalArgument(format!("Integer parsing error: {}", err))
	}
}

// Implement From for std::str::Utf8Error to handle UTF-8 decoding errors
impl From<std::str::Utf8Error> for NeoError {
	fn from(err: std::str::Utf8Error) -> Self {
		NeoError::InvalidEncoding(format!("UTF-8 error: {}", err))
	}
}
