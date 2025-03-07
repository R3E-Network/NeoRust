//! Wallet trait and implementation for the NeoRust SDK.

use std::fmt::Debug;
use serde::{Deserialize, Serialize};

/// A wallet that can be used to sign transactions.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct Wallet {
    /// The wallet's address.
    #[serde(default)]
    pub address: String,
}

impl Wallet {
    /// Creates a new wallet.
    pub fn new(address: String) -> Self {
        Self { address }
    }
}
