use neo3::neo_types::contract::ContractParameter;
use neo3::neo_types::address::Address;
use neo3::prelude::*;
use neo3::neo_utils::{network::{NeoNetwork, NetworkToken}};
use neo3::neo_types::address::AddressExtension;
use std::env;
use std::error::Error;
use std::str::FromStr;

/// This example demonstrates how to work with tokens across different Neo N3 networks
/// You can check balances, transfer tokens, and monitor transaction status
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("help");
    
    match command {
        "help" => print_help(),
        "balance" if args.len() >= 4 => {
            let network = parse_network(&args[2])?;
            let address = args[3].clone();
            check_token_balance(network, &address).await?;
        },
        "tokens" if args.len() >= 3 => {
            let network = parse_network(&args[2])?;
            list_available_tokens(network).await?;
        },
        "transfer" if args.len() >= 7 => {
            let network = parse_network(&args[2])?;
            let token_name = args[3].clone();
            let from_key = args[4].clone();
            let to_address = args[5].clone();
            let amount = args[6].clone();
            
            transfer_token(network, &token_name, &from_key, &to_address, &amount).await?;
        },
        _ => {
            println!("Unknown command or insufficient arguments: {:?}", args);
            print_help();
        }
    }
    
    Ok(())
}

fn print_help() {
    println!("Neo Token Transfer Examples");
    println!("==========================");
    println!();
    println!("Usage:");
    println!("  cargo run --features crypto-standard --example token_transfers [COMMAND] [ARGS]");
    println!();
    println!("Commands:");
    println!("  help                                         Show this help message");
    println!("  balance [NETWORK] [ADDRESS]                  Check token balances for an address");
    println!("  tokens [NETWORK]                             List available tokens on the network");
    println!("  transfer [NETWORK] [TOKEN] [FROM_WIF] [TO_ADDRESS] [AMOUNT]  Transfer tokens");
    println!();
    println!("Networks:");
    println!("  mainnet                                      Neo N3 MainNet");
    println!("  testnet                                      Neo N3 TestNet");
    println!();
    println!("Example:");
    println!("  cargo run --features crypto-standard --example token_transfers balance mainnet NVkg1yRMrTyY6QFnEkpP4WUFaviE1gFa3g");
    println!("  cargo run --features crypto-standard --example token_transfers tokens testnet");
    println!("  cargo run --features crypto-standard --example token_transfers transfer testnet gas YOUR_WIF NVkg1yRMrTyY6QFnEkpP4WUFaviE1gFa3g 1.5");
}

fn parse_network(network_str: &str) -> Result<NeoNetwork, Box<dyn Error>> {
    match network_str.to_lowercase().as_str() {
        "mainnet" => Ok(NeoNetwork::main_net()),
        "testnet" => Ok(NeoNetwork::test_net()),
        _ => Err(format!("Unknown network: {}", network_str).into()),
    }
}

async fn check_token_balance(network: NeoNetwork, address: &str) -> Result<(), Box<dyn Error>> {
    println!("Checking token balances for {} on {}", address, network);
    
    // Create client for the network
    let client = network.create_client()?;
    
    // Parse address
    let address_obj = Address::from_str(address)?;
    let script_hash = address_obj.to_script_hash()?;
    
    // Try common tokens
    println!("\nCommon Tokens:");
    println!("-------------");
    
    check_single_token_balance(&network, "neo", address).await?;
    check_single_token_balance(&network, "gas", address).await?;
    
    // Try network-specific tokens
    if network.name == "MainNet" {
        println!("\nMainNet Tokens:");
        println!("--------------");
        check_single_token_balance(&network, "flm", address).await?;
    } else if network.name == "TestNet" {
        println!("\nTestNet Tokens:");
        println!("--------------");
        check_single_token_balance(&network, "cneo", address).await?;
        check_single_token_balance(&network, "cgas", address).await?;
    }
    
    // Get all NEP-17 balances
    println!("\nAll NEP-17 Balances:");
    println!("------------------");
    
    match client.get_nep17_balances(&script_hash.to_string()).await {
        Ok(balances) => {
            if balances.balances.is_empty() {
                println!("No NEP-17 token balances found.");
            } else {
                for balance in balances.balances {
                    println!("  Asset: {}", balance.asset_hash);
                    println!("  Amount: {}", balance.amount);
                    println!("  Last Updated: Block {}", balance.last_updated_block);
                    println!();
                }
            }
        },
        Err(e) => println!("Error retrieving NEP-17 balances: {}", e),
    }
    
    Ok(())
}

async fn check_single_token_balance(
    network: &NeoNetwork, 
    token_name: &str, 
    address: &str
) -> Result<(), Box<dyn Error>> {
    // Try to create the token
    match NetworkToken::new(network.clone(), token_name) {
        Ok(token) => {
            match token.token_info().await {
                Ok(info) => {
                    println!("  {} ({}):", info.name, info.symbol);
                    
                    // Get balance
                    match token.balance_of(address).await {
                        Ok((balance, symbol, decimals)) => {
                            let formatted_balance = token.format_balance(balance, decimals);
                            println!("    Balance: {} {}", formatted_balance, symbol);
                        },
                        Err(e) => println!("    Failed to get balance: {}", e),
                    }
                },
                Err(e) => println!("  Failed to get {} token info: {}", token_name, e),
            }
        },
        Err(_) => {
            // Token not available on this network - ignore
        },
    }
    
    Ok(())
}

async fn list_available_tokens(network: NeoNetwork) -> Result<(), Box<dyn Error>> {
    println!("Available tokens on {}:", network);
    
    // Common tokens on all networks
    println!("\nCommon Tokens:");
    println!("-------------");
    println!("  NEO: {}", network.get_contract_hash("neo").unwrap_or_default());
    println!("  GAS: {}", network.get_contract_hash("gas").unwrap_or_default());
    
    // Network-specific tokens
    if network.name == "MainNet" {
        println!("\nMainNet Tokens:");
        println!("--------------");
        println!("  FLM: {}", network.get_contract_hash("flm").unwrap_or_default());
        println!("  Neo Name Service: {}", network.get_contract_hash("neo_ns").unwrap_or_default());
        println!("  NeoFS: {}", network.get_contract_hash("neofs").unwrap_or_default());
    } else if network.name == "TestNet" {
        println!("\nTestNet Tokens:");
        println!("--------------");
        println!("  cNEO: {}", network.get_contract_hash("cneo").unwrap_or_default());
        println!("  cGAS: {}", network.get_contract_hash("cgas").unwrap_or_default());
        println!("  Neo Name Service: {}", network.get_contract_hash("neo_ns").unwrap_or_default());
        println!("  NeoFS: {}", network.get_contract_hash("neofs").unwrap_or_default());
    }
    
    Ok(())
}

async fn transfer_token(
    network: NeoNetwork,
    token_name: &str,
    from_wif: &str,
    to_address: &str,
    amount_str: &str,
) -> Result<(), Box<dyn Error>> {
    println!("Transferring {} token on {}:", token_name, network);
    
    // Parse amount
    let amount = amount_str.parse::<f64>()?;
    
    // Get account from WIF
    let account = Account::from_wif(from_wif)?;
    println!("  From: {}", account.address());
    println!("  To: {}", to_address);
    println!("  Amount: {}", amount);
    
    // Get token info
    let token = NetworkToken::new(network.clone(), token_name)?;
    let info = token.token_info().await?;
    println!("  Token: {} ({})", info.name, info.symbol);
    
    // Get client for network
    let client = network.create_client()?;
    
    // Convert amount to raw value
    let raw_amount = (amount * 10_f64.powi(info.decimals as i32)) as i64;
    
    // Create recipient address hash
    let to_address_obj = Address::from_str(to_address)?;
    let to_script_hash = to_address_obj.to_script_hash()?;
    
    // Create parameters for transfer
    let params = vec![
        ContractParameter::h160(&account.to_script_hash()?), // from
        ContractParameter::h160(&to_script_hash),        // to
        ContractParameter::integer(raw_amount),            // amount
        ContractParameter::any(),                          // data
    ];
    
    // Build script for the transfer
    let script = ScriptBuilder::new()
        .contract_call(
            &info.contract_hash,
            "transfer",
            &params,
            None,
        )?
        .to_bytes();
    
    // Create a signer from the account
    let signer = AccountSigner::from_account(account.clone())
        .with_scopes(vec![WitnessScope::CalledByEntry]);
    
    // Build the transaction
    let mut tx_builder = TransactionBuilder::new();
    tx_builder
        .script(script)
        .add_signer(signer)
        .valid_until_block(client.get_block_count().await? + 5760)?; // ~1 day
    
    // Add system fee
    let invocation_result = client.invoke_script(&tx_builder.get_script(), None).await?;
    tx_builder.system_fee(invocation_result.gas_consumed.parse::<i64>()?);
    
    // Calculate network fee and sign the transaction
    let tx = account.sign_transaction(tx_builder)?;
    
    // Send the transaction to the network
    println!("\nSending transaction to network...");
    let response = client.send_raw_transaction(&tx).await?;
    
    println!("Transaction sent! Hash: {}", response.hash);
    println!("\nYou can check the transaction status on a Neo explorer:");
    
    if network.name == "MainNet" {
        println!("https://explorer.onegate.space/transactionInfo/{}", response.hash);
        println!("https://neo3.neotube.io/transaction/{}", response.hash);
    } else if network.name == "TestNet" {
        println!("https://testnet.explorer.onegate.space/transactionInfo/{}", response.hash);
        println!("https://testnet.neo3.neotube.io/transaction/{}", response.hash);
    }
    
    Ok(())
}                        