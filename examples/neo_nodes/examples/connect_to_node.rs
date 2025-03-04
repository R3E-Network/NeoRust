use neo::{
	neo_clients::{HttpProvider, JsonRpcProvider},
	prelude::RpcClient,
};
use neo3 as neo;

/// This example demonstrates how to connect to a Neo N3 node and retrieve basic blockchain information.
/// It shows different connection methods and how to query node status.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	println!("Neo N3 Node Connection Example");
	println!("==============================");

	// Connect to Neo N3 MainNet using HTTP
	println!("\nConnecting to Neo N3 MainNet...");
	let mainnet_provider = HttpProvider::new("https://mainnet1.neo.org:443");
	let mainnet_client = RpcClient::new(mainnet_provider);

	// Get basic blockchain information
	let block_count = mainnet_client.get_block_count().await?;
	println!("MainNet current block count: {}", block_count);

	let version = mainnet_client.get_version().await?;
	println!("Node version: {}", version.user_agent);

	// Connect to Neo N3 TestNet
	println!("\nConnecting to Neo N3 TestNet...");
	let testnet_provider = HttpProvider::new("https://testnet1.neo.org:443");
	let testnet_client = RpcClient::new(testnet_provider);

	// Get basic blockchain information from TestNet
	let block_count = testnet_client.get_block_count().await?;
	println!("TestNet current block count: {}", block_count);

	// Get the latest block
	println!("\nRetrieving latest block information...");
	let latest_block_hash = testnet_client.get_best_block_hash().await?;
	println!("Latest block hash: {}", latest_block_hash);

	let latest_block = testnet_client.get_block(latest_block_hash, true).await?;
	println!("Latest block index: {}", latest_block.index);
	println!("Latest block time: {}", latest_block.time);
	println!("Latest block transaction count: {}", latest_block.tx.len());

	// Get network information
	println!("\nRetrieving network information...");
	let peers = testnet_client.get_peers().await?;
	println!("Connected peers: {}", peers.connected.len());

	// Check node health
	println!("\nChecking node health...");
	match testnet_client.ping().await {
		Ok(_) => println!("Node is healthy and responding to ping"),
		Err(e) => println!("Node health check failed: {}", e),
	}

	println!("\nConnection example completed successfully!");
	Ok(())
}
