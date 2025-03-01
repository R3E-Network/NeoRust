//! # Neo Configuration
//!
//! Configuration management for the Neo N3 blockchain SDK.
//!
//! ## Overview
//!
//! The neo_config module provides configuration management for the Neo N3 blockchain SDK.
//! It includes:
//!
//! - Blockchain network configuration (MainNet, TestNet, etc.)
//! - Protocol constants and magic numbers
//! - Default settings for transaction building
//! - Test configuration utilities
//!
//! ## Feature Flags
//!
//! This module supports the following feature flags:
//!
//! - **std**: Core configuration functionality (always available)
//! - **http-client**: Network-specific configuration for HTTP clients
//! - **ws-client**: Network-specific configuration for WebSocket clients
//!
//! ## Examples
//!
//! ### Using configuration for network selection
//!
//! ```rust
//! use neo::prelude::*;
//!
//! // Get the network magic number for TestNet
//! let testnet_magic = NeoConstants::MAGIC_NUMBER_TESTNET;
//!
//! // Create a configuration for TestNet
//! let config = NeoConfig::test_net();
//!
//! // Access global configuration
//! let global_config = NEOCONFIG.lock().unwrap();
//! ```

// Core configuration types - always available
pub use constant::*;

// Network configuration - available with client features
#[cfg(any(feature = "http-client", feature = "ws-client"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "http-client", feature = "ws-client"))))]
pub use config::*;

// Test configuration - only available in test mode
#[cfg(test)]
pub use test_properties::*;

// Core configuration module - always available
mod constant;

// Network configuration - conditional on client features
#[cfg(any(feature = "http-client", feature = "ws-client"))]
mod config;

// Test configuration - only for tests
#[cfg(test)]
mod test_properties;

// Internal utility function for testing
#[doc(hidden)]
pub(crate) fn add(left: usize, right: usize) -> usize {
	left + right
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn it_works() {
		let result = add(2, 2);
		assert_eq!(result, 4);
	}
}
