//! # Neo Utilities
//!
//! Utility functions and types for the NeoRust SDK.
//!
//! ## Overview
//!
//! The neo_utils module provides various utility functions and types that are used throughout the SDK,
//! including:
//!
//! - Error handling utilities
//! - Common helper functions
//! - Cross-cutting concerns
//!
//! ## Feature Flags
//!
//! Most utilities in this module are always available as they are core to the SDK's functionality.

// Core error module - always available
pub mod error;

// Re-export error types for convenience
pub use error::*;
