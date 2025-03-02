//! Neo Types - Core types for the Neo blockchain
//!
//! This module contains the core types used in the Neo blockchain.

// Core types - always available
pub mod bytes;
pub mod error;
pub mod script_hash;

// Re-exports
pub use bytes::Bytes;
pub use error::TypeError;
pub use script_hash::ScriptHash;

// Conditional modules
#[cfg(feature = "utils")]
pub mod address;

#[cfg(feature = "utils")]
pub mod address_or_scripthash;

#[cfg(feature = "contract")]
pub mod contract;

#[cfg(feature = "serde-support")]
pub mod serde_with_utils;

#[cfg(feature = "string-ext")]
pub mod string;

#[cfg(feature = "utils")]
pub mod script_hash_extension;

// Alias for H160
pub type H160 = primitive_types::H160;
