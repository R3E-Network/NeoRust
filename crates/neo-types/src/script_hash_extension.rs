//! Extension traits for script hash operations
//!
//! This module provides extension traits for script hash operations.

use primitive_types::H160;
use std::str::FromStr;

/// Extension trait for script hash operations
pub trait ScriptHashExtension {
    /// Converts the script hash to a Base58 string
    fn to_bs58_string(&self) -> String;

    /// Creates a zero-value script hash
    fn zero() -> Self;

    /// Creates a script hash from a byte slice
    fn from_slice(slice: &[u8]) -> Result<Self, crate::TypeError>
    where
        Self: Sized;

    /// Creates a script hash from a hex string
    fn from_hex(hex: &str) -> Result<Self, hex::FromHexError>
    where
        Self: Sized;

    /// Creates a script hash from an address string
    fn from_address(address: &str) -> Result<Self, crate::TypeError>
    where
        Self: Sized;

    /// Converts the script hash to an address string
    fn to_address(&self) -> String;

    /// Converts the script hash to a hex string
    fn to_hex(&self) -> String;

    /// Converts the script hash to a big-endian hex string
    fn to_hex_big_endian(&self) -> String;

    /// Converts the script hash to a byte vector
    fn to_vec(&self) -> Vec<u8>;

    /// Converts the script hash to a little-endian byte vector
    fn to_le_vec(&self) -> Vec<u8>;

    /// Creates a script hash from a script byte slice
    fn from_script(script: &[u8]) -> Self;

    /// Creates a script hash from a public key
    fn from_public_key(public_key: &[u8]) -> Result<Self, crate::TypeError>
    where
        Self: Sized;
}
