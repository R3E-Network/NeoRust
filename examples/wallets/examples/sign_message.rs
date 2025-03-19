/// This example demonstrates the concept of message signing in Neo N3.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	println!("Neo N3 Wallet Message Signing Example");
	println!("====================================");
	
	println!("\nIn Neo N3, message signing is used to prove ownership of a private key");
	println!("without revealing the key itself. Common use cases include:");
	println!("- Authentication for dApps and services");
	println!("- Verifying identity on-chain");
	println!("- Authorizing off-chain operations");
	println!("- Proving ownership of an address");
	
	println!("\nThe process works as follows:");
	println!("1. The account owner creates a signature using their private key");
	println!("2. Anyone with the public key can verify the signature");
	println!("3. Verification confirms the message was signed by the private key owner");
	println!("4. The signature cannot be reused for different messages");
	
	println!("\nMessage signing is a cornerstone of blockchain security and identity management.");
	
	println!("\nIn a real implementation, you would:");
	println!("1. Create a wallet from a private key or mnemonic phrase");
	println!("   Example: let wallet = WalletSigner::new(key_pair);");
	
	println!("\n2. Sign a message using the wallet");
	println!("   Example: let signature = wallet.sign_message(message).await?;");
	
	println!("\n3. Verify the signature using the wallet's public key");
	println!("   Example: let is_valid = signature.verify(message, wallet.address())?;");
	
	println!("\n4. For added security, you can create wallets from mnemonic phrases");
	println!("   Example: let mnemonic_phrase = \"liberty village rhythm couch december axis barely model flag gym tortoise must\";");
	println!("            let account = create_account_from_mnemonic(mnemonic_phrase)?;");
	
	println!("\nMessage signing provides a cryptographic proof of identity without exposing");
	println!("private keys, making it a fundamental security feature in blockchain applications.");
	
	println!("\nMessage signing example completed!");
	Ok(())
}
