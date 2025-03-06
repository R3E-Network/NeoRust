//! Hard fork types for Neo blockchain
//!
//! This module provides types for hard forks in the Neo blockchain.

use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

/// Hard fork flags for Neo blockchain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
#[repr(u32)]
pub enum HardForks {
    /// Hardfork for enabling NEP-17 extensions
    #[strum(serialize = "hardfork_v17")]
    HardforkV17 = 0,

    /// Hardfork for enabling NEP-18 extensions
    #[strum(serialize = "hardfork_v18")]
    HardforkV18 = 1,

    /// Hardfork for enabling NEP-21 extensions
    #[strum(serialize = "hardfork_v21")]
    HardforkV21 = 2,

    /// Hardfork for enabling NEP-31 extensions
    #[strum(serialize = "hardfork_v31")]
    HardforkV31 = 3,
}

impl HardForks {
    /// Get the hard fork as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            HardForks::HardforkV17 => "hardfork_v17",
            HardForks::HardforkV18 => "hardfork_v18",
            HardForks::HardforkV21 => "hardfork_v21",
            HardForks::HardforkV31 => "hardfork_v31",
        }
    }

    /// Get the hard fork as a u32 value
    pub fn as_u32(&self) -> u32 {
        *self as u32
    }

    /// Create a hard fork from a u32 value
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(HardForks::HardforkV17),
            1 => Some(HardForks::HardforkV18),
            2 => Some(HardForks::HardforkV21),
            3 => Some(HardForks::HardforkV31),
            _ => None,
        }
    }
}
