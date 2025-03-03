// DeFi token operations for Neo CLI
//
// This module contains functions for handling token operations in the Neo CLI.

use crate::errors::CliError;
use crate::commands::wallet::CliState;
use crate::utils::{
    print_success, 
    print_info, 
    print_error, 
    ensure_account_loaded, 
    prompt_password,
    prompt_yes_no
};
use neo3::prelude::*;
use super::utils::{
    load_wallet,
    get_token_address_for_network, 
    resolve_token_to_scripthash_with_network,
    resolve_token_hash,
    get_token_decimals,
    parse_amount,
    format_token_amount,
    NetworkType
};
use std::str::FromStr;
use std::path::PathBuf;
use colored::*;
use primitive_types::H160;
use num_traits::ToPrimitive;
use base64;
use serde_json;
use neo3::builder::{CallFlags, ScriptBuilder, Signer};

/// Get token information
pub async fn handle_token_info(
    contract: &str,
    state: &mut CliState
) -> Result<(), CliError> {
    // Get RPC client
    let rpc_client = state.get_rpc_client()?;
    
    // Determine network type
    let network_type = network_type_from_state(state);
    
    // Resolve token address
    let token_hash = resolve_token_to_scripthash_with_network(
        contract, 
        &rpc_client, 
        network_type
    ).await?;
    
    print_info(&format!("Getting token information for: {}", contract));
    
    // Get token name
    match rpc_client.invoke_function(&token_hash, "name".to_string(), vec![], None).await {
        Ok(result) => {
            if let Some(item) = result.stack.first() {
                match item {
                    StackItem::ByteString { value } => {
                        if let Ok(bytes) = base64::decode(value.trim_end()) {
                            if let Ok(name) = String::from_utf8(bytes) {
                                print_info(&format!("Name: {}", name));
                            } else {
                                print_info("Name: <Invalid UTF-8>");
                            }
                        } else {
                            print_info("Name: <Invalid Base64>");
                        }
                    },
                    _ => print_info("Name: <Unknown format>"),
                }
            }
        },
        Err(e) => print_error(&format!("Failed to get token name: {}", e)),
    }
    
    // Get token symbol
    match rpc_client.invoke_function(&token_hash, "symbol".to_string(), vec![], None).await {
        Ok(result) => {
            if let Some(item) = result.stack.first() {
                match item {
                    StackItem::ByteString { value } => {
                        if let Ok(bytes) = base64::decode(value.trim_end()) {
                            if let Ok(symbol) = String::from_utf8(bytes) {
                                print_info(&format!("Symbol: {}", symbol));
                            } else {
                                print_info("Symbol: <Invalid UTF-8>");
                            }
                        } else {
                            print_info("Symbol: <Invalid Base64>");
                        }
                    },
                    _ => print_info("Symbol: <Unknown format>"),
                }
            }
        },
        Err(e) => print_error(&format!("Failed to get token symbol: {}", e)),
    }
    
    // Get decimals
    match get_token_decimals(&token_hash, &rpc_client, network_type).await {
        Ok(decimals) => {
            print_info(&format!("Decimals: {}", decimals));
        },
        Err(e) => print_error(&format!("Failed to get token decimals: {}", e)),
    }
    
    // Get total supply
    match rpc_client.invoke_function(&token_hash, "totalSupply".to_string(), vec![], None).await {
        Ok(result) => {
            if let Some(item) = result.stack.first() {
                match item {
                    StackItem::Integer { value } => {
                        // Get decimals for proper display
                        if let Ok(decimals) = get_token_decimals(&token_hash, &rpc_client, network_type).await {
                            let raw_supply = value.to_i64().unwrap_or(0);
                            let formatted_supply = format_token_amount(raw_supply, decimals);
                            print_info(&format!("Total Supply: {}", formatted_supply));
                        } else {
                            print_info(&format!("Total Supply: {}", value));
                        }
                    },
                    _ => print_info("Total Supply: <Unknown format>"),
                }
            }
        },
        Err(e) => print_error(&format!("Failed to get total supply: {}", e)),
    }
    
    // Display the contract hash
    print_info(&format!("Script Hash (Little-Endian): {}", token_hash));
    let address = token_hash.to_address();
    print_info(&format!("Contract Address: {}", address));
    
    Ok(())
}

/// Get token balance for an address
pub async fn handle_token_balance(
    contract: &str,
    address: Option<&str>,
    state: &mut CliState
) -> Result<(), CliError> {
    // Get RPC client
    let rpc_client = state.get_rpc_client()?;
    
    // Determine network type
    let network_type = network_type_from_state(state);
    
    // Resolve token address
    let token_hash = resolve_token_to_scripthash_with_network(
        contract, 
        &rpc_client, 
        network_type
    ).await?;
    
    // Resolve the address to check balance for
    let target_address = match address {
        Some(addr_str) => {
            // Check if the provided address is valid
            match Address::from_str(addr_str) {
                Ok(addr) => addr,
                Err(_) => return Err(CliError::InvalidArgument(
                    format!("Invalid address: {}", addr_str),
                    "Please provide a valid NEO address".to_string()
                )),
            }
        },
        None => {
            // Use the wallet's account address if available
            ensure_account_loaded(state, None, None).await?;
            let account = state.get_account()?;
            account.get_address()
        },
    };
    
    print_info(&format!("Checking {} balance for address: {}", contract, target_address));
    
    // Convert the address to script hash for the balanceOf call
    let addr_script_hash = target_address.address_to_script_hash().map_err(|e| {
        CliError::InvalidArgument(
            format!("Failed to convert address to script hash: {}", e),
            "The address may be invalid".to_string()
        )
    })?;
    
    // Call balanceOf on the token contract
    match rpc_client.invoke_function(
        &token_hash,
        "balanceOf".to_string(),
        vec![ContractParameter::h160(&addr_script_hash)],
        None
    ).await {
        Ok(result) => {
            if let Some(item) = result.stack.first() {
                match item {
                    StackItem::Integer { value } => {
                        // Get decimals for proper display
                        match get_token_decimals(&token_hash, &rpc_client, network_type).await {
                            Ok(decimals) => {
                                let raw_balance = value.to_i64().unwrap_or(0);
                                let formatted_balance = format_token_amount(raw_balance, decimals);
                                
                                // Try to get the token symbol for a better display
                                let token_symbol = match rpc_client.invoke_function(&token_hash, "symbol".to_string(), vec![], None).await {
                                    Ok(result) => {
                                        if let Some(stack_item) = result.stack.first() {
                                            if let Some(bytes) = stack_item.as_bytes() {
                                                String::from_utf8_lossy(&bytes).into_owned()
                                            } else {
                                                contract.to_string()
                                            }
                                        } else {
                                            contract.to_string()
                                        }
                                    },
                                    Err(_) => contract.to_string(),
                                };
                                
                                print_success(&format!("Balance: {} {}", formatted_balance, token_symbol));
                            },
                            Err(e) => {
                                // Just show the raw balance if we can't get decimals
                                print_error(&format!("Failed to get token decimals: {}", e));
                                print_info(&format!("Raw Balance: {}", value));
                            }
                        }
                    },
                    _ => print_error("Unexpected response format from token contract"),
                }
            } else {
                print_error("Empty response from token contract");
            }
        },
        Err(e) => {
            return Err(CliError::Rpc(format!("Failed to get token balance: {}", e)));
        }
    }
    
    Ok(())
}

/// Transfer tokens to an address
pub async fn handle_token_transfer(
    token: &str,
    to: &str,
    amount: &str,
    data: Option<&str>,
    state: &mut CliState
) -> Result<(), CliError> {
    // Get RPC client
    let rpc_client = state.get_rpc_client()?;
    
    // Determine network type
    let network_type = network_type_from_state(state);
    
    // Ensure wallet is loaded
    ensure_account_loaded(state, None, None).await?;
    let mut account = state.get_account()?;
    
    // Resolve token address
    let token_hash = resolve_token_to_scripthash_with_network(
        token, 
        &rpc_client, 
        network_type
    ).await?;
    
    // Resolve destination address to script hash
    let to_address = match Address::from_str(to) {
        Ok(addr) => addr,
        Err(_) => return Err(CliError::InvalidArgument(
            format!("Invalid recipient address: {}", to),
            "Please provide a valid NEO address".to_string()
        )),
    };
    
    let to_script_hash = to_address.address_to_script_hash().map_err(|e| {
        CliError::InvalidArgument(
            format!("Failed to convert address to script hash: {}", e),
            "The address may be invalid".to_string()
        )
    })?;
    
    // Parse amount
    let raw_amount = parse_amount(amount, &token_hash, &rpc_client, network_type).await?;
    
    // Get token symbol and decimals for better UX
    let token_symbol = match rpc_client.invoke_function(&token_hash, "symbol".to_string(), vec![], None).await {
        Ok(result) => {
            if let Some(stack_item) = result.stack.first() {
                if let Some(bytes) = stack_item.as_bytes() {
                    String::from_utf8_lossy(&bytes).into_owned()
                } else {
                    token.to_string()
                }
            } else {
                token.to_string()
            }
        },
        Err(_) => token.to_string(),
    };
    
    let decimals = get_token_decimals(&token_hash, &rpc_client, network_type).await.unwrap_or(0);
    let formatted_amount = format_token_amount(raw_amount, decimals);
    
    // Confirm the transfer
    print_info(&format!("Preparing to transfer {} {} to {}", formatted_amount, token_symbol, to_address));
    
    // Get the wallet password if needed
    let _password = if account.encrypted_private_key().is_some() {
        let pwd = prompt_password("Enter wallet password: ")?;
        account.decrypt_private_key(&pwd)?;
        Some(pwd)
    } else {
        None
    };
    
    // Create Script Builder
    let mut script_builder = ScriptBuilder::new();
    
    // Setup parameters for the token transfer
    let mut params = vec![
        ContractParameter::h160(account.get_script_hash()),
        ContractParameter::h160(to_script_hash),
        ContractParameter::integer(raw_amount),
    ];
    
    // Add data parameter if provided
    if let Some(data_str) = data {
        params.push(ContractParameter::string(data_str.to_string()));
    } else {
        params.push(ContractParameter::any(None)); // Optional data parameter as null
    }
    
    // Build the script
    script_builder.emit_contract_call(
        &token_hash,
        "transfer",
        &params,
        Some(CallFlags::All)
    )?;
    
    // Get the raw script bytes and encode them to base64
    let script_bytes = script_builder.to_bytes();
    let script_base64 = base64::encode(&script_bytes);
    
    // Use a more direct approach with the RPC client's invoke function
    print_info("Invoking transaction...");
    
    // Use the RPC client to invoke the transaction directly
    let signer = Signer::calledbyentry(account.get_script_hash());
    let result = rpc_client.invoke_function(
        &token_hash,
        "transfer".to_string(),
        params,
        Some(vec![signer])
    ).await;
    
    match result {
        Ok(invoke_result) => {
            if invoke_result.state.as_ref() == "HALT" && 
               invoke_result.stack.first().map_or(false, |item| match item {
                   StackItem::Boolean { value } => *value,
                   _ => false,
               }) {
                print_success(&format!("Transaction prepared successfully!"));
                
                // Ask for confirmation to send the transaction
                if prompt_yes_no("Do you want to send this transaction?")? {
                    // Format the invoke_function directly
                    let signed_tx = serde_json::json!({
                        "script": script_base64,
                        "signers": [{
                            "account": account.get_script_hash().to_string(),
                            "scopes": "CalledByEntry"
                        }],
                        "attributes": [],
                        "witnesses": []
                    });
                    
                    let tx_json = serde_json::to_string(&signed_tx)
                        .map_err(|e| CliError::Network(format!("Failed to serialize transaction: {}", e)))?;
                    
                    match rpc_client.send_raw_transaction(tx_json).await {
                        Ok(tx_id) => {
                            print_success(&format!("Transaction sent successfully!"));
                            print_info(&format!("Transaction ID: {:?}", tx_id));
                            Ok(())
                        },
                        Err(e) => {
                            Err(CliError::Rpc(format!("Failed to send transaction: {}", e)))
                        }
                    }
                } else {
                    print_info("Transaction aborted by user");
                    Ok(())
                }
            } else {
                Err(CliError::Rpc(format!("Invocation failed: {:?}", invoke_result)))
            }
        },
        Err(e) => {
            Err(CliError::Rpc(format!("Failed to invoke transaction: {}", e)))
        }
    }
}

async fn resolve_token_to_address(state: &mut CliState, token: &str) -> Result<String, CliError> {
    let network_type = network_type_from_state(state);
    let token_hash = resolve_token_to_scripthash_with_network(
        token, 
        &state.rpc_client.as_ref().ok_or(CliError::Config("RPC client not initialized".to_string()))?.clone(), 
        network_type
    ).await
        .map_err(|e| CliError::Config(format!("Failed to resolve token: {}", e)))?;

    Ok(token_hash.to_address())
}

/// Convert CliState.network_type to NetworkType
fn network_type_from_state(state: &CliState) -> NetworkType {
    state.network_type
        .map(NetworkType::from_network)
        .unwrap_or(NetworkType::N3Mainnet)
}
