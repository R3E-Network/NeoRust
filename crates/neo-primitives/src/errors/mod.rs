//! Error types for neo-primitives

use alloc::string::String;

/// Result type for neo-primitives operations
pub type Result<T> = core::result::Result<T, Error>;

/// Error type for neo-primitives
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum Error {
    /// Invalid address format
    #[cfg_attr(feature = "std", error("Invalid address: {0}"))]
    InvalidAddress(String),

    /// Invalid hash format
    #[cfg_attr(feature = "std", error("Invalid hash: {0}"))]
    InvalidHash(String),

    /// Invalid key format
    #[cfg_attr(feature = "std", error("Invalid key: {0}"))]
    InvalidKey(String),

    /// Invalid signature
    #[cfg_attr(feature = "std", error("Invalid signature: {0}"))]
    InvalidSignature(String),

    /// Encoding error
    #[cfg_attr(feature = "std", error("Encoding error: {0}"))]
    EncodingError(String),

    /// Decoding error
    #[cfg_attr(feature = "std", error("Decoding error: {0}"))]
    DecodingError(String),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Error::InvalidAddress(msg) => write!(f, "Invalid address: {}", msg),
            Error::InvalidHash(msg) => write!(f, "Invalid hash: ", msg),
            Error::InvalidKey(msg) => write!(f, "Invalid key: {}", msg),
            Error::InvalidSignature(msg) => write!(f, "Invalid signature: {}", msg),
            Error::EncodingError(msg) => write!(f, "Encoding error: {}", msg),
            Error::DecodingError(msg) => write!(f, "Decoding error: {}", msg),
        }
    }
}
