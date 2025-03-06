//! Error adapter for neo-clients
//!
//! This module provides conversion between neo-clients errors and neo-common errors.

use crate::errors::ProviderError as ClientProviderError;
use neo_common::ProviderError as CommonProviderError;

/// Convert a client-specific error to a common ProviderError
pub fn to_common_error(error: ClientProviderError) -> CommonProviderError {
    match error {
        ClientProviderError::NnsError(s) => CommonProviderError::CustomError(format!("NNS error: {}", s)),
        ClientProviderError::NnsNotOwned(s) => CommonProviderError::CustomError(format!("NNS not owned: {}", s)),
        ClientProviderError::SerdeJson(e) => CommonProviderError::SerializationError(format!("JSON error: {}", e)),
        ClientProviderError::HexError(e) => CommonProviderError::SerializationError(format!("Hex error: {}", e)),
        ClientProviderError::HTTPError(e) => CommonProviderError::NetworkError(format!("HTTP error: {}", e)),
        ClientProviderError::JsonRpcError(e) => CommonProviderError::RpcError(format!("JSON-RPC error: {}", e)),
        ClientProviderError::CustomError(s) => CommonProviderError::CustomError(s),
        ClientProviderError::UnsupportedRPC => CommonProviderError::RpcError("Unsupported RPC".to_string()),
        ClientProviderError::UnsupportedNodeClient => CommonProviderError::RpcError("Unsupported node client".to_string()),
        ClientProviderError::SignerUnavailable => CommonProviderError::CustomError("Signer unavailable".to_string()),
        ClientProviderError::IllegalState(s) => CommonProviderError::IllegalState(s),
        ClientProviderError::InvalidAddress => CommonProviderError::InvalidAddress,
        ClientProviderError::CryptoError(e) => CommonProviderError::CryptoError(format!("{:?}", e)),
        ClientProviderError::TypeError(e) => CommonProviderError::CustomError(format!("Type error: {:?}", e)),
        ClientProviderError::InvalidPassword => CommonProviderError::CustomError("Invalid password".to_string()),
        ClientProviderError::ParseError(s) => CommonProviderError::SerializationError(format!("Parse error: {}", s)),
        ClientProviderError::LockError => CommonProviderError::LockError,
        ClientProviderError::ProtocolNotFound => CommonProviderError::ProtocolNotFound,
        ClientProviderError::NetworkNotFound => CommonProviderError::NetworkNotFound,
        ClientProviderError::Other(s) => CommonProviderError::Other(s),
    }
}

/// Convert a common ProviderError to a client-specific error
pub fn from_common_error(error: CommonProviderError) -> ClientProviderError {
    match error {
        CommonProviderError::CustomError(s) => ClientProviderError::CustomError(s),
        CommonProviderError::InvalidAddress => ClientProviderError::InvalidAddress,
        CommonProviderError::IllegalState(s) => ClientProviderError::IllegalState(s),
        CommonProviderError::RpcError(s) => ClientProviderError::CustomError(format!("RPC error: {}", s)),
        CommonProviderError::SerializationError(s) => ClientProviderError::ParseError(s),
        CommonProviderError::NetworkError(s) => ClientProviderError::CustomError(format!("Network error: {}", s)),
        CommonProviderError::CryptoError(s) => ClientProviderError::CustomError(format!("Crypto error: {}", s)),
        CommonProviderError::LockError => ClientProviderError::LockError,
        CommonProviderError::ProtocolNotFound => ClientProviderError::ProtocolNotFound,
        CommonProviderError::NetworkNotFound => ClientProviderError::NetworkNotFound,
        CommonProviderError::Other(s) => ClientProviderError::CustomError(format!("Other error: {}", s)),
    }
}

/// Implement From trait for converting between error types
impl From<ClientProviderError> for CommonProviderError {
    fn from(error: ClientProviderError) -> Self {
        to_common_error(error)
    }
}

/// Implement From trait for converting between error types
impl From<CommonProviderError> for ClientProviderError {
    fn from(error: CommonProviderError) -> Self {
        from_common_error(error)
    }
}
