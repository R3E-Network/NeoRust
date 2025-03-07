//! Network Status Command Module
//!
//! This module provides comprehensive network diagnostics for checking the status of network
//! connections and token compatibility across Neo N3 and Neo X networks. It helps users verify
//! their network setup and ensure tokens are properly configured for cross-network operations.
//!
//! # Features
//!
//! * Checks current network connection status
//! * Verifies token support on the current network
//! * Tests cross-network token compatibility
//! * Validates bridge contract availability and status
//! * Performs connectivity tests to both Neo N3 and Neo X networks

use crate::{
    commands::wallet::CliState,
    errors::CliError,
    utils::{print_error, print_info, print_success, print_warning},
};
use neo3::{
    neo_clients::{HttpProvider, RpcClient},
    neo_types::ScriptHash,
};
use neo_x::evm::provider::NeoXProvider;
use std::str::FromStr;
use super::utils::{NetworkTypeCli, get_token_address_for_network, format_token_amount, get_token_decimals};

/// Performs a comprehensive network status check and token compatibility test
///
/// This function provides detailed diagnostics about the current network connection,
/// token support across networks, bridge contract status, and cross-network compatibility.
/// It serves as a troubleshooting tool for users working with both Neo N3 and Neo X.
///
/// # Arguments
///
/// * `state` - The CLI state containing network connection information
///
/// # Returns
///
/// * `Ok(())` - Status check completed successfully (even if issues were detected)
/// * `Err(CliError)` - Status check failed due to connection or other critical errors
///
/// # Example Output
///
/// ```text
/// Checking network status for: testnet
/// ✓ Successfully connected to testnet
/// Current block height: 123456
///
/// Checking token support on current network:
/// ✓ NEO is supported on testnet (0x..)
///   Decimals: 0
///   Example: 123456789 raw units = 123456789 NEO
/// ✓ GAS is supported on testnet (0x..)
///   Decimals: 8
///   Example: 123456789 raw units = 1.23456789 GAS
/// ```
pub async fn check_network_status(state: &mut CliState) -> Result<(), CliError> {
    // Get current network info
    let network_type_str = state.get_network_type_string();
    let network_type = NetworkTypeCli::from_network_string(&network_type_str);
    let is_testnet = network_type_str.to_lowercase().contains("test");
    let is_neox = state.is_neo_x();
    
    // Get RPC client
    let rpc_client = state.get_rpc_client()?;
    
    print_info(&format!("Checking network status for: {}", network_type_str));
    
    // Check current network connection
    match rpc_client.get_block_count().await {
        Ok(block_count) => {
            print_success(&format!("Successfully connected to {}", network_type_str));
            print_info(&format!("Current block height: {}", block_count));
        },
        Err(e) => {
            print_error(&format!("Failed to connect to {}: {}", network_type_str, e));
            return Err(CliError::Network(format!("Connection failed: {}", e)));
        }
    }
    
    // Check token support on current network
    print_info("\nChecking token support on current network:");
    
    // List of tokens to check (add more as needed)
    let tokens = vec!["NEO", "GAS", "NEOX"];
    
    for token in &tokens {
        match get_token_address_for_network(token, network_type) {
            Some(hash) => {
                print_success(&format!("✓ {} is supported on {} ({})", token, network_type_str, hash));
                
                // Get token decimals
                match get_token_decimals(&hash, rpc_client, network_type).await {
                    Ok(decimals) => {
                        print_info(&format!("  Decimals: {}", decimals));
                        
                        // Example amount converted
                        let example_amount = 123_456_789;
                        let formatted = format_token_amount(example_amount, decimals);
                        print_info(&format!("  Example: {} raw units = {} {}", example_amount, formatted, token));
                    },
                    Err(e) => {
                        print_warning(&format!("  Could not get decimals: {}", e));
                    }
                }
            },
            None => {
                print_warning(&format!("✗ {} is not supported on {}", token, network_type_str));
            }
        }
    }
    
    // Check cross-network token compatibility
    print_info("\nChecking cross-network token compatibility:");
    
    let opposite_network = if is_neox {
        if is_testnet {
            NetworkTypeCli::TestNet
        } else {
            NetworkTypeCli::MainNet
        }
    } else {
        NetworkTypeCli::NeoX
    };
    
    let opposite_network_name = opposite_network.to_network_string();
    
    print_info(&format!("Cross-network compatibility with {}", opposite_network_name));
    
    for token in &tokens {
        let current_hash = get_token_address_for_network(token, network_type);
        let opposite_hash = get_token_address_for_network(token, opposite_network);
        
        match (current_hash, opposite_hash) {
            (Some(curr), Some(opp)) => {
                print_success(&format!(
                    "✓ {} is available on both networks:", token
                ));
                print_info(&format!("  {} address: {}", network_type_str, curr));
                print_info(&format!("  {} address: {}", opposite_network_name, opp));
            },
            (Some(_), None) => {
                print_warning(&format!(
                    "⚠ {} is available on {} but not on {}", 
                    token, network_type_str, opposite_network_name
                ));
            },
            (None, Some(_)) => {
                print_warning(&format!(
                    "⚠ {} is available on {} but not on {}", 
                    token, opposite_network_name, network_type_str
                ));
            },
            (None, None) => {
                print_warning(&format!(
                    "✗ {} is not available on either network", token
                ));
            }
        }
    }
    
    // Bridge status (if on Neo N3)
    if !is_neox {
        print_info("\nChecking bridge contract status:");
        
        // Get bridge contract hash
        match super::utils::get_bridge_contract_hash(network_type) {
            Ok(bridge_hash) => {
                print_success(&format!("Bridge contract found: {}", bridge_hash));
                
                // Try to get bridge status
                use neo_x::bridge::bridge_contract::NeoXBridgeContract;
                let bridge_contract = NeoXBridgeContract::with_script_hash(bridge_hash, Some(rpc_client));
                
                // Check if we can get the bridge fee for NEO (as a test)
                if let Some(neo_hash) = get_token_address_for_network("NEO", network_type) {
                    match bridge_contract.get_fee(&neo_hash).await {
                        Ok(fee) => {
                            // Get token decimals for NEO
                            if let Ok(decimals) = get_token_decimals(&neo_hash, rpc_client, network_type).await {
                                let formatted_fee = format_token_amount(fee, decimals);
                                print_success(&format!("Bridge is operational. Fee for NEO: {}", formatted_fee));
                            } else {
                                print_success(&format!("Bridge is operational. Fee for NEO: {} (raw units)", fee));
                            }
                        },
                        Err(e) => {
                            print_warning(&format!("Bridge contract call failed: {}", e));
                        }
                    }
                }
            },
            Err(e) => {
                print_warning(&format!("Bridge contract not found: {}", e));
            }
        }
    }
    
    // Try to check Neo X chain (if we're on Neo N3)
    if !is_neox {
        print_info("\nAttempting to check Neo X chain status:");
        
        // Create Neo X provider
        let neox_url = if is_testnet {
            "https://testnet.neo-x.io" // Example Neo X testnet URL
        } else {
            "https://mainnet.neo-x.io" // Example Neo X mainnet URL
        };
        
        let neox_provider = NeoXProvider::new(neox_url, None, is_testnet);
        
        // Try to get chain ID
        match neox_provider.chain_id().await {
            Ok(chain_id) => {
                print_success(&format!(
                    "Successfully connected to Neo X chain (Chain ID: {})", 
                    chain_id
                ));
                
                print_info("Bridge operations should be functional");
            },
            Err(e) => {
                print_warning(&format!(
                    "Could not connect to Neo X chain: {}", e
                ));
                print_info("You may need to update your RPC endpoint for Neo X");
            }
        }
    }
    
    // Summary
    print_info("\nNetwork Status Summary:");
    if is_neox {
        print_info("- You are currently connected to a Neo X network");
        print_info("- For bridge operations (withdrawal to Neo N3), use Neo X wallet interfaces");
    } else {
        print_info("- You are currently connected to a Neo N3 network");
        print_info("- You can use the bridge commands for deposit operations to Neo X");
    }
    
    Ok(())
}
