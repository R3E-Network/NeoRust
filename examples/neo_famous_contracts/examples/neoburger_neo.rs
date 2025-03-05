use std::str::FromStr;

use neo3::prelude::*;
use neo_clients::{HttpProvider, JsonRpcProvider};
use neo_contract::famous::NeoburgerContract;
use neo_protocol::account::Account;
use neo_types::script_hash::ScriptHash;

/// This example demonstrates how to interact with the NeoburgerNeo (bNEO) contract on Neo N3.
/// It shows how to wrap NEO to bNEO, unwrap bNEO to NEO, claim GAS, and get the exchange rate.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	println!("NeoburgerNeo (bNEO) Contract Example");
	println!("==================================");

	// Connect to Neo N3 TestNet
	println!("\nConnecting to Neo N3 TestNet...");
	let provider = HttpProvider::new("https://testnet1.neo.org:443");
	let client = RpcClient::new(provider);

	// Create a NeoburgerNeo contract instance
	println!("\nInitializing NeoburgerNeo contract...");
	let neoburger = NeoburgerContract::new(Some(&client));
	println!("Contract hash: {}", neoburger.script_hash());
	println!("Token symbol: {}", neoburger.symbol().unwrap_or_default());
	println!("Token decimals: {}", neoburger.decimals().unwrap_or_default());

	// Create an account for the transaction signer
	// In a real application, you would load your private key securely
	println!("\nSetting up account...");
	let account = Account::from_wif("YOUR_WIF_KEY_HERE")?;
	println!("Account address: {}", account.get_address());

	// Example 1: Get the exchange rate
	println!("\nExample 1: Get the exchange rate");
	println!("Fetching the current NEO to bNEO exchange rate...");

	let rate = neoburger.get_rate().await?;

	println!("Current exchange rate: 1 NEO = {} bNEO", rate);

	// Example 2: Wrap NEO to bNEO
	println!("\nExample 2: Wrap NEO to bNEO");
	println!("Creating a transaction to wrap 1 NEO to bNEO...");
	let amount = 1_0000_0000; // 1 NEO (8 decimals)

	let wrap_tx = neoburger.wrap(amount, &account).await?;

	println!("Transaction created successfully!");
	println!("To execute this transaction, you would sign and send it:");
	println!("let signed_tx = wrap_tx.sign().await?;");
	println!("let result = signed_tx.send_tx().await?;");

	// Example 3: Unwrap bNEO to NEO
	println!("\nExample 3: Unwrap bNEO to NEO");
	println!("Creating a transaction to unwrap 1 bNEO to NEO...");
	let amount = 1_0000_0000; // 1 bNEO (8 decimals)

	let unwrap_tx = neoburger.unwrap(amount, &account).await?;

	println!("Transaction created successfully!");

	// Example 4: Claim GAS
	println!("\nExample 4: Claim GAS");
	println!("Creating a transaction to claim GAS rewards from bNEO holdings...");

	let claim_tx = neoburger.claim_gas(&account).await?;

	println!("Transaction created successfully!");

	println!("\nNeoburgerNeo example completed successfully!");
	Ok(())
}
