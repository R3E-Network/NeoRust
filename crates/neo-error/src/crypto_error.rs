use thiserror::Error;

/// Cryptography-related errors
#[derive(Debug, Error, PartialEq, Clone)]
pub enum CryptoError {
	/// Invalid passphrase error
	#[error("Invalid passphrase: {0}")]
	InvalidPassphrase(String),
	
	/// Invalid format error
	#[error("Invalid format: {0}")]
	InvalidFormat(String),
	
	/// Header out of range error
	#[error("invalid signature length, got {0}, expected 65")]
	HeaderOutOfRange(u8),
	
	/// Could not recover public key from signature
	#[error("Could not recover public key from signature")]
	RecoverFailed,
	
	/// Invalid public key error
	#[error("Invalid public key")]
	InvalidPublicKey,
	
	/// Invalid private key error
	#[error("Invalid private key")]
	InvalidPrivateKey,
	
	/// P256 elliptic curve error
	#[error("P256 error: {0}")]
	P256Error(String),
	
	/// Signing error
	#[error("Signing error")]
	SigningError,
	
	/// Signature verification error
	#[error("Signature verification error")]
	SignatureVerificationError,
	
	/// Hex conversion error
	#[error("Hex error: {0}")]
	FromHexError(String),
	
	/// Decryption error
	#[error("Decryption error: {0}")]
	DecryptionError(String),
	
	/// Key error
	#[error("Key error: {0}")]
	KeyError(String),
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum Nep2Error {
	#[error("Invalid passphrase: {0}")]
	InvalidPassphrase(String),
	#[error("Invalid format: {0}")]
	InvalidFormat(String),
}

#[derive(Error, Debug, PartialEq, Eq, Hash, Clone)]
pub enum SignError {
	#[error("Header byte out of range: {0}")]
	HeaderOutOfRange(u8),
	#[error("Could not recover public key from signature")]
	RecoverFailed,
}
