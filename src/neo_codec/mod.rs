//! # Neo Codec
//!
//! Encoding and decoding utilities for Neo N3 blockchain data.
//!
//! ## Overview
//!
//! The neo_codec module provides serialization and deserialization functionality for
//! Neo N3 blockchain data structures. It includes:
//!
//! - Binary encoding and decoding of blockchain structures
//! - Error handling for encoding/decoding operations
//! - Serialization format conversions
//!
//! ## Feature Flags
//!
//! This module supports the following feature flags:
//!
//! - **std**: Core encoding/decoding functionality (always available)
//! - **serde**: JSON serialization and deserialization support
//! - **binary-format**: Binary format encoding/decoding utilities
//! - **transaction**: Enhanced encoding for transaction types
//!
//! ## Examples
//!
//! ### Binary encoding and decoding
//!
//! ```rust
//! use neo::prelude::*;
//!
//! // Create a value to encode
//! let value = 12345u32;
//!
//! // Encode to binary
//! let mut encoder = BinaryEncoder::new();
//! encoder.write_u32(value);
//! let encoded_data = encoder.to_bytes();
//!
//! // Decode from binary
//! let mut decoder = BinaryDecoder::from_bytes(&encoded_data);
//! let decoded_value = decoder.read_u32().unwrap();
//!
//! assert_eq!(value, decoded_value);
//! ```

// Core codec functionality - always available
pub use encode::*;
pub use error::*;

// Binary encoding/decoding - available with binary-format feature or transaction feature
#[cfg(any(feature = "binary-format", feature = "transaction"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "binary-format", feature = "transaction"))))]
pub use binary_decoder::*;

#[cfg(any(feature = "binary-format", feature = "transaction"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "binary-format", feature = "transaction"))))]
pub use binary_encoder::*;

// JSON encoding/decoding - available with serde feature
#[cfg(feature = "serde")]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
pub use json_codec::*;

// Core modules - always available
mod encode;
mod error;

// Binary encoding/decoding modules - conditional on binary-format or transaction feature
#[cfg(any(feature = "binary-format", feature = "transaction"))]
mod binary_decoder;

#[cfg(any(feature = "binary-format", feature = "transaction"))]
mod binary_encoder;

// JSON encoding/decoding - conditional on serde feature
#[cfg(feature = "serde")]
mod json_codec;

// Internal utility function for testing
#[doc(hidden)]
pub(crate) fn add(left: usize, right: usize) -> usize {
	left + right
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn it_works() {
		let result = add(2, 2);
		assert_eq!(result, 4);
	}
}
