//! # Neo Utilities (v0.1.8)
//!
//! Utility functions and types for the NeoRust SDK.
//!
//! ## Overview
//!
//! The neo_utils module provides various utility functions and types that are used throughout the SDK.
//! These utilities include:
//!
//! - Error types and handling utilities
//! - Common helper functions
//! - Conversion utilities
//! - Formatting utilities
//!
//! This module serves as a foundation for the more specialized modules in the SDK.
//!
//! ## Examples
//!
//! ```rust
//! use neo_types::NeoError;
//!
//! // Error handling with specific error types
//! fn example() -> Result<(), NeoError> {
//!     // Create and return a specific error
//!     if some_condition {
//!         return Err(NeoError::InvalidFormat);
//!     }
//!     
//!     Ok(())
//! }
//! ```

pub mod error;
