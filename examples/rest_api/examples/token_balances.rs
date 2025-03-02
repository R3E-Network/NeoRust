use neo3::prelude::*;
use neo3::neo_clients::RestClient;
use std::env;

/// This example demonstrates how to use the REST API client to fetch token balances
/// for a Neo N3 address and display detailed information about each token
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Neo N3 REST API Token Balance Example");
    println!("====================================");
    
    // Parse command line arguments or use a default address
    let args: Vec<String> = env::args().collect();
    let address = if args.len() > 1 {
        args[1].clone()
    } else {
        "NVkg1yRMrTyY6QFnEkpP4WUFaviE1gFa3g".to_string() // Default address
    };
    
    // Create a REST client
    println!("Connecting to Neo N3 REST API...");
    let client = RestClient::new("https://testnet1.neo.org:443/api")?;
    
    // Get token balances for the address
    println!("\nFetching token balances for address: {}", address);
    match client.get_balances(&address).await {
        Ok(balances) => {
            if balances.balances.is_empty() {
                println!("No token balances found for this address");
            } else {
                println!("\nFound {} tokens:", balances.balances.len());
                println!("{:-<60}", "");
                
                for (i, token) in balances.balances.iter().enumerate() {
                    // Parse and format the balance
                    let amount = token.amount.parse::<f64>().unwrap_or(0.0);
                    let formatted_amount = format_balance(amount, token.decimals);
                    
                    println!("Token #{}", i+1);
                    println!("Name:        {}", token.name);
                    println!("Symbol:      {}", token.symbol);
                    println!("Balance:     {} {}", formatted_amount, token.symbol);
                    println!("Decimals:    {}", token.decimals);
                    println!("Asset Hash:  {}", token.asset_hash);
                    println!("{:-<60}", "");
                }
            }
        },
        Err(e) => {
            println!("Error fetching balances: {}", e);
        }
    }
    
    // Get blockchain information
    println!("Fetching blockchain information...");
    match client.get_height().await {
        Ok(height) => {
            println!("Current blockchain height: {}", height);
            
            // Get the latest block
            match client.get_block_by_index(height - 1).await {
                Ok(block) => {
                    println!("Latest block hash:        {}", block.hash);
                    println!("Latest block timestamp:   {}", block.time);
                    println!("Latest block tx count:    {}", block.transactions.len());
                },
                Err(e) => println!("Error fetching latest block: {}", e),
            }
        },
        Err(e) => println!("Error fetching blockchain height: {}", e),
    }
    
    println!("\nExample complete!");
    Ok(())
}

/// Format a balance with the correct number of decimals
fn format_balance(amount: f64, decimals: u8) -> String {
    let divisor = 10_f64.powi(decimals as i32);
    let formatted = amount / divisor;
    
    // Format with the correct number of decimal places
    match decimals {
        0 => format!("{:.0}", formatted),
        1 => format!("{:.1}", formatted),
        2 => format!("{:.2}", formatted),
        3 => format!("{:.3}", formatted),
        4 => format!("{:.4}", formatted),
        5 => format!("{:.5}", formatted),
        6 => format!("{:.6}", formatted),
        7 => format!("{:.7}", formatted),
        8 => format!("{:.8}", formatted),
        _ => format!("{}", formatted),
    }
} 