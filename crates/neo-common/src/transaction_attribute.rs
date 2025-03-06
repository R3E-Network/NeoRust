//! Transaction attribute utilities
//!
//! This module provides utilities for working with transaction attributes.

use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

/// Transaction attribute types in the Neo blockchain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[repr(u8)]
pub enum TransactionAttributeType {
    /// High priority attribute
    #[strum(serialize = "high_priority")]
    HighPriority = 0x01,
    
    /// Oracle response attribute
    #[strum(serialize = "oracle_response")]
    OracleResponse = 0x11,
    
    /// Not valid before attribute
    #[strum(serialize = "not_valid_before")]
    NotValidBefore = 0x20,
    
    /// Conflicts attribute
    #[strum(serialize = "conflicts")]
    Conflicts = 0x21,
}

impl TransactionAttributeType {
    /// Get the attribute type as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            TransactionAttributeType::HighPriority => "high_priority",
            TransactionAttributeType::OracleResponse => "oracle_response",
            TransactionAttributeType::NotValidBefore => "not_valid_before",
            TransactionAttributeType::Conflicts => "conflicts",
        }
    }
    
    /// Get the attribute type as a u8 value
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
    
    /// Create an attribute type from a u8 value
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(TransactionAttributeType::HighPriority),
            0x11 => Some(TransactionAttributeType::OracleResponse),
            0x20 => Some(TransactionAttributeType::NotValidBefore),
            0x21 => Some(TransactionAttributeType::Conflicts),
            _ => None,
        }
    }
}
