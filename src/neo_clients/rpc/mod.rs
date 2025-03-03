//! # Neo RPC Client Module (v0.1.4)
//!
//! The RPC module provides client implementations for interacting with Neo nodes 
//! through JSON-RPC API calls.
//!
//! ## Overview
//!
//! This module implements RPC client functionality for communicating with Neo blockchain nodes, including:
//!
//! - **Transport Protocols**: HTTP and WebSocket transport implementations
//! - **Connection Management**: Connection pooling and request batching
//! - **Pub/Sub Support**: Real-time notifications for blockchain events
//! - **Client Interfaces**: Type-safe wrappers for Neo's JSON-RPC methods
//!
//! ## Example
//!
//! ```no_run
//! use neo_rust::neo_clients::rpc::{RpcClient, HttpTransport};
//! 
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create an HTTP client connected to a Neo node
//!     let transport = HttpTransport::new("http://seed1.neo.org:10332");
//!     let client = RpcClient::new(transport);
//!     
//!     // Get current block count
//!     let block_count = client.get_block_count().await?;
//!     println!("Current block height: {}", block_count);
//!     
//!     // Get the hash of a specific block
//!     let block_hash = client.get_block_hash(1234).await?;
//!     println!("Block hash: {}", block_hash);
//!     
//!     Ok(())
//! }
//! ```

pub use connections::*;
pub use pubsub::{PubsubClient, SubscriptionStream};
pub use rpc_client::*;
pub use transports::*;

mod rpc_client;

mod connections;
mod pubsub;
mod transports;
