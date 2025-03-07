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

// Implement CommonRpcClient trait for JsonRpcProvider
impl neo_common::RpcClient for crate::rpc::RpcClient<crate::rpc::HttpProvider> {
    fn max_valid_until_block_increment(&self) -> u32 {
        2048 // Default value
    }
    
    fn invoke_script<'a>(&'a self, script: String, signers: Vec<String>) 
        -> Box<dyn std::future::Future<Output = Result<String, neo_common::provider_error::ProviderError>> + Send + 'a> {
        Box::new(async move {
            // Forward to the actual implementation
            let result = api_trait::APITrait::invoke_script(self, script, Vec::new())
                .await
                .map_err(|e| neo_common::provider_error::ProviderError::RpcError(e.to_string()))?;
            
            // Convert InvocationResult to String
            Ok(serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string()))
        })
    }
    
    fn calculate_network_fee<'a>(&'a self, tx_hex: String) 
        -> Box<dyn std::future::Future<Output = Result<u64, neo_common::provider_error::ProviderError>> + Send + 'a> {
        Box::new(async move {
            // Forward to the actual implementation
            let result = api_trait::APITrait::calculate_network_fee(self, tx_hex)
                .await
                .map_err(|e| neo_common::provider_error::ProviderError::RpcError(e.to_string()))?;
            
            // Extract fee value from NeoNetworkFee
            Ok(result.network_fee as u64)
        })
    }
    
    fn get_block_count<'a>(&'a self) 
        -> Box<dyn std::future::Future<Output = Result<u32, neo_common::provider_error::ProviderError>> + Send + 'a> {
        Box::new(async move {
            // Forward to the actual implementation
            api_trait::APITrait::get_block_count(self)
                .await
                .map_err(|e| neo_common::provider_error::ProviderError::RpcError(e.to_string()))
        })
    }
    
    fn invoke_function<'a>(&'a self, script_hash: String, operation: String, params: Vec<String>, signers: Vec<String>) 
        -> Box<dyn std::future::Future<Output = Result<String, neo_common::provider_error::ProviderError>> + Send + 'a> {
        Box::new(async move {
            // Convert script_hash from String to H160
            let script_hash_h160 = match std::str::FromStr::from_str(&script_hash) {
                Ok(h) => h,
                Err(_) => return Err(neo_common::provider_error::ProviderError::InvalidAddress),
            };
            
            // Convert params from Vec<String> to Vec<ContractParameter>
            let contract_params = Vec::new(); // Empty params for now
            
            // Forward to the actual implementation
            let result = api_trait::APITrait::invoke_function(
                self, 
                &script_hash_h160, 
                operation, 
                contract_params,
                None
            )
            .await
            .map_err(|e| neo_common::provider_error::ProviderError::RpcError(e.to_string()))?;
            
            // Convert InvocationResult to String
            Ok(serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string()))
        })
    }
    
    fn get_committee<'a>(&'a self) 
        -> Box<dyn std::future::Future<Output = Result<Vec<String>, neo_common::provider_error::ProviderError>> + Send + 'a> {
        Box::new(async move {
            // Forward to the actual implementation
            api_trait::APITrait::get_committee(self)
                .await
                .map_err(|e| neo_common::provider_error::ProviderError::RpcError(e.to_string()))
        })
    }
}
