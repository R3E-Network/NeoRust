//! Neo Crypto - Cryptographic primitives for the Neo blockchain
//!
//! This module contains the cryptographic primitives used in the Neo blockchain.

// Conditional modules
#[cfg(feature = "crypto-standard")]
pub mod error;

#[cfg(feature = "crypto-standard")]
pub mod hash;

#[cfg(feature = "crypto-standard")]
pub mod keys;

#[cfg(feature = "crypto-standard")]
pub mod key_pair;

// Re-exports
#[cfg(feature = "crypto-standard")]
pub use error::CryptoError;

#[cfg(feature = "crypto-standard")]
pub use hash::*;

#[cfg(feature = "crypto-standard")]
pub use keys::*;

#[cfg(feature = "crypto-standard")]
pub use key_pair::*;
