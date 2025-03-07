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
    /// Witness rules scope
    WitnessRules = 64,
    /// Global scope
    Global = 128,
}

/// Alias for WitnessScope to maintain compatibility
pub type WitnessScopeType = WitnessScope;

impl WitnessScope {
    /// Convert a u8 value to a WitnessScope
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(WitnessScope::None),
            1 => Some(WitnessScope::CalledByEntry),
            16 => Some(WitnessScope::CustomContracts),
            32 => Some(WitnessScope::CustomGroups),
            64 => Some(WitnessScope::WitnessRules),
            128 => Some(WitnessScope::Global),
            _ => None,
        }
    }

    /// Get the u8 value of the WitnessScope
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
    
    /// Combine multiple witness scopes into a single byte
    pub fn combine(scopes: &[WitnessScope]) -> u8 {
        let mut result = 0u8;
        for scope in scopes {
            result |= scope.as_u8();
        }
        result
    }
    
    /// Split a byte into multiple witness scopes
    pub fn split(byte: u8) -> Vec<WitnessScope> {
        let mut result = Vec::new();
        
        if byte & WitnessScope::None.as_u8() != 0 {
            result.push(WitnessScope::None);
        }
        if byte & WitnessScope::CalledByEntry.as_u8() != 0 {
            result.push(WitnessScope::CalledByEntry);
        }
        if byte & WitnessScope::CustomContracts.as_u8() != 0 {
            result.push(WitnessScope::CustomContracts);
        }
        if byte & WitnessScope::CustomGroups.as_u8() != 0 {
            result.push(WitnessScope::CustomGroups);
        }
        if byte & WitnessScope::WitnessRules.as_u8() != 0 {
            result.push(WitnessScope::WitnessRules);
        }
        if byte & WitnessScope::Global.as_u8() != 0 {
            result.push(WitnessScope::Global);
        }
        
        result
    }
}

impl fmt::Display for WitnessScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WitnessScope::None => write!(f, "None"),
            WitnessScope::CalledByEntry => write!(f, "CalledByEntry"),
            WitnessScope::CustomContracts => write!(f, "CustomContracts"),
            WitnessScope::CustomGroups => write!(f, "CustomGroups"),
            WitnessScope::WitnessRules => write!(f, "WitnessRules"),
            WitnessScope::Global => write!(f, "Global"),
        }
    }
}
