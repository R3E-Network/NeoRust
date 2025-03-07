//! Error conversion utilities for the NeoRust SDK.
//!
//! This module provides utilities for converting between different error types
//! in the NeoRust SDK.

use crate::provider_error::ProviderError;

/// A trait for converting errors to provider errors.
pub trait ProviderErrorConversion {
    /// Converts an error to a provider error.
    fn to_provider_error(self) -> ProviderError;
}

impl ProviderErrorConversion for serde_json::Error {
    fn to_provider_error(self) -> ProviderError {
        ProviderError::SerializationError(format!("JSON error: {}", self))
    }
}

impl ProviderErrorConversion for hex::FromHexError {
    fn to_provider_error(self) -> ProviderError {
        ProviderError::SerializationError(format!("Hex error: {}", self))
    }
}

impl ProviderErrorConversion for std::io::Error {
    fn to_provider_error(self) -> ProviderError {
        ProviderError::Other(format!("IO error: {}", self))
    }
}

impl<T> ProviderErrorConversion for std::sync::PoisonError<T> {
    fn to_provider_error(self) -> ProviderError {
        ProviderError::Other(format!("Poison error: {}", self))
    }
}
