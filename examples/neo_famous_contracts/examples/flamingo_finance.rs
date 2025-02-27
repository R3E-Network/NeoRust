use std::str::FromStr;

use neo::{
    neo_clients::{HttpProvider, JsonRpcProvider},
    neo_contract::famous::FlamingoContract,
    neo_protocol::account::Account,
    neo_types::script_hash::ScriptHash,
    prelude::RpcClient,
};

/// This example demonstrates how to interact with the Flamingo Finance contract on Neo N3.
/// It shows how to perform token swaps, add/remove liquidity, stake tokens, and claim rewards.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Flamingo Finance Contract Example");
    println!("================================");

    // Connect to Neo N3 TestNet
    println!("\nConnecting to Neo N3 TestNet...");
    let provider = HttpProvider::new("https://testnet1.neo.org:443");
    let client = RpcClient::new(provider);
    
    // Create a Flamingo Finance contract instance
    println!("\nInitializing Flamingo Finance contract...");
    let flamingo = FlamingoContract::new(Some(&client));
    println!("Contract hash: {}", flamingo.script_hash());
    
    // Create an account for the transaction signer
    // In a real application, you would load your private key securely
    println!("\nSetting up account...");
    let account = Account::from_wif("YOUR_WIF_KEY_HERE")?;
    println!("Account address: {}", account.get_address());
    
    // Define token script hashes (these are examples, use actual token hashes in production)
    let neo_token = ScriptHash::from_str("ef4073a0f2b305a38ec4050e4d3d28bc40ea63f5")?;
    let gas_token = ScriptHash::from_str("d2a4cff31913016155e38e474a2c06d08be276cf")?;
    
    // Example 1: Swap tokens
    println!("\nExample 1: Swap tokens");
    println!("Creating a transaction to swap 1 NEO for GAS...");
    let amount = 1_0000_0000; // 1 NEO (8 decimals)
    let min_return = 0; // Accept any amount of GAS in return
    
    let swap_tx = flamingo.swap(
        &neo_token,
        &gas_token,
        amount,
        min_return,
        &account,
    ).await?;
    
    println!("Transaction created successfully!");
    println!("To execute this transaction, you would sign and send it:");
    println!("let signed_tx = swap_tx.sign().await?;");
    println!("let result = signed_tx.send_tx().await?;");
    
    // Example 2: Add liquidity
    println!("\nExample 2: Add liquidity");
    println!("Creating a transaction to add liquidity to NEO/GAS pool...");
    let amount_neo = 1_0000_0000; // 1 NEO
    let amount_gas = 2_0000_0000; // 2 GAS
    
    let add_liquidity_tx = flamingo.add_liquidity(
        &neo_token,
        &gas_token,
        amount_neo,
        amount_gas,
        &account,
    ).await?;
    
    println!("Transaction created successfully!");
    
    // Example 3: Remove liquidity
    println!("\nExample 3: Remove liquidity");
    println!("Creating a transaction to remove liquidity from NEO/GAS pool...");
    let liquidity_amount = 5_0000_0000; // 5 LP tokens
    
    let remove_liquidity_tx = flamingo.remove_liquidity(
        &neo_token,
        &gas_token,
        liquidity_amount,
        &account,
    ).await?;
    
    println!("Transaction created successfully!");
    
    // Example 4: Stake tokens
    println!("\nExample 4: Stake tokens");
    println!("Creating a transaction to stake LP tokens...");
    let stake_amount = 5_0000_0000; // 5 LP tokens
    
    // Assuming LP token script hash (use actual LP token hash in production)
    let lp_token = ScriptHash::from_str("c36aee199dbba6c3f439983657558cfb67629599")?;
    
    let stake_tx = flamingo.stake(
        &lp_token,
        stake_amount,
        &account,
    ).await?;
    
    println!("Transaction created successfully!");
    
    // Example 5: Claim rewards
    println!("\nExample 5: Claim rewards");
    println!("Creating a transaction to claim staking rewards...");
    
    let claim_tx = flamingo.claim_rewards(&account).await?;
    
    println!("Transaction created successfully!");
    
    println!("\nFlamingo Finance example completed successfully!");
    Ok(())
}
