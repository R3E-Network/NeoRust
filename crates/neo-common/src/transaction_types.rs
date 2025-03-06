//! Transaction types for the NeoRust SDK.
//!
//! This module provides types for working with transactions in the Neo blockchain.

use serde::{Deserialize, Serialize};

/// Transaction type in the Neo blockchain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum TransactionType {
    /// Miner transaction
    MinerTransaction = 0x00,
    /// Issue transaction
    IssueTransaction = 0x01,
    /// Claim transaction
    ClaimTransaction = 0x02,
    /// Enrollment transaction
    EnrollmentTransaction = 0x20,
    /// Register transaction
    RegisterTransaction = 0x40,
    /// Contract transaction
    ContractTransaction = 0x80,
    /// State transaction
    StateTransaction = 0x90,
    /// Publish transaction
    PublishTransaction = 0xd0,
    /// Invocation transaction
    InvocationTransaction = 0xd1,
}

impl TransactionType {
    /// Convert a u8 value to a TransactionType
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(TransactionType::MinerTransaction),
            0x01 => Some(TransactionType::IssueTransaction),
            0x02 => Some(TransactionType::ClaimTransaction),
            0x20 => Some(TransactionType::EnrollmentTransaction),
            0x40 => Some(TransactionType::RegisterTransaction),
            0x80 => Some(TransactionType::ContractTransaction),
            0x90 => Some(TransactionType::StateTransaction),
            0xd0 => Some(TransactionType::PublishTransaction),
            0xd1 => Some(TransactionType::InvocationTransaction),
            _ => None,
        }
    }

    /// Get the u8 value of the TransactionType
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
}
