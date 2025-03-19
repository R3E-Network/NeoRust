use neo3::{
    neo_clients::{HttpProvider, JsonRpcProvider, RpcClient},
    neo_contract::SmartContract,
    neo_types::{
        script_hash::ScriptHash,
        contract::{ContractParameter, ContractParameterType},
    },
};
use std::str::FromStr;
use neo3 as neo;

/// This example demonstrates how to query information about the NEO token on the Neo blockchain.
/// It shows how to connect to the blockchain and query various token information directly.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Neo N3 NEO Token Query Example");
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

    // Initialize the NEO token contract
    println!("\n3. Initializing NEO token contract...");
    // NEO token hash on TestNet
    let neo_token_hash = ScriptHash::from_str("0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5")?;
    let neo_contract = SmartContract::new(neo_token_hash, client.clone());
    println!("NEO token contract initialized");

    // Query basic NEO token information
    println!("\n4. Querying NEO token information...");
    
    // Get symbol
    let symbol_result = neo_contract.call_function("symbol", Vec::new()).await?;
    let symbol = symbol_result.first().unwrap().value.as_str().unwrap_or("Unknown");
    println!("Token symbol: {}", symbol);
    
    // Get decimals
    let decimals_result = neo_contract.call_function("decimals", Vec::new()).await?;
    let decimals = decimals_result.first().unwrap().value.as_int().unwrap_or(0);
    println!("Token decimals: {}", decimals);
    
    // Get total supply
    let total_supply_result = neo_contract.call_function("totalSupply", Vec::new()).await?;
    let total_supply = total_supply_result.first().unwrap().value.as_int().unwrap_or(0);
    println!("Total supply: {}", total_supply);
    
    // Query NEO balance of a specific address
    println!("\n5. Querying NEO balance for a sample address...");
    let address = "NZNos2WqTbu5oCgyfss9kUJgBXJqhuYAaj"; // Example address
    let script_hash = ScriptHash::from_address(address)?;
    
    // Create parameter for the contract call
    let balance_params = vec![
        ContractParameter::new_hash160(script_hash)
    ];
    
    let balance_result = neo_contract.call_function("balanceOf", balance_params).await?;
    let balance = balance_result.first().unwrap().value.as_int().unwrap_or(0);
    println!("NEO balance of {}: {}", address, balance);
    
    // Get all candidates (committee members)
    println!("\n6. Retrieving NEO candidates...");
    let candidates_result = neo_contract.call_function("getCandidates", Vec::new()).await?;
    
    if let Some(candidate_array) = candidates_result.first().map(|r| r.value.as_array()) {
        if let Some(candidates) = candidate_array {
            println!("Number of candidates: {}", candidates.len());
            
            if !candidates.is_empty() && candidates.len() > 0 {
                if let Some(first_candidate) = candidates.first() {
                    if let Some(candidate_map) = first_candidate.as_map() {
                        let public_key = candidate_map.get("publicKey").and_then(|v| v.as_str()).unwrap_or("Unknown");
                        let votes = candidate_map.get("votes").and_then(|v| v.as_int()).unwrap_or(0);
                        println!("First candidate public key: {}", public_key);
                        println!("First candidate votes: {}", votes);
                    }
                }
            } else {
                println!("No candidates found");
            }
        }
    } else {
        println!("No candidates found or unexpected response format");
    }

    println!("\nNEO token query example completed successfully!");
    Ok(())
} 