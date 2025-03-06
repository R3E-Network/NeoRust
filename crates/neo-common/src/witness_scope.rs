//! Witness scope types for the NeoRust SDK.
//!
//! This module provides types for working with witness scopes in the Neo blockchain.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Witness scope in the Neo blockchain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum WitnessScope {
    /// None scope
    None = 0,
    /// Called by entry scope
    CalledByEntry = 1,
    /// Custom contracts scope
    CustomContracts = 16,
    /// Custom groups scope
    CustomGroups = 32,
    /// Global scope
    Global = 128,
}

impl WitnessScope {
    /// Convert a u8 value to a WitnessScope
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

    /// Get the u8 value of the WitnessScope
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
}

impl fmt::Display for WitnessScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WitnessScope::None => write!(f, "None"),
            WitnessScope::CalledByEntry => write!(f, "CalledByEntry"),
            WitnessScope::CustomContracts => write!(f, "CustomContracts"),
            WitnessScope::CustomGroups => write!(f, "CustomGroups"),
            WitnessScope::Global => write!(f, "Global"),
        }
    }
}
