//! Error adapter utilities
//!
//! This module provides utilities for adapting between different error types
//! to help break circular dependencies between crates.

use crate::ProviderError;

/// Convert a string error message to a common ProviderError
pub fn to_provider_error(message: &str) -> ProviderError {
    ProviderError::CustomError(message.to_string())
}

/// Convert a serialization error message to a common ProviderError
pub fn to_serialization_error(message: &str) -> ProviderError {
    ProviderError::SerializationError(message.to_string())
}

/// Convert a network error message to a common ProviderError
pub fn to_network_error(message: &str) -> ProviderError {
    ProviderError::NetworkError(message.to_string())
}

/// Convert an RPC error message to a common ProviderError
pub fn to_rpc_error(message: &str) -> ProviderError {
    ProviderError::RpcError(message.to_string())
}

/// Convert a crypto error message to a common ProviderError
pub fn to_crypto_error(message: &str) -> ProviderError {
    ProviderError::CryptoError(message.to_string())
}

/// Convert an illegal state error message to a common ProviderError
pub fn to_illegal_state_error(message: &str) -> ProviderError {
    ProviderError::IllegalState(message.to_string())
}

/// Convert a common ProviderError to a string representation
pub fn from_provider_error(error: &ProviderError) -> String {
    match error {
        ProviderError::CustomError(s) => s.clone(),
        ProviderError::InvalidAddress => "Invalid address".to_string(),
        ProviderError::IllegalState(s) => format!("Illegal state: {}", s),
        ProviderError::RpcError(s) => format!("RPC error: {}", s),
        ProviderError::SerializationError(s) => format!("Serialization error: {}", s),
        ProviderError::NetworkError(s) => format!("Network error: {}", s),
        ProviderError::CryptoError(s) => format!("Crypto error: {}", s),
        ProviderError::Other(s) => format!("Other error: {}", s),
    }
}
