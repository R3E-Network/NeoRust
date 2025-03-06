//! Oracle response code utilities
//!
//! This module provides utilities for working with oracle response codes.

use serde::{Deserialize, Serialize};
use std::hash::Hash;

/// Oracle response codes in the Neo blockchain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[repr(u8)]
pub enum OracleResponseCode {
    /// Success response code
    Success = 0x00,
    
    /// Protocol not supported response code
    ProtocolNotSupported = 0x10,
    
    /// Consensus unreachable response code
    ConsensusUnreachable = 0x12,
    
    /// Not found response code
    NotFound = 0x14,
    
    /// Timeout response code
    Timeout = 0x16,
    
    /// Forbidden response code
    Forbidden = 0x18,
    
    /// Response too large response code
    ResponseTooLarge = 0x1A,
    
    /// Insufficient funds response code
    InsufficientFunds = 0x1C,
    
    /// Content type not supported response code
    ContentTypeNotSupported = 0x1F,
    
    /// Error response code
    Error = 0xFF,
}

impl OracleResponseCode {
    /// Get the response code as a u8 value
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
    
    /// Create a response code from a u8 value
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(OracleResponseCode::Success),
            0x10 => Some(OracleResponseCode::ProtocolNotSupported),
            0x12 => Some(OracleResponseCode::ConsensusUnreachable),
            0x14 => Some(OracleResponseCode::NotFound),
            0x16 => Some(OracleResponseCode::Timeout),
            0x18 => Some(OracleResponseCode::Forbidden),
            0x1A => Some(OracleResponseCode::ResponseTooLarge),
            0x1C => Some(OracleResponseCode::InsufficientFunds),
            0x1F => Some(OracleResponseCode::ContentTypeNotSupported),
            0xFF => Some(OracleResponseCode::Error),
            _ => None,
        }
    }
}
