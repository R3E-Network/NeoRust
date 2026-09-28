//! Error types for neo-primitives

use alloc::string::String;

/// Result type for neo-primitives operations
pub type Result<T> = core::result::Result<T, Error>;

/// Error type for neo-primitives
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
	/// Invalid address format
	InvalidAddress(String),
	/// Invalid hash format
	InvalidHash(String),
	/// Invalid key format
	InvalidKey(String),
	/// Invalid signature
	InvalidSignature(String),
	/// Encoding error
	EncodingError(String),
	/// Decoding error
	DecodingError(String),
	/// Invalid script format
	InvalidScript(String),
}

impl core::fmt::Display for Error {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			Error::InvalidAddress(msg) => write!(f, "Invalid address: {}", msg),
			Error::InvalidHash(msg) => write!(f, "Invalid hash: {}", msg),
			Error::InvalidKey(msg) => write!(f, "Invalid key: {}", msg),
			Error::InvalidSignature(msg) => write!(f, "Invalid signature: {}", msg),
			Error::EncodingError(msg) => write!(f, "Encoding error: {}", msg),
			Error::DecodingError(msg) => write!(f, "Decoding error: {}", msg),
			Error::InvalidScript(msg) => write!(f, "Invalid script: {}", msg),
		}
	}
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}
