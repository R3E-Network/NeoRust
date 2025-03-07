// Bridge operations for Neo N3 and Neo X
//
// This module provides commands for interacting with bridge contracts
// between Neo N3 and Neo X networks. It supports token transfers across
// both networks in a secure and user-friendly manner.

use std::str::FromStr;

use neo3::{
    neo_types::{contract_parameter::ContractParameter, h160::H160, script_hash::ScriptHash},
    neo_protocol::{
        account::{Account, AccountSigner},
        address::Address,
        signer::Signer,
    },
    neo_transact::{call_flags::CallFlags, script_builder::ScriptBuilder},
    neo_providers::{http::HttpProvider, RpcClient},
};

use crate::{
    cli::CliState,
    commands::defi::{
        network_validator::{validate_address_for_network, NetworkTypeCli},
        utils::{parse_amount, resolve_token_to_scripthash_with_network},
    },
    constants::{tokens as token_constants, contracts as contract_constants},
    error::CliError,
};

// Import helper utilities
use super::utils::{
    print_success, print_info, print_error, 
    prompt_yes_no, prompt_password, ensure_account_loaded,
    network_type_from_state, validate_address,
};

/// Handle bridge commands
///
/// This function serves as the main entry point for bridge operations
/// in the modular DeFi architecture.
///
/// # Arguments
/// * `cmd` - Bridge subcommand to execute
/// * `args` - Command arguments
/// * `state` - CLI state containing wallet and RPC client
///
/// # Returns
/// * `Result<(), CliError>` - Success or error
pub async fn handle_bridge_command(
    cmd: &str,
    args: &[String],
    state: &mut CliState,
) -> Result<(), CliError> {
    match cmd {
        "deposit" | "d" => {
            if args.len() < 3 {
                return Err(CliError::MissingArgument(
                    "token symbol, recipient address, and amount required".to_string(),
                ));
            }
            bridge_deposit(&args[0], &args[1], &args[2], state).await
        },
        "withdraw" | "w" => {
            if args.len() < 3 {
                return Err(CliError::MissingArgument(
                    "token symbol, recipient address, and amount required".to_string(),
                ));
            }
            bridge_withdraw(&args[0], &args[1], &args[2], state).await
        },
        "info" | "i" => {
            list_bridge_info(state).await
        },
        _ => Err(CliError::InvalidCommand(format!("Unknown bridge command: {}", cmd))),
    }
}

/// Bridge deposit (from Neo N3 to Neo X)
///
/// This function handles depositing tokens from Neo N3 to Neo X through
/// the bridge contract.
///
/// # Arguments
/// * `token` - Token symbol or contract address
/// * `recipient_address` - Recipient address on Neo X
/// * `amount` - Amount to deposit
/// * `state` - CLI state
///
/// # Returns
/// * `Result<(), CliError>` - Success or error
pub async fn bridge_deposit(
    token: &str,
    recipient_address: &str,
    amount: &str,
    state: &mut CliState,
) -> Result<(), CliError> {
    // Validate current network is Neo N3
    let network_type = network_type_from_state(state);
    if !network_type.is_neo_n3() {
        return Err(CliError::InvalidNetwork(
            "Bridge deposit must be performed from a Neo N3 network".to_string(),
        ));
    }

    // Validate recipient address format for Neo X
    let target_network = if network_type.is_testnet() {
        NetworkTypeCli::NeoXTestNet
    } else {
        NetworkTypeCli::NeoXMainNet
    };
    
    validate_address_for_network(recipient_address, target_network)?;
    
    // Load account
    let account = ensure_account_loaded(state)?;
    let rpc_client = state.get_rpc_client()?;
    
    // Resolve token to script hash
    let token_hash = resolve_token_to_scripthash_with_network(
        token, 
        &rpc_client, 
        network_type
    ).await?;
    
    // Get bridge contract hash based on network
    let bridge_contract_hash = if network_type.is_testnet() {
        ScriptHash::from_str(&contract_constants::NEO_N3_TESTNET_BRIDGE_CONTRACT)
            .map_err(|_| CliError::Contract("Failed to parse bridge contract hash".to_string()))?
    } else {
        ScriptHash::from_str(&contract_constants::NEO_N3_MAINNET_BRIDGE_CONTRACT)
            .map_err(|_| CliError::Contract("Failed to parse bridge contract hash".to_string()))?
    };
    
    // Parse amount
    let raw_amount = parse_amount(amount, &token_hash, &rpc_client, network_type).await?;
    
    // Convert recipient address to script hash
    let recipient_script_hash = Address::from_str(recipient_address)
        .map_err(|_| CliError::Wallet(format!("Invalid recipient address: {}", recipient_address)))?
        .address_to_script_hash()
        .map_err(|e| CliError::Wallet(format!("Failed to convert address to script hash: {}", e)))?;
    
    // Create parameters for the bridge deposit
    let params = vec![
        ContractParameter::hash160(token_hash),
        ContractParameter::hash160(recipient_script_hash),
        ContractParameter::integer(raw_amount),
    ];
    
    // Confirm with user
    print_info(&format!(
        "You are about to deposit {} {} from Neo N3 to Neo X",
        amount, token
    ));
    print_info(&format!("Recipient: {}", recipient_address));
    
    if !prompt_yes_no("Proceed with deposit?") {
        return Ok(());
    }
    
    // Get password and decrypt account
    let password = prompt_password("Enter wallet password: ")?;
    let decrypted_account = account.decrypt(&password)
        .map_err(|_| CliError::Wallet("Failed to decrypt account".to_string()))?;
    
    // Build and invoke script
    let signer = AccountSigner::from_account(decrypted_account);
    let script = ScriptBuilder::build_contract_call(
        bridge_contract_hash, 
        "deposit", 
        params,
        CallFlags::All,
    );
    
    print_info("Submitting bridge deposit transaction...");
    
    // Invoke the script
    let result = rpc_client
        .invoke_script_with_signer(&script, &signer)
        .await
        .map_err(|e| CliError::Transaction(format!("Failed to execute bridge deposit: {}", e)))?;
    
    print_success(&format!(
        "Bridge deposit transaction submitted successfully!\nTransaction hash: 0x{}",
        result
    ));
    
    Ok(())
}

/// Bridge withdraw (from Neo X to Neo N3)
///
/// This function handles withdrawing tokens from Neo X to Neo N3 through
/// the bridge contract.
///
/// # Arguments
/// * `token` - Token symbol or contract address
/// * `recipient_address` - Recipient address on Neo N3
/// * `amount` - Amount to withdraw
/// * `state` - CLI state
///
/// # Returns
/// * `Result<(), CliError>` - Success or error
pub async fn bridge_withdraw(
    token: &str,
    recipient_address: &str,
    amount: &str,
    state: &mut CliState,
) -> Result<(), CliError> {
    // Validate current network is Neo X
    let network_type = network_type_from_state(state);
    if !network_type.is_neox() {
        return Err(CliError::InvalidNetwork(
            "Bridge withdrawal must be performed from a Neo X network".to_string(),
        ));
    }

    // Validate recipient address format for Neo N3
    let target_network = if network_type.is_testnet() {
        NetworkTypeCli::NeoN3TestNet
    } else {
        NetworkTypeCli::NeoN3MainNet
    };
    
    validate_address_for_network(recipient_address, target_network)?;
    
    // Load account
    let account = ensure_account_loaded(state)?;
    let rpc_client = state.get_rpc_client()?;
    
    // Resolve token to script hash
    let token_hash = resolve_token_to_scripthash_with_network(
        token, 
        &rpc_client, 
        network_type
    ).await?;
    
    // Get bridge contract hash based on network
    let bridge_contract_hash = if network_type.is_testnet() {
        ScriptHash::from_str(&contract_constants::NEO_X_TESTNET_BRIDGE_CONTRACT)
            .map_err(|_| CliError::Contract("Failed to parse bridge contract hash".to_string()))?
    } else {
        ScriptHash::from_str(&contract_constants::NEO_X_MAINNET_BRIDGE_CONTRACT)
            .map_err(|_| CliError::Contract("Failed to parse bridge contract hash".to_string()))?
    };
    
    // Parse amount
    let raw_amount = parse_amount(amount, &token_hash, &rpc_client, network_type).await?;
    
    // Convert recipient address to script hash
    let recipient_script_hash = Address::from_str(recipient_address)
        .map_err(|_| CliError::Wallet(format!("Invalid recipient address: {}", recipient_address)))?
        .address_to_script_hash()
        .map_err(|e| CliError::Wallet(format!("Failed to convert address to script hash: {}", e)))?;
    
    // Create parameters for the bridge withdrawal
    let params = vec![
        ContractParameter::hash160(token_hash),
        ContractParameter::hash160(recipient_script_hash),
        ContractParameter::integer(raw_amount),
    ];
    
    // Confirm with user
    print_info(&format!(
        "You are about to withdraw {} {} from Neo X to Neo N3",
        amount, token
    ));
    print_info(&format!("Recipient: {}", recipient_address));
    
    if !prompt_yes_no("Proceed with withdrawal?") {
        return Ok(());
    }
    
    // Get password and decrypt account
    let password = prompt_password("Enter wallet password: ")?;
    let decrypted_account = account.decrypt(&password)
        .map_err(|_| CliError::Wallet("Failed to decrypt account".to_string()))?;
    
    // Build and invoke script
    let signer = AccountSigner::from_account(decrypted_account);
    let script = ScriptBuilder::build_contract_call(
        bridge_contract_hash, 
        "withdraw", 
        params,
        CallFlags::All,
    );
    
    print_info("Submitting bridge withdrawal transaction...");
    
    // Invoke the script
    let result = rpc_client
        .invoke_script_with_signer(&script, &signer)
        .await
        .map_err(|e| CliError::Transaction(format!("Failed to execute bridge withdrawal: {}", e)))?;
    
    print_success(&format!(
        "Bridge withdrawal transaction submitted successfully!\nTransaction hash: 0x{}",
        result
    ));
    
    Ok(())
}

/// Display bridge information
///
/// Shows information about the bridge contracts and supported tokens
///
/// # Arguments
/// * `state` - CLI state
///
/// # Returns
/// * `Result<(), CliError>` - Success or error
pub async fn list_bridge_info(
    state: &CliState,
) -> Result<(), CliError> {
    let network_type = network_type_from_state(state);
    
    print_info("\n==== Neo Bridge Information ====");
    
    // Display current network
    if network_type.is_neo_n3() {
        print_info("Current Network: Neo N3");
        if network_type.is_testnet() {
            print_info("Network: TestNet");
        } else {
            print_info("Network: MainNet");
        }
        
        // Display bridge contract address
        print_info("\nBridge Contract:");
        if network_type.is_testnet() {
            print_info(&format!("  Address: {}", contract_constants::NEO_N3_TESTNET_BRIDGE_CONTRACT));
        } else {
            print_info(&format!("  Address: {}", contract_constants::NEO_N3_MAINNET_BRIDGE_CONTRACT));
        }
        
        // Display available operations
        print_info("\nAvailable Operations:");
        print_info("  - deposit: Transfer tokens from Neo N3 to Neo X");
        
        // Display supported tokens
        print_info("\nSupported Tokens for Bridge:");
        print_info("  - NEO");
        print_info("  - GAS");
        print_info("  - FLM (Flamingo)");
        
        // Display command examples
        print_info("\nCommand Examples:");
        print_info("  bridge deposit NEO NXNeoXAddressHere 10");
        print_info("  bridge deposit GAS NXNeoXAddressHere 5.5");
    } else if network_type.is_neox() {
        print_info("Current Network: Neo X");
        if network_type.is_testnet() {
            print_info("Network: TestNet");
        } else {
            print_info("Network: MainNet");
        }
        
        // Display bridge contract address
        print_info("\nBridge Contract:");
        if network_type.is_testnet() {
            print_info(&format!("  Address: {}", contract_constants::NEO_X_TESTNET_BRIDGE_CONTRACT));
        } else {
            print_info(&format!("  Address: {}", contract_constants::NEO_X_MAINNET_BRIDGE_CONTRACT));
        }
        
        // Display available operations
        print_info("\nAvailable Operations:");
        print_info("  - withdraw: Transfer tokens from Neo X to Neo N3");
        
        // Display supported tokens
        print_info("\nSupported Tokens for Bridge:");
        print_info("  - NEO");
        print_info("  - GAS");
        print_info("  - xFLM (Neo X Flamingo)");
        
        // Display command examples
        print_info("\nCommand Examples:");
        print_info("  bridge withdraw NEO NeoN3AddressHere 10");
        print_info("  bridge withdraw GAS NeoN3AddressHere 5.5");
    }
    
    print_info("\nBridge Fees:");
    print_info("  - NEO: 0 (No fee)");
    print_info("  - GAS: 0.1 GAS");
    print_info("  - Other tokens: Varies by token");
    
    print_info("\nProcessing Time:");
    print_info("  - Typically 1-2 minutes for confirmation");
    print_info("  - May take longer during network congestion");
    
    Ok(())
}
