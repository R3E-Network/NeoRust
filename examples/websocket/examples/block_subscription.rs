use neo3::prelude::*;
use neo3::neo_clients::WsProvider;
use std::time::Duration;

/// This example demonstrates how to use WebSocket connections to subscribe to blockchain events
/// It shows how to subscribe to new blocks as they are produced on the Neo N3 network
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Neo N3 WebSocket Block Subscription Example");
    println!("===========================================");
    
    // Connect to a Neo N3 WebSocket endpoint
    println!("Connecting to Neo N3 WebSocket...");
    let ws_url = "wss://testnet1.neo.org:60002/ws"; // Replace with an actual WebSocket endpoint
    let provider = WsProvider::connect(ws_url).await?;
    
    // Subscribe to new blocks
    println!("Subscribing to new blocks...");
    let mut block_subscription = provider.subscribe_blocks().await?;
    
    println!("Waiting for new blocks...");
    println!("(This will run for 5 minutes or until you press Ctrl+C)");
    
    // Run for 5 minutes or until Ctrl+C
    let end_time = std::time::Instant::now() + Duration::from_secs(300);
    
    while std::time::Instant::now() < end_time {
        // Use tokio::select to handle both the next block and a timeout
        tokio::select! {
            // Wait for the next block with a timeout
            block = tokio::time::timeout(Duration::from_secs(30), block_subscription.next()) => {
                match block {
                    Ok(Some(block)) => {
                        println!("\nNew block received!");
                        println!("Block Hash: {}", block.hash);
                        println!("Block Index: {}", block.index);
                        println!("Timestamp: {}", block.time);
                        println!("Transactions: {}", block.transactions.len());
                        
                        // Print first few transactions if any
                        if !block.transactions.is_empty() {
                            println!("First few transactions:");
                            for (i, tx) in block.transactions.iter().take(3).enumerate() {
                                println!("  {}. {}", i+1, tx);
                            }
                            if block.transactions.len() > 3 {
                                println!("  ... and {} more", block.transactions.len() - 3);
                            }
                        }
                    },
                    Ok(None) => {
                        println!("Subscription ended");
                        break;
                    },
                    Err(_) => {
                        println!("Timeout waiting for block, but connection is still active");
                    }
                }
            },
            // Also provide a way to exit if nothing happens for a while
            _ = tokio::time::sleep(Duration::from_secs(120)) => {
                println!("No blocks received for 2 minutes, exiting");
                break;
            }
        }
    }
    
    println!("Example complete!");
    Ok(())
} 