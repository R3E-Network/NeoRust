//! VM state
//!
//! This module provides the VMState enum for Neo VM state.

use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

/// VM state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum VMState {
    /// None
    None,
    /// Halt
    Halt,
    /// Fault
    Fault,
    /// Break
    Break,
}

impl Default for VMState {
    fn default() -> Self {
        VMState::None
    }
}
