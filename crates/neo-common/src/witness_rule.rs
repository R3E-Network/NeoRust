//! Witness rule types for the NeoRust SDK.
//!
//! This module provides types for working with witness rules in the Neo blockchain.

use std::hash::{Hash, Hasher};
use serde::{Deserialize, Serialize};

/// Witness action in the Neo blockchain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum WitnessAction {
    /// Deny action
    Deny = 0,
    /// Allow action
    Allow = 1,
}

impl WitnessAction {
    /// Convert a u8 value to a WitnessAction
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(WitnessAction::Deny),
            1 => Some(WitnessAction::Allow),
            _ => None,
        }
    }

    /// Get the u8 value of the WitnessAction
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
}

/// Witness condition in the Neo blockchain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum WitnessCondition {
    /// Boolean condition
    Boolean = 0,
    /// Not condition
    Not = 1,
    /// And condition
    And = 2,
    /// Or condition
    Or = 3,
    /// Script hash condition
    ScriptHash = 4,
    /// Group condition
    Group = 5,
    /// Called by entry condition
    CalledByEntry = 6,
    /// Called by contract condition
    CalledByContract = 7,
    /// Called by group condition
    CalledByGroup = 8,
}

impl WitnessCondition {
    /// Boolean condition byte
    pub const BOOLEAN_BYTE: u8 = 0;
    /// Not condition byte
    pub const NOT_BYTE: u8 = 1;
    /// And condition byte
    pub const AND_BYTE: u8 = 2;
    /// Or condition byte
    pub const OR_BYTE: u8 = 3;
    /// Script hash condition byte
    pub const SCRIPT_HASH_BYTE: u8 = 4;
    /// Group condition byte
    pub const GROUP_BYTE: u8 = 5;
    /// Called by entry condition byte
    pub const CALLED_BY_ENTRY_BYTE: u8 = 6;
    /// Called by contract condition byte
    pub const CALLED_BY_CONTRACT_BYTE: u8 = 7;
    /// Called by group condition byte
    pub const CALLED_BY_GROUP_BYTE: u8 = 8;

    /// Convert a u8 value to a WitnessCondition
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            Self::BOOLEAN_BYTE => Some(WitnessCondition::Boolean),
            Self::NOT_BYTE => Some(WitnessCondition::Not),
            Self::AND_BYTE => Some(WitnessCondition::And),
            Self::OR_BYTE => Some(WitnessCondition::Or),
            Self::SCRIPT_HASH_BYTE => Some(WitnessCondition::ScriptHash),
            Self::GROUP_BYTE => Some(WitnessCondition::Group),
            Self::CALLED_BY_ENTRY_BYTE => Some(WitnessCondition::CalledByEntry),
            Self::CALLED_BY_CONTRACT_BYTE => Some(WitnessCondition::CalledByContract),
            Self::CALLED_BY_GROUP_BYTE => Some(WitnessCondition::CalledByGroup),
            _ => None,
        }
    }

    /// Get the u8 value of the WitnessCondition
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
}

/// Witness rule in the Neo blockchain
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WitnessRule {
    /// The action to take
    pub action: WitnessAction,
    /// The condition to check
    pub condition: WitnessCondition,
}

impl WitnessRule {
    /// Create a new witness rule
    pub fn new(action: WitnessAction, condition: WitnessCondition) -> Self {
        Self { action, condition }
    }

    /// Get the action
    pub fn action(&self) -> &WitnessAction {
        &self.action
    }

    /// Get the condition
    pub fn condition(&self) -> &WitnessCondition {
        &self.condition
    }
}
