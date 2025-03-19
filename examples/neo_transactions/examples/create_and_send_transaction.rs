use std::str::FromStr;

use neo3::{
	neo_builder::TransactionBuilder,
	neo_clients::{HttpProvider, JsonRpcProvider},
	neo_contract::GasToken,
	neo_protocol::account::Account,
	neo_types::{contract::ContractParameter, script_hash::ScriptHash},
	prelude::{RpcClient, ScriptBuilder},
};

/// This example demonstrates how to create, sign, and send a transaction on the Neo N3 blockchain.
/// It shows the process of transferring GAS tokens from one account to another.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	println!("Neo N3 Transaction Creation and Sending Example");
	println!("==============================================");

	// Connect to Neo N3 TestNet
	println!("\nConnecting to Neo N3 TestNet...");
	let provider = HttpProvider::new("https://testnet1.neo.org:443");
	let client = RpcClient::new(provider);

	// Create accounts for the sender and recipient
	// In a real application, you would load your private key securely
	println!("\nSetting up accounts...");
	let sender = Account::from_wif("YOUR_SENDER_WIF_HERE")?;
	println!("Sender address: {}", sender.get_address());

	// The recipient can be any valid Neo N3 address
	let recipient_address = "NbTiM6h8r99kpRtb428XcsUk1TzKed2gTc";
	let recipient = ScriptHash::from_address(recipient_address)?;
	println!("Recipient address: {}", recipient_address);

	// Get the GAS token contract
	println!("\nPreparing GAS token transfer...");
	let gas_token_hash = ScriptHash::from_str("d2a4cff31913016155e38e474a2c06d08be276cf")?;
	let gas_token = GasToken::new(gas_token_hash, Some(&client));

	// Check sender's GAS balance
	let sender_balance = gas_token.balance_of(&sender.get_script_hash()).await?;
	println!("Sender's GAS balance: {}", sender_balance);

	// Create a transaction to transfer 1 GAS
	println!("\nCreating transaction...");
	let amount = 1_0000_0000; // 1 GAS (GAS has 8 decimals)

	// Build the transaction using the ScriptBuilder
	let script = ScriptBuilder::new()
		.contract_call(
			&gas_token_hash,
			"transfer",
			&[
				ContractParameter::hash160(&sender.get_script_hash()),
				ContractParameter::hash160(&recipient),
				ContractParameter::integer(amount),
				ContractParameter::any(None),
			],
			None,
		)?
		.to_bytes();

	// Create a transaction builder
	let mut tx_builder = TransactionBuilder::with_client(&client);

	// Configure the transaction
	tx_builder
		.script(Some(script))
		.set_signers(vec![sender.clone().into()])
		.valid_until_block(client.get_block_count().await? + 5760)?; // Valid for ~1 day

	// Sign the transaction
	println!("\nSigning transaction...");
	let tx = tx_builder.sign().await?;

	// In a real application, you would send the transaction
	// For this example, we'll just print the transaction details
	println!("\nTransaction created successfully!");
	println!("Transaction size: {} bytes", tx.size());
	println!("System fee: {} GAS", tx.sys_fee() as f64 / 100_000_000.0);
	println!("Network fee: {} GAS", tx.net_fee() as f64 / 100_000_000.0);
	println!("Valid until block: {}", tx.valid_until_block());

	// To actually send the transaction, uncomment the following code:
	/*
	println!("\nSending transaction...");
	let result = tx.send_tx().await?;
	println!("Transaction sent! Hash: {}", result.hash);

	// Wait for the transaction to be confirmed
	println!("\nWaiting for confirmation...");
	tx.track_tx(10).await?;
	println!("Transaction confirmed!");

	// Check the updated balance
	let new_balance = gas_token.balance_of(&sender.get_script_hash()).await?;
	println!("Sender's new GAS balance: {}", new_balance);
	*/

	println!("\nTransaction example completed successfully!");
	Ok(())
}
