use neo::prelude::*;
use neo::neo_utils::{constants, network::NeoNetwork};
use std::env;
use std::error::Error;
use std::str::FromStr;

/// An example demonstrating how to connect to different Neo N3 networks
/// and interact with various smart contracts
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("help");
    
    match command {
        "help" => print_help(),
        "mainnet" => connect_to_network(NeoNetwork::MainNet).await?,
        "testnet" => connect_to_network(NeoNetwork::TestNet).await?,
        "endpoints" => print_endpoints(),
        "contracts" => print_contracts(),
        "check-gas" if args.len() >= 3 => {
            let address = args[2].clone();
            check_gas_balance(&address).await?;
        },
        "check-neo" if args.len() >= 3 => {
            let address = args[2].clone();
            check_neo_balance(&address).await?;
        },
        "check-native" if args.len() >= 3 => {
            let address = args[2].clone();
            check_native_contracts(&address).await?;
        },
        _ => {
            println!("Unknown command or insufficient arguments: {}", command);
            print_help();
        }
    }
    
    Ok(())
}

fn print_help() {
    println!("Neo Network Examples");
    println!("==================");
    println!();
    println!("Usage:");
    println!("  cargo run --features crypto-standard --example neo_networks [COMMAND] [ARGS]");
    println!();
    println!("Commands:");
    println!("  help                     Show this help message");
    println!("  mainnet                  Connect to Neo N3 MainNet and get basic info");
    println!("  testnet                  Connect to Neo N3 TestNet and get basic info");
    println!("  endpoints                Show Neo N3 RPC endpoints");
    println!("  contracts                Show Neo N3 contract addresses");
    println!("  check-gas [ADDRESS]      Check GAS balance of an address");
    println!("  check-neo [ADDRESS]      Check NEO balance of an address");
    println!("  check-native [ADDRESS]   Check native contracts info for an address");
    println!();
    println!("Examples:");
    println!("  cargo run --features crypto-standard --example neo_networks mainnet");
    println!("  cargo run --features crypto-standard --example neo_networks check-gas NVkg1yRMrTyY6QFnEkpP4WUFaviE1gFa3g");
}

fn print_endpoints() {
    println!("Neo N3 Network Endpoints");
    println!("=======================");
    println!();
    
    println!("MainNet RPC Endpoints:");
    println!("-----------------");
    for endpoint in constants::rpc_endpoints::mainnet::ALL {
        println!("  {}", endpoint);
    }
    println!();
    
    println!("TestNet RPC Endpoints:");
    println!("-----------------");
    for endpoint in constants::rpc_endpoints::testnet::ALL {
        println!("  {}", endpoint);
    }
}

fn print_contracts() {
    println!("Neo N3 Contract Addresses");
    println!("========================");
    println!();
    
    println!("MainNet Contracts:");
    println!("----------------");
    println!("  NEO Token:     {}", constants::contracts::mainnet::NEO_TOKEN);
    println!("  GAS Token:     {}", constants::contracts::mainnet::GAS_TOKEN);
    println!("  NeoFS:         {}", constants::contracts::mainnet::NEOFS);
    println!("  Neo Name Service: {}", constants::contracts::mainnet::NEO_NS);
    println!("  Flamingo (FLM): {}", constants::contracts::mainnet::FLM_TOKEN);
    println!("  Oracle:        {}", constants::contracts::mainnet::ORACLE);
    println!();
    
    println!("TestNet Contracts:");
    println!("----------------");
    println!("  NEO Token:     {}", constants::contracts::testnet::NEO_TOKEN);
    println!("  GAS Token:     {}", constants::contracts::testnet::GAS_TOKEN);
    println!("  NeoFS:         {}", constants::contracts::testnet::NEOFS);
    println!("  Neo Name Service: {}", constants::contracts::testnet::NEO_NS);
    println!("  Oracle:        {}", constants::contracts::testnet::ORACLE);
    println!();
    
    println!("Native Contracts (same on all networks):");
    println!("-------------------------------------");
    println!("  Contract Management: {}", constants::contracts::native::CONTRACT_MANAGEMENT);
    println!("  Ledger:         {}", constants::contracts::native::LEDGER);
    println!("  Policy:         {}", constants::contracts::native::POLICY);
    println!("  Role Management: {}", constants::contracts::native::ROLE_MANAGEMENT);
}

async fn connect_to_network(network: NeoNetwork) -> Result<(), Box<dyn Error>> {
    println!("Connecting to Neo N3 {} network...", network);
    
    // Create a client for the specified network using the network utility
    let client = network.create_client()?;
    
    // Try getting version info
    match client.get_version().await {
        Ok(version) => {
            println!("✅ Successfully connected to {} ({})!", network, version.user_agent);
            println!();
            
            // Get more network info
            println!("Network Information:");
            println!("-------------------");
            println!("Protocol:");
            if let Some(p) = &version.protocol {
                println!("  Network Magic: {}", p.network_magic);
                println!("  Address Version: {}", p.address_version);
                println!("  Validators Count: {}", p.validators_count);
                println!("  Milliseconds Per Block: {}", p.milliseconds_per_block);
            }
            
            // Get block count
            if let Ok(block_count) = client.get_block_count().await {
                println!("Current Block Count: {}", block_count);
            }
            
            // Get connection count
            if let Ok(connection_count) = client.get_connection_count().await {
                println!("Connection Count: {}", connection_count);
            }
            
            // Get mempool size
            if let Ok(mempool) = client.get_raw_mempool().await {
                println!("Mempool Size: {} transactions", mempool.len());
            }
            
            Ok(())
        },
        Err(e) => {
            println!("❌ Failed to connect to {}: {}", network, e);
            Err(format!("Failed to connect to {}: {}", network, e).into())
        }
    }
}

async fn check_gas_balance(address: &str) -> Result<(), Box<dyn Error>> {
    println!("Checking GAS token balance for {}", address);
    
    // Create a NetworkToken for GAS on both MainNet and TestNet
    let mainnet_gas = NetworkToken::new(NeoNetwork::MainNet, "gas")?;
    let testnet_gas = NetworkToken::new(NeoNetwork::TestNet, "gas")?;
    
    // Get token info and balances
    println!("\nMainNet GAS Info:");
    match mainnet_gas.token_info().await {
        Ok(info) => {
            println!("  Token Name: {} ({})", info.name, info.symbol);
            println!("  Decimals: {}", info.decimals);
            println!("  Total Supply: {}", mainnet_gas.format_balance(info.total_supply, info.decimals));
            
            // Get balance on MainNet
            match mainnet_gas.balance_of(address).await {
                Ok((balance, symbol, decimals)) => {
                    let formatted_balance = mainnet_gas.format_balance(balance, decimals);
                    println!("  Balance: {} {}", formatted_balance, symbol);
                },
                Err(e) => println!("  Failed to get MainNet balance: {}", e),
            }
        },
        Err(e) => println!("  Failed to get MainNet token info: {}", e),
    }
    
    println!("\nTestNet GAS Info:");
    match testnet_gas.token_info().await {
        Ok(info) => {
            println!("  Token Name: {} ({})", info.name, info.symbol);
            println!("  Decimals: {}", info.decimals);
            println!("  Total Supply: {}", testnet_gas.format_balance(info.total_supply, info.decimals));
            
            // Get balance on TestNet
            match testnet_gas.balance_of(address).await {
                Ok((balance, symbol, decimals)) => {
                    let formatted_balance = testnet_gas.format_balance(balance, decimals);
                    println!("  Balance: {} {}", formatted_balance, symbol);
                },
                Err(e) => println!("  Failed to get TestNet balance: {}", e),
            }
        },
        Err(e) => println!("  Failed to get TestNet token info: {}", e),
    }
    
    Ok(())
}

async fn check_neo_balance(address: &str) -> Result<(), Box<dyn Error>> {
    println!("Checking NEO token balance for {}", address);
    
    // Create a NetworkToken for NEO on both MainNet and TestNet
    let mainnet_neo = NetworkToken::new(NeoNetwork::MainNet, "neo")?;
    let testnet_neo = NetworkToken::new(NeoNetwork::TestNet, "neo")?;
    
    // Get token info and balances
    println!("\nMainNet NEO Info:");
    match mainnet_neo.token_info().await {
        Ok(info) => {
            println!("  Token Name: {} ({})", info.name, info.symbol);
            println!("  Decimals: {}", info.decimals);
            println!("  Total Supply: {}", mainnet_neo.format_balance(info.total_supply, info.decimals));
            
            // Get balance on MainNet
            match mainnet_neo.balance_of(address).await {
                Ok((balance, symbol, decimals)) => {
                    let formatted_balance = mainnet_neo.format_balance(balance, decimals);
                    println!("  Balance: {} {}", formatted_balance, symbol);
                },
                Err(e) => println!("  Failed to get MainNet balance: {}", e),
            }
        },
        Err(e) => println!("  Failed to get MainNet token info: {}", e),
    }
    
    println!("\nTestNet NEO Info:");
    match testnet_neo.token_info().await {
        Ok(info) => {
            println!("  Token Name: {} ({})", info.name, info.symbol);
            println!("  Decimals: {}", info.decimals);
            println!("  Total Supply: {}", testnet_neo.format_balance(info.total_supply, info.decimals));
            
            // Get balance on TestNet
            match testnet_neo.balance_of(address).await {
                Ok((balance, symbol, decimals)) => {
                    let formatted_balance = testnet_neo.format_balance(balance, decimals);
                    println!("  Balance: {} {}", formatted_balance, symbol);
                },
                Err(e) => println!("  Failed to get TestNet balance: {}", e),
            }
        },
        Err(e) => println!("  Failed to get TestNet token info: {}", e),
    }
    
    Ok(())
}

async fn check_native_contracts(address: &str) -> Result<(), Box<dyn Error>> {
    println!("Checking native contracts info for {}", address);
    
    // Create clients for MainNet
    let mainnet_client = NeoNetwork::MainNet.create_client()?;
    
    // Get contract instances using the network utilities
    let neo_contract = get_network_contract(NeoNetwork::MainNet, "neo")?;
    let gas_contract = get_network_contract(NeoNetwork::MainNet, "gas")?;
    let policy_contract = get_network_contract(NeoNetwork::MainNet, "policy")?;
    
    // Parse address to script hash
    let address_obj = Address::from_str(address)?;
    let script_hash = address_obj.script_hash();
    
    // Get native contracts list
    println!("\nNative Contracts:");
    match mainnet_client.get_native_contracts().await {
        Ok(contracts) => {
            for contract in contracts {
                println!("  {} (Hash: {}, ID: {})", 
                    contract.manifest.name, 
                    contract.hash, 
                    contract.id);
            }
        },
        Err(e) => println!("  Error getting native contracts: {}", e),
    }
    
    // Create parameters for script hash
    let params = vec![
        ContractParameter::hash160(&script_hash),
    ];
    
    // Check Neo balance
    println!("\nNEO Balance:");
    match neo_contract.test_invoke("balanceOf", params.clone()).await {
        Ok(result) => {
            if let Some(balance_item) = result.stack.first() {
                if let Some(balance) = balance_item.get_int() {
                    println!("  Balance: {} NEO", balance);
                } else {
                    println!("  Unable to parse balance");
                }
            }
        },
        Err(e) => println!("  Error checking NEO balance: {}", e),
    }
    
    // Check GAS per block
    println!("\nGAS Policy:");
    match policy_contract.test_invoke("getFeePerByte", vec![]).await {
        Ok(result) => {
            if let Some(fee_item) = result.stack.first() {
                if let Some(fee) = fee_item.get_int() {
                    println!("  Fee per byte: {}", fee);
                }
            }
        },
        Err(e) => println!("  Error getting fee per byte: {}", e),
    }
    
    // Get unclaimed GAS
    println!("\nUnclaimed GAS:");
    let unclaimed_params = vec![
        ContractParameter::hash160(&script_hash)
    ];
    
    match neo_contract.test_invoke("unclaimedGas", unclaimed_params).await {
        Ok(result) => {
            if let Some(gas_item) = result.stack.first() {
                if let Some(gas) = gas_item.get_int() {
                    println!("  Unclaimed GAS: {:.8} GAS", gas as f64 / 100000000.0);
                } else {
                    println!("  Unable to parse unclaimed GAS");
                }
            }
        },
        Err(e) => println!("  Error checking unclaimed GAS: {}", e),
    }
    
    Ok(())
} 