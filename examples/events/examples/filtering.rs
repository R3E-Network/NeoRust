use neo3::prelude::*;
use neo3::neo_utils::network::NeoNetwork;
use primitive_types::H256;
use std::str::FromStr;

/// This example demonstrates how to filter and retrieve application logs from transactions
/// on the Neo N3 blockchain. It shows how to query transaction notifications and filter by contract.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Neo N3 Application Log Filtering Example");
    println!("======================================");

    // Connect to Neo N3 TestNet
    println!("\nConnecting to Neo N3 TestNet...");
    let client = NeoNetwork::TestNet.create_client()?;
    
    // Get recent blocks
    let current_height = client.get_block_count().await?;
    println!("Current block height: {}", current_height);
    
    // Let's get a block with some transactions
    let block = client.get_block_by_index(current_height - 1, true).await?;
    
    println!("\nFound block {} with {} transactions", block.hash, block.transactions.len());
    
    // Filter for transactions with application logs
    let mut notification_count = 0;
    
    for tx_hash in block.transactions.iter() {
        println!("\nChecking transaction: {}", tx_hash);
        
        // Get application logs for this transaction
        match client.get_application_log(H256::from_str(&tx_hash.to_string())?).await {
            Ok(app_log) => {
                println!("  Transaction executed with state: {}", app_log.execution.state);
                println!("  Gas consumed: {}", app_log.execution.gas_consumed);
                
                // Print notifications
                if !app_log.execution.notifications.is_empty() {
                    println!("  Notifications:");
                    for (i, notification) in app_log.execution.notifications.iter().enumerate() {
                        notification_count += 1;
                        println!("    {}: Contract: {}", i+1, notification.contract);
                        println!("       Event Name: {}", notification.event_name);
                        println!("       State Items: {}", notification.state.len());
                        
                        // Print state items (first few)
                        if !notification.state.is_empty() {
                            println!("       First state item: {:?}", notification.state[0]);
                            if notification.state.len() > 1 {
                                println!("       Second state item: {:?}", notification.state[1]);
                            }
                        }
                    }
                } else {
                    println!("  No notifications found");
                }
            },
            Err(e) => {
                println!("  Failed to get application log: {}", e);
            }
        }
    }
    
    println!("\nTotal notifications found: {}", notification_count);
    
    // How to filter for specific contract notifications (example only)
    println!("\nTo filter for specific contract notifications:");
    println!("1. Get transaction logs");
    println!("2. Filter by contract address");
    println!("3. Filter by event name");
    println!("4. Process the state array with the event parameters");
    
    Ok(())
}
