//! # Neo RPC Client Module (v0.1.8)
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
//! use neo_rust::prelude::*;
//! use neo_rust::neo_clients::rpc::{RpcClient, HttpTransport};
//! use neo_rust::neo_types::{Address, ScriptHash};
//! use std::str::FromStr;
//!
//! async fn rpc_examples() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create an HTTP client connected to a Neo TestNet node
//!     let transport = HttpTransport::new("https://testnet1.neo.org:443");
//!     let client = RpcClient::new(transport);
//!     
//!     // Get basic blockchain information
//!     let block_count = client.get_block_count().await?;
//!     println!("Current block height: {}", block_count);
//!     
//!     let best_block_hash = client.get_best_block_hash().await?;
//!     println!("Best block hash: {}", best_block_hash);
//!     
//!     // Get detailed block information
//!     let block = client.get_block(best_block_hash, true).await?;
//!     println!("Block time: {}, tx count: {}",
//!              block.time,
//!              block.tx.as_ref().map(|txs| txs.len()).unwrap_or(0));
//!     
//!     // Query account information
//!     let address = Address::from_str("NUVPACTpQvd2HHmBgFjJJRWwVXJiR3uAEh")?;
//!     let script_hash = ScriptHash::from_address(&address)?;
//!     
//!     // Get NEP-17 token balances
//!     let balances = client.get_nep17_balances(&script_hash).await?;
//!     
//!     for balance in balances.balances {
//!         println!("Token: {}, Amount: {}",
//!                  balance.asset_hash,
//!                  balance.amount);
//!     }
//!     
//!     // Get application logs for a transaction
//!     if let Some(tx) = block.tx.as_ref().and_then(|txs| txs.first()) {
//!         let app_log = client.get_application_log(&tx.hash).await?;
//!         println!("Transaction triggers: {} executions", app_log.executions.len());
//!         
//!         // Print any notifications emitted by the contract
//!         for execution in app_log.executions {
//!             for notification in execution.notifications {
//!                 println!("Contract {} emitted event: {}",
//!                          notification.contract,
//!                          notification.event_name);
//!             }
//!         }
//!     }
//!     
//!     // Use WebSocket for real-time updates (if supported by the node)
//!     #[cfg(feature = "ws")]
//!     {
//!         use neo_rust::neo_clients::rpc::{WebSocketTransport, PubsubClient};
//!         
//!         let ws_transport = WebSocketTransport::new("wss://testnet1.ws.neo.org:443").await?;
//!         let pubsub = PubsubClient::new(ws_transport);
//!         
//!         // Subscribe to new blocks
//!         let mut block_subscription = pubsub.subscribe_blocks().await?;
//!         
//!         // Listen for the next block (with timeout)
//!         if let Some(new_block) = block_subscription.next().await {
//!             println!("New block received: {}", new_block.hash);
//!         }
//!     }
//!     
//!     Ok(())
//! }
//! ```

pub use connections::*;
pub use pubsub::{PubsubClient, SubscriptionStream};
pub use rpc_client::*;
pub use transports::*;

pub mod rpc_client;
pub mod connections;
pub mod pubsub;
pub mod transports;
