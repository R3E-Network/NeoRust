use thiserror::Error;

use std::{error::Error as StdError, fmt};

// Define error types directly to avoid circular dependencies
#[cfg(feature = "crypto-standard")]
use crate::neo_crypto::error::{CryptoError, Nep2Error, SignError};

// These will be defined later when we implement the corresponding modules
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodecError(pub String);

impl fmt::Display for CodecError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "Codec error: {}", self.0)
	}
}

impl StdError for CodecError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractError(pub String);

impl fmt::Display for ContractError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "Contract error: {}", self.0)
	}
}

impl StdError for ContractError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolError(pub String);

impl fmt::Display for ProtocolError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "Protocol error: {}", self.0)
	}
}

impl StdError for ProtocolError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderError(pub String);

// Add these associated variants to make http-client feature work
impl ProviderError {
	pub fn CustomError(msg: String) -> Self {
		Self(format!("Custom error: {}", msg))
	}
	
	pub fn ParseError(msg: String) -> Self {
		Self(format!("Parse error: {}", msg))
	}
	
	pub fn JsonRpcError(msg: String) -> Self {
		Self(format!("JSON RPC error: {}", msg))
	}
	
	pub fn ConnectionError(msg: String) -> Self {
		Self(format!("Connection error: {}", msg))
	}
	
	pub fn DeserializationError(msg: String) -> Self {
		Self(format!("Deserialization error: {}", msg))
	}
	
	pub fn InvalidResponse(msg: String) -> Self {
		Self(format!("Invalid response: {}", msg))
	}
}

impl fmt::Display for ProviderError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "Provider error: {}", self.0)
	}
}

impl StdError for ProviderError {}

// Add implementations for conversion from common error types
impl From<serde_json::Error> for ProviderError {
	fn from(err: serde_json::Error) -> Self {
		Self(format!("JSON error: {}", err))
	}
}

// Make this impl conditional on http-client or websocket features
#[cfg(any(feature = "http-client", feature = "websocket"))]
impl From<reqwest::Error> for ProviderError {
	fn from(err: reqwest::Error) -> Self {
		Self(format!("HTTP error: {}", err))
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionError(pub String);

impl fmt::Display for TransactionError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "Transaction error: {}", self.0)
	}
}

impl StdError for TransactionError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalletError(pub String);

impl fmt::Display for WalletError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "Wallet error: {}", self.0)
	}
}

impl StdError for WalletError {}

// Create a specific conversion from wallet errors to this general error
#[cfg(feature = "wallet")]
impl From<crate::neo_wallets::WalletError> for WalletError {
	fn from(err: crate::neo_wallets::WalletError) -> Self {
		WalletError(format!("{}", err))
	}
}

// Backwards compatibility helpers
impl From<String> for WalletError {
	fn from(s: String) -> Self {
		WalletError(s)
	}
}

impl From<&str> for WalletError {
	fn from(s: &str) -> Self {
		WalletError(s.to_string())
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeError {
	InvalidNeoName(String),
	General(String)
}

impl fmt::Display for TypeError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			TypeError::InvalidNeoName(msg) => write!(f, "Invalid Neo name error: {}", msg),
			TypeError::General(msg) => write!(f, "Type error: {}", msg),
		}
	}
}

impl StdError for TypeError {}

// Define SignError for when crypto-standard is not enabled
#[cfg(not(feature = "crypto-standard"))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignError {
	RecoverFailed,
	InvalidSignature,
}

#[cfg(not(feature = "crypto-standard"))]
impl fmt::Display for SignError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			SignError::RecoverFailed => write!(f, "Failed to recover public key from signature"),
			SignError::InvalidSignature => write!(f, "Invalid signature"),
		}
	}
}

#[cfg(not(feature = "crypto-standard"))]
impl StdError for SignError {}

#[derive(Error, Debug)]
pub enum NeoError {
	#[error("Illegal argument: {0}")]
	IllegalArgument(String),
	#[error("Illegal state: {0}")]
	Deserialization(String),
	#[error("Illegal state: {0}")]
	IllegalState(String),
	#[error("Index out of bounds: {0}")]
	IndexOutOfBounds(String),
	#[error("Invalid configuration: {0}")]
	InvalidConfiguration(String),
	#[error("Runtime error: {0}")]
	Runtime(String),
	#[error("Invalid data: {0}")]
	InvalidData(String),
	#[error("Unsupported operation: {0}")]
	UnsupportedOperation(String),
	#[error("Transaction error: {0}")]
	Transaction(String),
	#[error("Invalid script: {0}")]
	InvalidScript(String),
	#[error("Invalid format")]
	InvalidFormat,
	#[error("neo-rs not initialized")]
	NeoNotInitialized,
	#[error("Contract error: {0}")]
	ContractError(#[from] ContractError),
	#[error("Wallet error: {0}")]
	WalletError(#[from] WalletError),
	#[error("Sign error: {0}")]
	SignError(#[from] SignError),
	#[error("Transaction error: {0}")]
	TransactionError(#[from] TransactionError),
	#[error("Unexpected returned type")]
	UnexpectedReturnType,
	#[error("Invalid private key")]
	InvalidPrivateKey,
	#[error("Invalid public key")]
	InvalidPublicKey,
	#[error("Invalid address")]
	InvalidAddress,
	#[error("Invalid signature")]
	InvalidSignature,
	#[error("Invalid encoding {0}")]
	InvalidEncoding(String),
	#[error("Invalid op code")]
	InvalidOpCode,
	#[error("Numeric overflow")]
	NumericOverflow,
	#[error("Wif error {0}")]
	WifError(String),
	#[error("Provider error: {0}")]
	ProviderError(#[from] ProviderError),
	#[error("Codec error: {0}")]
	CodecError(#[from] CodecError),
	#[error("Type error: {0}")]
	TypeError(#[from] TypeError),
	#[error("Protocol error: {0}")]
	ProtocolError(#[from] ProtocolError),
	#[error("JSON RPC error: {0}")]
	JsonRpcError(String),
	#[error("IO error: {0}")]
	IoError(#[from] std::io::Error),
	#[error("Serialization error: {0}")]
	SerializationError(String),
}

impl Into<TransactionError> for NeoError {
	fn into(self) -> TransactionError {
		TransactionError(self.to_string())
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

// Define BuilderError directly to avoid circular dependencies
#[derive(Debug)]
pub enum BuilderError {
	InvalidScript(String),
	InvalidOperation,
	InvalidArgument,
	InvalidState,
	InvalidInvocation,
	StackOverflow,
	OutOfGas,
	OutOfMemory,
	OutOfCycles,
	UnknownError,
	UnsupportedOperation(String),
	SignerConfiguration(String),
	TransactionConfiguration(String),
	InvalidConfiguration(String),
	TooManySigners(String),
	IllegalState(String),
	IllegalArgument(String),
	CodecError(CodecError),
	#[cfg(feature = "crypto-standard")]
	CryptoError(CryptoError),
	ProviderError(ProviderError),
	TransactionError(Box<TransactionError>),
}

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
			#[cfg(feature = "crypto-standard")]
			BuilderError::CryptoError(err) => NeoError::from(err),
			#[cfg(not(feature = "crypto-standard"))]
			_ => NeoError::IllegalState("Crypto feature not enabled".to_string()),
			BuilderError::ProviderError(err) => NeoError::ProviderError(err),
			BuilderError::TransactionError(err) => NeoError::TransactionError(*err),
		}
	}
}

#[cfg(feature = "crypto-standard")]
impl From<CryptoError> for NeoError {
	fn from(err: CryptoError) -> Self {
		match err {
			CryptoError::InvalidPassphrase(msg) =>
				NeoError::IllegalArgument(format!("Invalid passphrase: {}", msg)),
			CryptoError::InvalidFormat(_) => NeoError::InvalidFormat,
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

#[cfg(feature = "crypto-standard")]
impl From<Nep2Error> for NeoError {
	fn from(err: Nep2Error) -> Self {
		match err {
			Nep2Error::InvalidPassphrase(msg) =>
				NeoError::IllegalArgument(format!("Invalid NEP-2 passphrase: {}", msg)),
			Nep2Error::InvalidFormat(msg) =>
				NeoError::InvalidEncoding(format!("Invalid NEP-2 format: {}", msg)),
		}
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
