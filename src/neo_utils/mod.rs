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
//! - Network and contract constants
//! - Network utilities for working with different Neo N3 networks
//!
//! ## Feature Flags
//!
//! Most utilities in this module are always available as they are core to the SDK's functionality.

// Core error module - always available
pub mod error;

// Constants for Neo N3 networks and contracts
pub mod constants;

// Network utilities for working with different Neo N3 networks
pub mod network;

// Re-export error types for convenience
pub use error::*;

// Re-export constants for convenience
pub use constants::*;

// Re-export network utilities for convenience
pub use network::*;
