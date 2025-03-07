// Token operations for the Neo CLI (modular implementation)
//
// This module provides commands for interacting with NEP-17 tokens on both 
// Neo N3 and Neo X blockchains in a network-aware manner.
//
// It leverages the TokenHandlerFactory pattern to automatically select
// the appropriate implementation based on network type.

use std::str::FromStr;

use neo3::{
    neo_types::{contract_parameter::ContractParameter, h160::H160, script_hash::ScriptHash},
    neo_protocol::{
        account::{Account, AccountSigner},
        address::Address,
        signer::Signer,
    },
    neo_transact::{call_flags::CallFlags, script_builder::ScriptBuilder},
    neo_crypto::keys::Secp256r1PublicKey,
    neo_providers::{http::HttpProvider, RpcClient},
};
use base64::engine::general_purpose;

use crate::{
    cli::CliState,
    commands::defi::{
        tokens::TokenHandlerFactory,
        network_validator::{validate_address_for_network, NetworkTypeCli},
        utils::{
            get_token_decimals, parse_amount, resolve_token_to_scripthash_with_network,
            network_type_from_state, format_token_amount,
        },
    },
    constants::{tokens as token_constants, contracts as contract_constants},
    error::CliError,
};

// Import helper utilities
use super::utils::{
    print_success, print_info, print_error, 
    prompt_yes_no, prompt_password, ensure_account_loaded,
};

/// Handle token commands using the factory pattern to select appropriate implementations
///
/// This function serves as the main entry point for token operations in the modular
/// DeFi architecture.
///
/// # Arguments
/// * `cmd` - Token subcommand to execute
/// * `args` - Command arguments
/// * `state` - CLI state containing wallet and RPC client
///
/// # Returns
/// * `Result<(), CliError>` - Success or error
pub async fn handle_token_command(
    cmd: &str,
    args: &[String],
    state: &mut CliState,
) -> Result<(), CliError> {
    // Display network information if in verbose mode
    if state.verbose {
        match &state.network_type {
            Some(network) => {
                print_info(&format!("Connected to network: {}", network));
                if state.is_neo_x() {
                    print_info("Network type: Neo X");
                } else {
                    print_info("Network type: Neo N3");
                }
            },
            None => print_info("No network selected"),
        }
    }

    // Use the TokenHandlerFactory to route to the appropriate implementation
    match cmd {
        "info" | "i" => {
            if args.is_empty() {
                return Err(CliError::MissingArgument("token contract or symbol".to_string()));
            }
            TokenHandlerFactory::get_token_info(&args[0], state).await
        },
        "balance" | "b" => {
            if args.len() < 2 {
                return Err(CliError::MissingArgument(
                    "token contract/symbol and address required".to_string(),
                ));
            }
            TokenHandlerFactory::get_token_balance(&args[0], &args[1], state).await
        },
        "transfer" | "t" => {
            if args.len() < 3 {
                return Err(CliError::MissingArgument(
                    "token contract/symbol, recipient address, and amount required".to_string(),
                ));
            }
            TokenHandlerFactory::transfer_token(&args[0], &args[1], &args[2], state).await
        },
        "list" => {
            list_available_tokens(state)
        },
        _ => Err(CliError::InvalidCommand(format!("Unknown token command: {}", cmd))),
    }
}

/// List all available tokens on the current network
///
/// This function displays all tokens defined in the constants
/// that are available on the current network.
///
/// # Arguments
/// * `state` - CLI state containing wallet and RPC client
///
/// # Returns
/// * `Result<(), CliError>` - Success or error
fn list_available_tokens(state: &CliState) -> Result<(), CliError> {
    let network_type = network_type_from_state(state);
    
    print_info(&format!(
        "Available tokens on {} ({})",
        if network_type.is_neo_n3() { "Neo N3" } else { "Neo X" },
        if network_type.is_testnet() { "TestNet" } else { "MainNet" }
    ));
    
    // Get token list based on network type
    if network_type.is_neo_n3() {
        if network_type.is_testnet() {
            // Neo N3 TestNet tokens
            print_info("Native tokens:");
            print_info("  - NEO");
            print_info("  - GAS");
            
            print_info("\nStandard tokens:");
            print_info("  - FLM (Flamingo)");
            print_info("  - TEST (TestNet token)");
        } else {
            // Neo N3 MainNet tokens
            print_info("Native tokens:");
            print_info("  - NEO");
            print_info("  - GAS");
            
            print_info("\nStandard tokens:");
            print_info("  - FLM (Flamingo)");
            print_info("  - bNEO (Binance-Peg NEO Token)");
        }
    } else if network_type.is_neox() {
        if network_type.is_testnet() {
            // Neo X TestNet tokens
            print_info("Native tokens:");
            print_info("  - NEO");
            print_info("  - GAS");
            
            print_info("\nStandard tokens:");
            print_info("  - xFLM (Neo X Flamingo)");
            print_info("  - xTEST (Neo X TestNet token)");
        } else {
            // Neo X MainNet tokens
            print_info("Native tokens:");
            print_info("  - NEO");
            print_info("  - GAS");
            
            print_info("\nStandard tokens:");
            print_info("  - xFLM (Neo X Flamingo)");
        }
    }
    
    print_info("\nUse the 'token info <symbol>' command to get details about a specific token.");
    Ok(())
}

/// Helper function to retrieve the appropriate token address for the current network
///
/// This function encapsulates the token resolution logic in a reusable way.
///
/// # Arguments
/// * `token` - Token symbol or address
/// * `state` - CLI state containing wallet and RPC client
///
/// # Returns
/// * `Result<String, CliError>` - Token address or error
pub async fn resolve_token_address(
    token: &str,
    state: &mut CliState,
) -> Result<String, CliError> {
    let network_type = network_type_from_state(state);
    let rpc_client = state.get_rpc_client()?;
    
    // Resolve token to script hash
    let token_hash = resolve_token_to_scripthash_with_network(
        token, 
        rpc_client, 
        network_type
    ).await?;
    
    Ok(token_hash.to_string())
}
