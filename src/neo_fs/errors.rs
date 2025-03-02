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
		}
	}
}

impl Error for NeoFsError {}

/// Result type for NeoFS operations
pub type NeoFsResult<T> = Result<T, NeoFsError>;
