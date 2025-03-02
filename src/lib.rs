//! Neo Rust SDK - A Rust SDK for the Neo N3 blockchain
//!
//! This library provides a comprehensive set of tools for interacting with the Neo N3 blockchain.

// Core modules - always available
pub mod neo_types;
pub mod neo_error;

// Conditional modules
#[cfg(feature = "crypto-standard")]
pub mod neo_crypto;

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

// Re-exports
pub use neo_types::*;
// Export only specific items from neo_error to avoid conflicts
pub use neo_error::{NeoError, CodecError};

// Prelude module for convenient imports
pub mod prelude {
    pub use crate::neo_types::*;
    // Export only specific items from neo_error to avoid conflicts
    pub use crate::neo_error::{NeoError, CodecError};
    
    #[cfg(feature = "crypto-standard")]
    pub use crate::neo_crypto::*;
}
