// Constants Module
//
// This module centralizes all constant values used throughout the Neo CLI
// including token addresses, contract script hashes, and other fixed values.
//
// Using these centralized constants improves maintainability and reduces
// the risk of inconsistencies across the codebase.

pub mod tokens;
pub mod contracts;

// Re-export commonly used constants for easier access
pub use tokens::{neo_n3_mainnet, neo_n3_testnet, neo_x_mainnet, neo_x_testnet};
pub use contracts::{bridge, flamingo, neoburger};
