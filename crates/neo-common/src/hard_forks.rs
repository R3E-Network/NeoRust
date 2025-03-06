//! Hard forks in the Neo blockchain.
//!
//! This module defines the hard forks that have occurred in the Neo blockchain.

use serde::{Deserialize, Serialize};

/// Hard forks in the Neo blockchain
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HardForks {
    /// Aspidochelone hard fork
    Aspidochelone,
    /// Basilisk hard fork
    Basilisk,
    /// Cockatrice hard fork
    Cockatrice,
}
