//! Transaction attributes

use crate::{
	errors::{Error, Result},
	serialization::{Reader, Serializable, Writer},
};
use serde::{Deserialize, Serialize};

/// Optional metadata attached to a transaction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TransactionAttribute {
	/// Marks the transaction as high priority, letting it bypass the normal
	/// mempool fee ordering. Only committee members may use it.
	HighPriority,

	/// Response payload for an oracle request.
	OracleResponse {
		/// Id of the request being answered.
		id: u64,
		/// Result code for the request.
		code: OracleResponseCode,
		/// Response body; empty unless `code` is `Success`.
		result: alloc::vec::Vec<u8>,
	},

	/// Declares the transaction may only be validated after `height`.
	NotValidBefore {
		/// First block height at which the transaction becomes valid.
		height: u32,
	},

	/// Declares this transaction conflicts with another, so only one of them
	/// can be accepted.
	Conflicts {
		/// Hash of the conflicting transaction.
		hash: neo_primitives::Hash256,
	},
}

/// Outcome of an oracle request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum OracleResponseCode {
	/// Request succeeded; `result` holds the payload.
	Success = 0x00,
	/// The requested protocol is not supported.
	ProtocolNotSupported = 0x10,
	/// Could not reach consensus on the response.
	ConsensusUnreachable = 0x12,
	/// The requested resource does not exist.
	NotFound = 0x14,
	/// The request timed out.
	Timeout = 0x16,
	/// Access to the resource was denied.
	Forbidden = 0x18,
	/// The response exceeded the size limit.
	ResponseTooLarge = 0x1a,
	/// Insufficient funds to pay for the response.
	InsufficientFunds = 0x1c,
	/// The content type is not supported.
	ContentTypeNotSupported = 0x1f,
	/// The request failed for an unspecified reason.
	Error = 0xff,
}

impl OracleResponseCode {
	/// The wire byte for this code.
	pub const fn as_byte(self) -> u8 {
		self as u8
	}

	/// Parses a wire byte.
	pub fn from_byte(byte: u8) -> Result<Self> {
		match byte {
			0x00 => Ok(Self::Success),
			0x10 => Ok(Self::ProtocolNotSupported),
			0x12 => Ok(Self::ConsensusUnreachable),
			0x14 => Ok(Self::NotFound),
			0x16 => Ok(Self::Timeout),
			0x18 => Ok(Self::Forbidden),
			0x1a => Ok(Self::ResponseTooLarge),
			0x1c => Ok(Self::InsufficientFunds),
			0x1f => Ok(Self::ContentTypeNotSupported),
			0xff => Ok(Self::Error),
			other => Err(Error::UnknownVariant { kind: "OracleResponseCode", value: other }),
		}
	}
}

/// Maximum size of an oracle response payload, in bytes.
pub const MAX_ORACLE_RESULT_SIZE: usize = 0xffff;

impl TransactionAttribute {
	/// The attribute's type byte.
	pub const fn type_byte(&self) -> u8 {
		match self {
			Self::HighPriority => 0x01,
			Self::OracleResponse { .. } => 0x11,
			Self::NotValidBefore { .. } => 0x20,
			Self::Conflicts { .. } => 0x21,
		}
	}
}

impl Serializable for TransactionAttribute {
	fn serialize(&self, writer: &mut Writer) {
		writer.write_u8(self.type_byte());

		match self {
			Self::HighPriority => {},
			Self::OracleResponse { id, code, result } => {
				writer.write_u64(*id);
				writer.write_u8(code.as_byte());
				writer.write_var_bytes(result);
			},
			Self::NotValidBefore { height } => writer.write_u32(*height),
			Self::Conflicts { hash } => writer.write_bytes(hash.as_bytes()),
		}
	}

	fn deserialize(reader: &mut Reader<'_>) -> Result<Self> {
		let type_byte = reader.read_u8()?;

		match type_byte {
			0x01 => Ok(Self::HighPriority),
			0x11 => {
				let id = reader.read_u64()?;
				let code = OracleResponseCode::from_byte(reader.read_u8()?)?;
				let result = reader.read_var_bytes(MAX_ORACLE_RESULT_SIZE)?.to_vec();

				// A non-success code carries no payload; a body alongside one
				// would be ambiguous, so reject it rather than silently drop it.
				if code != OracleResponseCode::Success && !result.is_empty() {
					return Err(Error::InvalidData(alloc::format!(
						"oracle response with code {:?} must have an empty result",
						code
					)));
				}

				Ok(Self::OracleResponse { id, code, result })
			},
			0x20 => Ok(Self::NotValidBefore { height: reader.read_u32()? }),
			0x21 => Ok(Self::Conflicts {
				hash: neo_primitives::Hash256::from_slice(reader.read_bytes(32)?)?,
			}),
			other => Err(Error::UnknownVariant { kind: "TransactionAttribute", value: other }),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use alloc::vec;
	use neo_primitives::Hash256;

	#[test]
	fn test_high_priority_roundtrip() {
		let attr = TransactionAttribute::HighPriority;
		assert_eq!(attr.to_bytes(), vec![0x01]);
		assert_eq!(TransactionAttribute::from_bytes(&[0x01]).unwrap(), attr);
	}

	#[test]
	fn test_not_valid_before_roundtrip() {
		let attr = TransactionAttribute::NotValidBefore { height: 1000 };
		let bytes = attr.to_bytes();
		assert_eq!(bytes[0], 0x20);
		// Height is little-endian
		assert_eq!(&bytes[1..], &1000u32.to_le_bytes());
		assert_eq!(TransactionAttribute::from_bytes(&bytes).unwrap(), attr);
	}

	#[test]
	fn test_conflicts_roundtrip() {
		let hash =
			Hash256::from_hex("0x0000000000000000000000000000000000000000000000000000000000000042")
				.unwrap();
		let attr = TransactionAttribute::Conflicts { hash };
		let bytes = attr.to_bytes();

		assert_eq!(bytes[0], 0x21);
		assert_eq!(bytes.len(), 33);
		assert_eq!(TransactionAttribute::from_bytes(&bytes).unwrap(), attr);
	}

	#[test]
	fn test_oracle_response_success_roundtrip() {
		let attr = TransactionAttribute::OracleResponse {
			id: 7,
			code: OracleResponseCode::Success,
			result: b"{\"price\":42}".to_vec(),
		};
		let bytes = attr.to_bytes();
		assert_eq!(bytes[0], 0x11);
		assert_eq!(TransactionAttribute::from_bytes(&bytes).unwrap(), attr);
	}

	#[test]
	fn test_oracle_response_error_has_empty_result() {
		let attr = TransactionAttribute::OracleResponse {
			id: 9,
			code: OracleResponseCode::NotFound,
			result: vec![],
		};
		let bytes = attr.to_bytes();
		assert_eq!(TransactionAttribute::from_bytes(&bytes).unwrap(), attr);
	}

	#[test]
	fn test_oracle_response_rejects_payload_with_error_code() {
		// Hand-build: NotFound code carrying a body.
		let mut writer = Writer::new();
		writer.write_u8(0x11);
		writer.write_u64(1);
		writer.write_u8(OracleResponseCode::NotFound.as_byte());
		writer.write_var_bytes(b"unexpected");

		assert!(matches!(
			TransactionAttribute::from_bytes(writer.as_bytes()),
			Err(Error::InvalidData(_))
		));
	}

	#[test]
	fn test_rejects_unknown_attribute_type() {
		assert!(matches!(
			TransactionAttribute::from_bytes(&[0x99]),
			Err(Error::UnknownVariant { kind: "TransactionAttribute", value: 0x99 })
		));
	}

	#[test]
	fn test_rejects_unknown_oracle_code() {
		let mut writer = Writer::new();
		writer.write_u8(0x11);
		writer.write_u64(1);
		writer.write_u8(0x77);
		writer.write_var_bytes(&[]);

		assert!(matches!(
			TransactionAttribute::from_bytes(writer.as_bytes()),
			Err(Error::UnknownVariant { kind: "OracleResponseCode", value: 0x77 })
		));
	}

	#[test]
	fn test_oracle_code_byte_values() {
		assert_eq!(OracleResponseCode::Success.as_byte(), 0x00);
		assert_eq!(OracleResponseCode::NotFound.as_byte(), 0x14);
		assert_eq!(OracleResponseCode::Error.as_byte(), 0xff);
	}

	#[test]
	fn test_size_matches_encoding() {
		let attr = TransactionAttribute::NotValidBefore { height: 5 };
		assert_eq!(attr.size(), attr.to_bytes().len());
		assert_eq!(attr.size(), 5);
	}

	#[test]
	fn test_serde_roundtrip() {
		let attr = TransactionAttribute::NotValidBefore { height: 123 };
		let json = serde_json::to_string(&attr).unwrap();
		let back: TransactionAttribute = serde_json::from_str(&json).unwrap();
		assert_eq!(attr, back);
	}
}
