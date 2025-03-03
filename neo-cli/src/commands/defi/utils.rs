// DeFi utilities for Neo CLI
//
// This module contains utility functions for DeFi operations

use crate::errors::CliError;
use crate::commands::wallet::{CliState, Network};
use std::path::PathBuf;
use std::str::FromStr;
use neo3::prelude::*;
use neo3::neo_clients::{HttpProvider, RpcClient};
use num_traits::cast::ToPrimitive;
use primitive_types::H160;
use neo3::neo_types::AddressExtension;
use neo3::neo_wallets::Wallet;
use neo3::providers::APITrait;

/// Network types supported by the CLI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkType {
    N3Mainnet,
    N3Testnet,
    NeoX,
}

impl NetworkType {
    /// Get the network type from a magic number
    pub fn from_magic(magic: u32) -> Self {
        match magic {
            // Neo N3 Mainnet magic number
            860833102 => NetworkType::N3Mainnet,
            // Neo N3 Testnet magic number
            894710606 => NetworkType::N3Testnet,
            // Placeholder for NeoX
            _ => NetworkType::NeoX,
        }
    }
    
    /// Convert Neo3 Network enum to our NetworkType
    pub fn from_network(network: Network) -> Self {
        match network {
            Network::Mainnet => NetworkType::N3Mainnet,
            Network::Testnet => NetworkType::N3Testnet,
            _ => NetworkType::NeoX, // All other networks treated as NeoX for now
        }
    }
    
    /// Convert to Neo3 Network enum
    pub fn to_network(&self) -> Network {
        match self {
            NetworkType::N3Mainnet => Network::Mainnet,
            NetworkType::N3Testnet => Network::Testnet,
            NetworkType::NeoX => Network::Mainnet, // Placeholder until Neo X is officially supported
        }
    }
}

/// Load wallet from file
pub async fn load_wallet(wallet_path: &PathBuf, password: Option<&str>) -> Result<Wallet, CliError> {
    // Check if the wallet file exists
    if !wallet_path.exists() {
        return Err(CliError::Wallet(format!("Wallet file not found: {}", wallet_path.display())));
    }
    
    // Open wallet with or without password
    let wallet = match password {
        Some(pwd) => Wallet::open_wallet(wallet_path, pwd)
            .map_err(|e| CliError::Wallet(format!("Failed to open wallet: {}", e)))?,
        None => {
            // Read wallet file without password
            let wallet_json = std::fs::read_to_string(wallet_path)
                .map_err(|e| CliError::Wallet(format!("Failed to read wallet file: {}", e)))?;
            
            // Parse wallet JSON
            serde_json::from_str(&wallet_json)
                .map_err(|e| CliError::Wallet(format!("Failed to parse wallet file: {}", e)))?
        }
    };
    
    Ok(wallet)
}

/// Prepare a CLI state from an existing state
pub fn prepare_state_from_existing(existing_state: &CliState) -> CliState {
    let mut new_state = CliState::default();
    
    // Copy over relevant state
    if let Some(wallet) = &existing_state.wallet {
        new_state.wallet = Some(wallet.clone());
    }
    
    if let Some(rpc_client) = &existing_state.rpc_client {
        new_state.rpc_client = Some(rpc_client.clone());
    }
    
    new_state.network_type = existing_state.network_type;
    
    new_state
}

/// Get token hash for a token symbol based on network type
pub fn get_token_address_for_network(token_symbol: &str, network_type: NetworkType) -> Option<ScriptHash> {
    // Uppercase the token symbol for consistent matching
    let token_symbol = token_symbol.to_uppercase();
    
    match network_type {
        NetworkType::N3Mainnet => {
            // Token addresses for Neo N3 Mainnet
            match token_symbol.as_str() {
                "NEO" => ScriptHash::from_str("ef4073a0f2b305a38ec4050e4d3d28bc40ea63f5").ok(),
                "GAS" => ScriptHash::from_str("d2a4cff31913016155e38e474a2c06d08be276cf").ok(),
                "FLM" => ScriptHash::from_str("f0151f528127558851b39c2cd8aa47da7418ab28").ok(),
                _ => None,
            }
        },
        NetworkType::N3Testnet => {
            // Token addresses for Neo N3 Testnet
            match token_symbol.as_str() {
                "NEO" => ScriptHash::from_str("0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5").ok(),
                "GAS" => ScriptHash::from_str("0xd2a4cff31913016155e38e474a2c06d08be276cf").ok(),
                _ => None,
            }
        },
        NetworkType::NeoX => {
            // Placeholder for NeoX token addresses
            // Will be updated when available
            None
        }
    }
}

/// Parse amount string to raw integer value based on token decimals
pub async fn parse_amount(amount: &str, token_hash: &ScriptHash, rpc_client: &RpcClient<HttpProvider>, network_type: NetworkType) -> Result<i64, CliError> {
    // Try to parse as a simple float first
    let amount_float = f64::from_str(amount)
        .map_err(|_| CliError::InvalidArgument(
            format!("Invalid amount: {}", amount),
            "Please provide a valid number".to_string()
        ))?;
        
    // Get token decimals
    let token_decimals = get_token_decimals(token_hash, rpc_client, network_type).await?;
    
    // Calculate raw amount (amount * 10^decimals)
    let multiplier = 10_f64.powi(token_decimals as i32);
    let raw_amount = (amount_float * multiplier).round() as i64;
    
    Ok(raw_amount)
}

/// Get decimals for a token
pub async fn get_token_decimals(token_hash: &ScriptHash, rpc_client: &RpcClient<HttpProvider>, network_type: NetworkType) -> Result<u8, CliError> {
    // Check for well-known tokens first
    match network_type {
        NetworkType::N3Mainnet | NetworkType::N3Testnet => {
            // Try to convert token_hash to string for comparison
            let token_hash_str = token_hash.to_string();
            
            // Check Neo and Gas based on ScriptHash for N3
            if token_hash_str == "ef4073a0f2b305a38ec4050e4d3d28bc40ea63f5" || 
               token_hash_str == "0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5" {
                return Ok(0); // NEO has 0 decimals
            } else if token_hash_str == "d2a4cff31913016155e38e474a2c06d08be276cf" || 
                      token_hash_str == "0xd2a4cff31913016155e38e474a2c06d08be276cf" {
                return Ok(8); // GAS has 8 decimals
            }
        },
        NetworkType::NeoX => {
            // Will be updated when NeoX info is available
        }
    }
    
    // Call the decimals method on the token contract
    match rpc_client.invoke_function(token_hash, "decimals".to_string(), vec![], None).await {
        Ok(result) => {
            if let Some(item) = result.stack.first() {
                match item {
                    StackItem::Integer { value } => {
                        // Convert to u8
                        value.to_u8().ok_or_else(|| {
                            CliError::InvalidInput(
                                format!("Invalid decimals value: {}. Expected a small integer for decimals", value)
                            )
                        })
                    },
                    _ => Err(CliError::InvalidInput(
                        format!("Unexpected stack item type for decimals: {:?}. Expected an integer.", item)
                    )),
                }
            } else {
                Err(CliError::InvalidInput(
                    "Empty stack response for decimals call. Token contract may not be valid.".to_string()
                ))
            }
        },
        Err(e) => Err(CliError::Rpc(format!("Failed to get token decimals: {}", e))),
    }
}

/// Format token amount with proper decimal places
pub fn format_token_amount(raw_amount: i64, decimals: u8) -> String {
    let divisor = 10_f64.powi(decimals as i32);
    let formatted_amount = (raw_amount as f64) / divisor;
    
    if decimals == 0 {
        return format!("{}", raw_amount);
    } else {
        return format!("{:.1$}", formatted_amount, decimals as usize);
    }
}

/// Resolve token symbol or address to a script hash
pub async fn resolve_token_to_scripthash_with_network(
    token: &str, 
    _rpc_client: &RpcClient<HttpProvider>, 
    network_type: NetworkType
) -> Result<ScriptHash, CliError> {
    // Check if the input is a valid script hash
    if let Ok(script_hash) = ScriptHash::from_str(token) {
        return Ok(script_hash);
    }
    
    // Check if it's a valid address
    match Address::from_str(token) {
        Ok(address) => {
            return address.address_to_script_hash()
                .map_err(|e| CliError::InvalidArgument(
                    format!("Invalid address: {}", e),
                    "Please provide a valid NEO address".to_string()
                ));
        },
        Err(_) => {}
    }
    
    // Check if it's a well-known token symbol
    if let Some(script_hash) = get_token_address_for_network(token, network_type) {
        return Ok(script_hash);
    }
    
    // If we get here, we couldn't resolve the token
    Err(CliError::InvalidArgument(
        format!("Could not resolve token: {}", token),
        "Please provide a valid token address, symbol, or contract hash".to_string()
    ))
}

/// Resolve token symbol or address to a string hash
pub async fn resolve_token_hash(
    token: &str,
    rpc_client: &RpcClient<HttpProvider>,
    network_type: NetworkType
) -> Result<String, CliError> {
    let script_hash = resolve_token_to_scripthash_with_network(token, rpc_client, network_type).await?;
    Ok(script_hash.to_string())
}
