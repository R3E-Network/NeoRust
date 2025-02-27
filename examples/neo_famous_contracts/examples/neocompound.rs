use std::str::FromStr;

use neo::{
    neo_clients::{HttpProvider, JsonRpcProvider},
    neo_contract::famous::NeoCompoundContract,
    neo_protocol::account::Account,
    neo_types::script_hash::ScriptHash,
    prelude::RpcClient,
};

/// This example demonstrates how to interact with the NeoCompound contract on Neo N3.
/// It shows how to deposit tokens, withdraw tokens, compound interest, and get the APY.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("NeoCompound Contract Example");
    println!("===========================");

    // Connect to Neo N3 TestNet
    println!("\nConnecting to Neo N3 TestNet...");
    let provider = HttpProvider::new("https://testnet1.neo.org:443");
    let client = RpcClient::new(provider);
    
    // Create a NeoCompound contract instance
    println!("\nInitializing NeoCompound contract...");
    let neocompound = NeoCompoundContract::new(Some(&client));
    println!("Contract hash: {}", neocompound.script_hash());
    
    // Create an account for the transaction signer
    // In a real application, you would load your private key securely
    println!("\nSetting up account...");
    let account = Account::from_wif("YOUR_WIF_KEY_HERE")?;
    println!("Account address: {}", account.get_address());
    
    // Define token script hash (this is an example, use actual token hash in production)
    let gas_token = ScriptHash::from_str("d2a4cff31913016155e38e474a2c06d08be276cf")?;
    
    // Example 1: Get the APY for a token
    println!("\nExample 1: Get the APY for a token");
    println!("Fetching the current APY for GAS token...");
    
    let apy = neocompound.get_apy(&gas_token).await?;
    
    println!("Current APY for GAS: {}%", apy);
    
    // Example 2: Deposit tokens
    println!("\nExample 2: Deposit tokens");
    println!("Creating a transaction to deposit 10 GAS...");
    let amount = 10_0000_0000; // 10 GAS (8 decimals)
    
    let deposit_tx = neocompound.deposit(
        &gas_token,
        amount,
        &account,
    ).await?;
    
    println!("Transaction created successfully!");
    println!("To execute this transaction, you would sign and send it:");
    println!("let signed_tx = deposit_tx.sign().await?;");
    println!("let result = signed_tx.send_tx().await?;");
    
    // Example 3: Compound interest
    println!("\nExample 3: Compound interest");
    println!("Creating a transaction to compound interest for GAS token...");
    
    let compound_tx = neocompound.compound(
        &gas_token,
        &account,
    ).await?;
    
    println!("Transaction created successfully!");
    
    // Example 4: Withdraw tokens
    println!("\nExample 4: Withdraw tokens");
    println!("Creating a transaction to withdraw 5 GAS...");
    let amount = 5_0000_0000; // 5 GAS (8 decimals)
    
    let withdraw_tx = neocompound.withdraw(
        &gas_token,
        amount,
        &account,
    ).await?;
    
    println!("Transaction created successfully!");
    
    println!("\nNeoCompound example completed successfully!");
    Ok(())
}
