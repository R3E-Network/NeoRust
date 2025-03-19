use eyre::Result;

/// This example demonstrates how to use a local signer to sign messages in Neo N3.
#[tokio::main]
async fn main() -> Result<()> {
	println!("Neo N3 Local Signer Example");
	println!("===========================");
	
	// In a real implementation, you would:
	println!("\n1. Create an account:");
	println!("   let account = Account::create()?;");
	
	println!("\n2. Create a wallet signer from the account:");
	println!("   let wallet_signer = WalletSigner::from_account(account)?;");
	
	println!("\n3. Sign a message with the wallet signer:");
	println!("   let message = b\"Hello, Neo!\";");
	println!("   let signature = wallet_signer.sign_message(message).await?;");
	
	println!("\n4. Verify the signature with the public key:");
	println!("   let public_key = account.key_pair.unwrap().public_key;");
	println!("   let is_valid = public_key.verify_hash(message, &signature)?;");
	
	println!("\nLocal signers provide a secure way to sign transactions and messages");
	println!("without exposing private keys. Use them in wallets, dApps, and any");
	println!("application that requires cryptographic signing functionality.");
	
	println!("\nSee the Neo N3 documentation for more details on cryptographic operations:");
	println!("https://docs.neo.org/docs/en-us/index.html");
	
	Ok(())
}
