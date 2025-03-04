use std::str::FromStr;

use neo::{
	neo_clients::{HttpProvider, JsonRpcProvider},
	neo_contract::SmartContractTrait,
	neo_protocol::account::Account,
	neo_types::{contract::ContractParameter, script_hash::ScriptHash},
	prelude::{RpcClient, SmartContract},
};

/// This example demonstrates how to interact with a smart contract on the Neo N3 blockchain.
/// It shows how to call read-only methods and how to invoke methods that modify state.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	println!("Neo N3 Smart Contract Interaction Example");
	println!("========================================");

	// Connect to Neo N3 TestNet
	println!("\nConnecting to Neo N3 TestNet...");
	let provider = HttpProvider::new("https://testnet1.neo.org:443");
	let client = RpcClient::new(provider);

	// Load the account that will interact with the contract
	// In a real application, you would load your private key securely
	println!("\nSetting up account...");
	let account = Account::from_wif("YOUR_PRIVATE_KEY_WIF_HERE")?;
	println!("Account address: {}", account.get_address());

	// Define the smart contract to interact with
	// This example uses the GAS token contract
	println!("\nSetting up contract reference...");
	let contract_hash = ScriptHash::from_str("d2a4cff31913016155e38e474a2c06d08be276cf")?;
	let contract = SmartContract::new(contract_hash, Some(&client));

	// Read-only contract calls
	println!("\nPerforming read-only contract calls...");

	// Get contract name
	let name_result = contract.call_function("symbol", vec![]).await?;
	let name = name_result.stack[0].as_string().unwrap_or_default();
	println!("Contract symbol: {}", name);

	// Get contract decimals
	let decimals_result = contract.call_function("decimals", vec![]).await?;
	let decimals = decimals_result.stack[0].as_int().unwrap_or_default();
	println!("Contract decimals: {}", decimals);

	// Get account balance
	let balance_result = contract
		.call_function("balanceOf", vec![ContractParameter::hash160(&account.get_script_hash())])
		.await?;
	let balance = balance_result.stack[0].as_int().unwrap_or_default();
	println!("Account balance: {} (raw units)", balance);

	// Calculate the balance in token units
	let balance_in_tokens = balance as f64 / 10f64.powi(decimals as i32);
	println!("Account balance: {} {}", balance_in_tokens, name);

	// State-changing contract calls (these would require a transaction)
	println!("\nPreparing a state-changing contract call...");

	// Example: Transfer tokens to another address
	let recipient_address = "NbTiM6h8r99kpRtb428XcsUk1TzKed2gTc";
	let recipient = ScriptHash::from_address(recipient_address)?;
	let amount = 1_0000_0000; // 1 GAS (assuming 8 decimals)

	// Create a transaction builder for the transfer
	let tx_builder = contract
		.invoke_function(
			"transfer",
			vec![
				ContractParameter::hash160(&account.get_script_hash()),
				ContractParameter::hash160(&recipient),
				ContractParameter::integer(amount),
				ContractParameter::any(None),
			],
		)
		.await?;

	// In a real application, you would sign and send this transaction
	// For this example, we'll just print the transaction details
	println!("Transaction builder created for 'transfer' method");
	println!("Parameters:");
	println!("  From: {}", account.get_address());
	println!("  To: {}", recipient_address);
	println!("  Amount: {} (raw units)", amount);

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

	println!("\nSmart contract interaction example completed successfully!");
	Ok(())
}
