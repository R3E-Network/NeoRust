/// This example demonstrates message signing in Neo N3.
use neo_rust::prelude::*;
use neo_rust::neo_crypto::KeyPair;
use neo_rust::neo_wallets::WalletSigner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Neo N3 Message Signing Example");
    println!("==============================");
    
    println!("\nIn Neo N3, message signing is used to prove ownership of a private key");
    println!("without revealing the key itself. Common use cases include:");
    println!("- Authentication for dApps and services");
    println!("- Verifying identity on-chain");
    println!("- Authorizing off-chain operations");
    println!("- Proving ownership of an address");
    
    // Create a random key pair for demonstration
    println!("\nStep 1: Creating a key pair");
    let key_pair = KeyPair::new_random()?;
    let address = key_pair.get_address();
    println!("Generated address: {}", address);
    
    // Create a wallet signer
    println!("\nStep 2: Creating a wallet signer from the key pair");
    let wallet = WalletSigner::new_with_signer(key_pair.clone(), address.clone());
    
    // Message to sign
    println!("\nStep 3: Preparing a message to sign");
    let message = b"Hello, Neo N3!";
    println!("Message: {}", String::from_utf8_lossy(message));
    
    // Sign the message
    println!("\nStep 4: Signing the message");
    let signature = wallet.sign_message(message).await?;
    println!("Signature created successfully");
    println!("Signature: {:?}", signature);
    
    println!("\nStep 5: Verifying the signature (demonstration)");
    println!("In a real application, verification would involve:");
    println!("1. Recovering the public key from the signature");
    println!("2. Deriving the address from the public key");
    println!("3. Comparing with the claimed address");
    
    println!("\nFor security considerations:");
    println!("- Always hash messages before signing");
    println!("- Include domain separation in your message format");
    println!("- Consider adding a timestamp or nonce to prevent replay attacks");
    println!("- Use hardware wallets for high-value operations when possible");
    
    println!("\nMessage signing example completed!");
    Ok(())
}