// Neo X Bridge testing module for Neo CLI
//
// This module provides commands for testing Neo X Bridge connectivity and setup.

use crate::{
    commands::wallet::CliState,
    errors::CliError,
    utils::{print_info, print_success, print_warning},
};
use neo3::{
    neo_clients::{HttpProvider, RpcClient},
    neo_types::ScriptHash,
};
use neo_x::{
    bridge::bridge_contract::NeoXBridgeContract,
    evm::provider::NeoXProvider,
};
use std::str::FromStr;
use super::utils::{NetworkTypeCli, get_bridge_contract_hash, get_token_address_for_network};

/// Test bridge connectivity between Neo N3 and Neo X
pub async fn test_bridge_connectivity(state: &mut CliState) -> Result<(), CliError> {
    // Get current network info
    let network_type_str = state.get_network_type_string();
    let network_type = NetworkTypeCli::from_network_string(&network_type_str);
    let is_testnet = network_type_str.to_lowercase().contains("test");
    let is_neox = state.is_neo_x();
    
    print_info(&format!("Testing bridge connectivity for network: {}", network_type_str));
    
    // Get RPC client
    let rpc_client = state.get_rpc_client()?;
    
    if is_neox {
        // We're on Neo X network - test connectivity to Neo N3
        print_info("Currently connected to Neo X network");
        
        // Create Neo X provider
        let provider = NeoXProvider::new(
            rpc_client.get_provider().url(),
            Some(rpc_client),
            is_testnet
        );
        
        // Test chain ID retrieval
        match provider.chain_id().await {
            Ok(chain_id) => {
                print_success(&format!("Successfully connected to Neo X chain (Chain ID: {})", chain_id));
                
                // Determine correct N3 network based on current testnet status
                let n3_network = if is_testnet {
                    "Neo N3 TestNet"
                } else {
                    "Neo N3 MainNet"
                };
                
                print_info(&format!("To access the bridge from Neo X to {}, you'll need to:", n3_network));
                print_info("1. Use a Neo X compatible wallet");
                print_info("2. Interact with the bridge contract on Neo X");
                print_info("3. Specify a destination address on Neo N3");
            },
            Err(e) => {
                print_warning(&format!("Failed to connect to Neo X chain: {}", e));
                return Err(CliError::Network(format!("Neo X connection failed: {}", e)));
            }
        }
    } else {
        // We're on Neo N3 network - test bridge contract
        print_info("Currently connected to Neo N3 network");
        
        // Try to get bridge contract hash
        match get_bridge_contract_hash(network_type) {
            Ok(bridge_hash) => {
                print_success(&format!("Found bridge contract: {}", bridge_hash.to_string()));
                
                // Create bridge contract instance
                let bridge_contract = NeoXBridgeContract::with_script_hash(bridge_hash, Some(rpc_client));
                
                // Test supported tokens
                let tokens = vec!["NEO", "GAS"];
                
                for token_symbol in tokens {
                    // Resolve token hash
                    if let Some(token_hash) = get_token_address_for_network(token_symbol, network_type) {
                        print_info(&format!("Testing bridge capabilities for {}", token_symbol));
                        
                        // Test fee retrieval
                        match bridge_contract.get_fee(&token_hash).await {
                            Ok(fee) => {
                                print_success(&format!("Bridge fee for {}: {}", token_symbol, fee));
                            },
                            Err(e) => {
                                print_warning(&format!("Failed to get bridge fee for {}: {}", token_symbol, e));
                            }
                        }
                        
                        // Test cap retrieval
                        match bridge_contract.get_cap(&token_hash).await {
                            Ok(cap) => {
                                print_success(&format!("Bridge capacity for {}: {}", token_symbol, cap));
                            },
                            Err(e) => {
                                print_warning(&format!("Failed to get bridge capacity for {}: {}", token_symbol, e));
                            }
                        }
                    } else {
                        print_warning(&format!("Failed to resolve token hash for {}", token_symbol));
                    }
                }
                
                // Determine Neo X network based on current testnet status
                let neox_network = if is_testnet {
                    "Neo X TestNet"
                } else {
                    "Neo X MainNet"
                };
                
                // Provide bridge usage instructions
                print_info("\nTo use the bridge from Neo N3:");
                print_info(&format!("1. Connect to {} (current network)", network_type_str));
                print_info("2. Use the 'neo defi bridge deposit' command");
                print_info(&format!("3. Specify a destination address on {}", neox_network));
            },
            Err(e) => {
                print_warning(&format!("Failed to resolve bridge contract: {}", e));
                return Err(CliError::InvalidArgument(
                    "Bridge contract not found".to_string(),
                    "Make sure you're connected to a supported Neo N3 network".to_string(),
                ));
            }
        }
    }
    
    Ok(())
}
