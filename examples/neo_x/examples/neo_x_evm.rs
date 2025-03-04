use neo::prelude::*;
use primitive_types::H160;
use std::str::FromStr;

/// Example demonstrating Neo X EVM compatibility layer usage
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	// Initialize the JSON-RPC provider for Neo N3
	let neo_provider = HttpProvider::new("https://mainnet1.neo.org:443")?;
	let neo_client = RpcClient::new(neo_provider);

	// Initialize the Neo X EVM provider
	let neo_x_provider = NeoXProvider::new("https://rpc.neo-x.org", Some(&neo_client));

	// Get the chain ID for Neo X
	let chain_id = neo_x_provider.chain_id().await?;
	println!("Neo X Chain ID: {}", chain_id);

	// Create a destination address for the transaction
	let destination = H160::from_str("0x1234567890123456789012345678901234567890")?;

	// Create transaction data (example: transfer function call)
	let data = vec![
		0xa9, 0x05, 0x9c, 0xbb, // Function selector for "transfer(address,uint256)"
		0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
		0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
		0x00, 0x01, // Recipient address padded to 32 bytes
		0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
		0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
		0x00, 0x01, // Amount (1) padded to 32 bytes
	];

	// Create a Neo X transaction
	let transaction = NeoXTransaction::new(
		Some(destination),
		data,
		0,              // Value (in wei)
		21000,          // Gas limit
		20_000_000_000, // Gas price (20 Gwei)
	);

	// Print transaction details
	println!("Transaction to: {:?}", transaction.to());
	println!("Transaction data length: {} bytes", transaction.data().len());
	println!("Transaction value: {} wei", transaction.value());
	println!("Transaction gas limit: {}", transaction.gas_limit());
	println!("Transaction gas price: {} wei", transaction.gas_price());

	// In a real scenario, you would sign and send the transaction
	// This is just a demonstration of the Neo X EVM compatibility layer

	Ok(())
}
