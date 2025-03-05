//! Implementation of script hash operations
//!
//! This module provides implementation of script hash operations.

use crate::{ScriptHashExtension, TypeError};
use bs58;
use byte_slice_cast::AsByteSlice;
use primitive_types::H160;
use rustc_serialize::hex::ToHex;
use sha2::{Digest, Sha256};
use std::str::FromStr;

impl ScriptHashExtension for H160 {
    fn to_bs58_string(&self) -> String {
        let mut data = Vec::with_capacity(25);
        // Add address version byte
        data.push(0x35); // Default Neo address version
        // Add script hash bytes (in reverse order for Neo)
        data.extend_from_slice(&self.0);
        
        // Calculate checksum (SHA256 twice)
        let checksum = {
            let mut hasher = Sha256::new();
            hasher.update(&data);
            let first_hash = hasher.finalize();
            
            let mut hasher = Sha256::new();
            hasher.update(first_hash);
            let second_hash = hasher.finalize();
            
            second_hash[0..4].to_vec()
        };
        
        // Append checksum
        data.extend_from_slice(&checksum);
        
        // Encode with Base58
        bs58::encode(data).into_string()
    }

    fn zero() -> Self {
        let arr = [0u8; 20];
        Self(arr)
    }

    fn from_slice(slice: &[u8]) -> Result<Self, TypeError> {
        if slice.len() != 20 {
            return Err(TypeError::InvalidAddress);
        }

        let mut arr = [0u8; 20];
        arr.copy_from_slice(slice);
        Ok(Self(arr))
    }

    fn from_hex(hex: &str) -> Result<Self, hex::FromHexError> {
        if hex.starts_with("0x") {
            let mut bytes = hex::decode(&hex[2..])?;
            bytes.reverse();
            if bytes.len() != 20 {
                return Err(hex::FromHexError::InvalidStringLength);
            }
            let mut arr = [0u8; 20];
            arr.copy_from_slice(&bytes);
            Ok(Self(arr))
        } else {
            let bytes = hex::decode(hex)?;
            if bytes.len() != 20 {
                return Err(hex::FromHexError::InvalidStringLength);
            }
            let mut arr = [0u8; 20];
            arr.copy_from_slice(&bytes);
            Ok(Self(arr))
        }
    }

    fn from_address(address: &str) -> Result<Self, TypeError> {
        let bytes = match bs58::decode(address).into_vec() {
            Ok(bytes) => bytes,
            Err(_) => return Err(TypeError::InvalidAddress),
        };

        if bytes.len() != 25 {
            return Err(TypeError::InvalidAddress);
        }

        let _salt = bytes[0];
        let hash = &bytes[1..21];
        
        // Calculate checksum
        let mut checksum_data = Vec::with_capacity(21);
        checksum_data.push(bytes[0]);
        checksum_data.extend_from_slice(hash);
        
        let checksum = &bytes[21..25];
        let check = {
            let mut hasher = Sha256::new();
            hasher.update(&checksum_data);
            let first_hash = hasher.finalize();
            
            let mut hasher = Sha256::new();
            hasher.update(first_hash);
            let second_hash = hasher.finalize();
            
            second_hash[0..4].to_vec()
        };
        
        if checksum != check {
            return Err(TypeError::InvalidAddress);
        }

        let mut rev = [0u8; 20];
        rev.clone_from_slice(hash);
        rev.reverse();
        Ok(Self::from_slice(&rev))
    }

    fn to_address(&self) -> String {
        let mut data = vec![0x35]; // Default Neo address version
        let mut reversed_bytes = self.as_bytes().to_vec();
        reversed_bytes.reverse();
        data.extend_from_slice(&reversed_bytes);
        
        // Calculate checksum (SHA256 twice)
        let checksum = {
            let mut hasher = Sha256::new();
            hasher.update(&data);
            let first_hash = hasher.finalize();
            
            let mut hasher = Sha256::new();
            hasher.update(first_hash);
            let second_hash = hasher.finalize();
            
            second_hash[0..4].to_vec()
        };
        
        data.extend_from_slice(&checksum);
        bs58::encode(data).into_string()
    }

    fn to_hex(&self) -> String {
        self.0.to_hex()
    }

    fn to_hex_big_endian(&self) -> String {
        let mut cloned = self.0.clone();
        cloned.reverse();
        "0x".to_string() + &cloned.to_hex()
    }

    fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    fn to_le_vec(&self) -> Vec<u8> {
        let vec = self.0.to_vec();
        vec
    }

    fn from_script(script: &[u8]) -> Self {
        // Calculate SHA256 hash
        let mut hasher = Sha256::new();
        hasher.update(script);
        let sha256_result = hasher.finalize();
        
        // Calculate RIPEMD160 hash
        let mut hasher = ripemd::Ripemd160::new();
        hasher.update(sha256_result);
        let ripemd_result = hasher.finalize();
        
        // Convert to H160
        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(&ripemd_result);
        bytes.reverse(); // Reverse for Neo format
        H160(bytes)
    }

    fn from_public_key(public_key: &[u8]) -> Result<Self, TypeError> {
        if public_key.len() != 33 && public_key.len() != 65 {
            return Err(TypeError::InvalidAddress);
        }
        
        // Calculate SHA256 hash
        let mut hasher = Sha256::new();
        hasher.update(public_key);
        let sha256_result = hasher.finalize();
        
        // Calculate RIPEMD160 hash
        let mut hasher = ripemd::Ripemd160::new();
        hasher.update(sha256_result);
        let ripemd_result = hasher.finalize();
        
        // Convert to H160
        let mut bytes = [0u8; 20];
        bytes.copy_from_slice(&ripemd_result);
        bytes.reverse(); // Reverse for Neo format
        Ok(H160(bytes))
    }
}
