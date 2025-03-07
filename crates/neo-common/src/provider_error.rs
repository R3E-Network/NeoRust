//! Provider error types for the NeoRust SDK.
//!
//! This module provides error types for RPC providers.

use thiserror::Error;

/// Errors that can occur when using a provider.
#[derive(Error, Debug, Clone, PartialEq)]
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
    
    /// Custom error with message
    #[error("Custom error: {0}")]
    CustomError(String),
    
    /// Illegal state error
    #[error("Illegal state: {0}")]
    IllegalState(String),
    
    /// Crypto error
    #[error("Crypto error: {0}")]
    CryptoError(String),
    
    /// Invalid address error
    #[error("Invalid address")]
    InvalidAddress,
    
    /// RPC error
    #[error("RPC error: {0}")]
    RpcError(String),
    
    /// Lock error
    #[error("Lock error")]
    LockError,
    
    /// Protocol not found
    #[error("Protocol not found")]
    ProtocolNotFound,
    
    /// Network not found
    #[error("Network not found")]
    NetworkNotFound,
    
    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),
    
    /// Other error
    #[error("Provider error: {0}")]
    Other(String),
}
