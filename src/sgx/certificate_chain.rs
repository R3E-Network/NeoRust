//! X.509 path and revocation validation for an SGX PCK certificate chain.
//!
//! This module validates the bounded certificate-path responsibilities it can
//! perform locally: exact PEM/DER parsing, P-256/SHA-256 signatures, issuer names,
//! CA constraints and path lengths, validity periods, root-key pinning, and
//! caller-supplied signed CRLs. Intel TCB Info, QE Identity, and the remaining
//! DCAP collateral still require a complete Intel DCAP QVL integration.

#![cfg(feature = "sgx")]

use std::time::{SystemTime, UNIX_EPOCH};

use const_oid::ObjectIdentifier;
use der::{Decode, DecodePem, Encode, Reader, SliceReader};
use ring::signature::{UnparsedPublicKey, ECDSA_P256_SHA256_ASN1};
use x509_cert::{
	certificate::Certificate,
	crl::CertificateList,
	ext::pkix::{BasicConstraints, KeyUsage},
};

use super::crl_checker::CrLChecker;

const MAX_CHAIN_BYTES: usize = 1024 * 1024;
const MAX_CERTIFICATES: usize = 8;
const ECDSA_WITH_SHA256_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("1.2.840.10045.4.3.2");
const INTEL_SGX_PCK_EXTENSIONS_OID: ObjectIdentifier =
	ObjectIdentifier::new_unwrap("1.2.840.113741.1.13.1");
const DELTA_CRL_INDICATOR_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.5.29.27");
const ISSUING_DISTRIBUTION_POINT_OID: ObjectIdentifier = ObjectIdentifier::new_unwrap("2.5.29.28");
const PEM_CERT_BEGIN: &[u8] = b"-----BEGIN CERTIFICATE-----";
const PEM_CERT_END: &[u8] = b"-----END CERTIFICATE-----";

/// Intel SGX Provisioning Certification Root CA public key.
///
/// This is the complete SEC1 uncompressed NIST P-256 point published in Intel's
/// Provisioning Certification Root CA certificate (`0x04 || X || Y`).
pub const INTEL_SGX_PROVISIONING_ROOT_CA_PUBKEY: [u8; 65] = [
	0x04, 0x0b, 0xa9, 0xc4, 0xc0, 0xc0, 0xc8, 0x61, 0x93, 0xa3, 0xfe, 0x23, 0xd6, 0xb0, 0x2c, 0xda,
	0x10, 0xa8, 0xbb, 0xd4, 0xe8, 0x8e, 0x48, 0xb4, 0x45, 0x85, 0x61, 0xa3, 0x6e, 0x70, 0x55, 0x25,
	0xf5, 0x67, 0x91, 0x8e, 0x2e, 0xdc, 0x88, 0xe4, 0x0d, 0x86, 0x0b, 0xd0, 0xcc, 0x4e, 0xe2, 0x6a,
	0xac, 0xc9, 0x88, 0xe5, 0x05, 0xa9, 0x53, 0x55, 0x8c, 0x45, 0x3f, 0x6b, 0x09, 0x04, 0xae, 0x73,
	0x94,
];

/// Compatibility alias for the historical placeholder constant.
///
/// Despite the historical name, this now contains the genuine Intel root key.
#[deprecated(note = "use INTEL_SGX_PROVISIONING_ROOT_CA_PUBKEY; the placeholder was replaced")]
pub const INTEL_SGX_ROOT_CA_PUBKEY_PLACEHOLDER: [u8; 65] = INTEL_SGX_PROVISIONING_ROOT_CA_PUBKEY;

/// Certificate-chain validation failures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CertChainValidationError {
	ParseError(String),
	InvalidChainFormat(String),
	IssuerNameMismatch {
		expected: String,
		actual: String,
	},
	MissingBasicConstraints,
	InvalidKeyUsage,
	PathLengthViolation,
	UnsupportedSignatureAlgorithm,
	SignatureVerificationFailed,
	ExpiredBefore,
	NotYetValid,
	Revoked(u64),
	RevokedSerial(String),
	TcbLevelTooLow {
		current: String,
		required: String,
	},
	UntrustedRoot,
	MissingCrl(String),
	InvalidCrlSignature,
	ExpiredCrl,
	CrlNotYetValid,
	UnsupportedCrlExtension(String),
	/// Retained for source compatibility with previous callers.
	CrlCheckNotImplemented,
}

impl std::fmt::Display for CertChainValidationError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::ParseError(message) => write!(f, "Certificate parsing error: {message}"),
			Self::InvalidChainFormat(message) => {
				write!(f, "Invalid certificate chain format: {message}")
			},
			Self::IssuerNameMismatch { expected, actual } => {
				write!(f, "Issuer/Subject mismatch - Expected: '{expected}', Got: '{actual}'")
			},
			Self::MissingBasicConstraints => {
				write!(f, "CA certificate missing Basic Constraints")
			},
			Self::InvalidKeyUsage => write!(f, "Certificate key usage is invalid for its role"),
			Self::PathLengthViolation => write!(f, "Path length constraint violated"),
			Self::UnsupportedSignatureAlgorithm => {
				write!(f, "Certificate signature algorithm is not ECDSA P-256/SHA-256")
			},
			Self::SignatureVerificationFailed => {
				write!(f, "Certificate signature verification failed")
			},
			Self::ExpiredBefore => write!(f, "Certificate has expired"),
			Self::NotYetValid => write!(f, "Certificate is not yet valid"),
			Self::Revoked(serial) => write!(f, "Certificate with serial {serial} is revoked"),
			Self::RevokedSerial(serial) => {
				write!(f, "Certificate with serial {serial} is revoked")
			},
			Self::TcbLevelTooLow { current, required } => {
				write!(f, "TCB level {current} is below required {required}")
			},
			Self::UntrustedRoot => write!(f, "Root CA certificate not trusted"),
			Self::MissingCrl(issuer) => write!(f, "No valid CRL covers issuer '{issuer}'"),
			Self::InvalidCrlSignature => write!(f, "CRL signature verification failed"),
			Self::ExpiredCrl => write!(f, "CRL is stale or has no nextUpdate"),
			Self::CrlNotYetValid => write!(f, "CRL is not yet valid"),
			Self::UnsupportedCrlExtension(extension) => {
				write!(f, "unsupported CRL extension: {extension}")
			},
			Self::CrlCheckNotImplemented => {
				write!(f, "CRL revocation checking was requested but is not implemented")
			},
		}
	}
}

impl std::error::Error for CertChainValidationError {}

/// Legacy application-defined scalar thresholds supplied entirely by the caller.
///
/// This is not an Intel CPU SVN or a parsed TCB Info representation. A complete
/// DCAP QVL must evaluate the platform's signed TCB collateral.
#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTcbLevel {
	pub cpu_svn: u16,
	pub pce_svn: u16,
	pub qe_svn: u16,
	pub misc_select: u32,
}

impl MinimumTcbLevel {
	pub fn new(cpu_svn: u16, pce_svn: u16, qe_svn: u16, misc_select: u32) -> Self {
		Self { cpu_svn, pce_svn, qe_svn, misc_select }
	}

	pub fn satisfies(&self, other: &MinimumTcbLevel) -> bool {
		self.cpu_svn <= other.cpu_svn
			&& self.pce_svn <= other.pce_svn
			&& self.qe_svn <= other.qe_svn
			&& (self.misc_select == 0 || self.misc_select == other.misc_select)
	}
}

/// Successful certificate-path validation details.
#[derive(Debug, Clone)]
pub struct CertValidationResult {
	pub is_valid: bool,
	pub chain_length: usize,
	pub leaf_serial: Option<String>,
	pub issued_by: Option<String>,
	pub valid_until: Option<u64>,
}

#[derive(Clone, Debug)]
struct TrustedRootKey {
	name: String,
	key_bytes: Vec<u8>,
}

/// Validates an ordered certificate chain (`leaf -> ... -> root`).
pub struct CertChainValidator {
	trusted_roots: Vec<TrustedRootKey>,
	tcb_requirements: Option<MinimumTcbLevel>,
	crl_checker: Option<CrLChecker>,
	strict_crl_checking: bool,
}

impl CertChainValidator {
	/// Construct a validator pinned to Intel's SGX Provisioning root.
	///
	/// CRL validation is strict by default. Callers must install current CRLs or
	/// explicitly opt out with [`Self::set_strict_crl_checking`].
	pub fn new() -> Self {
		Self {
			trusted_roots: vec![TrustedRootKey {
				name: "Intel SGX Provisioning Certification Root CA".to_string(),
				key_bytes: INTEL_SGX_PROVISIONING_ROOT_CA_PUBKEY.to_vec(),
			}],
			tcb_requirements: None,
			crl_checker: None,
			strict_crl_checking: true,
		}
	}

	pub fn set_tcb_requirements(&mut self, tcb: MinimumTcbLevel) {
		self.tcb_requirements = Some(tcb);
	}

	/// Install signed CRLs and/or an additive local serial denylist.
	pub fn set_crl_checker<C: Into<CrLChecker>>(&mut self, crl: C) {
		self.crl_checker = Some(crl.into());
	}

	/// Enable or explicitly opt out of signed CRL coverage.
	pub fn set_strict_crl_checking(&mut self, strict: bool) {
		self.strict_crl_checking = strict;
	}

	/// Register another trusted root by exact SEC1 public-key bytes.
	pub fn add_trusted_root(&mut self, name: impl Into<String>, key_bytes: &[u8]) {
		self.trusted_roots
			.push(TrustedRootKey { name: name.into(), key_bytes: key_bytes.to_vec() });
	}

	/// Remove all configured trust anchors, including the built-in Intel root.
	pub fn clear_trusted_roots(&mut self) {
		self.trusted_roots.clear();
	}

	/// Validate certificates ordered leaf first, root last.
	///
	/// Intel quote certification data type 5 uses concatenated PEM certificates.
	/// Concatenated DER remains accepted for compatibility with existing callers.
	pub fn validate(
		&self,
		cert_chain_data: &[u8],
		enclave_tcb_info: &MinimumTcbLevel,
	) -> Result<CertValidationResult, CertChainValidationError> {
		let certs = self.parse_certificates(cert_chain_data)?;
		if certs.len() != 3 {
			return Err(CertChainValidationError::InvalidChainFormat(
				"an SGX PCK chain must contain leaf, PCK CA, and root certificates".into(),
			));
		}

		self.verify_issuer_chain(&certs)?;
		self.validate_pck_leaf_profile(&certs[0])?;
		self.validate_leaf_certificate(&certs[0])?;
		for (index, cert) in certs.iter().enumerate().skip(1) {
			self.validate_ca_certificate(cert, index, &certs)?;
		}
		for cert in &certs {
			self.check_validity_period(cert)?;
		}
		self.check_revocation(&certs)?;
		self.verify_tcb_level(enclave_tcb_info)?;

		let leaf = &certs[0];
		Ok(CertValidationResult {
			is_valid: true,
			chain_length: certs.len(),
			leaf_serial: Some(leaf.tbs_certificate.serial_number.to_string()),
			issued_by: Some(leaf.tbs_certificate.issuer.to_string()),
			valid_until: Some(leaf.tbs_certificate.validity.not_after.to_unix_duration().as_secs()),
		})
	}

	fn parse_certificates(
		&self,
		data: &[u8],
	) -> Result<Vec<Certificate>, CertChainValidationError> {
		if data.is_empty() {
			return Err(CertChainValidationError::InvalidChainFormat(
				"certificate data is empty".into(),
			));
		}
		if data.len() > MAX_CHAIN_BYTES {
			return Err(CertChainValidationError::InvalidChainFormat(
				"certificate data exceeds the 1 MiB limit".into(),
			));
		}

		let mut pem_candidate = trim_ascii_whitespace(data);
		if pem_candidate.starts_with(PEM_CERT_BEGIN) {
			// Intel QCNL emits type-5 data as the PEM chain followed by one C-string
			// terminator. Accept that exact wire representation without weakening the
			// parser's rejection of any other trailing data.
			if pem_candidate.last() == Some(&0) {
				pem_candidate = trim_ascii_whitespace(&pem_candidate[..pem_candidate.len() - 1]);
			}
			return self.parse_pem_certificates(pem_candidate);
		}

		self.parse_der_certificates(data)
	}

	fn parse_pem_certificates(
		&self,
		data: &[u8],
	) -> Result<Vec<Certificate>, CertChainValidationError> {
		let mut certs = Vec::new();
		let mut remaining = data;
		while !remaining.is_empty() {
			if certs.len() == MAX_CERTIFICATES {
				return Err(CertChainValidationError::InvalidChainFormat(format!(
					"certificate count exceeds {MAX_CERTIFICATES}"
				)));
			}
			if !remaining.starts_with(PEM_CERT_BEGIN) {
				return Err(CertChainValidationError::ParseError(
					"unexpected data between PEM certificates".into(),
				));
			}
			let boundary_offset = remaining
				.windows(PEM_CERT_END.len())
				.position(|window| window == PEM_CERT_END)
				.ok_or_else(|| {
					CertChainValidationError::ParseError(
						"PEM certificate has no end boundary".into(),
					)
				})?;
			let certificate_end =
				boundary_offset.checked_add(PEM_CERT_END.len()).ok_or_else(|| {
					CertChainValidationError::ParseError("PEM length overflow".into())
				})?;
			let encoded = &remaining[..certificate_end];
			let cert = Certificate::from_pem(encoded).map_err(|error| {
				CertChainValidationError::ParseError(format!(
					"failed to parse PEM certificate: {error}"
				))
			})?;
			certs.push(cert);
			remaining = trim_ascii_whitespace(&remaining[certificate_end..]);
		}
		Ok(certs)
	}

	fn parse_der_certificates(
		&self,
		data: &[u8],
	) -> Result<Vec<Certificate>, CertChainValidationError> {
		let mut reader = SliceReader::new(data)
			.map_err(|error| CertChainValidationError::ParseError(error.to_string()))?;
		let mut certs = Vec::new();
		while !reader.is_finished() {
			if certs.len() == MAX_CERTIFICATES {
				return Err(CertChainValidationError::InvalidChainFormat(format!(
					"certificate count exceeds {MAX_CERTIFICATES}"
				)));
			}
			let offset = usize::try_from(reader.position())
				.map_err(|_| CertChainValidationError::ParseError("DER offset overflow".into()))?;
			let encoded = reader.tlv_bytes().map_err(|error| {
				CertChainValidationError::ParseError(format!(
					"failed to read certificate at offset {offset}: {error}"
				))
			})?;
			let cert = Certificate::from_der(encoded).map_err(|error| {
				CertChainValidationError::ParseError(format!(
					"failed to parse certificate at offset {offset}: {error}"
				))
			})?;
			certs.push(cert);
		}

		Ok(certs)
	}

	fn verify_issuer_chain(&self, certs: &[Certificate]) -> Result<(), CertChainValidationError> {
		for pair in certs.windows(2) {
			let child = &pair[0];
			let issuer = &pair[1];
			if child.tbs_certificate.issuer != issuer.tbs_certificate.subject {
				return Err(CertChainValidationError::IssuerNameMismatch {
					expected: issuer.tbs_certificate.subject.to_string(),
					actual: child.tbs_certificate.issuer.to_string(),
				});
			}
			self.verify_cert_signature(child, issuer)?;
		}

		let root = certs.last().ok_or_else(|| {
			CertChainValidationError::InvalidChainFormat("empty certificate chain".into())
		})?;
		if root.tbs_certificate.issuer != root.tbs_certificate.subject {
			return Err(CertChainValidationError::UntrustedRoot);
		}
		if !self.is_root_trusted(root) {
			return Err(CertChainValidationError::UntrustedRoot);
		}
		self.verify_cert_signature(root, root)
	}

	fn verify_cert_signature(
		&self,
		cert: &Certificate,
		issuer: &Certificate,
	) -> Result<(), CertChainValidationError> {
		validate_signature_algorithm(&cert.signature_algorithm, &cert.tbs_certificate.signature)?;
		let tbs = cert.tbs_certificate.to_der().map_err(|error| {
			CertChainValidationError::ParseError(format!(
				"failed to encode certificate TBS: {error}"
			))
		})?;
		let signature = cert.signature.as_bytes().ok_or_else(|| {
			CertChainValidationError::ParseError("certificate signature is not byte-aligned".into())
		})?;
		verify_p256_signature(
			issuer.tbs_certificate.subject_public_key_info.subject_public_key.raw_bytes(),
			&tbs,
			signature,
			CertChainValidationError::SignatureVerificationFailed,
		)
	}

	fn is_root_trusted(&self, root: &Certificate) -> bool {
		let root_key = root.tbs_certificate.subject_public_key_info.subject_public_key.raw_bytes();
		self.trusted_roots.iter().any(|anchor| {
			let _diagnostic_name = &anchor.name;
			anchor.key_bytes == root_key
		})
	}

	fn validate_leaf_certificate(
		&self,
		cert: &Certificate,
	) -> Result<(), CertChainValidationError> {
		if let Some((_, constraints)) = cert
			.tbs_certificate
			.get::<BasicConstraints>()
			.map_err(|error| CertChainValidationError::ParseError(error.to_string()))?
		{
			if constraints.ca || constraints.path_len_constraint.is_some() {
				return Err(CertChainValidationError::MissingBasicConstraints);
			}
		}
		if let Some((_, key_usage)) = cert
			.tbs_certificate
			.get::<KeyUsage>()
			.map_err(|error| CertChainValidationError::ParseError(error.to_string()))?
		{
			if !key_usage.digital_signature() && !key_usage.non_repudiation() {
				return Err(CertChainValidationError::InvalidKeyUsage);
			}
		}
		Ok(())
	}

	fn validate_pck_leaf_profile(
		&self,
		cert: &Certificate,
	) -> Result<(), CertChainValidationError> {
		let has_sgx_extensions =
			cert.tbs_certificate.extensions.as_ref().into_iter().flatten().any(|extension| {
				extension.extn_id == INTEL_SGX_PCK_EXTENSIONS_OID
					&& !extension.extn_value.as_bytes().is_empty()
			});
		if !has_sgx_extensions {
			return Err(CertChainValidationError::InvalidChainFormat(
				"leaf certificate is missing the Intel SGX PCK extensions".into(),
			));
		}
		Ok(())
	}

	fn validate_ca_certificate(
		&self,
		cert: &Certificate,
		index: usize,
		chain: &[Certificate],
	) -> Result<(), CertChainValidationError> {
		let (critical, constraints) = cert
			.tbs_certificate
			.get::<BasicConstraints>()
			.map_err(|error| CertChainValidationError::ParseError(error.to_string()))?
			.ok_or(CertChainValidationError::MissingBasicConstraints)?;
		if !critical || !constraints.ca {
			return Err(CertChainValidationError::MissingBasicConstraints);
		}

		let (_, key_usage) = cert
			.tbs_certificate
			.get::<KeyUsage>()
			.map_err(|error| CertChainValidationError::ParseError(error.to_string()))?
			.ok_or(CertChainValidationError::InvalidKeyUsage)?;
		if !key_usage.key_cert_sign() {
			return Err(CertChainValidationError::InvalidKeyUsage);
		}

		if let Some(limit) = constraints.path_len_constraint {
			let subordinate_non_self_issued_cas = chain[1..index]
				.iter()
				.filter(|subordinate| {
					subordinate.tbs_certificate.subject != subordinate.tbs_certificate.issuer
				})
				.count();
			if subordinate_non_self_issued_cas > usize::from(limit) {
				return Err(CertChainValidationError::PathLengthViolation);
			}
		}
		Ok(())
	}

	fn check_validity_period(&self, cert: &Certificate) -> Result<(), CertChainValidationError> {
		let now = unix_time()?;
		let validity = &cert.tbs_certificate.validity;
		if now < validity.not_before.to_unix_duration().as_secs() {
			return Err(CertChainValidationError::NotYetValid);
		}
		if now > validity.not_after.to_unix_duration().as_secs() {
			return Err(CertChainValidationError::ExpiredBefore);
		}
		Ok(())
	}

	fn check_revocation(&self, certs: &[Certificate]) -> Result<(), CertChainValidationError> {
		let checker = match &self.crl_checker {
			Some(checker) => checker,
			None if self.strict_crl_checking => {
				return Err(CertChainValidationError::MissingCrl(
					certs[0].tbs_certificate.issuer.to_string(),
				));
			},
			None => return Ok(()),
		};

		for pair in certs.windows(2) {
			let cert = &pair[0];
			let issuer = &pair[1];
			let serial = cert.tbs_certificate.serial_number.as_bytes();
			if checker.is_serial_revoked(serial) {
				return Err(CertChainValidationError::RevokedSerial(
					cert.tbs_certificate.serial_number.to_string(),
				));
			}

			if !self.strict_crl_checking {
				continue;
			}

			let mut issuer_crl_seen = false;
			let mut valid_issuer_crl_seen = false;
			let mut last_crl_error = None;
			for encoded in checker.encoded_crls() {
				let crl = CertificateList::from_der(encoded).map_err(|error| {
					CertChainValidationError::ParseError(format!("invalid configured CRL: {error}"))
				})?;
				if crl.tbs_cert_list.issuer != issuer.tbs_certificate.subject {
					continue;
				}
				issuer_crl_seen = true;
				validate_crl_scope(&crl)?;
				match self.validate_crl(&crl, issuer) {
					Ok(()) => {
						valid_issuer_crl_seen = true;
						if crl
							.tbs_cert_list
							.revoked_certificates
							.as_ref()
							.into_iter()
							.flatten()
							.any(|entry| entry.serial_number.as_bytes() == serial)
						{
							return Err(CertChainValidationError::RevokedSerial(
								cert.tbs_certificate.serial_number.to_string(),
							));
						}
					},
					Err(error) => last_crl_error = Some(error),
				}
			}

			if !issuer_crl_seen {
				return Err(CertChainValidationError::MissingCrl(
					issuer.tbs_certificate.subject.to_string(),
				));
			}
			if !valid_issuer_crl_seen {
				return Err(last_crl_error.unwrap_or_else(|| {
					CertChainValidationError::MissingCrl(issuer.tbs_certificate.subject.to_string())
				}));
			}
		}
		Ok(())
	}

	fn validate_crl(
		&self,
		crl: &CertificateList,
		issuer: &Certificate,
	) -> Result<(), CertChainValidationError> {
		let (_, key_usage) = issuer
			.tbs_certificate
			.get::<KeyUsage>()
			.map_err(|error| CertChainValidationError::ParseError(error.to_string()))?
			.ok_or(CertChainValidationError::InvalidKeyUsage)?;
		if !key_usage.crl_sign() {
			return Err(CertChainValidationError::InvalidKeyUsage);
		}

		validate_signature_algorithm(&crl.signature_algorithm, &crl.tbs_cert_list.signature)?;
		let tbs = crl.tbs_cert_list.to_der().map_err(|error| {
			CertChainValidationError::ParseError(format!("failed to encode CRL TBS: {error}"))
		})?;
		let signature = crl.signature.as_bytes().ok_or_else(|| {
			CertChainValidationError::ParseError("CRL signature is not byte-aligned".into())
		})?;
		verify_p256_signature(
			issuer.tbs_certificate.subject_public_key_info.subject_public_key.raw_bytes(),
			&tbs,
			signature,
			CertChainValidationError::InvalidCrlSignature,
		)?;

		let now = unix_time()?;
		if now < crl.tbs_cert_list.this_update.to_unix_duration().as_secs() {
			return Err(CertChainValidationError::CrlNotYetValid);
		}
		let next_update =
			crl.tbs_cert_list.next_update.ok_or(CertChainValidationError::ExpiredCrl)?;
		if now > next_update.to_unix_duration().as_secs() {
			return Err(CertChainValidationError::ExpiredCrl);
		}
		Ok(())
	}

	fn verify_tcb_level(
		&self,
		enclave_tcb: &MinimumTcbLevel,
	) -> Result<(), CertChainValidationError> {
		if let Some(required) = &self.tcb_requirements {
			if !required.satisfies(enclave_tcb) {
				return Err(CertChainValidationError::TcbLevelTooLow {
					current: format!(
						"cpu:{},pce:{},qe:{}",
						enclave_tcb.cpu_svn, enclave_tcb.pce_svn, enclave_tcb.qe_svn
					),
					required: format!(
						"cpu:{},pce:{},qe:{}",
						required.cpu_svn, required.pce_svn, required.qe_svn
					),
				});
			}
		}
		Ok(())
	}
}

fn validate_crl_scope(crl: &CertificateList) -> Result<(), CertChainValidationError> {
	for extension in crl.tbs_cert_list.crl_extensions.as_ref().into_iter().flatten() {
		let unsupported = match extension.extn_id {
			DELTA_CRL_INDICATOR_OID => Some("deltaCRLIndicator"),
			ISSUING_DISTRIBUTION_POINT_OID => Some("IssuingDistributionPoint"),
			_ => None,
		};
		if let Some(name) = unsupported {
			return Err(CertChainValidationError::UnsupportedCrlExtension(name.to_string()));
		}
	}
	Ok(())
}

impl Default for CertChainValidator {
	fn default() -> Self {
		Self::new()
	}
}

fn validate_signature_algorithm(
	outer: &x509_cert::spki::AlgorithmIdentifierOwned,
	tbs: &x509_cert::spki::AlgorithmIdentifierOwned,
) -> Result<(), CertChainValidationError> {
	if outer != tbs || outer.oid != ECDSA_WITH_SHA256_OID || outer.parameters.is_some() {
		return Err(CertChainValidationError::UnsupportedSignatureAlgorithm);
	}
	Ok(())
}

fn verify_p256_signature(
	public_key: &[u8],
	message: &[u8],
	signature_bytes: &[u8],
	error: CertChainValidationError,
) -> Result<(), CertChainValidationError> {
	UnparsedPublicKey::new(&ECDSA_P256_SHA256_ASN1, public_key)
		.verify(message, signature_bytes)
		.map_err(|_| error)
}

fn unix_time() -> Result<u64, CertChainValidationError> {
	SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.map(|duration| duration.as_secs())
		.map_err(|error| {
			CertChainValidationError::ParseError(format!("system time error: {error}"))
		})
}

fn trim_ascii_whitespace(mut data: &[u8]) -> &[u8] {
	while data.first().is_some_and(u8::is_ascii_whitespace) {
		data = &data[1..];
	}
	while data.last().is_some_and(u8::is_ascii_whitespace) {
		data = &data[..data.len() - 1];
	}
	data
}

#[cfg(test)]
mod tests {
	use std::{str::FromStr, time::Duration};

	use ::signature::Signer;
	use const_oid::AssociatedOid;
	use der::{
		asn1::{BitString, OctetString},
		pem::LineEnding,
		Encode, EncodePem,
	};
	use p256::{
		ecdsa::{DerSignature, SigningKey},
		pkcs8::EncodePublicKey,
	};
	use x509_cert::{
		builder::{Builder, CertificateBuilder, Profile},
		crl::{CertificateList, RevokedCert, TbsCertList},
		ext::{AsExtension, Extension},
		name::Name,
		serial_number::SerialNumber,
		spki::{DynSignatureAlgorithmIdentifier, SubjectPublicKeyInfoOwned},
		time::{Time, Validity},
		Version,
	};

	use super::*;

	struct TestPath {
		leaf: Vec<u8>,
		intermediate: Vec<u8>,
		root: Vec<u8>,
		leaf_crl: Vec<u8>,
		intermediate_crl: Vec<u8>,
		root_public_key: Vec<u8>,
	}

	#[derive(Clone, Debug, Eq, PartialEq)]
	struct TestSgxExtensions(OctetString);

	impl AssociatedOid for TestSgxExtensions {
		const OID: ObjectIdentifier = INTEL_SGX_PCK_EXTENSIONS_OID;
	}

	x509_cert::impl_newtype!(TestSgxExtensions, OctetString);

	impl AsExtension for TestSgxExtensions {
		fn critical(&self, _subject: &Name, _extensions: &[Extension]) -> bool {
			false
		}
	}

	fn signing_key(byte: u8) -> SigningKey {
		SigningKey::from_slice(&[byte; 32]).expect("valid deterministic scalar")
	}

	fn public_key(key: &SigningKey) -> SubjectPublicKeyInfoOwned {
		let document = key.verifying_key().to_public_key_der().expect("encode public key");
		SubjectPublicKeyInfoOwned::try_from(document.as_bytes()).expect("parse public key")
	}

	fn valid_now() -> Validity {
		Validity::from_now(Duration::from_secs(86_400)).expect("validity")
	}

	fn expired() -> Validity {
		let now = SystemTime::now();
		Validity {
			not_before: Time::try_from(now - Duration::from_secs(7_200)).expect("not before"),
			not_after: Time::try_from(now - Duration::from_secs(3_600)).expect("not after"),
		}
	}

	fn build_cert(
		profile: Profile,
		serial: u64,
		validity: Validity,
		subject: Name,
		subject_key: &SigningKey,
		issuer_key: &SigningKey,
		include_sgx_extensions: bool,
	) -> Certificate {
		let mut builder = CertificateBuilder::new(
			profile,
			SerialNumber::from(serial),
			validity,
			subject,
			public_key(subject_key),
			issuer_key,
		)
		.expect("certificate builder");
		if include_sgx_extensions {
			builder
				.add_extension(&TestSgxExtensions(
					OctetString::new([0x30, 0x00]).expect("test SGX extension"),
				))
				.expect("add SGX extension");
		}
		builder.build::<DerSignature>().expect("certificate")
	}

	fn build_crl(issuer_name: Name, issuer_key: &SigningKey, revoked_serials: &[u64]) -> Vec<u8> {
		build_crl_with_extensions(issuer_name, issuer_key, revoked_serials, None)
	}

	fn build_crl_with_extensions(
		issuer_name: Name,
		issuer_key: &SigningKey,
		revoked_serials: &[u64],
		crl_extensions: Option<Vec<Extension>>,
	) -> Vec<u8> {
		let now = SystemTime::now();
		let algorithm = issuer_key.signature_algorithm_identifier().expect("signature algorithm");
		let revoked = (!revoked_serials.is_empty()).then(|| {
			revoked_serials
				.iter()
				.map(|serial| RevokedCert {
					serial_number: SerialNumber::from(*serial),
					revocation_date: Time::try_from(now - Duration::from_secs(60))
						.expect("revocation date"),
					crl_entry_extensions: None,
				})
				.collect()
		});
		let tbs = TbsCertList {
			version: Version::V2,
			signature: algorithm.clone(),
			issuer: issuer_name,
			this_update: Time::try_from(now - Duration::from_secs(60)).expect("this update"),
			next_update: Some(
				Time::try_from(now + Duration::from_secs(3_600)).expect("next update"),
			),
			revoked_certificates: revoked,
			crl_extensions,
		};
		let tbs_der = tbs.to_der().expect("CRL TBS DER");
		let signature: DerSignature = issuer_key.sign(&tbs_der);
		CertificateList {
			tbs_cert_list: tbs,
			signature_algorithm: algorithm,
			signature: BitString::from_bytes(signature.as_bytes()).expect("signature bit string"),
		}
		.to_der()
		.expect("CRL DER")
	}

	fn test_path_with_profile(
		intermediate_validity: Validity,
		root_path_len: Option<u8>,
		include_sgx_extensions: bool,
	) -> TestPath {
		let root_key = signing_key(1);
		let intermediate_key = signing_key(2);
		let leaf_key = signing_key(3);
		let root_name = Name::from_str("CN=Test SGX Root,O=R3E,C=US").expect("root name");
		let intermediate_name =
			Name::from_str("CN=Test SGX Intermediate,O=R3E,C=US").expect("intermediate name");
		let leaf_name = Name::from_str("CN=Test PCK,O=R3E,C=US").expect("leaf name");

		let root_profile = match root_path_len {
			Some(limit) => {
				Profile::SubCA { issuer: root_name.clone(), path_len_constraint: Some(limit) }
			},
			None => Profile::Root,
		};
		let root = build_cert(
			root_profile,
			100,
			valid_now(),
			root_name.clone(),
			&root_key,
			&root_key,
			false,
		);
		let intermediate = build_cert(
			Profile::SubCA { issuer: root_name.clone(), path_len_constraint: Some(0) },
			200,
			intermediate_validity,
			intermediate_name.clone(),
			&intermediate_key,
			&root_key,
			false,
		);
		let leaf = build_cert(
			Profile::Leaf {
				issuer: intermediate_name.clone(),
				enable_key_agreement: false,
				enable_key_encipherment: false,
			},
			300,
			valid_now(),
			leaf_name,
			&leaf_key,
			&intermediate_key,
			include_sgx_extensions,
		);

		TestPath {
			leaf: leaf.to_der().expect("leaf DER"),
			intermediate: intermediate.to_der().expect("intermediate DER"),
			root: root.to_der().expect("root DER"),
			leaf_crl: build_crl(intermediate_name, &intermediate_key, &[]),
			intermediate_crl: build_crl(root_name, &root_key, &[]),
			root_public_key: root_key.verifying_key().to_encoded_point(false).as_bytes().to_vec(),
		}
	}

	fn test_path(intermediate_validity: Validity, root_path_len: Option<u8>) -> TestPath {
		test_path_with_profile(intermediate_validity, root_path_len, true)
	}

	fn concatenate(path: &TestPath) -> Vec<u8> {
		[&path.leaf[..], &path.intermediate[..], &path.root[..]].concat()
	}

	fn concatenate_pem(path: &TestPath) -> Vec<u8> {
		[&path.leaf, &path.intermediate, &path.root]
			.into_iter()
			.map(|der| {
				Certificate::from_der(der)
					.expect("parse generated certificate")
					.to_pem(LineEnding::LF)
					.expect("PEM certificate")
			})
			.collect::<String>()
			.into_bytes()
	}

	fn validator_for(path: &TestPath, with_crls: bool) -> CertChainValidator {
		let mut validator = CertChainValidator::new();
		validator.clear_trusted_roots();
		validator.add_trusted_root("test root", &path.root_public_key);
		if with_crls {
			let mut checker = CrLChecker::new();
			checker.add_crl_der(&path.leaf_crl).expect("leaf CRL");
			checker.add_crl_der(&path.intermediate_crl).expect("intermediate CRL");
			validator.set_crl_checker(checker);
		}
		validator
	}

	#[test]
	fn valid_signed_chain_with_fresh_crls_is_accepted() {
		let path = test_path(valid_now(), None);
		let result = validator_for(&path, true)
			.validate(&concatenate(&path), &MinimumTcbLevel::new(0, 0, 0, 0))
			.expect("valid chain");
		assert!(result.is_valid);
		assert_eq!(result.chain_length, 3);
	}

	#[test]
	fn non_pck_leaf_under_a_trusted_root_is_rejected() {
		let path = test_path_with_profile(valid_now(), None, false);
		assert!(matches!(
			validator_for(&path, true)
				.validate(&concatenate(&path), &MinimumTcbLevel::new(0, 0, 0, 0)),
			Err(CertChainValidationError::InvalidChainFormat(message))
				if message.contains("SGX PCK extensions")
		));
	}

	#[test]
	fn exact_der_lengths_parse_differently_sized_certificates() {
		let path = test_path(valid_now(), None);
		assert_ne!(path.leaf.len(), path.root.len());
		let certs = validator_for(&path, true)
			.parse_certificates(&concatenate(&path))
			.expect("parse chain");
		assert_eq!(certs.len(), 3);
	}

	#[test]
	fn quote_type_five_pem_chain_is_accepted() {
		let path = test_path(valid_now(), None);
		let result = validator_for(&path, true)
			.validate(&concatenate_pem(&path), &MinimumTcbLevel::new(0, 0, 0, 0))
			.expect("valid PEM chain");
		assert_eq!(result.chain_length, 3);
	}

	#[test]
	fn quote_type_five_pem_chain_with_qcnl_terminator_is_accepted() {
		let path = test_path(valid_now(), None);
		let mut chain = concatenate_pem(&path);
		chain.push(0);
		let result = validator_for(&path, true)
			.validate(&chain, &MinimumTcbLevel::new(0, 0, 0, 0))
			.expect("valid QCNL PEM chain");
		assert_eq!(result.chain_length, 3);
	}

	#[test]
	fn quote_type_five_pem_chain_with_two_terminators_is_rejected() {
		let path = test_path(valid_now(), None);
		let mut chain = concatenate_pem(&path);
		chain.extend_from_slice(&[0, 0]);
		assert!(validator_for(&path, true)
			.validate(&chain, &MinimumTcbLevel::new(0, 0, 0, 0))
			.is_err());
	}

	#[test]
	fn pem_chain_with_non_whitespace_trailing_data_is_rejected() {
		let path = test_path(valid_now(), None);
		let mut chain = concatenate_pem(&path);
		chain.extend_from_slice(b"x");
		assert!(matches!(
			validator_for(&path, true).validate(&chain, &MinimumTcbLevel::new(0, 0, 0, 0)),
			Err(CertChainValidationError::ParseError(_))
		));
	}

	#[test]
	fn trailing_or_partial_der_is_rejected() {
		let path = test_path(valid_now(), None);
		let mut chain = concatenate(&path);
		chain.push(0);
		assert!(matches!(
			validator_for(&path, true).validate(&chain, &MinimumTcbLevel::new(0, 0, 0, 0)),
			Err(CertChainValidationError::ParseError(_))
		));
	}

	#[test]
	fn tampered_certificate_signature_is_rejected() {
		let path = test_path(valid_now(), None);
		let mut chain = concatenate(&path);
		let signature_byte = path.leaf.len() - 1;
		chain[signature_byte] ^= 1;
		assert_eq!(
			validator_for(&path, true)
				.validate(&chain, &MinimumTcbLevel::new(0, 0, 0, 0))
				.unwrap_err(),
			CertChainValidationError::SignatureVerificationFailed
		);
	}

	#[test]
	fn untrusted_root_is_rejected() {
		let path = test_path(valid_now(), None);
		assert_eq!(
			CertChainValidator::new()
				.validate(&concatenate(&path), &MinimumTcbLevel::new(0, 0, 0, 0))
				.unwrap_err(),
			CertChainValidationError::UntrustedRoot
		);
	}

	#[test]
	fn default_crl_policy_fails_closed() {
		let path = test_path(valid_now(), None);
		assert!(matches!(
			validator_for(&path, false)
				.validate(&concatenate(&path), &MinimumTcbLevel::new(0, 0, 0, 0)),
			Err(CertChainValidationError::MissingCrl(_))
		));
	}

	#[test]
	fn signed_crl_revocation_is_rejected() {
		let path = test_path(valid_now(), None);
		let root_key = signing_key(1);
		let intermediate_key = signing_key(2);
		let root_name = Name::from_str("CN=Test SGX Root,O=R3E,C=US").expect("root name");
		let intermediate_name =
			Name::from_str("CN=Test SGX Intermediate,O=R3E,C=US").expect("intermediate name");
		let mut checker = CrLChecker::new();
		checker
			.add_crl_der(&build_crl(intermediate_name, &intermediate_key, &[300]))
			.expect("revoking CRL");
		checker.add_crl_der(&build_crl(root_name, &root_key, &[])).expect("root CRL");
		let mut validator = validator_for(&path, false);
		validator.set_crl_checker(checker);

		assert!(matches!(
			validator.validate(&concatenate(&path), &MinimumTcbLevel::new(0, 0, 0, 0)),
			Err(CertChainValidationError::RevokedSerial(_))
		));
	}

	#[test]
	fn later_overlapping_crl_cannot_hide_a_revocation() {
		let path = test_path(valid_now(), None);
		let intermediate_key = signing_key(2);
		let intermediate_name =
			Name::from_str("CN=Test SGX Intermediate,O=R3E,C=US").expect("intermediate name");
		let mut checker = CrLChecker::new();
		checker.add_crl_der(&path.leaf_crl).expect("non-revoking leaf CRL");
		checker
			.add_crl_der(&build_crl(intermediate_name, &intermediate_key, &[300]))
			.expect("revoking leaf CRL");
		checker.add_crl_der(&path.intermediate_crl).expect("root CRL");
		let mut validator = validator_for(&path, false);
		validator.set_crl_checker(checker);

		assert!(matches!(
			validator.validate(&concatenate(&path), &MinimumTcbLevel::new(0, 0, 0, 0)),
			Err(CertChainValidationError::RevokedSerial(_))
		));
	}

	#[test]
	fn delta_crl_is_rejected_even_when_a_full_crl_for_the_issuer_is_present() {
		let path = test_path(valid_now(), None);
		let intermediate_key = signing_key(2);
		let intermediate_name =
			Name::from_str("CN=Test SGX Intermediate,O=R3E,C=US").expect("intermediate name");
		let delta_indicator = Extension {
			extn_id: DELTA_CRL_INDICATOR_OID,
			critical: true,
			extn_value: OctetString::new([0x02, 0x01, 0x01]).expect("delta CRL number"),
		};
		let mut checker = CrLChecker::new();
		checker.add_crl_der(&path.leaf_crl).expect("full leaf CRL");
		checker
			.add_crl_der(&build_crl_with_extensions(
				intermediate_name,
				&intermediate_key,
				&[],
				Some(vec![delta_indicator]),
			))
			.expect("delta CRL");
		checker.add_crl_der(&path.intermediate_crl).expect("root CRL");
		let mut validator = validator_for(&path, false);
		validator.set_crl_checker(checker);

		assert_eq!(
			validator
				.validate(&concatenate(&path), &MinimumTcbLevel::new(0, 0, 0, 0))
				.unwrap_err(),
			CertChainValidationError::UnsupportedCrlExtension("deltaCRLIndicator".into())
		);
	}

	#[test]
	fn issuing_distribution_point_crl_is_rejected() {
		let path = test_path(valid_now(), None);
		let intermediate_key = signing_key(2);
		let intermediate_name =
			Name::from_str("CN=Test SGX Intermediate,O=R3E,C=US").expect("intermediate name");
		let issuing_distribution_point = Extension {
			extn_id: ISSUING_DISTRIBUTION_POINT_OID,
			critical: true,
			extn_value: OctetString::new([0x30, 0x03, 0x82, 0x01, 0xFF])
				.expect("issuing distribution point"),
		};
		let mut checker = CrLChecker::new();
		checker
			.add_crl_der(&build_crl_with_extensions(
				intermediate_name,
				&intermediate_key,
				&[],
				Some(vec![issuing_distribution_point]),
			))
			.expect("partitioned CRL");
		checker.add_crl_der(&path.intermediate_crl).expect("root CRL");
		let mut validator = validator_for(&path, false);
		validator.set_crl_checker(checker);

		assert_eq!(
			validator
				.validate(&concatenate(&path), &MinimumTcbLevel::new(0, 0, 0, 0))
				.unwrap_err(),
			CertChainValidationError::UnsupportedCrlExtension("IssuingDistributionPoint".into())
		);
	}

	#[test]
	fn tampered_crl_signature_is_rejected() {
		let path = test_path(valid_now(), None);
		let mut tampered_crl = path.leaf_crl.clone();
		let last = tampered_crl.len() - 1;
		tampered_crl[last] ^= 1;
		let mut checker = CrLChecker::new();
		checker.add_crl_der(&tampered_crl).expect("parse tampered CRL");
		checker.add_crl_der(&path.intermediate_crl).expect("root CRL");
		let mut validator = validator_for(&path, false);
		validator.set_crl_checker(checker);

		assert_eq!(
			validator
				.validate(&concatenate(&path), &MinimumTcbLevel::new(0, 0, 0, 0))
				.unwrap_err(),
			CertChainValidationError::InvalidCrlSignature
		);
	}

	#[test]
	fn expired_intermediate_is_rejected() {
		let path = test_path(expired(), None);
		assert_eq!(
			validator_for(&path, true)
				.validate(&concatenate(&path), &MinimumTcbLevel::new(0, 0, 0, 0))
				.unwrap_err(),
			CertChainValidationError::ExpiredBefore
		);
	}

	#[test]
	fn root_path_length_constraint_is_enforced() {
		let path = test_path(valid_now(), Some(0));
		assert_eq!(
			validator_for(&path, true)
				.validate(&concatenate(&path), &MinimumTcbLevel::new(0, 0, 0, 0))
				.unwrap_err(),
			CertChainValidationError::PathLengthViolation
		);
	}

	#[test]
	fn built_in_intel_root_key_is_a_valid_p256_point() {
		p256::PublicKey::from_sec1_bytes(&INTEL_SGX_PROVISIONING_ROOT_CA_PUBKEY)
			.expect("Intel trust anchor must be a valid P-256 public key");
	}
}
