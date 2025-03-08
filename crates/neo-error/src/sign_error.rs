use thiserror::Error;

/// Signing-related errors
#[derive(Debug, Error, Clone, PartialEq, Eq, Hash)]
pub enum SignError {
	/// Header byte out of range.
	#[error("Header byte out of range: {0}")]
	HeaderOutOfRange(u8),
	
	/// Could not recover public key from signature.
	#[error("Could not recover public key from signature")]
	RecoverFailed,
} 