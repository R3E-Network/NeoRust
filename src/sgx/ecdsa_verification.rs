//! SGX ECDSA Signature Verification Module
//!
//! This module provides the low-level P-256 operation needed to verify an Intel
//! DCAP quote signature. It is not, by itself, a complete quote verifier; see
//! the [`crate::sgx`] module-level security boundary.

use crate::sgx::sgx_types::QuoteSignature;
use crate::sgx::QuoteVerificationError;
use crate::sgx::VerificationResult;

#[cfg(feature = "sgx")]
use crate::sgx::{
	certificate_chain::{CertChainValidator, MinimumTcbLevel},
	crl_checker::CrLChecker,
};

/// Low-level helper for separate quote-signature and PCK-path operations.
///
/// This type does not bind the quote attestation key to the QE report or PCK
/// path and therefore cannot produce an end-to-end attestation verdict.
pub struct EcdsaQuoteVerifier {
	/// ECDSA attestation public key used to verify the quote signature.
	quote_signing_key: Option<ring::signature::UnparsedPublicKey<Vec<u8>>>,

	#[cfg(feature = "sgx")]
	certificate_chain_validator: CertChainValidator,
}

impl EcdsaQuoteVerifier {
	/// Create a new low-level verification helper.
	pub fn new() -> Result<Self, &'static str> {
		Ok(Self {
			quote_signing_key: None,
			#[cfg(feature = "sgx")]
			certificate_chain_validator: CertChainValidator::new(),
		})
	}

	/// Add a caller-configured certificate trust anchor.
	///
	/// This does not configure the ephemeral quote attestation key. Call
	/// [`Self::set_quote_signing_key`] separately, and do not treat two successful
	/// low-level checks as proof of the missing QE-report/key binding.
	pub fn set_root_ca_key(&mut self, key_bytes: &[u8]) -> VerificationResult<()> {
		#[cfg(feature = "sgx")]
		{
			p256::PublicKey::from_sec1_bytes(key_bytes).map_err(|_| {
				QuoteVerificationError::CertificateChainError(
					"Invalid P-256 certificate root key".into(),
				)
			})?;
			self.certificate_chain_validator
				.add_trusted_root("caller-configured root", key_bytes);
			Ok(())
		}

		#[cfg(not(feature = "sgx"))]
		{
			let _ = key_bytes;
			Err(QuoteVerificationError::CertificateChainError(
				"certificate root configuration requires the `sgx` feature".into(),
			))
		}
	}

	/// Set the P-256 attestation key used for the fixed-width quote signature.
	///
	/// This accepts either Intel's 64-byte big-endian `X || Y` quote field or a
	/// 65-byte uncompressed SEC1 point.
	pub fn set_quote_signing_key(&mut self, key_bytes: &[u8]) -> VerificationResult<()> {
		let normalized_key = match key_bytes.len() {
			64 => {
				let mut sec1 = Vec::with_capacity(65);
				sec1.push(0x04);
				sec1.extend_from_slice(key_bytes);
				sec1
			},
			65 if key_bytes[0] == 0x04 => key_bytes.to_vec(),
			_ => {
				return Err(QuoteVerificationError::SignatureVerificationError(
					"Invalid P-256 attestation public key encoding".into(),
				));
			},
		};
		p256::PublicKey::from_sec1_bytes(&normalized_key).map_err(|_| {
			QuoteVerificationError::SignatureVerificationError(
				"Invalid P-256 attestation public key".into(),
			)
		})?;
		let public_key = ring::signature::UnparsedPublicKey::new(
			&ring::signature::ECDSA_P256_SHA256_FIXED,
			normalized_key,
		);
		self.quote_signing_key = Some(public_key);
		Ok(())
	}

	/// Install signed CRLs used by certificate-chain validation.
	#[cfg(feature = "sgx")]
	pub fn set_crl_checker(&mut self, checker: CrLChecker) {
		self.certificate_chain_validator.set_crl_checker(checker);
	}

	/// Enable or explicitly opt out of signed CRL coverage.
	#[cfg(feature = "sgx")]
	pub fn set_strict_crl_checking(&mut self, strict: bool) {
		self.certificate_chain_validator.set_strict_crl_checking(strict);
	}

	/// Access advanced certificate-path configuration.
	#[cfg(feature = "sgx")]
	pub fn certificate_chain_validator_mut(&mut self) -> &mut CertChainValidator {
		&mut self.certificate_chain_validator
	}

	/// Verify the low-level ECDSA signature bytes from an SGX quote.
	///
	/// `quote_data` must be the exact quote bytes covered by the DCAP signature
	/// (the 48-byte header followed by the report body). The signature is the
	/// fixed-width, big-endian `r || s` encoding used by Intel quote v3.
	pub fn verify_signature(
		&self,
		signature: &QuoteSignature,
		quote_data: &[u8],
	) -> VerificationResult<()> {
		// Check that we have a valid public key
		let public_key = self.get_trusted_signing_key()?;

		// Verify the signature using constant-time operation
		// Note: Ring returns ring::error::VerificationError on failure, we map to our error type
		public_key.verify(quote_data, &signature.epid_or_ecdsa_sig).map_err(|_| {
			QuoteVerificationError::SignatureVerificationError("Invalid signature".into())
		})?;

		Ok(())
	}

	/// Validate a PCK certificate path against the configured trust anchors.
	///
	/// Current signed CRLs are required by default. Successful path validation is
	/// only one part of DCAP quote verification; see the module-level boundary.
	pub fn verify_certificate_chain(&self, cert_chain: &[u8]) -> VerificationResult<()> {
		#[cfg(feature = "sgx")]
		{
			self.certificate_chain_validator
				.validate(cert_chain, &MinimumTcbLevel::new(0, 0, 0, 0))
				.map(|_| ())
				.map_err(|error| QuoteVerificationError::CertificateChainError(error.to_string()))
		}

		#[cfg(not(feature = "sgx"))]
		{
			let _ = cert_chain;
			Err(QuoteVerificationError::CertificateChainError(
				"certificate validation requires the `sgx` feature".to_string(),
			))
		}
	}

	/// Get the trusted signing public key
	fn get_trusted_signing_key(
		&self,
	) -> VerificationResult<&ring::signature::UnparsedPublicKey<Vec<u8>>> {
		self.quote_signing_key.as_ref().ok_or_else(|| {
			QuoteVerificationError::ParseError(
				"No quote attestation signing key configured".to_string(),
			)
		})
	}
}

impl Default for EcdsaQuoteVerifier {
	fn default() -> Self {
		Self::new().expect("Failed to create ECDSA verifier")
	}
}

/// ECDSA signature verification error type
#[derive(Debug)]
pub struct EcdsaVerificationError {
	message: String,
}

impl From<ring::error::Unspecified> for EcdsaVerificationError {
	fn from(_error: ring::error::Unspecified) -> Self {
		Self { message: "Signature verification failed".to_string() }
	}
}

impl std::fmt::Display for EcdsaVerificationError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.message)
	}
}

impl std::error::Error for EcdsaVerificationError {}

#[cfg(test)]
mod tests {
	use super::*;
	use ring::{
		rand::SystemRandom,
		signature::{EcdsaKeyPair, KeyPair, ECDSA_P256_SHA256_FIXED_SIGNING},
	};

	#[test]
	fn test_verifier_creation() {
		let verifier = EcdsaQuoteVerifier::new();
		assert!(verifier.is_ok());
	}

	#[test]
	fn signature_without_configured_attestation_key_is_rejected() {
		let verifier = EcdsaQuoteVerifier::new().unwrap();
		let bad_sig = QuoteSignature { epid_or_ecdsa_sig: [0u8; 64], certificate_chain: vec![] };
		let result = verifier.verify_signature(&bad_sig, b"test data");
		assert!(result.is_err());
	}

	#[cfg(feature = "sgx")]
	#[test]
	fn certificate_root_is_not_reused_as_quote_signing_key() {
		let rng = SystemRandom::new();
		let document = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &rng)
			.expect("generate test key");
		let key_pair =
			EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, document.as_ref(), &rng)
				.expect("parse test key");
		let mut verifier = EcdsaQuoteVerifier::new().expect("verifier");
		verifier
			.set_root_ca_key(key_pair.public_key().as_ref())
			.expect("valid root key");
		let signature =
			QuoteSignature { epid_or_ecdsa_sig: [0u8; 64], certificate_chain: Vec::new() };

		assert!(verifier.verify_signature(&signature, b"quote bytes").is_err());
	}

	#[test]
	fn fixed_width_quote_signature_is_verified_and_tampering_is_rejected() {
		let rng = SystemRandom::new();
		let document = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, &rng)
			.expect("generate test key");
		let key_pair =
			EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, document.as_ref(), &rng)
				.expect("parse test key");
		let signed_data = b"quote header and report body";
		let encoded_signature = key_pair.sign(&rng, signed_data).expect("sign test quote");
		let mut raw_signature = [0u8; 64];
		raw_signature.copy_from_slice(encoded_signature.as_ref());
		let quote_signature =
			QuoteSignature { epid_or_ecdsa_sig: raw_signature, certificate_chain: Vec::new() };
		let mut verifier = EcdsaQuoteVerifier::new().expect("verifier");
		let sec1_public_key = key_pair.public_key().as_ref();
		verifier.set_quote_signing_key(&sec1_public_key[1..]).expect("valid P-256 key");

		assert!(verifier.verify_signature(&quote_signature, signed_data).is_ok());
		assert!(verifier.verify_signature(&quote_signature, b"tampered data").is_err());
	}

	#[test]
	fn test_arbitrary_nonempty_certificate_data_is_rejected() {
		let verifier = EcdsaQuoteVerifier::new().unwrap();
		let result = verifier.verify_certificate_chain(b"arbitrary non-empty bytes");
		assert!(matches!(result, Err(QuoteVerificationError::CertificateChainError(_))));
	}
}
