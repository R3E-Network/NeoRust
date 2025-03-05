use neo3::prelude::*;
use std::str::FromStr;

/// Example demonstrating Neo X Bridge contract interactions
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	// Initialize the JSON-RPC provider for Neo N3
	let neo_provider = HttpProvider::new("https://mainnet1.neo.org:443")?;
	let neo_client = RpcClient::new(neo_provider);

	// Create an account for signing transactions
	// In a real scenario, you would use your own private key
	let account = Account::from_wif("L1WMhxazScMhUrdv34JqQb1HFSQmWeN2Kpc1R9JGKwL7CDNP21uR")?;

	// Initialize the Neo X Bridge contract
	let bridge = NeoXBridgeContract::new(Some(&neo_client));

	// Get the GAS token script hash
	let gas_token = ScriptHash::from_str("d2a4cff31913016155e38e474a2c06d08be276cf")?;

	// Get the bridge fee for GAS token
	let fee = bridge.get_fee(&gas_token).await?;
	println!("Bridge fee for GAS: {} GAS", fee as f64 / 100_000_000.0);

	// Get the bridge cap for GAS token
	let cap = bridge.get_cap(&gas_token).await?;
	println!("Bridge cap for GAS: {} GAS", cap as f64 / 100_000_000.0);

	// Example: Deposit GAS from Neo N3 to Neo X
	// In a real scenario, you would use your own Neo X address
	let neo_x_address = "0x1234567890123456789012345678901234567890";
	let amount = 1_0000_0000; // 1 GAS (8 decimals)

	println!(
		"Preparing to deposit {} GAS to Neo X address: {}",
		amount as f64 / 100_000_000.0,
		neo_x_address
	);

	// Build the deposit transaction
	// Note: In a real scenario, you would actually send this transaction
	let deposit_builder = bridge.deposit(&gas_token, amount, neo_x_address, &account).await?;

	println!("Deposit transaction prepared successfully");

	// Example: Withdraw GAS from Neo X to Neo N3
	// In a real scenario, you would use your own Neo N3 address
	let neo_n3_address = "NbTiM6h8r99kpRtb428XcsUk1TzKed2gTc";

	println!(
		"Preparing to withdraw {} GAS to Neo N3 address: {}",
		amount as f64 / 100_000_000.0,
		neo_n3_address
	);

	// Build the withdraw transaction
	// Note: In a real scenario, you would actually send this transaction
	let withdraw_builder = bridge.withdraw(&gas_token, amount, neo_n3_address, &account).await?;

	println!("Withdraw transaction prepared successfully");

	// In a real scenario, you would sign and send these transactions
	// This is just a demonstration of the Neo X Bridge contract interactions

	Ok(())
}
