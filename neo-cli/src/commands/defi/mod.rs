// DeFi Module for Neo CLI
//
// This module contains commands for various DeFi operations on the Neo blockchain

mod utils;
mod tokens;
mod types;

// Re-export utility types and functions
pub use utils::{
    NetworkType,
    load_wallet,
    prepare_state_from_existing,
    get_token_address_for_network,
    resolve_token_to_scripthash_with_network,
    resolve_token_hash,
    parse_amount,
    format_token_amount,
    get_token_decimals,
};

// Re-export command types
pub use types::*;

use crate::errors::CliError;
use crate::commands::wallet::CliState;
use std::path::PathBuf;
use clap::Args;
use neo3::prelude::*;
use primitive_types::H160;
use std::str::FromStr;

/// DeFi operations on Neo blockchain
#[derive(Args, Debug, Clone)]
pub struct DefiArgs {
    /// Path to wallet file
    #[arg(short, long)]
    pub wallet: Option<PathBuf>,
    
    /// Wallet password
    #[arg(short, long)]
    pub password: Option<String>,
    
    #[clap(subcommand)]
    pub command: DefiCommands,
}

#[derive(clap::Subcommand, Debug, Clone)]
pub enum DefiCommands {
    /// Get token information
    Token {
        /// Token contract address or symbol
        contract: String,
    },
    
    /// Check token balance for an address
    Balance {
        /// Token contract address or symbol
        contract: String,
        
        /// Optional address to check balance for (defaults to wallet's address)
        address: Option<String>,
    },
    
    /// Transfer tokens to an address
    Transfer {
        /// Token contract address or symbol
        token: String,
        
        /// Destination address
        to: String,
        
        /// Amount to transfer
        amount: String,
        
        /// Optional data to include with the transfer
        data: Option<String>,
    },
}

// Add the create_h160_param function needed by contract.rs
pub fn create_h160_param(value: &str) -> Result<ContractParameter, CliError> {
    use neo3::prelude::*;
    use primitive_types::H160;
    use std::str::FromStr;
    
    // First try to parse as Neo address
    match Address::from_str(value) {
        Ok(address) => {
            match address.address_to_script_hash() {
                Ok(script_hash) => return Ok(ContractParameter::h160(&script_hash)),
                Err(e) => {
                    // Address format is valid but conversion failed
                    return Err(CliError::InvalidArgument(
                        format!("Invalid address: {}", e),
                        "Please provide a valid NEO address".to_string()
                    ));
                }
            }
        },
        Err(_) => {
            // Not an address, try other formats
        }
    }
    
    // Check if input is a valid script hash
    if let Ok(script_hash) = ScriptHash::from_str(value) {
        return Ok(ContractParameter::h160(&script_hash));
    }
    
    // Check if input is a valid H160
    if let Ok(h160) = H160::from_str(value.trim_start_matches("0x")) {
        let script_hash = ScriptHash::from(h160);
        return Ok(ContractParameter::h160(&script_hash));
    }
    
    Err(CliError::InvalidArgument(
        format!("Invalid address or script hash: {}", value),
        "Please provide a valid NEO address or script hash".to_string()
    ))
}

// Main entry point for handling DeFi commands
pub async fn handle_defi_command(args: DefiArgs, state: &mut CliState) -> Result<(), CliError> {
    match args.command {
        DefiCommands::Token { contract } => {
            tokens::handle_token_info(&contract, state).await
        },
        DefiCommands::Balance { contract, address } => {
            tokens::handle_token_balance(&contract, address.as_deref(), state).await
        },
        DefiCommands::Transfer { token, to, amount, data } => {
            tokens::handle_token_transfer(&token, &to, &amount, data.as_deref(), state).await
        },
    }
}
