//! Certificate-revocation inputs used by the SGX certificate validator.
//!
//! A manually populated serial denylist is additive only: it never counts as
//! proof that an issuer's complete revocation state was checked. In strict mode
//! the certificate validator also requires a fresh, issuer-signed DER CRL for
//! every non-root certificate in the path. Delta and partitioned CRLs are not
//! supported as complete coverage and are rejected by the validator.

use std::collections::HashMap;

#[cfg(feature = "sgx")]
use der::Decode;
#[cfg(feature = "sgx")]
use x509_cert::crl::CertificateList;

#[cfg(any(feature = "sgx", test))]
const MAX_CRL_DER_BYTES: usize = 8 * 1024 * 1024;
#[cfg(any(feature = "sgx", test))]
const MAX_SIGNED_CRLS: usize = 8;
#[cfg(any(feature = "sgx", test))]
const MAX_TOTAL_CRL_DER_BYTES: usize = 16 * 1024 * 1024;

/// In-memory collection of signed CRLs plus an optional local serial denylist.
#[derive(Clone, Debug, Default)]
pub struct CrLChecker {
	revoked_serials: HashMap<Vec<u8>, u64>,
	encoded_crls: Vec<Vec<u8>>,
	#[cfg(feature = "sgx")]
	encoded_crl_bytes: usize,
}

impl CrLChecker {
	/// Create an empty checker.
	///
	/// An empty checker is not authoritative and strict validation will reject
	/// chains until applicable signed CRLs are added.
	pub fn new() -> Self {
		Self::default()
	}

	/// Check the compatibility `u64` denylist.
	pub fn is_revoked(&self, serial: u64) -> bool {
		self.is_serial_revoked(&unsigned_integer_bytes(serial))
	}

	/// Add a serial to the local denylist.
	///
	/// This is useful as an emergency denylist, but does not replace signed CRL
	/// coverage when strict checking is enabled.
	pub fn add_revoked(&mut self, serial: u64, timestamp: u64) {
		self.add_revoked_serial(&unsigned_integer_bytes(serial), timestamp);
	}

	/// Check an arbitrary-size X.509 serial number in canonical big-endian form.
	pub fn is_serial_revoked(&self, serial: &[u8]) -> bool {
		self.revoked_serials.contains_key(&canonical_serial(serial))
	}

	/// Add an arbitrary-size X.509 serial number to the local denylist.
	pub fn add_revoked_serial(&mut self, serial: &[u8], timestamp: u64) {
		self.revoked_serials.insert(canonical_serial(serial), timestamp);
	}

	/// Add one complete DER-encoded X.509 CRL.
	///
	/// At most eight CRLs may be stored. Each input is limited to 8 MiB and the
	/// cumulative encoded size is limited to 16 MiB.
	///
	/// Structural parsing happens here. Issuer matching, signature validation,
	/// freshness, and certificate lookup happen during chain validation.
	#[cfg(feature = "sgx")]
	pub fn add_crl_der(&mut self, crl_der: &[u8]) -> Result<(), String> {
		let updated_total = checked_crl_storage_size(
			self.encoded_crls.len(),
			self.encoded_crl_bytes,
			crl_der.len(),
		)?;
		CertificateList::from_der(crl_der).map_err(|error| format!("invalid DER CRL: {error}"))?;
		self.encoded_crls.push(crl_der.to_vec());
		self.encoded_crl_bytes = updated_total;
		Ok(())
	}

	/// Return the number of signed CRLs currently configured.
	pub fn crl_count(&self) -> usize {
		self.encoded_crls.len()
	}

	#[cfg(feature = "sgx")]
	pub(crate) fn encoded_crls(&self) -> &[Vec<u8>] {
		&self.encoded_crls
	}
}

#[cfg(any(feature = "sgx", test))]
fn checked_crl_storage_size(
	existing_count: usize,
	existing_bytes: usize,
	new_crl_bytes: usize,
) -> Result<usize, String> {
	if new_crl_bytes > MAX_CRL_DER_BYTES {
		return Err(format!("CRL exceeds the {MAX_CRL_DER_BYTES}-byte size limit"));
	}
	if existing_count >= MAX_SIGNED_CRLS {
		return Err(format!("CRL count exceeds the limit of {MAX_SIGNED_CRLS}"));
	}
	let updated_total = existing_bytes
		.checked_add(new_crl_bytes)
		.ok_or_else(|| "total CRL size overflow".to_string())?;
	if updated_total > MAX_TOTAL_CRL_DER_BYTES {
		return Err(format!("total CRL size exceeds the {MAX_TOTAL_CRL_DER_BYTES}-byte limit"));
	}
	Ok(updated_total)
}

impl From<()> for CrLChecker {
	fn from((): ()) -> Self {
		Self::new()
	}
}

fn unsigned_integer_bytes(serial: u64) -> Vec<u8> {
	canonical_serial(&serial.to_be_bytes())
}

fn canonical_serial(serial: &[u8]) -> Vec<u8> {
	let first_nonzero = serial.iter().position(|byte| *byte != 0).unwrap_or(serial.len());
	if first_nonzero == serial.len() {
		vec![0]
	} else {
		serial[first_nonzero..].to_vec()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn local_denylist_canonicalizes_leading_zeroes() {
		let mut checker = CrLChecker::new();
		checker.add_revoked_serial(&[0, 0, 0x12, 0x34], 1);
		assert!(checker.is_serial_revoked(&[0x12, 0x34]));
		assert!(checker.is_serial_revoked(&[0, 0x12, 0x34]));

		checker.add_revoked(0xDEAD_BEEF, 2);
		assert!(checker.is_revoked(0xDEAD_BEEF));
	}

	#[test]
	fn crl_storage_limits_cover_individual_count_total_and_overflow_boundaries() {
		assert_eq!(
			checked_crl_storage_size(0, 0, MAX_CRL_DER_BYTES).expect("single limit"),
			MAX_CRL_DER_BYTES
		);
		assert!(checked_crl_storage_size(0, 0, MAX_CRL_DER_BYTES + 1)
			.unwrap_err()
			.contains("size limit"));
		assert!(checked_crl_storage_size(MAX_SIGNED_CRLS, 0, 1).unwrap_err().contains("count"));
		assert!(checked_crl_storage_size(1, MAX_TOTAL_CRL_DER_BYTES, 1)
			.unwrap_err()
			.contains("total CRL size"));
		assert!(checked_crl_storage_size(1, usize::MAX, 1).unwrap_err().contains("overflow"));
	}

	#[cfg(feature = "sgx")]
	#[test]
	fn oversized_crl_is_rejected_before_parsing_or_storage() {
		let mut checker = CrLChecker::new();
		let oversized = vec![0u8; MAX_CRL_DER_BYTES + 1];
		assert!(checker.add_crl_der(&oversized).unwrap_err().contains("size limit"));
		assert_eq!(checker.crl_count(), 0);
		assert_eq!(checker.encoded_crl_bytes, 0);
	}
}
