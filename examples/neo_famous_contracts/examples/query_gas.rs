use neo3::{
    neo_clients::{HttpProvider, JsonRpcProvider, RpcClient},
    neo_types::{
        script_hash::ScriptHash,
        contract::{ContractParameter, ContractParameterType},
    },
};
use std::str::FromStr;
use neo3 as neo;

/// This example demonstrates how to query information about the GAS token on the Neo blockchain.
/// It shows how to connect to the blockchain and query token information directly using the contract interface.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Neo N3 GAS Token Query Example");
    println!("==============================");

    // Connect to Neo N3 TestNet
    println!("\n1. Connecting to Neo N3 TestNet...");
    let provider = HttpProvider::new("https://testnet1.neo.org:443")?;
    let client = RpcClient::new(provider);
    println!("Connected to TestNet");

    // Get the TestNet blockchain height
    println!("\n2. Getting current block height...");
    let block_count = client.get_block_count().await?;
    println!("Current block height: {}", block_count);

    // Initialize the GAS token contract
    println!("\n3. Initializing GAS token contract...");
    
    // GAS token hash on TestNet
    let gas_token_hash = ScriptHash::from_str("0xd2a4cff31913016155e38e474a2c06d08be276cf")?;
    
    use neo3::neo_contract::SmartContract;
    let gas_contract = SmartContract::new(gas_token_hash, client.clone());
    println!("GAS token contract initialized");

    // Query basic GAS token information
    println!("\n4. Querying GAS token information...");
    
    // Get symbol
    let symbol_result = gas_contract.call_function("symbol", Vec::new()).await?;
    let symbol = symbol_result.first().unwrap().value.as_str().unwrap_or("Unknown");
    println!("Token symbol: {}", symbol);
    
    // Get decimals
    let decimals_result = gas_contract.call_function("decimals", Vec::new()).await?;
    let decimals = decimals_result.first().unwrap().value.as_int().unwrap_or(0);
    println!("Token decimals: {}", decimals);
    
    // Get total supply
    let total_supply_result = gas_contract.call_function("totalSupply", Vec::new()).await?;
    let total_supply = total_supply_result.first().unwrap().value.as_int().unwrap_or(0);
    println!("Total supply: {}", total_supply);
    
    // Query GAS balance of a specific address
    println!("\n5. Querying GAS balance for a sample address...");
    let address = "NZNos2WqTbu5oCgyfss9kUJgBXJqhuYAaj"; // Example address
    let script_hash = ScriptHash::from_address(address)?;
    
    // Create parameter for the contract call
    let balance_params = vec![
        ContractParameter::new_hash160(script_hash)
    ];
    
    let balance_result = gas_contract.call_function("balanceOf", balance_params).await?;
    let balance = balance_result.first().unwrap().value.as_int().unwrap_or(0);
    println!("GAS balance of {}: {}", address, balance);
    
    println!("\nGAS token query example completed successfully!");
    Ok(())
} 