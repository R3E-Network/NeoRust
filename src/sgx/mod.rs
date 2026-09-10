//! Experimental Intel SGX DCAP parsing and validation primitives.
//!
//! This module can parse a complete SGX ECDSA quote v3 and, with the `sgx`
//! feature, validate an ordered PCK X.509 path against the pinned Intel root
//! and caller-supplied signed CRLs. The certificate validator fails closed on
//! missing revocation collateral by default.
//!
//! This is not a complete DCAP quote-verification library. It does not validate
//! the quote/QE-report signature binding, QE Identity, TCB Info, or all Intel
//! collateral. Applications making an attestation trust decision must use a
//! complete Intel DCAP QVL integration and bind a caller-generated challenge to
//! REPORTDATA. Quote v3 itself contains neither a protocol timestamp nor a
//! protocol nonce.

// External crates used in this module

#[cfg(feature = "sgx")]
pub mod certificate_chain;
pub mod crl_checker;
pub mod ecdsa_verification;
pub mod nonce_tracker;
pub mod sgx_types;

use thiserror::Error;

/// Core error types for SGX quote verification failures
#[derive(Error, Debug)]
pub enum QuoteVerificationError {
	#[error("Failed to parse SGX quote: {0}")]
	ParseError(String),

	#[error("Certificate chain validation failed: {0}")]
	CertificateChainError(String),

	#[error("ECDSA signature verification failed: {0}")]
	SignatureVerificationError(String),

	#[error("CRL check failed: {0}")]
	CrlCheckError(String),

	#[error("Nonce replay attack detected")]
	ReplayAttackDetected,

	#[error("MRENCLAVE not in whitelist: {0}")]
	MrenclaveNotWhitelisted(String),

	#[error("Timestamp validation failed: {0}")]
	TimestampValidationError(String),

	#[error("Quote version unsupported (required: 3+): {0}")]
	UnsupportedQuoteVersion(u16),

	#[error("Network error fetching CRL: {0}")]
	NetworkError(String),

	#[error("Memory safety violation during verification")]
	MemorySafetyViolation,
}

// Implement From<String> for each variant with string parameter
impl From<String> for QuoteVerificationError {
	fn from(s: String) -> Self {
		// Default to ParseError, callers should use specific constructors
		QuoteVerificationError::ParseError(s)
	}
}

// Additional From implementations for better error handling
impl From<&str> for QuoteVerificationError {
	fn from(s: &str) -> Self {
		QuoteVerificationError::ParseError(s.to_string())
	}
}

/// Result type alias for SGX verification operations
pub type VerificationResult<T> = Result<T, QuoteVerificationError>;

/// Legacy compatibility configuration that is not consumed by an SDK verifier.
///
/// These fields are not enforced security controls. Quote v3 has no protocol
/// timestamp or nonce, and complete policy evaluation belongs in a DCAP QVL
/// integration.
#[derive(Debug, Clone)]
pub struct QuoteVerifierConfig {
	/// Application-supplied identities; not checked by this crate.
	pub mrenclave_whitelist: Vec<[u8; 32]>,

	/// Compatibility value; the parser currently accepts exact quote version 3.
	pub min_quote_version: u16,

	/// Compatibility value; quote v3 contains no protocol timestamp.
	pub max_timestamp_age_seconds: u64,

	/// Compatibility value; configure [`certificate_chain::CertChainValidator`] directly.
	pub strict_crl_checking: bool,
}

impl Default for QuoteVerifierConfig {
	fn default() -> Self {
		Self {
			mrenclave_whitelist: Vec::new(),
			min_quote_version: 3,
			max_timestamp_age_seconds: 2_592_000, // 30 days
			strict_crl_checking: true,
		}
	}
}
