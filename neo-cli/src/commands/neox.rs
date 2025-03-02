/*
 * NeoX module for Neo CLI
 * 
 * This module provides Neo X chain integration for the Neo CLI, including:
 * - Support for address conversion between Neo and Ethereum formats
 * - Chain info and balance checking
 * - Cross-chain bridging capabilities between Neo N3 and Neo X
 * 
 * The implementation uses feature flags to ensure that the Neo X features
 * are only available when the ethereum-compat feature is enabled.
 */

use clap::{Args, Subcommand};
use neo::prelude::*;
use neo::neo_x::{NeoXProvider, NeoXTransaction, NeoXBridgeContract};
use primitive_types::H160;
use crate::utils::error::{CliError, CliResult};
use crate::commands::wallet::CliState;
use crate::utils::{print_success, print_error, print_info, print_warning, prompt_password};
use std::str::FromStr;
use neo::prelude::ScriptHash;

#[derive(Debug, Args)]
pub struct NeoXArgs {
    #[command(subcommand)]
    pub command: NeoXCommands,
}

#[derive(Debug, Subcommand)]
pub enum NeoXCommands {
    /// Get Neo X chain information
    ChainInfo,
    
    /// Get Neo X account balance
    Balance {
        /// Account address in EVM format (0x...)
        #[arg(short, long)]
        address: String,
    },
    
    /// Bridge assets from Neo N3 to Neo X
    BridgeToNeoX {
        /// Token to bridge (NEO, GAS, or contract hash)
        #[arg(short, long)]
        token: String,
        
        /// Amount to bridge
        #[arg(short, long)]
        amount: String,
        
        /// Destination address on Neo X (0x...)
        #[arg(short, long)]
        to: String,
        
        /// Account to use for the bridge operation
        #[arg(short, long)]
        account: Option<String>,
    },
    
    /// Bridge assets from Neo X to Neo N3
    BridgeToNeoN3 {
        /// Token to bridge (NEO, GAS, or contract hash)
        #[arg(short, long)]
        token: String,
        
        /// Amount to bridge
        #[arg(short, long)]
        amount: String,
        
        /// Destination address on Neo N3
        #[arg(short, long)]
        to: String,
        
        /// Account to use for the bridge operation
        #[arg(short, long)]
        account: Option<String>,
    },
    
    /// Check bridge transaction status
    BridgeStatus {
        /// Transaction hash
        #[arg(short, long)]
        hash: String,
    },
    
    /// Convert Ethereum address to Neo address
    ConvertAddress {
        /// Ethereum address to convert (with or without 0x prefix)
        ethereum_address: String,
    },
    
    /// Convert Neo address to Ethereum address
    ConvertNeoAddress {
        /// Neo address to convert
        neo_address: String,
    },
}

pub async fn handle_neox_command(args: NeoXArgs, state: &mut CliState) -> CliResult<()> {
    match args.command {
        NeoXCommands::ChainInfo => get_chain_info(state).await,
        NeoXCommands::Balance { address } => get_balance(address, state).await,
        NeoXCommands::BridgeToNeoX { token, amount, to, account } => 
            bridge_to_neox(token, amount, to, account, state).await,
        NeoXCommands::BridgeToNeoN3 { token, amount, to, account } => 
            bridge_to_neon3(token, amount, to, account, state).await,
        NeoXCommands::BridgeStatus { hash } => check_bridge_status(hash, state).await,
        NeoXCommands::ConvertAddress { ethereum_address } => {
            let eth_addr = ethereum_address.trim_start_matches("0x");
            
            // Parse the Ethereum address
            let eth_address = match H160::from_str(eth_addr) {
                Ok(addr) => addr,
                Err(_) => return Err(CliError::Input(format!("Invalid Ethereum address format: {}", ethereum_address)))
            };
            
            // Convert Ethereum address to Neo address using the ethereum-compat feature
            if let Some(neo_address) = try_eth_to_neo_address(&eth_address) {
                println!("Ethereum address: {}", ethereum_address);
                println!("Neo address:      {}", neo_address);
                
                print_success("Address conversion completed successfully");
                Ok(())
            } else {
                print_error("Ethereum address conversion is not supported in this build");
                Err(CliError::Input("Ethereum-compat feature is not available".to_string()))
            }
        }
        NeoXCommands::ConvertNeoAddress { neo_address } => {
            // Validate Neo address format
            let script_hash = match ScriptHash::from_address(&neo_address) {
                Ok(hash) => hash,
                Err(_) => return Err(CliError::Input(format!("Invalid Neo address format: {}", neo_address)))
            };
            
            // Convert Neo address to Ethereum address
            if let Some(eth_address) = try_neo_to_eth_address(&script_hash) {
                println!("Neo address:      {}", neo_address);
                println!("Ethereum address: 0x{}", eth_address.to_string());
                
                print_success("Address conversion completed successfully");
                Ok(())
            } else {
                print_error("Neo address conversion is not supported in this build");
                Err(CliError::Input("Ethereum-compat feature is not available".to_string()))
            }
        }
    }
}

// Helper function to try to convert Ethereum address to Neo address
fn try_eth_to_neo_address(eth_address: &H160) -> Option<String> {
    #[cfg(feature = "ethereum-compat")]
    {
        neo::ethereum_compat::eth_to_neo_address(eth_address).ok()
    }
    
    #[cfg(not(feature = "ethereum-compat"))]
    {
        None
    }
}

// Helper function to try to convert Neo address to Ethereum address
fn try_neo_to_eth_address(script_hash: &ScriptHash) -> Option<H160> {
    #[cfg(feature = "ethereum-compat")]
    {
        Some(neo::ethereum_compat::neo_to_eth_address(script_hash))
    }
    
    #[cfg(not(feature = "ethereum-compat"))]
    {
        None
    }
}

async fn get_chain_info(state: &mut CliState) -> CliResult<()> {
    #[cfg(not(feature = "ethereum-compat"))]
    {
        print_error("Neo X support is not enabled in this build");
        return Err(CliError::Input("Neo X feature is not enabled in this build".to_string()));
    }
    
    #[cfg(feature = "ethereum-compat")]
    {
        if state.rpc_client.is_none() {
            print_error("No RPC client is connected. Please connect to a node first.");
            return Err(CliError::Network("No RPC client is connected".to_string()));
        }
        
        print_info("Getting Neo X chain information...");
        
        // Get Neo X provider
        let neo_client = state.rpc_client.as_ref().unwrap();
        let neox_provider = NeoXProvider::new("https://rpc.neo-x.org", Some(neo_client));
        
        // Get chain ID
        let chain_id = neox_provider.chain_id().await
            .map_err(|e| CliError::Network(format!("Failed to get Neo X chain ID: {}", e)))?;
        
        // Get latest block
        let block_number = neox_provider.block_number().await
            .map_err(|e| CliError::Network(format!("Failed to get Neo X block number: {}", e)))?;
        
        // Get gas price
        let gas_price = neox_provider.gas_price().await
            .map_err(|e| CliError::Network(format!("Failed to get Neo X gas price: {}", e)))?;
        
        // Display information
        println!("Neo X Chain Information:");
        println!("  Chain ID: {}", chain_id);
        println!("  Latest Block: {}", block_number);
        println!("  Gas Price: {} gwei", gas_price / 1_000_000_000);
        
        print_success("Neo X chain information retrieved successfully");
        Ok(())
    }
}

async fn get_balance(address: String, state: &mut CliState) -> CliResult<()> {
    #[cfg(not(feature = "ethereum-compat"))]
    {
        print_error("Neo X support is not enabled in this build");
        return Err(CliError::Input("Neo X feature is not enabled in this build".to_string()));
    }
    
    #[cfg(feature = "ethereum-compat")]
    {
        if state.rpc_client.is_none() {
            print_error("No RPC client is connected. Please connect to a node first.");
            return Err(CliError::Network("No RPC client is connected".to_string()));
        }
        
        print_info(&format!("Getting Neo X balance for address: {}", address));
        
        // Parse address
        let address = address.parse::<primitive_types::H160>()
            .map_err(|_| CliError::Input(format!("Invalid Ethereum address format: {}", address)))?;
        
        // Get Neo X provider
        let neo_client = state.rpc_client.as_ref().unwrap();
        let neox_provider = NeoXProvider::new("https://rpc.neo-x.org", Some(neo_client));
        
        // Get ETH balance
        let balance = neox_provider.balance(&address).await
            .map_err(|e| CliError::Network(format!("Failed to get balance: {}", e)))?;
        
        // Display balance
        println!("Neo X Balance:");
        println!("  ETH: {} ({} wei)", 
            balance as f64 / 1_000_000_000_000_000_000.0, 
            balance);
        
        print_success("Neo X balance retrieved successfully");
        Ok(())
    }
}

async fn bridge_to_neox(
    token: String, 
    amount: String, 
    to: String, 
    account: Option<String>, 
    state: &mut CliState
) -> CliResult<()> {
    #[cfg(not(feature = "ethereum-compat"))]
    {
        print_error("Neo X support is not enabled in this build");
        return Err(CliError::Input("Neo X feature is not enabled in this build".to_string()));
    }
    
    #[cfg(feature = "ethereum-compat")]
    {
        if state.wallet.is_none() {
            print_error("No wallet is currently open");
            return Err(CliError::Wallet("No wallet is currently open".to_string()));
        }
        
        if state.rpc_client.is_none() {
            print_error("No RPC client is connected. Please connect to a node first.");
            return Err(CliError::Network("No RPC client is connected".to_string()));
        }
        
        print_info(&format!("Bridging {} {} to Neo X address: {}", amount, token, to));
        
        // Parse token
        let token_hash = if token.to_uppercase() == "NEO" {
            H160::from_str("0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5")
                .map_err(|_| CliError::Input("Failed to parse NEO token hash".to_string()))?
        } else if token.to_uppercase() == "GAS" {
            H160::from_str("0xd2a4cff31913016155e38e474a2c06d08be276cf")
                .map_err(|_| CliError::Input("Failed to parse GAS token hash".to_string()))?
        } else {
            H160::from_str(&token)
                .map_err(|_| CliError::Input(format!("Invalid token format: {}", token)))?
        };
        
        // Parse amount
        let amount = if token.to_uppercase() == "NEO" {
            // NEO is indivisible
            amount.parse::<i64>()
                .map_err(|_| CliError::Input(format!("Invalid amount: {}", amount)))?
        } else {
            // GAS and other tokens use 8 decimals
            let decimal_amount = amount.parse::<f64>()
                .map_err(|_| CliError::Input(format!("Invalid amount: {}", amount)))?;
            (decimal_amount * 100_000_000.0) as i64
        };
        
        // Get Neo N3 account
        let wallet = state.wallet.as_ref().unwrap();
        let account_address = match account {
            Some(addr) => addr,
            None => {
                // If no account specified, use the first account in the wallet
                let accounts = wallet.get_accounts();
                if accounts.is_empty() {
                    print_error("No accounts in wallet");
                    return Err(CliError::Wallet("No accounts in wallet".to_string()));
                }
                accounts[0].get_address().to_string()
            }
        };
        
        // Find account in wallet
        let account_obj = wallet.get_accounts().iter()
            .find(|a| a.get_address() == account_address)
            .ok_or_else(|| CliError::Wallet(format!("Account not found: {}", account_address)))?
            .clone();
        
        // Get password for signing
        let password = prompt_password("Enter wallet password")?;
        
        // Initialize Neo X bridge contract
        let neo_client = state.rpc_client.as_ref().unwrap();
        let bridge = NeoXBridgeContract::new(Some(neo_client));
        
        // Show bridge fees
        let fee = bridge.get_fee(&token_hash).await
            .map_err(|e| CliError::Network(format!("Failed to get bridge fee: {}", e)))?;
        
        let fee_formatted = if token.to_uppercase() == "NEO" {
            format!("{} NEO", fee)
        } else {
            format!("{} {}", fee as f64 / 100_000_000.0, token)
        };
        
        print_info(&format!("Bridge fee: {}", fee_formatted));
        
        // Confirm the operation
        if !crate::utils::prompt_yes_no(&format!("Are you sure you want to bridge {} {} to Neo X?", amount, token))? {
            print_info("Bridge operation cancelled");
            return Ok(());
        }
        
        // Execute the deposit
        account_obj.decrypt_private_key(&password)
            .map_err(|e| CliError::Wallet(format!("Failed to decrypt private key: {}", e)))?;
        
        let tx = bridge.deposit(
            &token_hash,
            amount,
            &to,
            &account_obj
        ).await.map_err(|e| CliError::Transaction(format!("Failed to deposit to bridge: {}", e)))?;
        
        print_success(&format!("Bridge transaction sent successfully! Transaction hash: {}", tx.hash));
        print_info("Please wait for the transaction to be confirmed on both Neo N3 and Neo X networks.");
        
        Ok(())
    }
}

async fn bridge_to_neon3(
    token: String, 
    amount: String, 
    to: String, 
    account: Option<String>, 
    state: &mut CliState
) -> CliResult<()> {
    #[cfg(not(feature = "ethereum-compat"))]
    {
        print_error("Neo X support is not enabled in this build");
        return Err(CliError::Input("Neo X feature is not enabled in this build".to_string()));
    }
    
    #[cfg(feature = "ethereum-compat")]
    {
        print_warning("Bridging from Neo X to Neo N3 is not fully implemented in the CLI");
        print_info("Please use the Neo X wallet interface to bridge assets back to Neo N3");
        
        Ok(())
    }
}

async fn check_bridge_status(hash: String, state: &mut CliState) -> CliResult<()> {
    #[cfg(not(feature = "ethereum-compat"))]
    {
        print_error("Neo X support is not enabled in this build");
        return Err(CliError::Input("Neo X feature is not enabled in this build".to_string()));
    }
    
    #[cfg(feature = "ethereum-compat")]
    {
        if state.rpc_client.is_none() {
            print_error("No RPC client is connected. Please connect to a node first.");
            return Err(CliError::Network("No RPC client is connected".to_string()));
        }
        
        print_info(&format!("Checking bridge status for transaction: {}", hash));
        
        // Initialize Neo X bridge contract
        let neo_client = state.rpc_client.as_ref().unwrap();
        let bridge = NeoXBridgeContract::new(Some(neo_client));
        
        // Check transaction status
        let status = bridge.get_transaction_status(&hash).await
            .map_err(|e| CliError::Network(format!("Failed to get transaction status: {}", e)))?;
        
        // Display status
        println!("Bridge Transaction Status:");
        println!("  Hash: {}", hash);
        println!("  Status: {}", status.status);
        
        if let Some(neo_n3_hash) = status.neo_n3_hash {
            println!("  Neo N3 Transaction: {}", neo_n3_hash);
        }
        
        if let Some(neo_x_hash) = status.neo_x_hash {
            println!("  Neo X Transaction: {}", neo_x_hash);
        }
        
        print_success("Bridge transaction status retrieved successfully");
        Ok(())
    }
} 