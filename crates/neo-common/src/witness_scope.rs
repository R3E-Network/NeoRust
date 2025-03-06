//! Witness scope types for Neo blockchain
//!
//! This module provides types for witness scopes in the Neo blockchain.

use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

/// Witness scope flags for transaction verification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[repr(u8)]
pub enum WitnessScope {
    /// No scope
    #[strum(serialize = "none")]
    None = 0,

    /// Called by entry
    #[strum(serialize = "called_by_entry")]
    CalledByEntry = 1,

    /// Custom contracts
    #[strum(serialize = "custom_contracts")]
    CustomContracts = 16,

    /// Custom groups
    #[strum(serialize = "custom_groups")]
    CustomGroups = 32,

    /// Global scope
    #[strum(serialize = "global")]
    Global = 128,
}

impl WitnessScope {
    /// Get the scope as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            WitnessScope::None => "none",
            WitnessScope::CalledByEntry => "called_by_entry",
            WitnessScope::CustomContracts => "custom_contracts",
            WitnessScope::CustomGroups => "custom_groups",
            WitnessScope::Global => "global",
        }
    }

    /// Get the scope as a u8 value
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }

    /// Create a scope from a u8 value
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(WitnessScope::None),
            1 => Some(WitnessScope::CalledByEntry),
            16 => Some(WitnessScope::CustomContracts),
            32 => Some(WitnessScope::CustomGroups),
            128 => Some(WitnessScope::Global),
            _ => None,
        }
    }
}
