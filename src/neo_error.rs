//! Neo Error - Error types for the Neo blockchain
//!
//! This module contains the error types used in the Neo blockchain.

use std::fmt;

#[cfg(feature = "thiserror")]
use thiserror::Error;

/// Neo Error
#[cfg(feature = "thiserror")]
#[derive(Debug, Error)]
pub enum NeoError {
    /// Client error
    #[error("Client error: {0}")]
    ClientError(String),

    /// Other error
    #[error("Other error: {0}")]
    OtherError(String),
}

/// Neo Error (without thiserror)
#[cfg(not(feature = "thiserror"))]
#[derive(Debug)]
pub enum NeoError {
    /// Client error
    ClientError(String),

    /// Other error
    OtherError(String),
}

#[cfg(not(feature = "thiserror"))]
impl fmt::Display for NeoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NeoError::ClientError(s) => write!(f, "Client error: {}", s),
            NeoError::OtherError(s) => write!(f, "Other error: {}", s),
        }
    }
}

#[cfg(not(feature = "thiserror"))]
impl std::error::Error for NeoError {}

/// Type Error
#[cfg(feature = "thiserror")]
#[derive(Debug, Error)]
pub enum TypeError {
    /// Invalid format
    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    /// Invalid length
    #[error("Invalid length: {0}")]
    InvalidLength(String),

    /// Invalid value
    #[error("Invalid value: {0}")]
    InvalidValue(String),

    /// Invalid encoding
    #[error("Invalid encoding: {0}")]
    InvalidEncoding(String),

    /// Invalid checksum
    #[error("Invalid checksum: {0}")]
    InvalidChecksum(String),

    /// Invalid signature
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    /// Invalid key
    #[error("Invalid key: {0}")]
    InvalidKey(String),

    /// Invalid address
    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    /// Invalid script hash
    #[error("Invalid script hash: {0}")]
    InvalidScriptHash(String),

    /// Invalid public key
    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),

    /// Invalid private key
    #[error("Invalid private key: {0}")]
    InvalidPrivateKey(String),

    /// Invalid signature
    #[error("Invalid signature: {0}")]
    InvalidSignatureFormat(String),

    /// Other error
    #[error("Other error: {0}")]
    OtherError(String),
    
    /// Unexpected return type
    #[error("Unexpected return type: {0}")]
    UnexpectedReturnType(String),
    
    /// Codec error
    #[error("Codec error: {0}")]
    CodecError(String),
}

/// Type Error (without thiserror)
#[cfg(not(feature = "thiserror"))]
#[derive(Debug)]
pub enum TypeError {
    /// Invalid format
    InvalidFormat(String),

    /// Invalid length
    InvalidLength(String),

    /// Invalid value
    InvalidValue(String),

    /// Invalid encoding
    InvalidEncoding(String),

    /// Invalid checksum
    InvalidChecksum(String),

    /// Invalid signature
    InvalidSignature(String),

    /// Invalid key
    InvalidKey(String),

    /// Invalid address
    InvalidAddress(String),

    /// Invalid script hash
    InvalidScriptHash(String),

    /// Invalid public key
    InvalidPublicKey(String),

    /// Invalid private key
    InvalidPrivateKey(String),

    /// Invalid signature
    InvalidSignatureFormat(String),

    /// Other error
    OtherError(String),
    
    /// Unexpected return type
    UnexpectedReturnType(String),
    
    /// Codec error
    CodecError(String),
}

#[cfg(not(feature = "thiserror"))]
impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeError::InvalidFormat(s) => write!(f, "Invalid format: {}", s),
            TypeError::InvalidLength(s) => write!(f, "Invalid length: {}", s),
            TypeError::InvalidValue(s) => write!(f, "Invalid value: {}", s),
            TypeError::InvalidEncoding(s) => write!(f, "Invalid encoding: {}", s),
            TypeError::InvalidChecksum(s) => write!(f, "Invalid checksum: {}", s),
            TypeError::InvalidSignature(s) => write!(f, "Invalid signature: {}", s),
            TypeError::InvalidKey(s) => write!(f, "Invalid key: {}", s),
            TypeError::InvalidAddress(s) => write!(f, "Invalid address: {}", s),
            TypeError::InvalidScriptHash(s) => write!(f, "Invalid script hash: {}", s),
            TypeError::InvalidPublicKey(s) => write!(f, "Invalid public key: {}", s),
            TypeError::InvalidPrivateKey(s) => write!(f, "Invalid private key: {}", s),
            TypeError::InvalidSignatureFormat(s) => write!(f, "Invalid signature format: {}", s),
            TypeError::OtherError(s) => write!(f, "Other error: {}", s),
            TypeError::UnexpectedReturnType(s) => write!(f, "Unexpected return type: {}", s),
            TypeError::CodecError(s) => write!(f, "Codec error: {}", s),
        }
    }
}

#[cfg(not(feature = "thiserror"))]
impl std::error::Error for TypeError {}

/// Codec Error
pub struct CodecError {
    pub message: String,
}

impl CodecError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl fmt::Display for CodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Codec error: {}", self.message)
    }
}

impl fmt::Debug for CodecError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Codec error: {}", self.message)
    }
}

impl std::error::Error for CodecError {}

#[cfg(feature = "crypto-standard")]
impl From<crate::neo_crypto::error::CryptoError> for NeoError {
    fn from(err: crate::neo_crypto::error::CryptoError) -> Self {
        match err {
            crate::neo_crypto::error::CryptoError::ProviderError(err) => NeoError::ClientError(err),
            crate::neo_crypto::error::CryptoError::TransactionError(err) => NeoError::OtherError(err),
            _ => NeoError::OtherError(format!("{:?}", err)),
        }
    }
}

#[cfg(feature = "crypto-standard")]
impl From<NeoError> for crate::neo_crypto::error::CryptoError {
    fn from(err: NeoError) -> Self {
        match err {
            NeoError::ClientError(err) => crate::neo_crypto::error::CryptoError::ProviderError(err),
            NeoError::OtherError(err) => crate::neo_crypto::error::CryptoError::TransactionError(err),
        }
    }
}
