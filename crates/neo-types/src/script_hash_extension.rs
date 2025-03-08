//! Extension traits for script hash operations
//!
//! This module provides extension traits for script hash operations.

use primitive_types::H160;
use std::str::FromStr;
use neo_error::TypeError;
use neo_common::HashableForVec;
use hex::{FromHexError};

/// Extension trait for script hash operations
pub trait ScriptHashExtension {
    /// Converts the script hash to a Base58 string
    fn to_bs58_string(&self) -> String;

    /// Creates a zero-value script hash
    fn zero() -> Self;

    /// Creates a script hash from a byte slice
    fn from_slice(slice: &[u8]) -> Result<Self, TypeError>
    where
        Self: Sized;

    /// Creates a script hash from a hex string
    fn from_hex(hex: &str) -> Result<Self, FromHexError>
    where
        Self: Sized;

    /// Creates a script hash from an address string
    fn from_address(address: &str) -> Result<Self, TypeError>
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
    fn from_public_key(public_key: &[u8]) -> Result<Self, TypeError>
    where
        Self: Sized;
}

impl ScriptHashExtension for H160 {
    fn to_bs58_string(&self) -> String {
        bs58::encode(self.as_bytes()).into_string()
    }

    fn zero() -> Self {
        H160::zero()
    }

    fn from_slice(slice: &[u8]) -> Result<Self, TypeError> {
        if slice.len() != 20 {
            return Err(TypeError::InvalidArgError(String::from("Script hash must be 20 bytes")));
        }
        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(slice);
        Ok(H160::from(bytes))
    }

    fn from_hex(hex: &str) -> Result<Self, FromHexError> {
        let bytes = hex::decode(hex.trim_start_matches("0x"))?;
        if bytes.len() != 20 {
            return Err(FromHexError::InvalidStringLength);
        }
        let mut arr = [0u8; 20];
        arr.copy_from_slice(&bytes);
        Ok(H160::from(arr))
    }

    fn from_address(address: &str) -> Result<Self, TypeError> {
        let decoded = bs58::decode(address).into_vec().map_err(|_| 
            TypeError::InvalidArgError(String::from("Failed to decode base58 address")))?;
        if decoded.len() != 25 {
            return Err(TypeError::InvalidArgError(String::from("Invalid address length")));
        }
        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(&decoded[1..21]);
        Ok(H160::from(bytes))
    }

    fn to_address(&self) -> String {
        let mut data = vec![0x35]; // Address version
        data.extend_from_slice(self.as_bytes());
        // Simplified checksum (real implementation would add proper checksum)
        bs58::encode(data).into_string()
    }

    fn to_hex(&self) -> String {
        hex::encode(self.as_bytes())
    }

    fn to_hex_big_endian(&self) -> String {
        let mut bytes = self.as_bytes().to_vec();
        bytes.reverse();
        hex::encode(bytes)
    }

    fn to_vec(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }

    fn to_le_vec(&self) -> Vec<u8> {
        let mut bytes = self.as_bytes().to_vec();
        bytes.reverse();
        bytes
    }

    fn from_script(script: &[u8]) -> Self {
        // Create from script data
        let hash = script.to_vec().hash160();
        
        // Create a new H160 from the bytes
        let mut arr = [0u8; 20];
        if hash.len() == 20 {
            arr.copy_from_slice(&hash);
            H160::from(arr)
        } else {
            Self::zero()
        }
    }

    fn from_public_key(public_key: &[u8]) -> Result<Self, TypeError> {
        // Create from public key data
        let hash = public_key.to_vec().hash160();
        
        // Create a new H160 from the bytes
        if hash.len() != 20 {
            return Err(TypeError::InvalidArgError(String::from("Invalid hash length")));
        }
        
        let mut arr = [0u8; 20];
        arr.copy_from_slice(&hash);
        Ok(H160::from(arr))
    }
}
