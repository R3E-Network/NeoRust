//! Provider error types
//!
//! This module defines error types for providers.

use thiserror::Error;
use std::fmt;

/// Error type for provider operations
#[derive(Debug, Clone, Error)]
pub enum ProviderError {
    /// Custom error with message
    #[error("Custom error: {0}")]
    CustomError(String),
    
    /// Invalid address format
    #[error("Invalid address format")]
    InvalidAddress,
    
    /// Illegal state
    #[error("Illegal state: {0}")]
    IllegalState(String),
    
    /// RPC error
    #[error("RPC error: {0}")]
    RpcError(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),
    
    /// Crypto error
    #[error("Crypto error: {0}")]
    CryptoError(String),
    
    /// Lock error
    #[error("Lock error")]
    LockError,
    
    /// Protocol not found
    #[error("Protocol not found")]
    ProtocolNotFound,
    
    /// Network not found
    #[error("Network not found")]
    NetworkNotFound,
    
    /// Other error
    #[error("Other error: {0}")]
    Other(String),
}

impl From<String> for ProviderError {
    fn from(msg: String) -> Self {
        ProviderError::CustomError(msg)
    }
}

impl From<&str> for ProviderError {
    fn from(msg: &str) -> Self {
        ProviderError::CustomError(msg.to_string())
    }
}

// This will be implemented in neo-crypto for CryptoError
// but we need to provide a way for other crates to convert errors
// without creating circular dependencies
#[cfg(feature = "with-crypto")]
impl From<neo_crypto::CryptoError> for ProviderError {
    fn from(err: neo_crypto::CryptoError) -> Self {
        ProviderError::CryptoError(format!("{:?}", err))
    }
}
