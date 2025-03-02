use std::{error::Error, fmt::Debug, sync::Arc};

// Direct imports instead of using prelude
#[cfg(feature = "crypto-standard")]
use crate::neo_crypto::error::CryptoError;

#[derive(Debug)]
/// An error thrown when making a call to the provider
pub enum ProviderError {
	/// An error during NNS name resolution
	NnsError(String),
	/// Invalid reverse NNS name
	NnsNotOwned(String),
	/// Error in underlying lib `serde_json`
	SerdeJson(serde_json::Error),
	/// Error in underlying lib `hex`
	HexError(hex::FromHexError),
	/// Error in underlying lib `reqwest`
	HTTPError(Arc<reqwest::Error>),
	/// JSON-RPC error
	JsonRpcError(String),
	/// Custom error from unknown source
	CustomError(String),
	/// RPC method is not supported by this provider
	UnsupportedRPC,
	/// Node is not supported by this provider
	UnsupportedNodeClient,
	/// Signer is not available to this provider.
	SignerUnavailable,
	IllegalState(String),
	InvalidAddress,
	/// Type error
	TypeError(String),
	InvalidPassword,
	/// Error parsing data
	ParseError(String),
	/// Error locking a mutex
	LockError,
	/// Protocol not found
	ProtocolNotFound,
	/// Network not found
	NetworkNotFound,
	/// Connection error
	ConnectionError(String),
	/// RPC error
	RpcError(String),
	/// Deserialization error
	DeserializationError(String),
	/// Invalid response
	InvalidResponse(String),
}

// Implement Display manually
impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderError::NnsError(msg) => write!(f, "nns name not found: {}", msg),
            ProviderError::NnsNotOwned(msg) => write!(f, "reverse nns name not pointing to itself: {}", msg),
            ProviderError::SerdeJson(err) => write!(f, "{}", err),
            ProviderError::HexError(err) => write!(f, "{}", err),
            ProviderError::HTTPError(err) => write!(f, "{}", err),
            ProviderError::JsonRpcError(msg) => write!(f, "JSON-RPC error: {}", msg),
            ProviderError::CustomError(msg) => write!(f, "custom error: {}", msg),
            ProviderError::UnsupportedRPC => write!(f, "unsupported RPC"),
            ProviderError::UnsupportedNodeClient => write!(f, "unsupported node client"),
            ProviderError::SignerUnavailable => write!(f, "Attempted to sign a transaction with no available signer. Hint: did you mean to use a SignerMiddleware?"),
            ProviderError::IllegalState(msg) => write!(f, "Illegal state: {}", msg),
            ProviderError::InvalidAddress => write!(f, "Invalid address"),
            ProviderError::TypeError(msg) => write!(f, "Type error: {}", msg),
            ProviderError::InvalidPassword => write!(f, "Invalid password"),
            ProviderError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            ProviderError::LockError => write!(f, "Lock error"),
            ProviderError::ProtocolNotFound => write!(f, "Protocol not found"),
            ProviderError::NetworkNotFound => write!(f, "Network not found"),
            ProviderError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            ProviderError::RpcError(msg) => write!(f, "RPC error: {}", msg),
            ProviderError::DeserializationError(msg) => write!(f, "Deserialization error: {}", msg),
            ProviderError::InvalidResponse(msg) => write!(f, "Invalid response: {}", msg),
        }
    }
}

// Implement Error trait
impl Error for ProviderError {}

// Implement From for serde_json::Error
impl From<serde_json::Error> for ProviderError {
    fn from(err: serde_json::Error) -> Self {
        ProviderError::SerdeJson(err)
    }
}

// Implement From for hex::FromHexError
impl From<hex::FromHexError> for ProviderError {
    fn from(err: hex::FromHexError) -> Self {
        ProviderError::HexError(err)
    }
}

// Implement From for Arc<reqwest::Error>
impl From<Arc<reqwest::Error>> for ProviderError {
    fn from(err: Arc<reqwest::Error>) -> Self {
        ProviderError::HTTPError(err)
    }
}

// Implement From for CryptoError

impl PartialEq for ProviderError {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(ProviderError::SerdeJson(a), ProviderError::SerdeJson(b)) =>
				a.to_string() == b.to_string(),
			(ProviderError::HTTPError(a), ProviderError::HTTPError(b)) => a.status() == b.status(),
			(ProviderError::CustomError(a), ProviderError::CustomError(b)) => a == b,
			(ProviderError::UnsupportedRPC, ProviderError::UnsupportedRPC) => true,
			(ProviderError::UnsupportedNodeClient, ProviderError::UnsupportedNodeClient) => true,
			(ProviderError::SignerUnavailable, ProviderError::SignerUnavailable) => true,
			(ProviderError::IllegalState(a), ProviderError::IllegalState(b)) => a == b,
			(ProviderError::InvalidAddress, ProviderError::InvalidAddress) => true,
			(ProviderError::TypeError(a), ProviderError::TypeError(b)) => a == b,
			(ProviderError::InvalidPassword, ProviderError::InvalidPassword) => true,
			_ => false,
		}
	}
}

// Implementing Clone manually for `ProviderError`
impl Clone for ProviderError {
	fn clone(&self) -> Self {
		match self {
			ProviderError::NnsError(message) => ProviderError::NnsError(message.clone()),
			ProviderError::NnsNotOwned(message) => ProviderError::NnsNotOwned(message.clone()),
			ProviderError::SerdeJson(error) => ProviderError::SerdeJson(serde_json::Error::io(
				std::io::Error::new(std::io::ErrorKind::Other, error.to_string()),
			)),
			ProviderError::HexError(error) => ProviderError::HexError(error.clone()),
			ProviderError::HTTPError(error) => ProviderError::HTTPError(Arc::clone(error)),
			ProviderError::JsonRpcError(error) => ProviderError::JsonRpcError(error.clone()),
			ProviderError::CustomError(message) => ProviderError::CustomError(message.clone()),
			ProviderError::UnsupportedRPC => ProviderError::UnsupportedRPC,
			ProviderError::UnsupportedNodeClient => ProviderError::UnsupportedNodeClient,
			ProviderError::SignerUnavailable => ProviderError::SignerUnavailable,
			ProviderError::IllegalState(message) => ProviderError::IllegalState(message.clone()),
			ProviderError::InvalidAddress => ProviderError::InvalidAddress,
			ProviderError::TypeError(error) => ProviderError::TypeError(error.clone()),
			ProviderError::InvalidPassword => ProviderError::InvalidPassword,
			ProviderError::ParseError(message) => ProviderError::ParseError(message.clone()),
			ProviderError::LockError => ProviderError::LockError,
			ProviderError::ProtocolNotFound => ProviderError::ProtocolNotFound,
			ProviderError::NetworkNotFound => ProviderError::NetworkNotFound,
			ProviderError::ConnectionError(message) => ProviderError::ConnectionError(message.clone()),
			ProviderError::RpcError(message) => ProviderError::RpcError(message.clone()),
			ProviderError::DeserializationError(message) => ProviderError::DeserializationError(message.clone()),
			ProviderError::InvalidResponse(message) => ProviderError::InvalidResponse(message.clone()),
		}
	}
}

// Fix error conversion between neo_error::ProviderError and neo_clients::errors::ProviderError
impl From<crate::neo_error::ProviderError> for ProviderError {
    fn from(err: crate::neo_error::ProviderError) -> Self {
        match err {
            crate::neo_error::ProviderError(msg) => ProviderError::CustomError(msg),
        }
    }
}

// Implement conversion from this ProviderError to neo_error::ProviderError to satisfy JsonRpcProvider
impl From<ProviderError> for crate::neo_error::ProviderError {
    fn from(err: ProviderError) -> Self {
        crate::neo_error::ProviderError(err.to_string())
    }
}

// Implement From for RetryClientError
#[cfg(feature = "http-client")]
impl From<crate::neo_clients::rpc::transports::retry::RetryClientError> for ProviderError {
    fn from(err: crate::neo_clients::rpc::transports::retry::RetryClientError) -> Self {
        ProviderError::CustomError(format!("RetryClient error: {}", err))
    }
}

// Implement From for RwClientError with proper trait bounds
#[cfg(feature = "http-client")]
impl<R, W> From<crate::neo_clients::rpc::transports::rw::RwClientError<R, W>> for ProviderError
where
    R: crate::neo_clients::rpc::connections::JsonRpcProvider,
    R::Error: std::fmt::Display + Send + Sync + 'static,
    W: crate::neo_clients::rpc::connections::JsonRpcProvider,
    W::Error: std::fmt::Display + Send + Sync + 'static,
{
    fn from(err: crate::neo_clients::rpc::transports::rw::RwClientError<R, W>) -> Self {
        ProviderError::CustomError(format!("RwClient error: {}", err))
    }
}
