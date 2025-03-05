//! # Neo FS
//!
//! NeoFS integration for the NeoRust SDK.
//!
//! This crate provides utilities for interacting with NeoFS, the distributed, decentralized 
//! object storage network for the Neo blockchain, including:
//!
//! - Container management
//! - Object storage and retrieval
//! - Access control lists (ACLs)
//! - Client functionality for NeoFS operations
//! - Type definitions for NeoFS entities
//!
//! ## Usage
//!
//! ```rust,ignore
//! use neo_fs::{NeoFSClient, Container, ACL};
//! use neo_types::ScriptHash;
//! use std::str::FromStr;
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a NeoFS client
//!     let client = NeoFSClient::new("https://neofs.example.com");
//!     
//!     // List containers
//!     let containers = client.list_containers().await?;
//!     
//!     // Create a new container
//!     let container = Container::new("my-container", "My container description");
//!     let container_id = client.create_container(&container).await?;
//!     
//!     // Upload an object
//!     let object_id = client.put_object(container_id, "my-file.txt", b"Hello, NeoFS!").await?;
//!     
//!     // Download an object
//!     let data = client.get_object(container_id, object_id).await?;
//!     
//!     Ok(())
//! }
//! ```

#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms, unreachable_pub)]
#![deny(rustdoc::broken_intra_doc_links)]

mod acl;
mod client;
mod container;
mod error;
mod object;
mod types;

// Re-export all public items
pub use acl::*;
pub use client::*;
pub use container::*;
pub use error::*;
pub use object::*;
pub use types::*;
