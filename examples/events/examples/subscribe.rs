use neo3::prelude::*;
use neo3::neo_utils::network::NeoNetwork;
use std::sync::Arc;

/// This example demonstrates how to subscribe to and handle events from Neo N3 blockchain.
/// It shows how to set up a notification subscription for transaction and block events.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Neo N3 Event Subscription Example");
    println!("================================");

    // Connect to Neo N3 TestNet
    println!("\nConnecting to Neo N3 TestNet...");
    let network = NeoNetwork::TestNet;
    let client = network.create_client()?;
    let arc_client = Arc::new(client);
    
    // Subscribe to new blocks
    println!("\nSubscribing to new blocks...");
    // Note: In production code, you would need a websocket-enabled endpoint
    // This is just a demonstration of the API pattern
    
    // In an actual implementation with websocket support:
    // let ws_provider = WebSocketProvider::connect("wss://testnet.neoline.io:10331").await?;
    // let mut block_stream = ws_provider.subscribe_blocks().await?;
    // while let Some(block) = block_stream.next().await {
    //     println!("New block: {}", block.hash);
    // }
    
    // As an alternative, for demonstration purposes, we'll poll for new blocks
    let current_block = arc_client.get_block_count().await?;
    println!("Current block number: {}", current_block);
    
    println!("\nPolling for new blocks (simulating subscription)...");
    for _ in 0..5 {
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        let new_block = arc_client.get_block_count().await?;
        if new_block > current_block {
            println!("New block detected: {}", new_block);
        } else {
            println!("No new block yet, still at: {}", new_block);
        }
    }
    
    println!("\nNote: For real-time event notifications, you'd need a WebSocket connection to a Neo N3 node that supports notifications.");
    
    Ok(())
}
