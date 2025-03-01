// examples/nep17_token.rs
//
// This example demonstrates how to interact with NEP-17 tokens.
//
// Required features:
// - nep17
// - http-client
// - transaction
// - wallet
//
// Run with:
// cargo run --example nep17_token --features="nep17,http-client,transaction,wallet"

use neo::prelude::*;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Interacting with NEP-17 tokens on Neo N3...");
    
    // Connect to the Neo N3 TestNet
    let provider = HttpProvider::new("https://testnet1.neo.org:443")?;
    let client = RpcClient::new(provider);
    
    // Create a reference to the GAS token (NEP-17)
    let gas_hash = ScriptHash::from_str("d2a4cff31913016155e38e474a2c06d08be276cf")?;
    let gas_token = Nep17Token::new(gas_hash, client.clone());
    
    // Get token information
    let symbol = gas_token.symbol().await?;
    let decimals = gas_token.decimals().await?;
    let total_supply = gas_token.total_supply().await?;
    
    println!("Token: {} (Decimals: {})", symbol, decimals);
    println!("Total Supply: {}", total_supply);
    
    // Create a demo account
    let account = Account::create()?;
    println!("Created account with address: {}", account.address());
    
    // Check the account's token balance
    let balance = gas_token.balance_of(&account.get_script_hash()).await?;
    
    // Format the balance with proper decimal places
    let formatted_balance = format_token_amount(balance, decimals);
    println!("Account balance: {} {}", formatted_balance, symbol);
    
    // For a real transfer, you would need actual tokens:
    // 
    // // Create a transfer transaction (recipient would be another address)
    // let recipient = ScriptHash::from_str("0x5c9c3a340f4c28262e7042b908b5f7e7a4bcd7e7")?;
    // let amount = 1_0000_0000; // 1 GAS (8 decimals)
    //
    // let tx_builder = gas_token.create_transfer_tx(
    //     &account,
    //     &recipient,
    //     amount,
    //     None,
    // ).await?;
    //
    // // Sign and send the transaction
    // let tx = tx_builder.sign(&account).await?;
    // let tx_hash = client.send_raw_transaction(&tx).await?;
    // println!("Transfer transaction sent: {}", tx_hash);
    
    println!("NEP-17 token operations completed successfully!");
    Ok(())
}

// Helper function to format token amounts with decimals
fn format_token_amount(amount: u64, decimals: u8) -> String {
    let divisor = 10u64.pow(decimals as u32);
    let integer_part = amount / divisor;
    let fractional_part = amount % divisor;
    
    let fractional_str = format!("{:0width$}", fractional_part, width = decimals as usize);
    
    // Trim trailing zeros if needed
    let trimmed_fractional = fractional_str.trim_end_matches('0');
    
    if trimmed_fractional.is_empty() {
        format!("{}", integer_part)
    } else {
        format!("{}.{}", integer_part, trimmed_fractional)
    }
} 