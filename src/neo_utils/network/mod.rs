//! Network module for Neo Rust SDK
//!
//! This module provides network utilities for working with different Neo N3 networks.

use std::fmt;
use crate::neo_error::NeoError;

/// Neo network client
#[derive(Debug, Clone)]
pub struct NeoNetwork {
    /// Network name
    pub name: String,
    /// RPC endpoint
    pub rpc_endpoint: String,
}

impl NeoNetwork {
    /// Create a new Neo network client for MainNet
    pub fn main_net() -> Self {
        Self {
            name: "MainNet".to_string(),
            rpc_endpoint: "https://mainnet1.neo.org:443".to_string(),
        }
    }

    /// Create a new Neo network client for TestNet
    pub fn test_net() -> Self {
        Self {
            name: "TestNet".to_string(),
            rpc_endpoint: "https://testnet1.neo.org:443".to_string(),
        }
    }
    
    /// Create a client for this network
    #[cfg(feature = "tokio-support")]
    pub fn create_client(&self) -> Result<crate::neo_provider::JsonRpcClient, NeoError> {
        Ok(crate::neo_provider::JsonRpcClient::new(&self.rpc_endpoint))
    }
    
    /// Get MainNet network
    pub fn MainNet() -> Self {
        Self::main_net()
    }
    
    /// Get TestNet network
    pub fn TestNet() -> Self {
        Self::test_net()
    }
    
    /// Get contract hash for a given token name
    pub fn get_contract_hash(&self, token_name: &str) -> Result<String, String> {
        get_network_contract(self.clone(), token_name)
    }
}

impl fmt::Display for NeoNetwork {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Network token
#[derive(Debug, Clone)]
pub struct NetworkToken {
    /// Network
    pub network: NeoNetwork,
    /// Token name
    pub token_name: String,
    /// Contract hash
    pub contract_hash: String,
}

impl NetworkToken {
    /// Create a new network token
    pub fn new(network: NeoNetwork, token_name: &str) -> Result<Self, String> {
        let contract_hash = get_network_contract(network.clone(), token_name)?;
        
        Ok(Self {
            network,
            token_name: token_name.to_string(),
            contract_hash,
        })
    }
    
    /// Format token balance
    pub fn format_balance(&self, balance: u64, decimals: u8) -> String {
        let divisor = 10u64.pow(decimals as u32);
        let whole_part = balance / divisor;
        let fractional_part = balance % divisor;
        
        format!("{}.{:0width$}", whole_part, fractional_part, width = decimals as usize)
    }
    
    /// Get token info
    #[cfg(feature = "tokio-support")]
    pub async fn token_info(&self) -> Result<TokenInfo, String> {
        // This is a placeholder implementation
        Ok(TokenInfo {
            name: format!("{} Token", self.token_name.to_uppercase()),
            symbol: self.token_name.to_uppercase(),
            decimals: 8,
            total_supply: 100_000_000 * 100_000_000, // 100M with 8 decimals
            contract_hash: self.contract_hash.clone(),
        })
    }
    
    /// Get balance of address
    #[cfg(feature = "tokio-support")]
    pub async fn balance_of(&self, _address: &str) -> Result<(u64, String, u8), String> {
        // This is a placeholder implementation
        let balance = 1_000 * 100_000_000; // 1,000 tokens with 8 decimals
        let symbol = self.token_name.to_uppercase();
        let decimals = 8;
        
        Ok((balance, symbol, decimals))
    }
}

/// Token information
#[derive(Debug, Clone)]
pub struct TokenInfo {
    /// Token name
    pub name: String,
    /// Token symbol
    pub symbol: String,
    /// Token decimals
    pub decimals: u8,
    /// Total supply
    pub total_supply: u64,
    /// Contract hash
    pub contract_hash: String,
}

/// Get a network contract by name
pub fn get_network_contract(network: NeoNetwork, token_name: &str) -> Result<String, String> {
    use crate::neo_utils::constants::contracts;
    
    match (network.name.as_str(), token_name) {
        ("MainNet", "neo") => Ok(contracts::mainnet::NEO_TOKEN.to_string()),
        ("MainNet", "gas") => Ok(contracts::mainnet::GAS_TOKEN.to_string()),
        ("MainNet", "policy") => Ok(contracts::native::POLICY.to_string()),
        ("TestNet", "neo") => Ok(contracts::testnet::NEO_TOKEN.to_string()),
        ("TestNet", "gas") => Ok(contracts::testnet::GAS_TOKEN.to_string()),
        ("TestNet", "policy") => Ok(contracts::native::POLICY.to_string()),
        _ => Err(format!("Unknown token {} on network {}", token_name, network.name)),
    }
}
