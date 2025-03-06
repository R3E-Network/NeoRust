//! Error conversion utilities
//!
//! This module provides conversion implementations between different error types
//! to help break circular dependencies between crates.

use crate::ProviderError;

/// Trait for converting external provider errors to neo-common ProviderError
pub trait IntoProviderError {
    /// Convert the error to a ProviderError
    fn into_provider_error(self) -> ProviderError;
}

/// Trait for converting neo-common ProviderError to external provider errors
pub trait FromProviderError<T> {
    /// Convert a ProviderError to the external error type
    fn from_provider_error(error: ProviderError) -> T;
}

/// Implement conversion from serde_json::Error to neo-common ProviderError
impl From<serde_json::Error> for ProviderError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError(format!("JSON error: {}", err))
    }
}

/// Implement conversion from hex::FromHexError to neo-common ProviderError
impl From<hex::FromHexError> for ProviderError {
    fn from(err: hex::FromHexError) -> Self {
        Self::SerializationError(format!("Hex error: {}", err))
    }
}
