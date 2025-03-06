//! Hard forks serialization utilities
//!
//! This module provides utilities for serializing and deserializing hard forks.

use serde::{Deserialize, Deserializer};
use crate::HardForks;

/// Deserializes hard forks from a string.
pub fn deserialize_hardforks<'de, D>(deserializer: D) -> Result<Vec<HardForks>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let parts: Vec<&str> = s.split(',').collect();
    
    let mut result = Vec::new();
    for part in parts {
        let trimmed = part.trim();
        if !trimmed.is_empty() {
            match trimmed {
                "Hardfork_0" | "hardfork_v17" => result.push(HardForks::HardforkV17),
                "Hardfork_1" | "hardfork_v18" => result.push(HardForks::HardforkV18),
                "Hardfork_2" | "hardfork_v21" => result.push(HardForks::HardforkV21),
                "Hardfork_3" | "hardfork_v31" => result.push(HardForks::HardforkV31),
                _ => return Err(serde::de::Error::custom(format!("Unknown hardfork: {}", trimmed))),
            }
        }
    }
    
    Ok(result)
}
