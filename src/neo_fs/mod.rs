//! # NeoFS Module
//!
//! This module provides support for interacting with NeoFS, Neo's decentralized
//! storage system. It allows users to:
//!
//! - Create and manage containers
//! - Upload and download objects
//! - Manage access control policies
//! - Search for data in the NeoFS network
//! - Monitor storage node health and metrics

pub mod client;
pub mod container;
pub mod errors;
pub mod object;
pub mod policy;
pub mod types;

pub use client::*;
pub use container::*;
pub use errors::*;
pub use object::*;
pub use policy::*;
pub use types::*;

#[cfg(test)]
mod tests {
	#[test]
	fn basic_test() {
		assert!(true);
	}
}
