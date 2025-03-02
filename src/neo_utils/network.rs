use crate::neo_clients::{JsonRpcClient, HttpProvider, RpcClient};
use crate::neo_contract::{ContractParameter, SmartContract};
use crate::neo_protocol::stack_item::StackItem;
use crate::neo_types::{Address, ScriptHash};
use crate::neo_utils::constants;

use std::str::FromStr;
use std::error::Error;

/// Enum representing Neo N3 networks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeoNetwork {
    /// Neo N3 MainNet
    MainNet,
    /// Neo N3 TestNet
    TestNet,
    /// Neo N3 PrivateNet (custom network)
    PrivateNet,
}

impl NeoNetwork {
    /// Get the network magic number for this network
    pub fn get_magic(&self) -> u32 {
        match self {
            NeoNetwork::MainNet => constants::network_magic::MAINNET,
            NeoNetwork::TestNet => constants::network_magic::TESTNET,
            NeoNetwork::PrivateNet => 0, // Default for private networks, user should override
        }
    }
    
    /// Get default RPC endpoints for this network
    pub fn get_endpoints(&self) -> &'static [&'static str] {
        match self {
            NeoNetwork::MainNet => constants::rpc_endpoints::mainnet::NEO_OFFICIAL,
            NeoNetwork::TestNet => constants::rpc_endpoints::testnet::NEO_OFFICIAL,
            NeoNetwork::PrivateNet => &["http://localhost:10332"], // Default for private networks
        }
    }
    
    /// Get the script hash for a well-known contract on this network
    pub fn get_contract_hash(&self, contract_name: &str) -> Option<String> {
        match (self, contract_name.to_lowercase().as_str()) {
            // Native contracts are the same across all networks
            (_, "neo") => Some(constants::contracts::native::NEO_TOKEN.to_string()),
            (_, "gas") => Some(constants::contracts::native::GAS_TOKEN.to_string()),
            (_, "oracle") => Some(constants::contracts::native::ORACLE.to_string()),
            (_, "policy") => Some(constants::contracts::native::POLICY.to_string()),
            (_, "ledger") => Some(constants::contracts::native::LEDGER.to_string()),
            (_, "role_management") => Some(constants::contracts::native::ROLE_MANAGEMENT.to_string()),
            (_, "contract_management") => Some(constants::contracts::native::CONTRACT_MANAGEMENT.to_string()),
            
            // Network-specific contracts
            (NeoNetwork::MainNet, "neofs") => Some(constants::contracts::mainnet::NEOFS.to_string()),
            (NeoNetwork::MainNet, "neo_ns") => Some(constants::contracts::mainnet::NEO_NS.to_string()),
            (NeoNetwork::MainNet, "flm") => Some(constants::contracts::mainnet::FLM_TOKEN.to_string()),
            (NeoNetwork::MainNet, "neoline_swap") => Some(constants::contracts::mainnet::NEOLINE_SWAP.to_string()),
            
            (NeoNetwork::TestNet, "neofs") => Some(constants::contracts::testnet::NEOFS.to_string()),
            (NeoNetwork::TestNet, "neo_ns") => Some(constants::contracts::testnet::NEO_NS.to_string()),
            (NeoNetwork::TestNet, "cneo") => Some(constants::contracts::testnet::CNEO_TOKEN.to_string()),
            (NeoNetwork::TestNet, "cgas") => Some(constants::contracts::testnet::CGAS_TOKEN.to_string()),
            
            // Contract not found
            _ => None,
        }
    }
    
    /// Create a RPC client for this network
    pub fn create_client(&self) -> Result<RpcClient<HttpProvider>, Box<dyn Error>> {
        // Try each endpoint until one works
        let endpoints = self.get_endpoints();
        let mut last_error = None;
        
        for endpoint in endpoints {
            match HttpProvider::new(*endpoint) {
                Ok(provider) => {
                    return Ok(RpcClient::new(provider));
                },
                Err(e) => {
                    last_error = Some(e.to_string());
                    // Try next endpoint
                }
            }
        }
        
        // If we got here, all endpoints failed
        Err(format!("Failed to connect to any {} endpoint. Last error: {}", 
            self, last_error.unwrap_or_else(|| "Unknown error".to_string())).into())
    }
}

impl std::fmt::Display for NeoNetwork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NeoNetwork::MainNet => write!(f, "MainNet"),
            NeoNetwork::TestNet => write!(f, "TestNet"),
            NeoNetwork::PrivateNet => write!(f, "PrivateNet"),
        }
    }
}

/// Builder for creating a network-specific client
pub struct NetworkBuilder {
    network: NeoNetwork,
    endpoints: Option<Vec<String>>,
    magic: Option<u32>,
}

impl NetworkBuilder {
    /// Create a new network builder for the specified network
    pub fn new(network: NeoNetwork) -> Self {
        Self {
            network,
            endpoints: None,
            magic: None,
        }
    }
    
    /// Set custom endpoints for this network
    pub fn endpoints(mut self, endpoints: Vec<String>) -> Self {
        self.endpoints = Some(endpoints);
        self
    }
    
    /// Set custom magic number for this network
    pub fn magic(mut self, magic: u32) -> Self {
        self.magic = Some(magic);
        self
    }
    
    /// Build a client for this network
    pub fn build_client(&self) -> Result<RpcClient<HttpProvider>, Box<dyn Error>> {
        // If custom endpoints provided, try them first
        if let Some(endpoints) = &self.endpoints {
            let mut last_error = None;
            
            for endpoint in endpoints {
                match HttpProvider::new(endpoint) {
                    Ok(provider) => {
                        return Ok(RpcClient::new(provider));
                    },
                    Err(e) => {
                        last_error = Some(e.to_string());
                        // Try next endpoint
                    }
                }
            }
            
            // If all custom endpoints failed, fall back to default endpoints
            if let Ok(client) = self.network.create_client() {
                return Ok(client);
            }
            
            // If even default endpoints failed, return the last error
            return Err(format!("Failed to connect to any {} endpoint. Last error: {}", 
                self.network, last_error.unwrap_or_else(|| "Unknown error".to_string())).into());
        }
        
        // No custom endpoints, use default ones
        self.network.create_client()
    }
    
    /// Get RPC endpoints for this network
    pub fn get_endpoints(&self) -> Vec<String> {
        if let Some(endpoints) = &self.endpoints {
            endpoints.clone()
        } else {
            self.network.get_endpoints().iter().map(|&s| s.to_string()).collect()
        }
    }
    
    /// Get the network magic number
    pub fn get_magic(&self) -> u32 {
        self.magic.unwrap_or_else(|| self.network.get_magic())
    }
}

/// Helper structure for working with tokens across networks
pub struct NetworkToken {
    network: NeoNetwork,
    client: RpcClient<HttpProvider>,
    contract_hash: ScriptHash,
}

impl NetworkToken {
    /// Create a new network token instance
    pub fn new(network: NeoNetwork, contract_name: &str) -> Result<Self, Box<dyn Error>> {
        // Get contract hash for this network
        let contract_hash_str = network.get_contract_hash(contract_name)
            .ok_or_else(|| format!("Contract '{}' not found on {}", contract_name, network))?;
        
        // Convert to script hash
        let contract_hash = ScriptHash::from_str(&contract_hash_str)?;
        
        // Create client for this network
        let client = network.create_client()?;
        
        Ok(Self {
            network,
            client,
            contract_hash,
        })
    }
    
    /// Get the balance of the token for a specific address
    pub async fn balance_of(&self, address: &str) -> Result<(u64, String, u8), Box<dyn Error>> {
        // Convert address to script hash
        let address_obj = Address::from_str(address)?;
        let address_hash = address_obj.script_hash();
        
        // Create parameter for contract invocation
        let params = vec![
            ContractParameter::hash160(&address_hash),
        ];
        
        // Invoke balanceOf
        let balance_result = self.client.invoke_function(
            &self.contract_hash, 
            "balanceOf".to_string(), 
            params, 
            None
        ).await?;
        
        if let Some(balance_item) = balance_result.stack.first() {
            // Get token decimals
            let decimals_result = self.client.invoke_function(
                &self.contract_hash, 
                "decimals".to_string(), 
                vec![], 
                None
            ).await?;
            
            // Get token symbol
            let symbol_result = self.client.invoke_function(
                &self.contract_hash, 
                "symbol".to_string(), 
                vec![], 
                None
            ).await?;
            
            // Parse results
            let balance = balance_item.get_int().ok_or("Invalid balance format")?;
            let decimals = decimals_result.stack.first()
                .ok_or("No decimals returned")?
                .get_int().ok_or("Invalid decimals format")? as u8;
            let symbol = symbol_result.stack.first()
                .ok_or("No symbol returned")?
                .get_string().ok_or("Invalid symbol format")?;
            
            Ok((balance as u64, symbol, decimals))
        } else {
            Err("No balance returned".into())
        }
    }
    
    /// Format the balance with the correct number of decimals
    pub fn format_balance(&self, balance: u64, decimals: u8) -> f64 {
        balance as f64 / 10_f64.powi(decimals as i32)
    }
    
    /// Get token information (name, symbol, decimals, etc.)
    pub async fn token_info(&self) -> Result<TokenInfo, Box<dyn Error>> {
        // Get token name
        let name_result = self.client.invoke_function(
            &self.contract_hash, 
            "name".to_string(), 
            vec![], 
            None
        ).await?;
        
        // Get token symbol
        let symbol_result = self.client.invoke_function(
            &self.contract_hash, 
            "symbol".to_string(), 
            vec![], 
            None
        ).await?;
        
        // Get token decimals
        let decimals_result = self.client.invoke_function(
            &self.contract_hash, 
            "decimals".to_string(), 
            vec![], 
            None
        ).await?;
        
        // Get token total supply
        let total_supply_result = self.client.invoke_function(
            &self.contract_hash, 
            "totalSupply".to_string(), 
            vec![], 
            None
        ).await?;
        
        // Parse results
        let name = name_result.stack.first()
            .ok_or("No name returned")?
            .get_string().ok_or("Invalid name format")?;
        
        let symbol = symbol_result.stack.first()
            .ok_or("No symbol returned")?
            .get_string().ok_or("Invalid symbol format")?;
        
        let decimals = decimals_result.stack.first()
            .ok_or("No decimals returned")?
            .get_int().ok_or("Invalid decimals format")? as u8;
        
        let total_supply = total_supply_result.stack.first()
            .ok_or("No total supply returned")?
            .get_int().ok_or("Invalid total supply format")? as u64;
        
        Ok(TokenInfo {
            name,
            symbol,
            decimals,
            total_supply,
            contract_hash: self.contract_hash.clone(),
        })
    }
}

/// Token information structure
#[derive(Debug, Clone)]
pub struct TokenInfo {
    /// Token name
    pub name: String,
    /// Token symbol
    pub symbol: String,
    /// Token decimals
    pub decimals: u8,
    /// Token total supply
    pub total_supply: u64,
    /// Token contract hash
    pub contract_hash: ScriptHash,
}

/// Helper function to get a client for a specific network
pub fn get_network_client(network: NeoNetwork) -> Result<RpcClient<HttpProvider>, Box<dyn Error>> {
    network.create_client()
}

/// Helper function to create a client for MainNet
pub fn get_mainnet_client() -> Result<RpcClient<HttpProvider>, Box<dyn Error>> {
    NeoNetwork::MainNet.create_client()
}

/// Helper function to create a client for TestNet
pub fn get_testnet_client() -> Result<RpcClient<HttpProvider>, Box<dyn Error>> {
    NeoNetwork::TestNet.create_client()
}

/// Helper function to get a smart contract instance for a well-known contract
pub fn get_network_contract(
    network: NeoNetwork,
    contract_name: &str,
) -> Result<SmartContract<RpcClient<HttpProvider>>, Box<dyn Error>> {
    // Get contract hash for this network
    let contract_hash_str = network.get_contract_hash(contract_name)
        .ok_or_else(|| format!("Contract '{}' not found on {}", contract_name, network))?;
    
    // Convert to script hash
    let contract_hash = ScriptHash::from_str(&contract_hash_str)?;
    
    // Create client for this network
    let client = network.create_client()?;
    
    // Create contract instance
    Ok(SmartContract::new(contract_hash, &client))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_network_magic() {
        assert_eq!(NeoNetwork::MainNet.get_magic(), constants::network_magic::MAINNET);
        assert_eq!(NeoNetwork::TestNet.get_magic(), constants::network_magic::TESTNET);
    }
    
    #[test]
    fn test_get_contract_hash() {
        // Test native contracts (same on all networks)
        assert_eq!(
            NeoNetwork::MainNet.get_contract_hash("neo"),
            Some(constants::contracts::native::NEO_TOKEN.to_string())
        );
        assert_eq!(
            NeoNetwork::TestNet.get_contract_hash("neo"),
            Some(constants::contracts::native::NEO_TOKEN.to_string())
        );
        
        // Test network-specific contracts
        assert_eq!(
            NeoNetwork::MainNet.get_contract_hash("flm"),
            Some(constants::contracts::mainnet::FLM_TOKEN.to_string())
        );
        assert_eq!(
            NeoNetwork::TestNet.get_contract_hash("cneo"),
            Some(constants::contracts::testnet::CNEO_TOKEN.to_string())
        );
        
        // Test non-existent contract
        assert_eq!(NeoNetwork::MainNet.get_contract_hash("nonexistent"), None);
    }
    
    #[test]
    fn test_get_endpoints() {
        assert_eq!(
            NeoNetwork::MainNet.get_endpoints(),
            constants::rpc_endpoints::mainnet::NEO_OFFICIAL
        );
        assert_eq!(
            NeoNetwork::TestNet.get_endpoints(),
            constants::rpc_endpoints::testnet::NEO_OFFICIAL
        );
    }
} 