//! Error types for neo-protocol

use alloc::string::String;

/// Result type for protocol operations.
pub type Result<T> = core::result::Result<T, Error>;

/// Errors produced while encoding or decoding protocol types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
	/// Input ended before the required number of bytes was available.
	UnexpectedEof {
		/// Bytes the read required.
		needed: usize,
		/// Bytes actually available.
		available: usize,
	},
	/// A variable-length integer used a longer form than necessary.
	///
	/// Neo requires the shortest encoding; longer forms are rejected rather
	/// than normalized, because accepting them would let two different byte
	/// strings hash to the same transaction.
	NonCanonicalVarInt {
		/// Total byte length of the offending form (3, 5 or 9).
		form: u8,
		/// The value that was encoded.
		value: u64,
	},
	/// A length or item count exceeded the protocol limit.
	LengthLimitExceeded {
		/// Declared length.
		len: u64,
		/// Maximum allowed.
		max: usize,
	},
	/// Bytes remained after decoding a value that should consume all input.
	TrailingBytes {
		/// Number of unconsumed bytes.
		remaining: usize,
	},
	/// A field held a value the protocol does not allow.
	InvalidData(String),
	/// A discriminant byte did not match any known variant.
	UnknownVariant {
		/// Name of the enum being decoded.
		kind: &'static str,
		/// The unrecognized discriminant.
		value: u8,
	},
	/// An error surfaced from neo-primitives.
	Primitive(neo_primitives::Error),
}

impl core::fmt::Display for Error {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			Error::UnexpectedEof { needed, available } => {
				write!(
					f,
					"unexpected end of input: needed {} bytes, {} available",
					needed, available
				)
			},
			Error::NonCanonicalVarInt { form, value } => {
				write!(f, "non-canonical VarInt: {}-byte form used for value {:#x}", form, value)
			},
			Error::LengthLimitExceeded { len, max } => {
				write!(f, "length {} exceeds maximum {}", len, max)
			},
			Error::TrailingBytes { remaining } => {
				write!(f, "{} unexpected trailing bytes", remaining)
			},
			Error::InvalidData(msg) => write!(f, "invalid data: {}", msg),
			Error::UnknownVariant { kind, value } => {
				write!(f, "unknown {} discriminant: {:#x}", kind, value)
			},
			Error::Primitive(err) => write!(f, "{}", err),
		}
	}
}

impl From<neo_primitives::Error> for Error {
	fn from(err: neo_primitives::Error) -> Self {
		Error::Primitive(err)
	}
}

#[cfg(feature = "std")]
impl std::error::Error for Error {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match self {
			Error::Primitive(err) => Some(err),
			_ => None,
		}
	}
}
