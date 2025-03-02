#![feature(inherent_associated_types)]
#![doc = include_str!("../../README.md")]
#![allow(clippy::type_complexity)]
#![warn(missing_docs)]
#![deny(unsafe_code, rustdoc::broken_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! # Neo Clients
//!
//! Client interfaces for interacting with Neo N3 blockchain nodes.
//!
//! ## Overview
//!
//! The neo_clients module provides a comprehensive set of client interfaces for connecting to
//! and interacting with Neo N3 blockchain nodes. It includes:
//!
//! - RPC clients for making JSON-RPC calls to Neo nodes
//! - Multiple transport providers (HTTP, WebSocket, IPC)
//! - Subscription support for real-time blockchain events
//! - Mock clients for testing
//! - Extension traits for domain-specific functionality
//! - Error handling for network and protocol issues
//!
//! The module is designed to be flexible, allowing developers to choose the appropriate
//! client implementation and transport mechanism for their specific use case.
//!
//! ## Feature Flags
//!
//! This module supports various feature flags to customize client functionality:
//!
//! - **http-client**: Enables HTTP transport for interacting with Neo nodes (default)
//! - **ws-client**: Enables WebSocket transport for real-time updates and subscriptions
//! - **full-node-client**: Enables both HTTP and WebSocket transports
//!
//! ## Examples
//!
//! ### Connecting to a Neo N3 node using HTTP
//!
//! ```rust
//! use neo::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create an HTTP provider connected to a Neo N3 TestNet node
//!     let provider = HttpProvider::new("https://testnet1.neo.org:443");
//!     
//!     // Create an RPC client with the provider
//!     let client = RpcClient::new(provider);
//!     
//!     // Get the current block count
//!     let block_count = client.get_block_count().await?;
//!     println!("Current block count: {}", block_count);
//!     
//!     // Get information about the blockchain
//!     let version = client.get_version().await?;
//!     println!("Node version: {}", version.user_agent);
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### Using WebSocket for real-time updates
//!
//! ```rust
//! use neo::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Connect to a Neo N3 node using WebSocket
//!     let ws = WebSocketProvider::connect("wss://testnet1.neo.org:4443/ws").await?;
//!     let client = RpcClient::new(ws);
//!     
//!     // Subscribe to new blocks
//!     let mut block_subscription = client.subscribe_to_new_blocks().await?;
//!     
//!     // Process the first 5 new blocks
//!     for _ in 0..5 {
//!         if let Some(block) = block_subscription.next().await {
//!             println!("New block: {}", block.hash);
//!         }
//!     }
//!     
//!     Ok(())
//! }
//! ```

#[cfg(any(feature = "http-client", feature = "websocket"))]
use lazy_static::lazy_static;

// Error module definition
pub mod errors;
pub use errors::ProviderError;

// Core modules
pub mod rpc;
pub use rpc::JsonRpcProvider;
pub use rpc::RpcClient;

// HTTP client functionality
#[cfg(feature = "http-client")]
pub mod http;
#[cfg(feature = "http-client")]
pub use http::*;

// Mock client for testing
#[cfg(feature = "http-client")]
pub mod mock_client;
#[cfg(feature = "http-client")]
pub use mock_client::*;

// WebSocket client functionality
#[cfg(feature = "websocket")]
pub mod ws_provider;
#[cfg(feature = "websocket")]
pub use ws_provider::{WebSocketProvider, Subscription};

// Rest client
#[cfg(feature = "rest-client")]
pub mod rest_client;
#[cfg(feature = "rest-client")]
pub use rest_client::*;

#[cfg(test)]
#[allow(dead_code)]
mod test_provider {
	use super::*;
	use lazy_static::lazy_static;
	use std::{
		iter::{Cycle, Iter},
		sync::Mutex,
	};

	lazy_static! {
		pub static ref MAINNET: TestProvider = TestProvider::new(&[], "mainnet");
		pub static ref TESTNET: TestProvider = TestProvider::new(&[], "testnet");
	}

	pub struct TestProvider {
		network: String,
		keys: Mutex<Cycle<Iter<'static, &'static str>>>,
	}

	impl TestProvider {
		pub fn new(keys: &'static [&'static str], network: impl Into<String>) -> Self {
			Self { network: network.into(), keys: Mutex::new(keys.iter().cycle()) }
		}

		pub fn url(&self) -> String {
			"http://localhost:3000".to_string()
		}

		#[cfg(feature = "http-client")]
		pub fn provider(&self) -> RpcClient<rpc::transports::http_provider::HttpProvider> {
			let provider = rpc::transports::http_provider::HttpProvider::new(self.url().as_str()).unwrap();
			RpcClient::new(provider)
		}

		#[cfg(feature = "websocket")]
		pub async fn ws(&self) -> RpcClient<WebSocketProvider> {
			let provider = WebSocketProvider::connect(self.url().as_str()).await.unwrap();
			RpcClient::new(provider)
		}
	}
}
