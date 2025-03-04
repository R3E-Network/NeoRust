use std::str::FromStr;

use neo::{
	neo_clients::{HttpProvider, JsonRpcProvider},
	neo_contract::{FungibleTokenContract, FungibleTokenTrait},
	neo_protocol::account::Account,
	neo_types::script_hash::ScriptHash,
	prelude::RpcClient,
};

/// This example demonstrates how to work with NEP-17 tokens on the Neo N3 blockchain.
/// It shows how to check balances, transfer tokens, and get token information.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	println!("Neo N3 NEP-17 Token Operations Example");
	println!("=====================================");

	// Connect to Neo N3 TestNet
	println!("\nConnecting to Neo N3 TestNet...");
	let provider = HttpProvider::new("https://testnet1.neo.org:443");
	let client = RpcClient::new(provider);

	// Load the account that will interact with tokens
	// In a real application, you would load your private key securely
	println!("\nSetting up account...");
	let account = Account::from_wif("YOUR_PRIVATE_KEY_WIF_HERE")?;
	println!("Account address: {}", account.get_address());

	// Define the NEP-17 tokens to interact with
	println!("\nSetting up token references...");

	// GAS token
	let gas_hash = ScriptHash::from_str("d2a4cff31913016155e38e474a2c06d08be276cf")?;
	let gas_token = FungibleTokenContract::new(gas_hash, Some(&client));

	// NEO token
	let neo_hash = ScriptHash::from_str("ef4073a0f2b305a38ec4050e4d3d28bc40ea63f5")?;
	let neo_token = FungibleTokenContract::new(neo_hash, Some(&client));

	// Get token information
	println!("\nRetrieving token information...");

	// GAS token info
	let gas_symbol = gas_token.symbol().await?;
	let gas_decimals = gas_token.decimals().await?;
	let gas_total_supply = gas_token.total_supply().await?;

	println!("GAS Token:");
	println!("  Symbol: {}", gas_symbol);
	println!("  Decimals: {}", gas_decimals);
	println!("  Total Supply: {}", gas_total_supply);

	// NEO token info
	let neo_symbol = neo_token.symbol().await?;
	let neo_decimals = neo_token.decimals().await?;
	let neo_total_supply = neo_token.total_supply().await?;

	println!("\nNEO Token:");
	println!("  Symbol: {}", neo_symbol);
	println!("  Decimals: {}", neo_decimals);
	println!("  Total Supply: {}", neo_total_supply);

	// Check token balances
	println!("\nChecking token balances...");

	// GAS balance
	let gas_balance = gas_token.balance_of(&account.get_script_hash()).await?;
	let gas_balance_formatted = gas_balance as f64 / 10f64.powi(gas_decimals as i32);
	println!("GAS Balance: {} {}", gas_balance_formatted, gas_symbol);

	// NEO balance
	let neo_balance = neo_token.balance_of(&account.get_script_hash()).await?;
	let neo_balance_formatted = neo_balance as f64 / 10f64.powi(neo_decimals as i32);
	println!("NEO Balance: {} {}", neo_balance_formatted, neo_symbol);

	// Transfer tokens
	println!("\nPreparing token transfer...");
	let recipient_address = "NbTiM6h8r99kpRtb428XcsUk1TzKed2gTc";
	let recipient = ScriptHash::from_address(recipient_address)?;

	// Prepare a GAS transfer
	let transfer_amount = 1_0000_0000; // 1 GAS (assuming 8 decimals)

	// Create a transaction builder for the transfer
	let tx_builder = gas_token
		.transfer(&account.get_script_hash(), &recipient, transfer_amount, None)
		.await?;

	// In a real application, you would sign and send this transaction
	// For this example, we'll just print the transaction details
	println!("Transaction builder created for token transfer");
	println!("Transfer details:");
	println!("  Token: {}", gas_symbol);
	println!("  From: {}", account.get_address());
	println!("  To: {}", recipient_address);
	println!(
		"  Amount: {} {} ({} raw units)",
		transfer_amount as f64 / 10f64.powi(gas_decimals as i32),
		gas_symbol,
		transfer_amount
	);

	// To actually execute the transfer, you would:
	/*
	// Add the account as a signer
	let tx_builder = tx_builder
		.set_signers(vec![account.into()])
		.valid_until_block(client.get_block_count().await? + 5760)?;

	// Sign and send the transaction
	let tx = tx_builder.sign().await?;
	let result = tx.send_tx().await?;

	println!("Transfer transaction sent! Hash: {}", result.hash);
	*/

	println!("\nNEP-17 token operations example completed successfully!");
	Ok(())
}
