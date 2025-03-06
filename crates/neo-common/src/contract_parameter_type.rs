//! Contract parameter types for Neo blockchain
//!
//! This module provides types for contract parameters in the Neo blockchain.

use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

/// Contract parameter types in the Neo blockchain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[repr(u8)]
pub enum ContractParameterType {
    /// Any type
    #[strum(serialize = "any")]
    Any = 0x00,
    
    /// Boolean type
    #[strum(serialize = "boolean")]
    Boolean = 0x10,
    
    /// Integer type
    #[strum(serialize = "integer")]
    Integer = 0x11,
    
    /// ByteArray type
    #[strum(serialize = "byte_array")]
    ByteArray = 0x12,
    
    /// String type
    #[strum(serialize = "string")]
    String = 0x13,
    
    /// Hash160 type
    #[strum(serialize = "hash160")]
    Hash160 = 0x14,
    
    /// Hash256 type
    #[strum(serialize = "hash256")]
    Hash256 = 0x15,
    
    /// PublicKey type
    #[strum(serialize = "public_key")]
    PublicKey = 0x16,
    
    /// Signature type
    #[strum(serialize = "signature")]
    Signature = 0x17,
    
    /// Array type
    #[strum(serialize = "array")]
    Array = 0x20,
    
    /// Map type
    #[strum(serialize = "map")]
    Map = 0x22,
    
    /// InteropInterface type
    #[strum(serialize = "interop_interface")]
    InteropInterface = 0x30,
    
    /// Void type
    #[strum(serialize = "void")]
    Void = 0xff,
}

impl ContractParameterType {
    /// Get the parameter type as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            ContractParameterType::Any => "any",
            ContractParameterType::Boolean => "boolean",
            ContractParameterType::Integer => "integer",
            ContractParameterType::ByteArray => "byte_array",
            ContractParameterType::String => "string",
            ContractParameterType::Hash160 => "hash160",
            ContractParameterType::Hash256 => "hash256",
            ContractParameterType::PublicKey => "public_key",
            ContractParameterType::Signature => "signature",
            ContractParameterType::Array => "array",
            ContractParameterType::Map => "map",
            ContractParameterType::InteropInterface => "interop_interface",
            ContractParameterType::Void => "void",
        }
    }
    
    /// Get the parameter type as a u8 value
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
    
    /// Create a parameter type from a u8 value
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(ContractParameterType::Any),
            0x10 => Some(ContractParameterType::Boolean),
            0x11 => Some(ContractParameterType::Integer),
            0x12 => Some(ContractParameterType::ByteArray),
            0x13 => Some(ContractParameterType::String),
            0x14 => Some(ContractParameterType::Hash160),
            0x15 => Some(ContractParameterType::Hash256),
            0x16 => Some(ContractParameterType::PublicKey),
            0x17 => Some(ContractParameterType::Signature),
            0x20 => Some(ContractParameterType::Array),
            0x22 => Some(ContractParameterType::Map),
            0x30 => Some(ContractParameterType::InteropInterface),
            0xff => Some(ContractParameterType::Void),
            _ => None,
        }
    }
}

// ToString is automatically implemented for types that implement Display
// No need for a manual implementation
