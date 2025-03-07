//! Transaction attribute types for the NeoRust SDK.
//!
//! This module provides types for working with transaction attributes in the Neo blockchain.

use serde::{Deserialize, Serialize};

/// Transaction attribute type in the Neo blockchain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum TransactionAttributeType {
    /// High priority attribute
    HighPriority = 0x01,
    /// Oracle response attribute
    OracleResponse = 0x11,
    /// Not valid before attribute
    NotValidBefore = 0x20,
    /// Conflicts attribute
    Conflicts = 0x21,
}

impl TransactionAttributeType {
    /// Convert a u8 value to a TransactionAttributeType
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x01 => Some(TransactionAttributeType::HighPriority),
            0x11 => Some(TransactionAttributeType::OracleResponse),
            0x20 => Some(TransactionAttributeType::NotValidBefore),
            0x21 => Some(TransactionAttributeType::Conflicts),
            _ => None,
        }
    }

    /// Get the u8 value of the TransactionAttributeType
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
}

/// Transaction attribute in the Neo blockchain
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransactionAttribute {
    /// The type of the attribute
    pub attribute_type: TransactionAttributeType,
    /// The data of the attribute
    pub data: Vec<u8>,
}

impl TransactionAttribute {
    /// Create a new transaction attribute
    pub fn new(attribute_type: TransactionAttributeType, data: Vec<u8>) -> Self {
        Self {
            attribute_type,
            data,
        }
    }

    /// Create a high priority transaction attribute
    pub fn high_priority() -> Self {
        Self {
            attribute_type: TransactionAttributeType::HighPriority,
            data: Vec::new(),
        }
    }

    /// Create a not valid before transaction attribute
    pub fn not_valid_before(block_height: u32) -> Self {
        let data = block_height.to_le_bytes().to_vec();
        Self {
            attribute_type: TransactionAttributeType::NotValidBefore,
            data,
        }
    }
}
