//! Error adapter for converting between different error types.
//!
//! This module provides utilities for converting between different error types
//! in the NeoRust SDK.

use crate::provider_error::ProviderError;
use std::fmt::Display;

/// A trait for adapting errors from one type to another.
pub trait ErrorAdapter<T, E> {
    /// Adapts an error of type `E` to type `T`.
    fn adapt_error(error: E) -> T;
}

/// Converts a provider error to a string.
///
/// This function is used to convert a provider error to a string
/// for display purposes.
///
/// # Arguments
///
/// * `error` - The provider error to convert.
///
/// # Returns
///
/// A string representation of the provider error.
pub fn provider_error_to_string(error: &ProviderError) -> String {
    match error {
        ProviderError::JsonError(s) => format!("JSON error: {}", s),
        ProviderError::HttpError(s) => format!("HTTP error: {}", s),
        ProviderError::WebSocketError(s) => format!("WebSocket error: {}", s),
        ProviderError::IpcError(s) => format!("IPC error: {}", s),
        ProviderError::Timeout => "Request timeout".to_string(),
        ProviderError::NotImplemented(s) => format!("Not implemented: {}", s),
        ProviderError::SerializationError(s) => format!("Serialization error: {}", s),
        ProviderError::Other(s) => format!("Other error: {}", s),
    }
}

/// Converts any error to a provider error.
///
/// This function is used to convert any error that implements `Display`
/// to a provider error.
///
/// # Arguments
///
/// * `error` - The error to convert.
///
/// # Returns
///
/// A provider error with the string representation of the input error.
pub fn to_provider_error<E: Display>(error: E) -> ProviderError {
    ProviderError::Other(error.to_string())
}
