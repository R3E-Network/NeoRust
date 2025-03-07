//! Implementation of error conversion for neo-clients
//!
//! This module provides error conversion utilities for the neo-clients crate.

use crate::errors::ProviderError as ClientProviderError;
use neo_common::{ProviderError as CommonProviderError, to_provider_error, to_serialization_error, to_network_error, to_rpc_error};

/// Convert a client-specific error to a common ProviderError
pub fn client_to_common_error(error: &ClientProviderError) -> CommonProviderError {
    match error {
        ClientProviderError::NnsError(s) => to_provider_error(&format!("NNS error: {}", s)),
        ClientProviderError::NnsNotOwned(s) => to_provider_error(&format!("NNS not owned: {}", s)),
        ClientProviderError::SerdeJson(e) => to_serialization_error(&format!("JSON error: {}", e)),
        ClientProviderError::HexError(e) => to_serialization_error(&format!("Hex error: {}", e)),
        ClientProviderError::HTTPError(e) => to_network_error(&format!("HTTP error: {}", e)),
        ClientProviderError::JsonRpcError(e) => to_rpc_error(&format!("JSON-RPC error: {}", e)),
        ClientProviderError::CustomError(s) => to_provider_error(s),
        ClientProviderError::UnsupportedRPC => to_rpc_error("Unsupported RPC"),
        ClientProviderError::UnsupportedNodeClient => to_rpc_error("Unsupported node client"),
        ClientProviderError::SignerUnavailable => to_provider_error("Signer unavailable"),
        ClientProviderError::IllegalState(s) => CommonProviderError::IllegalState(s.clone()),
        ClientProviderError::InvalidAddress => CommonProviderError::InvalidAddress,
        ClientProviderError::CryptoError(e) => CommonProviderError::CryptoError(format!("{:?}", e)),
        ClientProviderError::TypeError(e) => to_provider_error(&format!("Type error: {:?}", e)),
        ClientProviderError::InvalidPassword => to_provider_error("Invalid password"),
        ClientProviderError::ParseError(s) => to_serialization_error(&format!("Parse error: {}", s)),
        ClientProviderError::LockError => to_provider_error("Lock error"),
        ClientProviderError::ProtocolNotFound => to_provider_error("Protocol not found"),
        ClientProviderError::NetworkNotFound => to_provider_error("Network not found"),
    }
}

/// Convert a common ProviderError to a client-specific error
pub fn common_to_client_error(error: &CommonProviderError) -> ClientProviderError {
    match error {
        CommonProviderError::CustomError(s) => ClientProviderError::CustomError(s.clone()),
        CommonProviderError::InvalidAddress => ClientProviderError::InvalidAddress,
        CommonProviderError::IllegalState(s) => ClientProviderError::IllegalState(s.clone()),
        CommonProviderError::RpcError(s) => ClientProviderError::CustomError(format!("RPC error: {}", s)),
        CommonProviderError::SerializationError(s) => ClientProviderError::ParseError(s.clone()),
        CommonProviderError::NetworkError(s) => ClientProviderError::CustomError(format!("Network error: {}", s)),
        CommonProviderError::CryptoError(s) => ClientProviderError::CustomError(format!("Crypto error: {}", s)),
        CommonProviderError::Other(s) => ClientProviderError::CustomError(format!("Other error: {}", s)),
    }
}
