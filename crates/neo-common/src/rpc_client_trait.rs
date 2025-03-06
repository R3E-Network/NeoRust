//! RPC Client Trait
//!
//! This module defines common types for RPC clients to break circular dependencies
//! between neo-protocol and neo-clients.

use std::future::Future;

/// RPC Client trait that defines methods available on all RPC clients
pub trait RpcClient: Send + Sync + std::fmt::Debug {
    /// Returns the maximum valid until block increment
    fn max_valid_until_block_increment(&self) -> u32;
    
    /// Invokes a script with the given signers
    fn invoke_script<'a>(&'a self, script: String, signers: Vec<String>) 
        -> Box<dyn Future<Output = Result<String, crate::provider_error::ProviderError>> + Send + 'a>;
    
    /// Calculates the network fee for a transaction
    fn calculate_network_fee<'a>(&'a self, tx_hex: String) 
        -> Box<dyn Future<Output = Result<u64, crate::provider_error::ProviderError>> + Send + 'a>;
    
    /// Gets the current block count
    fn get_block_count<'a>(&'a self) 
        -> Box<dyn Future<Output = Result<u32, crate::provider_error::ProviderError>> + Send + 'a>;
    
    /// Invokes a contract function with the given parameters and signers
    fn invoke_function<'a>(&'a self, script_hash: String, operation: String, params: Vec<String>, signers: Vec<String>) 
        -> Box<dyn Future<Output = Result<String, crate::provider_error::ProviderError>> + Send + 'a>;
    
    /// Gets the committee members
    fn get_committee<'a>(&'a self) 
        -> Box<dyn Future<Output = Result<Vec<String>, crate::provider_error::ProviderError>> + Send + 'a>;
        
    /// Gets the network magic number
    fn network<'a>(&'a self)
        -> Box<dyn Future<Output = Result<u32, crate::provider_error::ProviderError>> + Send + 'a>;
        
    /// Gets the block hash for a given block index
    fn get_block_hash<'a>(&'a self, block_index: u32)
        -> Box<dyn Future<Output = Result<String, crate::provider_error::ProviderError>> + Send + 'a>;
        
    /// Gets a block by its hash
    fn get_block<'a>(&'a self, block_hash: String, full_transactions: bool)
        -> Box<dyn Future<Output = Result<String, crate::provider_error::ProviderError>> + Send + 'a>;
        
    /// Sends a raw transaction
    fn send_raw_transaction<'a>(&'a self, hex: String)
        -> Box<dyn Future<Output = Result<String, crate::provider_error::ProviderError>> + Send + 'a>;
        
    /// Gets the application log for a transaction
    fn get_application_log<'a>(&'a self, tx_hash: String)
        -> Box<dyn Future<Output = Result<String, crate::provider_error::ProviderError>> + Send + 'a>;
}

/// Helper trait for method chaining
pub trait RpcClientExt: RpcClient {
    /// Returns a reference to self for method chaining
    fn rpc_client(&self) -> &Self {
        self
    }
}

/// Default implementation for any type that implements RpcClient
impl<T: RpcClient> RpcClientExt for T {}


/// Neo version information
#[derive(Debug, Clone)]
pub struct NeoVersion {
    /// The protocol configuration
    pub protocol: ProtocolSettings,
    /// The network
    pub network: u32,
    /// The node port
    pub port: u16,
    /// The node nonce
    pub nonce: u64,
    /// The user agent
    pub user_agent: String,
}

/// Protocol settings
#[derive(Debug, Clone)]
pub struct ProtocolSettings {
    /// The network magic number
    pub network_magic: u32,
    /// The address version
    pub address_version: u8,
    /// The standby validators
    pub standby_validators: Vec<String>,
    /// The committee members
    pub committee_members: Vec<String>,
    /// The seed list
    pub seed_list: Vec<String>,
    /// The milliseconds per block
    pub milliseconds_per_block: u32,
    /// The memory pool maximum transactions
    pub memory_pool_max_transactions: u32,
}
