//! # Neo Clients
//!
//! Client implementations for interacting with Neo N3 blockchain nodes.
//!
//! This crate provides various client implementations for connecting to and interacting with Neo N3 blockchain nodes, including:
//!
//! - JSON-RPC client for communicating with Neo nodes
//! - WebSocket client for real-time event subscriptions
//! - HTTP client for standard API requests
//! - Provider abstractions for different connection types
//! - Middleware support for request/response processing
//!
//! ## Usage
//!
//! ```rust,ignore
//! use neo_clients::{JsonRpcProvider, RpcClient};
//! use std::str::FromStr;
//!
//! // Create a JSON-RPC provider
//! let provider = JsonRpcProvider::new("https://mainnet.neoline.io:443");
//!
//! // Get the latest block height
//! let block_height = provider.get_block_count().await?;
//!
//! // Create a WebSocket client for subscriptions
//! let ws_client = RpcClient::new_websocket("wss://mainnet.neoline.io:4443/ws");
//!
//! // Subscribe to new blocks
//! let subscription = ws_client.subscribe_to_new_blocks().await?;
//! ```

#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms, unreachable_pub)]
#![deny(rustdoc::broken_intra_doc_links)]

mod api_trait;
mod errors;
mod error_adapter;
mod ext;
mod mock_blocks;
mod mock_client;
mod nep17_provider_impl;
mod rpc;
mod rx;
mod utils;

// Re-export all public items
pub use api_trait::*;
pub use errors::*;
pub use error_adapter::*;
pub use ext::*;
pub use mock_blocks::*;
pub use mock_client::*;
pub use rpc::*;
pub use rx::*;
pub use utils::*;

// Implement RpcClient trait for JsonRpcProvider
impl neo_common::RpcClient for crate::rpc::RpcClient<crate::rpc::transports::http_provider::HttpProvider> {
    fn max_valid_until_block_increment(&self) -> u32 {
        2048 // Default value
    }
    
    fn invoke_script(&self, script: String, signers: Vec<String>) -> Result<String, neo_common::provider_error::ProviderError> {
        // Forward to the actual implementation
        self.invoke_script(script, signers)
            .map_err(|e| neo_common::provider_error::ProviderError::RpcError(e.to_string()))
    }
    
    fn calculate_network_fee(&self, tx_hex: String) -> Result<u64, neo_common::provider_error::ProviderError> {
        // Forward to the actual implementation
        self.calculate_network_fee(tx_hex)
            .map_err(|e| neo_common::provider_error::ProviderError::RpcError(e.to_string()))
    }
    
    fn get_block_count(&self) -> Result<u32, neo_common::provider_error::ProviderError> {
        // Forward to the actual implementation
        self.get_block_count()
            .map_err(|e| neo_common::provider_error::ProviderError::RpcError(e.to_string()))
    }
    
    fn invoke_function(&self, script_hash: String, operation: String, params: Vec<String>, signers: Vec<String>) -> Result<String, neo_common::provider_error::ProviderError> {
        // Forward to the actual implementation
        self.invoke_function(script_hash, operation, params, signers)
            .map_err(|e| neo_common::provider_error::ProviderError::RpcError(e.to_string()))
    }
    
    fn get_committee(&self) -> Result<Vec<String>, neo_common::provider_error::ProviderError> {
        // Forward to the actual implementation
        self.get_committee()
            .map_err(|e| neo_common::provider_error::ProviderError::RpcError(e.to_string()))
    }
}

// Implement RpcClientExt for JsonRpcProvider
impl neo_common::RpcClientExt for crate::rpc::RpcClient<crate::rpc::transports::http_provider::HttpProvider> {}
