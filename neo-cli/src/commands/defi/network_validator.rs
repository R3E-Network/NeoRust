//! Neo Network Validator Module
//!
//! This module provides utilities to validate addresses and network connections
//! across Neo N3 and Neo X networks. It ensures operations are performed on the
//! correct network with valid address formats, helping prevent common errors
//! when transferring tokens between networks.
//!
//! # Features
//!
//! * Validation of address formats for both Neo N3 and Neo X
//! * Network compatibility checking for bridge operations
//! * Bridge limits and fee retrieval
//! * Cross-network address validation

use crate::{
    commands::wallet::CliState,
    errors::CliError,
    utils::{print_info, print_success, print_warning},
};
use neo3::{
    neo_clients::{HttpProvider, RpcClient},
    neo_types::{Address, ScriptHash},
};
use neo_x::evm::utils::{is_valid_evm_address, script_hash_to_evm_address};
use std::str::FromStr;
use super::utils::NetworkTypeCli;

/// Validates whether an address is appropriate for a given network type
///
/// This function checks if the provided address is in the correct format for the
/// specified network type. It can also identify when an address from one network
/// is being incorrectly used on another network.
///
/// # Arguments
///
/// * `address` - The address string to validate
/// * `network_type` - The network type (MainNet, TestNet, or NeoX) to validate against
///
/// # Returns
///
/// * `Ok(true)` - Address is valid for the specified network
/// * `Ok(false)` - Address is valid but for a different network
/// * `Err(CliError)` - Address is invalid for any supported network
pub fn validate_address_for_network(
    address: &str,
    network_type: NetworkTypeCli,
) -> Result<bool, CliError> {
    if network_type.is_neo_n3() {
        // Validate Neo N3 address
        match Address::from_str(address) {
            Ok(_) => Ok(true),
            Err(_) => {
                // Check if it's a Neo X address being used on Neo N3
                if is_valid_evm_address(address) {
                    Ok(false) // It's a Neo X address being used on Neo N3
                } else {
                    Err(CliError::InvalidArgument(
                        format!("Invalid address: {}", address),
                        "Please provide a valid Neo N3 address".to_string(),
                    ))
                }
            }
        }
    } else if network_type.is_neox() {
        // Validate Neo X address (EVM format)
        if is_valid_evm_address(address) {
            Ok(true)
        } else {
            // Check if it's a Neo N3 address being used on Neo X
            match Address::from_str(address) {
                Ok(_) => Ok(false), // It's a Neo N3 address being used on Neo X
                Err(_) => Err(CliError::InvalidArgument(
                    format!("Invalid address: {}", address),
                    "Please provide a valid Neo X address (0x...)".to_string(),
                )),
            }
        }
    } else {
        // Unknown network type
        Err(CliError::InvalidArgument(
            format!("Unknown network type: {:?}", network_type),
            "Please use a valid network type".to_string(),
        ))
    }
}

/// Performs a network compatibility check for bridge operations
///
/// Ensures that bridge operations are being executed on the correct network type.
/// Bridge operations must be initiated from Neo N3 networks rather than Neo X.
///
/// # Arguments
///
/// * `state` - The CLI state containing network connection information
///
/// # Returns
///
/// * `Ok(())` - Network is compatible with bridge operations
/// * `Err(CliError)` - Network is incompatible, with specific error message
pub async fn check_bridge_network_compatibility(
    state: &CliState,
) -> Result<(), CliError> {
    let network_type_str = state.get_network_type_string();
    let network_type = NetworkTypeCli::from_network_string(&network_type_str);
    
    // Bridge must be accessed from Neo N3
    if network_type.is_neox() {
        return Err(CliError::InvalidOperation(
            "Cannot perform bridge operations from Neo X network".to_string(),
            "Please connect to a Neo N3 network (mainnet or testnet)".to_string(),
        ));
    }
    
    // Check if RPC client is connected
    state.get_rpc_client()?;
    
    Ok(())
}

/// Validates a destination address for bridge operations
///
/// Ensures the destination address is in the correct format for the target network.
/// For bridge operations, the destination network is always the opposite of the origin.
///
/// # Arguments
///
/// * `destination` - The destination address string
/// * `origin_network` - The originating network type
///
/// # Returns
///
/// * `Ok(())` - Destination address is valid for the target network
/// * `Err(CliError)` - Destination address is invalid, with specific error message
pub fn validate_bridge_destination(
    destination: &str,
    origin_network: NetworkTypeCli,
) -> Result<(), CliError> {
    // For bridge operations, the destination network is always the opposite type of origin
    let destination_network = if origin_network.is_neo_n3() {
        // If coming from Neo N3, destination should be Neo X
        // Preserve testnet/mainnet distinction
        if origin_network.is_mainnet() {
            NetworkTypeCli::NeoXMain
        } else {
            NetworkTypeCli::NeoXTest
        }
    } else {
        // If coming from Neo X, destination should be Neo N3
        // Preserve testnet/mainnet distinction
        if origin_network.is_mainnet() {
            NetworkTypeCli::MainNet
        } else {
            NetworkTypeCli::TestNet
        }
    };
    
    match validate_address_for_network(destination, destination_network) {
        Ok(true) => Ok(()), // Address is valid for destination network
        Ok(false) => Err(CliError::InvalidArgument(
            format!("Destination address {} is not valid for the target network", destination),
            format!("Please provide a valid {} address", 
                if destination_network.is_neox() {
                    "Neo X (0x...)"
                } else {
                    "Neo N3"
                }
            ),
        )),
        Err(e) => Err(e),
    }
}

/// Retrieves fees and capacity limits for token bridge operations
///
/// Connects to the appropriate bridge contract based on the network type
/// and retrieves the current fee and capacity limit for the specified token.
///
/// # Arguments
///
/// * `token_hash` - The script hash of the token
/// * `rpc_client` - The RPC client connected to the Neo network
/// * `is_testnet` - Whether we're operating on testnet
///
/// # Returns
///
/// * `Ok(BridgeLimits)` - The fee and capacity limits for the token
/// * `Err(CliError)` - Failed to retrieve limits, with specific error message
pub async fn get_bridge_limits_for_token(
    token_hash: &ScriptHash,
    rpc_client: &RpcClient<HttpProvider>,
    is_testnet: bool,
) -> Result<BridgeLimits, CliError> {
    // Create bridge contract instance
    use neo_x::bridge::bridge_contract::NeoXBridgeContract;
    
    // Use appropriate bridge contract based on network
    let bridge_contract = if is_testnet {
        NeoXBridgeContract::new(Some(rpc_client), true)
            .map_err(|e| CliError::ContractError(format!("Failed to create bridge contract: {}", e)))?
    } else {
        NeoXBridgeContract::new(Some(rpc_client), false)
            .map_err(|e| CliError::ContractError(format!("Failed to create bridge contract: {}", e)))?
    };
    
    // Get fee
    let fee = bridge_contract.get_fee(token_hash)
        .await
        .map_err(|e| CliError::ContractError(format!("Failed to get bridge fee: {}", e)))?;
    
    // Get cap
    let cap = bridge_contract.get_cap(token_hash)
        .await
        .map_err(|e| CliError::ContractError(format!("Failed to get bridge capacity: {}", e)))?;
    
    Ok(BridgeLimits { fee, cap })
}

/// Structure holding bridge fee and capacity information for a token
///
/// Contains the current fee required for bridging operations and the
/// maximum capacity the bridge supports for the token.
///
/// # Fields
///
/// * `fee` - The fee amount in raw units (not adjusted for decimals)
/// * `cap` - The capacity limit in raw units (not adjusted for decimals)
pub struct BridgeLimits {
    pub fee: i64,
    pub cap: i64,
}
