/// This example demonstrates how to interact with smart contracts on the Neo N3 blockchain.
use neo3::prelude::*;
use neo3::neo_contract::{NeoToken, GasToken, FungibleTokenContract};
use std::str::FromStr;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Neo N3 Smart Contract Interaction Example");
    println!("========================================");
    
    // Connect to Neo N3 TestNet
    println!("\nStep 1: Connecting to Neo N3 TestNet...");
    let provider = HttpProvider::new("https://testnet1.neo.org:443")?;
    let client = RpcClient::new(provider);
    let block_count = client.get_block_count().await?;
    println!("Connected to Neo N3 TestNet at block height: {}", block_count);
    
    // System Contracts Example
    println!("\nStep 2: Interacting with System Contracts");
    
    // NeoToken contract
    println!("\n2.1: NEO Token Contract");
    let neo_token = NeoToken::new(&client);
    
    let symbol = neo_token.symbol().await?;
    let decimals = neo_token.decimals().await?;
    let total_supply = neo_token.total_supply().await?;
    
    println!("Token: {} (Decimals: {})", symbol, decimals);
    println!("Total Supply: {}", total_supply);
    
    // Get committee members
    let committee = neo_token.get_committee().await?;
    println!("Committee Members: {}", committee.len());
    if !committee.is_empty() {
        println!("First committee member: {}", committee[0]);
    }
    
    // GasToken contract
    println!("\n2.2: GAS Token Contract");
    let gas_token = GasToken::new(&client);
    
    let symbol = gas_token.symbol().await?;
    let decimals = gas_token.decimals().await?;
    let total_supply = gas_token.total_supply().await?;
    
    println!("Token: {} (Decimals: {})", symbol, decimals);
    println!("Total Supply: {}", total_supply);
    
    // Custom NEP-17 Token Example
    println!("\nStep 3: Interacting with Custom NEP-17 Tokens");
    
    // This example uses Flamingo (FLM) token on TestNet
    // Replace with an actual token contract address on the network you're connected to
    let token_hash = ScriptHash::from_str("d2a4cff31913016155e38e474a2c06d08be276cf")?;
    println!("Using token with hash: {}", token_hash);
    
    let token = FungibleTokenContract::new(token_hash, client.clone());
    
    // Try to get token information - note this might fail if the token doesn't exist on TestNet
    println!("Attempting to fetch token information...");
    match token.symbol().await {
        Ok(symbol) => {
            let decimals = token.decimals().await?;
            let total_supply = token.total_supply().await?;
            
            println!("Token: {} (Decimals: {})", symbol, decimals);
            println!("Total Supply: {}", total_supply);
        },
        Err(e) => {
            println!("Could not fetch token information: {}", e);
            println!("This is expected if the token contract doesn't exist on TestNet.");
            println!("Try with a different token hash or network.");
        }
    }
    
    // Direct Contract Invocation
    println!("\nStep 4: Direct Contract Invocation");
    println!("Demonstrating direct contract invocation using ScriptBuilder...");
    
    // Create a script to call the 'symbol' method on the GAS token contract
    let gas_hash = ScriptHash::from_str("d2a4cff31913016155e38e474a2c06d08be276cf")?;
    
    let script = ScriptBuilder::build_contract_call(
        &gas_hash,
        "symbol",
        &[],
    )?;
    
    // Invoke the script
    println!("Invoking 'symbol' method on GAS token contract...");
    let result = client.invoke_script(&script, None).await?;
    
    // Process the result
    if let Some(stack_item) = result.stack.first() {
        if let Some(value) = stack_item.as_string() {
            println!("Result from direct invocation: {}", value);
        } else {
            println!("Unexpected result type");
        }
    } else {
        println!("No result returned");
    }
    
    // Check Account Balance Example
    println!("\nStep 5: Checking Account Balance");
    
    // Use a well-known TestNet address
    let address = "NZNos2WqTbu5oCgyfss9kUJgBXJqhuYAaj";
    println!("Checking balance for address: {}", address);
    
    let account_hash = ScriptHash::from_address(address)?;
    
    // Check NEO balance
    let neo_balance = neo_token.balance_of(&account_hash).await?;
    println!("NEO Balance: {}", neo_balance);
    
    // Check GAS balance
    let gas_balance = gas_token.balance_of(&account_hash).await?;
    println!("GAS Balance: {}", gas_balance);
    
    // Transfer Tokens Example (commented out to avoid actual transfers)
    println!("\nStep 6: Token Transfer Example (code shown but not executed)");
    println!("To transfer tokens, you would use code like this:");
    println!("
    // Load your account with private key
    let account = Account::from_wif(\"your-private-key-wif\")?;
    
    // Recipient address
    let recipient = ScriptHash::from_address(\"NZNos2WqTbu5oCgyfss9kUJgBXJqhuYAaj\")?;
    
    // Amount to transfer (e.g., 1 GAS)
    let amount = 1_0000_0000; // 1 GAS (with 8 decimals)
    
    // Execute the transfer
    let tx_hash = gas_token.transfer(
        &account,
        &recipient,
        amount,
        None, // No data
    ).await?;
    
    println!(\"Transfer transaction sent: {}\", tx_hash);
    ");
    
    println!("\nSmart contract interaction example completed!");
    Ok(())
}