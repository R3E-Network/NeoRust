//! Provider error types for the NeoRust SDK.
//!
//! This module provides error types for RPC providers.

use thiserror::Error;

/// Errors that can occur when using a provider.
#[derive(Error, Debug)]
pub enum ProviderError {
    /// Method not implemented
    #[error("Method not implemented: {0}")]
    NotImplemented(String),
    
    /// JSON serialization error
    #[error("JSON error: {0}")]
    JsonError(String),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    /// HTTP error
    #[error("HTTP error: {0}")]
    HttpError(String),
    
    /// WebSocket error
    #[error("WebSocket error: {0}")]
    WebSocketError(String),
    
    /// IPC error
    #[error("IPC error: {0}")]
    IpcError(String),
    
    /// Request timeout
    #[error("Request timeout")]
    Timeout,
    
    /// Other error
    #[error("Provider error: {0}")]
    Other(String),
}
