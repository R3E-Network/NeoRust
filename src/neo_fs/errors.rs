use crate::neo_fs::types::ResponseStatus;
use std::{error::Error, fmt};

/// Error types specific to NeoFS operations
#[derive(Debug)]
pub enum NeoFsError {
	/// Error during HTTP request
	HttpError(String),
	/// Invalid response from NeoFS API
	InvalidResponse(String),
	/// Authentication error
	AuthError(String),
	/// Container operation error
	ContainerError(String),
	/// Object operation error
	ObjectError(String),
	/// Network error
	NetworkError(String),
	/// Permission denied
	PermissionDenied(String),
	/// Resource not found
	NotFound(String),
	/// API response error
	ApiError(ResponseStatus, String),
	/// Serialization or deserialization error
	SerializationError(String),
	/// Required feature is disabled
	FeatureDisabled(String),
	/// Response error with details
	ResponseError(String),
	/// Request failed with error details
	RequestFailed(String),
	/// Deserialization error with details
	DeserializationError(String),
}

impl fmt::Display for NeoFsError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::HttpError(msg) => write!(f, "HTTP error: {}", msg),
			Self::InvalidResponse(msg) => write!(f, "Invalid response: {}", msg),
			Self::AuthError(msg) => write!(f, "Authentication error: {}", msg),
			Self::ContainerError(msg) => write!(f, "Container error: {}", msg),
			Self::ObjectError(msg) => write!(f, "Object error: {}", msg),
			Self::NetworkError(msg) => write!(f, "Network error: {}", msg),
			Self::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
			Self::NotFound(msg) => write!(f, "Not found: {}", msg),
			Self::ApiError(status, msg) => write!(f, "API error (status {:?}): {}", status, msg),
			Self::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
			Self::FeatureDisabled(msg) => write!(f, "Feature disabled: {}", msg),
			Self::ResponseError(msg) => write!(f, "Response error: {}", msg),
			Self::RequestFailed(msg) => write!(f, "Request failed: {}", msg),
			Self::DeserializationError(msg) => write!(f, "Deserialization error: {}", msg),
		}
	}
}

impl Error for NeoFsError {}

// Add From implementations for common error types
impl From<serde_json::Error> for NeoFsError {
	fn from(err: serde_json::Error) -> Self {
		NeoFsError::SerializationError(err.to_string())
	}
}

impl From<hex::FromHexError> for NeoFsError {
	fn from(err: hex::FromHexError) -> Self {
		NeoFsError::DeserializationError(err.to_string())
	}
}

impl From<std::io::Error> for NeoFsError {
	fn from(err: std::io::Error) -> Self {
		NeoFsError::HttpError(err.to_string())
	}
}

/// Result type for NeoFS operations
pub type NeoFsResult<T> = Result<T, NeoFsError>;
