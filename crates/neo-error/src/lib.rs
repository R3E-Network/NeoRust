pub mod transaction_error;
pub mod type_error;
pub mod crypto_error;
pub mod builder_error;
pub mod codec_error;
pub mod contract_error;
pub mod provider_error;
pub mod nep2_error;
pub mod sign_error;

use thiserror::Error;
pub use crate::builder_error::BuilderError;
pub use crate::crypto_error::CryptoError;
pub use crate::transaction_error::TransactionError;
pub use crate::type_error::TypeError;
pub use crate::codec_error::CodecError;
pub use crate::contract_error::ContractError;
pub use crate::provider_error::{
    ProviderError, 
    to_provider_error, 
    to_serialization_error, 
    to_network_error, 
    to_rpc_error
};
pub use crate::nep2_error::Nep2Error;
pub use crate::sign_error::SignError;

/// The main error type for the Neo Rust SDK.
#[derive(Debug, Error)]
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

	/// An error for invalid configuration.
	#[error("Invalid configuration: {0}")]
	InvalidConfiguration(String),

	/// A runtime error.
	#[error("Runtime error: {0}")]
	Runtime(String),

	/// An error for invalid data.
	#[error("Invalid data: {0}")]
	InvalidData(String),

	/// An error for unsupported operations.
	#[error("Unsupported operation: {0}")]
	UnsupportedOperation(String),

	/// A transaction error.
	#[error("Transaction error: {0}")]
	Transaction(String),

	/// An error for invalid scripts.
	#[error("Invalid script: {0}")]
	InvalidScript(String),

	/// An error for invalid format.
	#[error("Invalid format: {0}")]
	InvalidFormat(String),

	/// An error for when Neo is not initialized.
	#[error("Neo not initialized")]
	NeoNotInitialized,

	/// An error for unexpected return types.
	#[error("Unexpected returned type: {0}")]
	UnexpectedReturnType(String),

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
	#[error("Invalid encoding: {0}")]
	InvalidEncoding(String),

	/// An error for invalid op codes.
	#[error("Invalid op code")]
	InvalidOpCode,

	/// An error for invalid arguments.
	#[error("Invalid argument: {0}")]
	InvalidArgError(String),

	/// An error for invalid Neo names.
	#[error("Invalid Neo name: {0}")]
	InvalidNeoName(String),

	/// An error for numeric overflow.
	#[error("Numeric overflow")]
	NumericOverflow,

	/// An error for WIF-related issues.
	#[error("WIF error: {0}")]
	WifError(String),

	/// A codec error.
	#[error(transparent)]
	CodecError(#[from] CodecError),

	/// A crypto error.
	#[error(transparent)]
	CryptoError(#[from] CryptoError),

	/// A transaction error.
	#[error(transparent)]
	TransactionError(#[from] TransactionError),

	/// A builder error.
	#[error(transparent)]
	BuilderError(#[from] BuilderError),

	/// A type error.
	#[error(transparent)]
	TypeError(#[from] TypeError),

	/// A provider error.
	#[error(transparent)]
	ProviderError(#[from] ProviderError),

	/// A reqwest error.
	#[error("HTTP error: {0}")]
	ReqwestError(String),

	/// A serde JSON error.
	#[error("JSON error: {0}")]
	SerdeJsonError(String),

	/// A URL parse error.
	#[error("URL parse error: {0}")]
	UrlParseError(String),

	/// An IO error.
	#[error("IO error: {0}")]
	IoError(String),

	/// A UTF-8 error.
	#[error("UTF-8 error: {0}")]
	Utf8Error(String),

	/// A base64 decode error.
	#[error("Base64 decode error: {0}")]
	Base64DecodeError(String),

	/// A hex decode error.
	#[error("Hex decode error: {0}")]
	HexDecodeError(String),

	/// A NEP-2 error.
	#[error(transparent)]
	Nep2Error(#[from] Nep2Error),

	/// A sign error.
	#[error(transparent)]
	SignError(#[from] SignError),

	/// A custom error.
	#[error("{0}")]
	Custom(String),
}

// Common result type
pub type Result<T> = std::result::Result<T, NeoError>; 