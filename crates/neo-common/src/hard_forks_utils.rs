//! Hard forks utilities for the NeoRust SDK.
//!
//! This module provides utilities for working with hard forks in the Neo blockchain.

use crate::hard_forks::HardForks;
use serde::{Deserialize, Deserializer};

/// Checks if a hard fork is active at a given block height.
pub fn is_hard_fork_active(hard_fork: HardForks, block_height: u32) -> bool {
    match hard_fork {
        HardForks::Aspidochelone => block_height >= 1_730_000,
        HardForks::Basilisk => block_height >= 2_320_000,
        HardForks::Cockatrice => block_height >= 3_150_000,
    }
}

/// Deserialize hard forks from a JSON value.
pub fn deserialize_hardforks<'de, D>(deserializer: D) -> Result<Vec<HardForks>, D::Error>
where
    D: Deserializer<'de>,
{
    let hard_forks_str: Vec<String> = Deserialize::deserialize(deserializer)?;
    let mut result = Vec::new();
    
    for fork_str in hard_forks_str {
        match fork_str.as_str() {
            "Aspidochelone" => result.push(HardForks::Aspidochelone),
            "Basilisk" => result.push(HardForks::Basilisk),
            "Cockatrice" => result.push(HardForks::Cockatrice),
            _ => {} // Ignore unknown hard forks
        }
    }
    
    Ok(result)
}
