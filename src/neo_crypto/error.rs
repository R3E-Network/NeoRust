//! Error types for the Neo crypto module
//!
//! This module contains the error types used in the Neo crypto module.

use std::fmt;

#[cfg(feature = "thiserror")]
use thiserror::Error;

/// Crypto Error
#[cfg(feature = "thiserror")]
#[derive(Debug, Error)]
pub enum CryptoError {
    /// Provider error
    #[error("Provider error: {0}")]
    ProviderError(String),

    /// Transaction error
    #[error("Transaction error: {0}")]
    TransactionError(String),

    /// Invalid public key
    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),

    /// Invalid private key
    #[error("Invalid private key: {0}")]
    InvalidPrivateKey(String),

    /// Invalid format
    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    /// Signature verification error
    #[error("Signature verification error: {0}")]
    SignatureVerificationError(String),

    /// Signing error
    #[error("Signing error: {0}")]
    SigningError(String),

    /// Other error
    #[error("Other error: {0}")]
    OtherError(String),
}

/// Crypto Error (without thiserror)
#[cfg(not(feature = "thiserror"))]
#[derive(Debug)]
pub enum CryptoError {
    /// Provider error
    ProviderError(String),

    /// Transaction error
    TransactionError(String),

    /// Invalid public key
    InvalidPublicKey(String),

    /// Invalid private key
    InvalidPrivateKey(String),

    /// Invalid format
    InvalidFormat(String),

    /// Signature verification error
    SignatureVerificationError(String),

    /// Signing error
    SigningError(String),

    /// Other error
    OtherError(String),
}

impl CryptoError {
    pub fn invalid_public_key() -> Self {
        Self::InvalidPublicKey("Invalid public key".to_string())
    }

    pub fn invalid_private_key() -> Self {
        Self::InvalidPrivateKey("Invalid private key".to_string())
    }

    pub fn signature_verification_error() -> Self {
        Self::SignatureVerificationError("Signature verification failed".to_string())
    }

    pub fn signing_error() -> Self {
        Self::SigningError("Signing failed".to_string())
    }
}

#[cfg(not(feature = "thiserror"))]
impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CryptoError::ProviderError(s) => write!(f, "Provider error: {}", s),
            CryptoError::TransactionError(s) => write!(f, "Transaction error: {}", s),
            CryptoError::InvalidPublicKey(s) => write!(f, "Invalid public key: {}", s),
            CryptoError::InvalidPrivateKey(s) => write!(f, "Invalid private key: {}", s),
            CryptoError::InvalidFormat(s) => write!(f, "Invalid format: {}", s),
            CryptoError::SignatureVerificationError(s) => write!(f, "Signature verification error: {}", s),
            CryptoError::SigningError(s) => write!(f, "Signing error: {}", s),
            CryptoError::OtherError(s) => write!(f, "Other error: {}", s),
        }
    }
}

#[cfg(not(feature = "thiserror"))]
impl std::error::Error for CryptoError {}

// Implement From<signature::Error> for CryptoError
#[cfg(feature = "crypto-standard")]
impl From<signature::Error> for CryptoError {
    fn from(_: signature::Error) -> Self {
        Self::SignatureVerificationError("Signature verification failed".to_string())
    }
}
