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

mod client;
mod container;
mod object;
mod types;
mod errors;
mod policy;

pub use client::*;
pub use container::*;
pub use object::*;
pub use types::*;
pub use errors::*;
pub use policy::*;

#[cfg(test)]
mod tests {
    #[test]
    fn basic_test() {
        assert!(true);
    }
} 