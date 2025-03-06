//! Hard forks utilities for the NeoRust SDK.
//!
//! This module provides utilities for working with hard forks in the Neo blockchain.

use crate::hard_forks::HardForks;

/// Checks if a hard fork is active at a given block height.
pub fn is_hard_fork_active(hard_fork: HardForks, block_height: u32) -> bool {
    match hard_fork {
        HardForks::Aspidochelone => block_height >= 1_730_000,
        HardForks::Basilisk => block_height >= 2_320_000,
        HardForks::Cockatrice => block_height >= 3_150_000,
    }
}
