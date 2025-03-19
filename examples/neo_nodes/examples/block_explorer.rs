use neo3::{
    neo_clients::{HttpProvider, JsonRpcProvider, RpcClient},
};
use neo3 as neo;

/// This example demonstrates how to build a simple block explorer to view block and transaction details.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Neo N3 Block Explorer Example");
    println!("============================");

    // Connect to Neo N3 TestNet
    println!("\n1. Connecting to Neo N3 TestNet...");
    let provider = HttpProvider::new("https://testnet1.neo.org:443");
    let client = RpcClient::new(provider);
    println!("Connected to TestNet");

    // Get the current block height
    println!("\n2. Getting block information...");
    let block_count = client.get_block_count().await?;
    println!("Current block count: {}", block_count);

    // Get the latest block
    let latest_block_index = block_count - 1;
    println!("\n3. Retrieving latest block (index {})...", latest_block_index);
    let latest_block = client.get_block_by_index(latest_block_index, 1).await?;
    
    println!("Block hash: {}", latest_block.hash);
    println!("Block size: {} bytes", latest_block.size);
    println!("Block time: {}", latest_block.time);
    println!("Block version: {}", latest_block.version);
    println!("Previous block: {}", latest_block.prev_block_hash);
    println!("Merkle root: {}", latest_block.merkle_root);
    println!("Transaction count: {}", latest_block.tx.len());

    // Display transaction information
    println!("\n4. Examining transactions in this block...");
    for (i, tx) in latest_block.tx.iter().take(3).enumerate() {
        println!("\nTransaction #{}", i + 1);
        println!("Hash: {}", tx.hash);
        println!("Size: {} bytes", tx.size);
        println!("Version: {}", tx.version);
        println!("Nonce: {}", tx.nonce);
        println!("Sender: {}", tx.sender);
        println!("System fee: {}", tx.sys_fee);
        println!("Network fee: {}", tx.net_fee);
        println!("Valid until block: {}", tx.valid_until_block);
    }

    // If there are more than 3 transactions, show a message
    if latest_block.tx.len() > 3 {
        println!("\n... and {} more transactions", latest_block.tx.len() - 3);
    }

    // Get network information
    println!("\n5. Retrieving network information...");
    let peers = client.get_peers().await?;
    println!("Connected peers: {}", peers.connected.len());
    println!("Unconnected peers: {}", peers.unconnected.len());
    println!("Bad peers: {}", peers.bad.len());

    println!("\nBlock explorer example completed successfully!");
    Ok(())
} 