use std::str::FromStr;

use neo3::prelude::*;
use neo_clients::{HttpProvider, JsonRpcProvider};
use neo_contract::famous::GrandShareContract;
use neo_protocol::account::Account;
use neo_types::ScriptHash;

/// This example demonstrates how to interact with the GrandShare contract on Neo N3.
/// It shows how to submit proposals, vote on proposals, fund projects, and claim funds.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	println!("GrandShare Contract Example");
	println!("==========================");

	// Connect to Neo N3 TestNet
	println!("\nConnecting to Neo N3 TestNet...");
	let provider = HttpProvider::new("https://testnet1.neo.org:443");
	let client = RpcClient::new(provider);

	// Create a GrandShare contract instance
	println!("\nInitializing GrandShare contract...");
	let grandshare = GrandShareContract::new(Some(&client));
	println!("Contract hash: {}", grandshare.script_hash());

	// Create an account for the transaction signer
	// In a real application, you would load your private key securely
	println!("\nSetting up account...");
	let account = Account::from_wif("YOUR_WIF_KEY_HERE")?;
	println!("Account address: {}", account.get_address());

	// Example 1: Submit a proposal
	println!("\nExample 1: Submit a proposal");
	println!("Creating a transaction to submit a new proposal...");
	let title = "Enhance Neo N3 SDK";
	let description = "Develop comprehensive Rust SDK for Neo N3 blockchain";
	let requested_amount = 1000_0000_0000; // 1000 GAS (8 decimals)

	let submit_tx = grandshare
		.submit_proposal(title, description, requested_amount, &account)
		.await?;

	println!("Transaction created successfully!");
	println!("To execute this transaction, you would sign and send it:");
	println!("let signed_tx = submit_tx.sign().await?;");
	println!("let result = signed_tx.send_tx().await?;");

	// Example 2: Vote on a proposal
	println!("\nExample 2: Vote on a proposal");
	println!("Creating a transaction to vote on a proposal...");
	let proposal_id = 42; // Example proposal ID
	let vote_type = true; // true for yes, false for no

	let vote_tx = grandshare.vote(proposal_id, vote_type, &account).await?;

	println!("Transaction created successfully!");

	// Example 3: Fund a project
	println!("\nExample 3: Fund a project");
	println!("Creating a transaction to fund a project...");
	let project_id = 42; // Example project ID
	let amount = 100_0000_0000; // 100 GAS (8 decimals)

	let fund_tx = grandshare.fund_project(project_id, amount, &account).await?;

	println!("Transaction created successfully!");

	// Example 4: Claim funds
	println!("\nExample 4: Claim funds");
	println!("Creating a transaction to claim funds from a project...");
	let project_id = 42; // Example project ID

	let claim_tx = grandshare.claim_funds(project_id, &account).await?;

	println!("Transaction created successfully!");

	println!("\nGrandShare example completed successfully!");
	Ok(())
}
