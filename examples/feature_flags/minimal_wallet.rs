// examples/feature_flags/minimal_wallet.rs
//
// This example demonstrates how to create a wallet using minimal features.
//
// Required features:
// - wallet
// - crypto-standard
//
// Run with:
// cargo run --example feature_flags/minimal_wallet --no-default-features --features="wallet,crypto-standard"

use neo::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating a minimal Neo N3 wallet...");
    
    // Create a new wallet
    let wallet = Wallet::new()?;
    println!("Created wallet with name: {}", wallet.name());
    
    // Generate a new account
    let account = Account::create()?;
    println!("Generated account with address: {}", account.address());
    
    // Add the account to the wallet
    wallet.add_account(account.clone());
    println!("Added account to wallet");
    
    // Print the wallet's accounts
    let accounts = wallet.accounts();
    println!("Wallet now has {} accounts", accounts.len());
    
    // Print the account's script hash
    println!("Account script hash: {}", account.get_script_hash());
    
    // Sign a message with the account
    let message = b"Hello, Neo!";
    let signature = account.sign(message)?;
    println!("Signed message with signature: {:?}", signature);
    
    // Verify the signature
    let is_valid = account.verify(message, &signature)?;
    println!("Signature verification result: {}", is_valid);
    
    println!("Wallet operations completed successfully!");
    Ok(())
} 