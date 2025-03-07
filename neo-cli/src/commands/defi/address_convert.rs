// Neo X Address Conversion Utilities
//
// This module provides utilities to convert between Neo N3 and Neo X address formats.

use crate::{
    commands::wallet::CliState,
    errors::CliError,
    utils::{print_info, print_success, print_warning},
};
use clap::Args;
use neo3::{
    neo_types::{Address, AddressExtension, ScriptHash},
    networks::NeoAddressFormat,
};
use neo_x::evm::utils::script_hash_to_evm_address;
use std::str::FromStr;

#[derive(Args, Debug, Clone)]
pub struct AddressConvertArgs {
    /// Address to convert (Neo N3 or Neo X format)
    pub address: String,
}

/// Convert between Neo N3 and Neo X address formats
pub async fn handle_address_convert(args: AddressConvertArgs, state: &mut CliState) -> Result<(), CliError> {
    let address_str = &args.address;
    
    // Try to parse as a Neo N3 address
    match Address::from_str(address_str) {
        Ok(n3_address) => {
            print_info(&format!("Detected Neo N3 address: {}", n3_address.to_string()));
            
            // Convert to script hash
            let script_hash = n3_address.address_to_script_hash().map_err(|e| {
                CliError::InvalidArgument(
                    format!("Failed to convert address to script hash: {}", e),
                    "Please provide a valid Neo N3 address".to_string(),
                )
            })?;
            
            // Convert to Neo X address (EVM format)
            let evm_address = script_hash_to_evm_address(&script_hash);
            print_success(&format!("Equivalent Neo X address: {}", evm_address));
            
            Ok(())
        },
        Err(_) => {
            // Try to parse as an EVM address
            if address_str.starts_with("0x") && address_str.len() == 42 {
                print_info(&format!("Detected Neo X address: {}", address_str));
                
                // Convert from hex string to bytes
                let address_bytes = hex::decode(&address_str[2..]).map_err(|e| {
                    CliError::InvalidArgument(
                        format!("Failed to decode EVM address: {}", e),
                        "Please provide a valid Neo X address".to_string(),
                    )
                })?;
                
                // Create a script hash from the address bytes (reversed)
                let mut script_hash_bytes = [0u8; 20];
                script_hash_bytes.copy_from_slice(&address_bytes);
                
                let script_hash = ScriptHash::from(script_hash_bytes);
                
                // Create Neo N3 address from script hash
                let network_magic = if state.network_type.to_lowercase().contains("test") {
                    neo3::networks::NEO_TESTNET
                } else {
                    neo3::networks::NEO_MAINNET
                };
                
                let n3_address = Address::from_script_hash(&script_hash, network_magic);
                print_success(&format!("Equivalent Neo N3 address: {}", n3_address.to_string()));
                
                Ok(())
            } else {
                Err(CliError::InvalidArgument(
                    "Invalid address format".to_string(),
                    "Please provide either a valid Neo N3 address or Neo X address (0x...)".to_string(),
                ))
            }
        }
    }
}
