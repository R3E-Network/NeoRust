use thiserror::Error;

#[derive(Debug, Error, PartialEq, Clone)]
pub enum CryptoError {
	#[error("Invalid passphrase: {0}")]
	InvalidPassphrase(String),
	#[error("Invalid format: {0}")]
	InvalidFormat(String),
	#[error("invalid signature length, got {0}, expected 65")]
	HeaderOutOfRange(u8),
	#[error("Could not recover public key from signature")]
	RecoverFailed,
	#[error("Invalid public key")]
	InvalidPublicKey,
	#[error("Invalid private key")]
	InvalidPrivateKey,
	#[error("Invalid private key")]
	P256Error(#[from] p256::elliptic_curve::Error),
	#[error("Signing error")]
	SigningError,
	#[error("Signature verification error")]
	SignatureVerificationError,
	#[error(transparent)]
	FromHexError(#[from] hex::FromHexError),
	#[error("Decryption error: {0}")]
	DecryptionError(String),
	#[error("Key error: {0}")]
	KeyError(String),
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum Nep2Error {
	#[error("Invalid passphrase: {0}")]
	InvalidPassphrase(String),
	#[error("Invalid format: {0}")]
	InvalidFormat(String),
	#[error("Invalid private key: {0}")]
	InvalidPrivateKey(String),
	#[error("Encryption error: {0}")]
	EncryptionError(String),
	#[error("Decryption error: {0}")]
	DecryptionError(String),
	#[error("Verification failed: {0}")]
	VerificationFailed(String),
	#[error("Scrypt error: {0}")]
	ScryptError(String),
	#[error("Base58 error: {0}")]
	Base58Error(String),
}

#[derive(Error, Debug, PartialEq, Eq, Hash, Clone)]
pub enum SignError {
	#[error("Header byte out of range: {0}")]
	HeaderOutOfRange(u8),
	#[error("Could not recover public key from signature")]
	RecoverFailed,
}
