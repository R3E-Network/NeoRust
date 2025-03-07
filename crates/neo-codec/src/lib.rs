//! # Neo Codec
//!
//! Serialization and deserialization utilities for Neo N3 blockchain data.
//!
//! This crate provides encoding and decoding functionality for Neo N3 blockchain data structures, including:
//!
//! - Binary encoding and decoding
//! - Serialization traits for Neo data types
//! - Utility functions for data conversion
//! - Error handling for codec operations
//!
//! ## Usage
//!
//! ```rust,ignore
//! use neo_codec::{Decoder, Encoder, NeoSerializable};
//!
//! // Implement the NeoSerializable trait for your custom type
//! impl NeoSerializable for MyType {
//!     fn size(&self) -> usize {
//!         // Return the size in bytes
//!     }
//!
//!     fn encode(&self, encoder: &mut Encoder) -> Result<(), CodecError> {
//!         // Encode your type
//!         encoder.write_u32(self.value)?;
//!         Ok(())
//!     }
//!
//!     fn decode(decoder: &mut Decoder) -> Result<Self, CodecError> {
//!         // Decode your type
//!         let value = decoder.read_u32()?;
//!         Ok(Self { value })
//!     }
//! }
//!
//! // Encode a value
//! let mut encoder = Encoder::new();
//! my_value.encode(&mut encoder)?;
//! let encoded_bytes = encoder.to_bytes();
//!
//! // Decode a value
//! let mut decoder = Decoder::new(&encoded_bytes);
//! let decoded_value = MyType::decode(&mut decoder)?;
//! ```

#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms, unreachable_pub)]
#![deny(rustdoc::broken_intra_doc_links)]

mod binary_decoder;
mod binary_encoder;
mod constants;
mod encode;
mod error;
mod opcode;

// Re-export all public items
pub use binary_decoder::*;
pub use binary_encoder::*;
pub use constants::*;
pub use encode::*;
pub use error::*;
pub use opcode::*;
