//! # Neo Utils
//!
//! Utility functions and helpers for the NeoRust SDK.
//!
//! This crate provides various utility functions and helpers for working with the Neo N3 blockchain, including:
//!
//! - Conversion utilities
//! - Formatting helpers
//! - Validation functions
//! - Common patterns and abstractions
//! - Testing utilities
//! - Logging and debugging helpers
//!
//! ## Usage
//!
//! ```rust,ignore
//! use neo_utils::{format_neo_amount, validate_address, hex_to_bytes};
//! use neo_types::ScriptHash;
//! use std::str::FromStr;
//!
//! // Format Neo amounts with proper decimal places
//! let formatted = format_neo_amount(1000000000, 8);
//! assert_eq!(formatted, "10.00000000");
//!
//! // Validate Neo addresses
//! let is_valid = validate_address("NZNos2WqTbu5oCgyfss9kUJgBXJqhuYAaj");
//! assert!(is_valid);
//!
//! // Convert hex strings to byte arrays
//! let bytes = hex_to_bytes("0123456789abcdef").unwrap();
//! ```

#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms, unreachable_pub)]
#![deny(rustdoc::broken_intra_doc_links)]

mod error;

// Re-export all public items
pub use error::*;

// Utility functions
pub fn format_neo_amount(amount: u64, decimals: u8) -> String {
    let divisor = 10u64.pow(decimals as u32);
    let whole = amount / divisor;
    let fractional = amount % divisor;
    
    format!("{}.{:0width$}", whole, fractional, width = decimals as usize)
}

pub fn validate_address(address: &str) -> bool {
    // Simple validation - in a real implementation, this would do proper validation
    address.starts_with('N') && address.len() == 34
}

pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, hex::FromHexError> {
    hex::decode(hex)
}
