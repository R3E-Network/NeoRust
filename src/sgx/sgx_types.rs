//! Intel SGX ECDSA quote v3 data types and parser.
//!
//! The parser follows the packed `sgx_quote3_t`/`sgx_ql_ecdsa_sig_data_t`
//! layout. Every variable-length field is bounds checked and the declared
//! lengths must consume the complete input.

use serde::{Deserialize, Serialize};

const QUOTE_HEADER_LEN: usize = 48;
const REPORT_BODY_LEN: usize = 384;
const SIGNATURE_DATA_LEN_OFFSET: usize = QUOTE_HEADER_LEN + REPORT_BODY_LEN;
const QUOTE_FIXED_LEN: usize = SIGNATURE_DATA_LEN_OFFSET + 4;

const QUOTE_SIGNATURE_LEN: usize = 64;
const ATTESTATION_PUBLIC_KEY_LEN: usize = 64;
const QE_REPORT_BODY_LEN: usize = 384;
const QE_REPORT_SIGNATURE_LEN: usize = 64;
const AUTHENTICATION_DATA_LEN_FIELD: usize = 2;
const CERTIFICATION_DATA_HEADER_LEN: usize = 6;
const ECDSA_SIGNATURE_DATA_FIXED_LEN: usize = QUOTE_SIGNATURE_LEN
	+ ATTESTATION_PUBLIC_KEY_LEN
	+ QE_REPORT_BODY_LEN
	+ QE_REPORT_SIGNATURE_LEN
	+ AUTHENTICATION_DATA_LEN_FIELD
	+ CERTIFICATION_DATA_HEADER_LEN;

const ECDSA_P256_ATTESTATION_KEY_TYPE: u16 = 2;
const SGX_TEE_TYPE: u32 = 0;
const PCK_CERT_CHAIN_DATA_TYPE: u16 = 5;
const MAX_CERTIFICATION_DATA_LEN: usize = 1024 * 1024;

/// SGX quote version 3 structure (ECDSA P-256 signature).
#[derive(Debug, Clone)]
pub struct SgxQuote {
	/// Quote header.
	pub header: QuoteHeader,

	/// SGX enclave report body.
	pub body: QuoteBody,

	/// Parsed ECDSA signature and certification data.
	pub signature: QuoteSignature,
}

impl SgxQuote {
	/// Parse one complete SGX ECDSA quote v3.
	pub fn parse(data: &[u8]) -> Result<Self, &'static str> {
		if data.len() < QUOTE_FIXED_LEN {
			return Err("Quote too short");
		}

		let mut header = Self::parse_header(&data[..QUOTE_HEADER_LEN])?;
		if header.version != 3 {
			return Err("Only SGX quote version 3 is supported");
		}
		if header.attestation_key_type != ECDSA_P256_ATTESTATION_KEY_TYPE {
			return Err("Unsupported SGX attestation key type");
		}
		if header.tee_type != SGX_TEE_TYPE {
			return Err("Quote is not an SGX quote");
		}

		let body_bytes = &data[QUOTE_HEADER_LEN..SIGNATURE_DATA_LEN_OFFSET];
		let body = Self::parse_body(body_bytes)?;
		// `xfrm` lives in `sgx_report_body_t`, not the quote header. Keep this
		// legacy public field populated with the correctly parsed value.
		header.xfrm =
			u64::from_le_bytes(body_bytes[56..64].try_into().map_err(|_| "Invalid XFRM field")?);

		let declared_signature_len = u32::from_le_bytes(
			data[SIGNATURE_DATA_LEN_OFFSET..QUOTE_FIXED_LEN]
				.try_into()
				.map_err(|_| "Invalid signature length field")?,
		) as usize;
		let expected_quote_len = QUOTE_FIXED_LEN
			.checked_add(declared_signature_len)
			.ok_or("Quote length overflow")?;
		if data.len() != expected_quote_len {
			return Err(if data.len() < expected_quote_len {
				"Quote signature data is truncated"
			} else {
				"Quote contains trailing data"
			});
		}

		let signature = Self::parse_signature(&data[QUOTE_FIXED_LEN..])?;
		Ok(Self { header, body, signature })
	}

	/// Extract MRENCLAVE from the SGX report body.
	pub fn mrenclave(&self) -> [u8; 32] {
		let mut measurement = [0u8; 32];
		measurement.copy_from_slice(&self.body.report_data[64..96]);
		measurement
	}

	/// Extract MRSIGNER from the SGX report body.
	pub fn mr_signer(&self) -> [u8; 32] {
		let mut measurement = [0u8; 32];
		measurement.copy_from_slice(&self.body.report_data[128..160]);
		measurement
	}

	/// Extract the 64-byte application-defined REPORTDATA field.
	pub fn enclave_report_data(&self) -> [u8; 64] {
		let mut report_data = [0u8; 64];
		report_data.copy_from_slice(&self.body.report_data[320..384]);
		report_data
	}

	/// Return the first 16 bytes of the quote header's caller-defined user data.
	///
	/// Quote v3 does not define a protocol nonce field. This compatibility
	/// accessor must only be used when the caller that produced the quote also
	/// defines these user-data bytes as a nonce.
	pub fn get_nonce(&self) -> [u8; 16] {
		self.header.dynamic_data.nonce
	}

	/// Return a caller-defined timestamp projection, if present.
	///
	/// Quote v3 has no timestamp field, so the parser leaves this value at zero.
	pub fn get_timestamp_seconds(&self) -> u64 {
		u64::from_le_bytes(self.header.dynamic_data.timestamp)
	}

	fn parse_header(data: &[u8]) -> Result<QuoteHeader, &'static str> {
		if data.len() != QUOTE_HEADER_LEN {
			return Err("Invalid quote header length");
		}

		let mut qe_vendor_id = [0u8; 16];
		qe_vendor_id.copy_from_slice(&data[12..28]);
		let mut user_data = [0u8; 20];
		user_data.copy_from_slice(&data[28..48]);
		let mut nonce = [0u8; 16];
		nonce.copy_from_slice(&user_data[..16]);

		Ok(QuoteHeader {
			version: u16::from_le_bytes([data[0], data[1]]),
			attestation_key_type: u16::from_le_bytes([data[2], data[3]]),
			tee_type: u32::from_le_bytes([data[4], data[5], data[6], data[7]]),
			qe_svn: u16::from_le_bytes([data[8], data[9]]),
			pce_svn: u16::from_le_bytes([data[10], data[11]]),
			qe_vendor_id,
			user_data,
			xfrm: 0,
			dynamic_data: DynamicDataFields { nonce, timestamp: [0u8; 8] },
		})
	}

	fn parse_body(data: &[u8]) -> Result<QuoteBody, &'static str> {
		if data.len() != REPORT_BODY_LEN {
			return Err("Invalid SGX report body length");
		}

		let mut report_data = [0u8; REPORT_BODY_LEN];
		report_data.copy_from_slice(data);

		Ok(QuoteBody {
			report_data,
			isv_product_id: u16::from_le_bytes([data[256], data[257]]),
			isvsvn: u16::from_le_bytes([data[258], data[259]]),
		})
	}

	fn parse_signature(data: &[u8]) -> Result<QuoteSignature, &'static str> {
		if data.len() < ECDSA_SIGNATURE_DATA_FIXED_LEN {
			return Err("ECDSA signature data is too short");
		}

		let mut cursor = 0usize;
		let epid_or_ecdsa_sig = take_array::<QUOTE_SIGNATURE_LEN>(data, &mut cursor)?;
		let _attestation_public_key = take_array::<ATTESTATION_PUBLIC_KEY_LEN>(data, &mut cursor)?;
		let _qe_report_body = take_array::<QE_REPORT_BODY_LEN>(data, &mut cursor)?;
		let _qe_report_signature = take_array::<QE_REPORT_SIGNATURE_LEN>(data, &mut cursor)?;

		let authentication_data_len = read_u16(data, &mut cursor)? as usize;
		let authentication_data_end = cursor
			.checked_add(authentication_data_len)
			.ok_or("Authentication data length overflow")?;
		let _authentication_data = data
			.get(cursor..authentication_data_end)
			.ok_or("Authentication data is truncated")?
			.to_vec();
		cursor = authentication_data_end;

		let certification_data_type = read_u16(data, &mut cursor)?;
		if certification_data_type != PCK_CERT_CHAIN_DATA_TYPE {
			return Err("Unsupported quote certification data type");
		}
		let certification_data_len = read_u32(data, &mut cursor)? as usize;
		if certification_data_len == 0 {
			return Err("PCK certificate chain is empty");
		}
		if certification_data_len > MAX_CERTIFICATION_DATA_LEN {
			return Err("PCK certificate chain exceeds the 1 MiB limit");
		}
		let certification_data_end = cursor
			.checked_add(certification_data_len)
			.ok_or("Certification data length overflow")?;
		if certification_data_end != data.len() {
			return Err(if certification_data_end > data.len() {
				"Certification data is truncated"
			} else {
				"Signature data contains trailing bytes"
			});
		}

		Ok(QuoteSignature {
			epid_or_ecdsa_sig,
			certificate_chain: data[cursor..certification_data_end].to_vec(),
		})
	}
}

fn take_array<const N: usize>(data: &[u8], cursor: &mut usize) -> Result<[u8; N], &'static str> {
	let end = cursor.checked_add(N).ok_or("Signature field length overflow")?;
	let value = data
		.get(*cursor..end)
		.ok_or("ECDSA signature data is truncated")?
		.try_into()
		.map_err(|_| "Invalid signature field length")?;
	*cursor = end;
	Ok(value)
}

fn read_u16(data: &[u8], cursor: &mut usize) -> Result<u16, &'static str> {
	Ok(u16::from_le_bytes(take_array::<2>(data, cursor)?))
}

fn read_u32(data: &[u8], cursor: &mut usize) -> Result<u32, &'static str> {
	Ok(u32::from_le_bytes(take_array::<4>(data, cursor)?))
}

/// Parsed fields from the quote v3 header.
#[derive(Debug, Clone)]
pub struct QuoteHeader {
	pub version: u16,
	pub attestation_key_type: u16,
	pub tee_type: u32,
	pub qe_svn: u16,
	pub pce_svn: u16,
	pub qe_vendor_id: [u8; 16],
	pub user_data: [u8; 20],
	/// XFRM from the report body's ATTRIBUTES field (retained here for API compatibility).
	pub xfrm: u64,
	/// Compatibility projection of caller-defined quote-header user data.
	pub dynamic_data: DynamicDataFields,
}

/// Parsed SGX report body.
#[derive(Debug, Clone)]
pub struct QuoteBody {
	/// Complete raw 384-byte `sgx_report_body_t`.
	pub report_data: [u8; REPORT_BODY_LEN],
	pub isv_product_id: u16,
	pub isvsvn: u16,
}

/// Parsed `sgx_ql_ecdsa_sig_data_t` fields.
#[derive(Debug, Clone)]
pub struct QuoteSignature {
	/// Quote signature as fixed-width, big-endian `r || s` integers.
	pub epid_or_ecdsa_sig: [u8; QUOTE_SIGNATURE_LEN],
	/// PCK certificate-chain certification data (type 5, normally PEM).
	pub certificate_chain: Vec<u8>,
}

/// Compatibility view over caller-defined quote-header user data.
#[derive(Debug, Clone)]
pub struct DynamicDataFields {
	pub nonce: [u8; 16],
	pub timestamp: [u8; 8],
}

/// Serialized compatibility representation for logging/monitoring.
///
/// Conversion from [`SgxQuote`] does not perform freshness or replay checks.
#[derive(Debug, Serialize, Deserialize)]
pub struct QuoteSummary {
	/// Parsed quote version.
	pub version: u16,
	/// Hex-encoded enclave measurement.
	pub mrenclave_hex: String,
	/// Hex-encoded signer measurement.
	pub mr_signer_hex: String,
	/// Compatibility placeholder; currently always `None`.
	pub age_seconds: Option<u64>,
	/// Compatibility placeholder; currently always `false`.
	pub nonce_seen_before: bool,
}

impl From<&SgxQuote> for QuoteSummary {
	fn from(quote: &SgxQuote) -> Self {
		Self {
			version: quote.header.version,
			mrenclave_hex: hex::encode(quote.mrenclave()),
			mr_signer_hex: hex::encode(quote.mr_signer()),
			age_seconds: None,
			nonce_seen_before: false,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn valid_quote() -> Vec<u8> {
		let cert_chain = b"not-a-real-chain";
		let authentication_data = [0xA5, 0x5A, 0x11];
		let signature_len =
			ECDSA_SIGNATURE_DATA_FIXED_LEN + authentication_data.len() + cert_chain.len();
		let mut quote = vec![0u8; QUOTE_FIXED_LEN + signature_len];

		quote[0..2].copy_from_slice(&3u16.to_le_bytes());
		quote[2..4].copy_from_slice(&ECDSA_P256_ATTESTATION_KEY_TYPE.to_le_bytes());
		quote[4..8].copy_from_slice(&SGX_TEE_TYPE.to_le_bytes());
		quote[8..10].copy_from_slice(&7u16.to_le_bytes());
		quote[10..12].copy_from_slice(&9u16.to_le_bytes());
		quote[12..28].fill(0xCC);
		quote[28..48].fill(0xDD);

		let body = &mut quote[QUOTE_HEADER_LEN..SIGNATURE_DATA_LEN_OFFSET];
		body[56..64].copy_from_slice(&0x1122_3344_5566_7788u64.to_le_bytes());
		body[64..96].fill(0x11);
		body[128..160].fill(0x22);
		body[256..258].copy_from_slice(&42u16.to_le_bytes());
		body[258..260].copy_from_slice(&17u16.to_le_bytes());
		body[320..384].fill(0x33);

		quote[SIGNATURE_DATA_LEN_OFFSET..QUOTE_FIXED_LEN]
			.copy_from_slice(&(signature_len as u32).to_le_bytes());
		let mut cursor = QUOTE_FIXED_LEN
			+ QUOTE_SIGNATURE_LEN
			+ ATTESTATION_PUBLIC_KEY_LEN
			+ QE_REPORT_BODY_LEN
			+ QE_REPORT_SIGNATURE_LEN;
		quote[cursor..cursor + 2]
			.copy_from_slice(&(authentication_data.len() as u16).to_le_bytes());
		cursor += 2;
		quote[cursor..cursor + authentication_data.len()].copy_from_slice(&authentication_data);
		cursor += authentication_data.len();
		quote[cursor..cursor + 2].copy_from_slice(&PCK_CERT_CHAIN_DATA_TYPE.to_le_bytes());
		cursor += 2;
		quote[cursor..cursor + 4].copy_from_slice(&(cert_chain.len() as u32).to_le_bytes());
		cursor += 4;
		quote[cursor..].copy_from_slice(cert_chain);

		quote
	}

	#[test]
	fn parses_quote_v3_at_specified_offsets_without_panicking() {
		let quote = SgxQuote::parse(&valid_quote()).expect("valid quote must parse");

		assert_eq!(quote.header.version, 3);
		assert_eq!(quote.header.attestation_key_type, 2);
		assert_eq!(quote.header.qe_svn, 7);
		assert_eq!(quote.header.pce_svn, 9);
		assert_eq!(quote.header.xfrm, 0x1122_3344_5566_7788);
		assert_eq!(quote.mrenclave(), [0x11; 32]);
		assert_eq!(quote.mr_signer(), [0x22; 32]);
		assert_eq!(quote.body.isv_product_id, 42);
		assert_eq!(quote.body.isvsvn, 17);
		assert_eq!(quote.enclave_report_data(), [0x33; 64]);
		assert_eq!(quote.signature.certificate_chain, b"not-a-real-chain");
	}

	#[test]
	fn rejects_truncated_and_trailing_signature_data() {
		let quote = valid_quote();
		assert_eq!(
			SgxQuote::parse(&quote[..quote.len() - 1]).unwrap_err(),
			"Quote signature data is truncated"
		);

		let mut trailing = quote;
		trailing.push(0);
		assert_eq!(SgxQuote::parse(&trailing).unwrap_err(), "Quote contains trailing data");
	}

	#[test]
	fn rejects_inner_certification_length_mismatch() {
		let mut quote = valid_quote();
		let cert_len_offset = quote.len() - b"not-a-real-chain".len() - 4;
		let declared_len = (b"not-a-real-chain".len() + 1) as u32;
		quote[cert_len_offset..cert_len_offset + 4].copy_from_slice(&declared_len.to_le_bytes());

		assert_eq!(SgxQuote::parse(&quote).unwrap_err(), "Certification data is truncated");
	}

	#[test]
	fn rejects_oversized_certification_length_before_copying() {
		let mut quote = valid_quote();
		let cert_len_offset = quote.len() - b"not-a-real-chain".len() - 4;
		quote[cert_len_offset..cert_len_offset + 4]
			.copy_from_slice(&((MAX_CERTIFICATION_DATA_LEN + 1) as u32).to_le_bytes());

		assert_eq!(
			SgxQuote::parse(&quote).unwrap_err(),
			"PCK certificate chain exceeds the 1 MiB limit"
		);
	}
}
