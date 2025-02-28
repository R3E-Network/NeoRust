//! # SGX Wallet Example
//!
//! This example demonstrates how to use the SGX-enabled wallet functionality
//! to securely manage Neo blockchain accounts within an Intel SGX enclave.

use neo::prelude::*;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Neo SGX Wallet Example");
    println!("======================");
    
    // Path to the enclave shared object
    let enclave_path = "path/to/enclave.so";
    
    // Check if the enclave file exists
    if !Path::new(enclave_path).exists() {
        println!("Error: Enclave file not found at {}", enclave_path);
        println!("Note: This example requires a compiled SGX enclave.");
        println!("Please build the enclave using the provided Makefile.");
        return Ok(());
    }
    
    // Initialize the SGX enclave
    println!("\nInitializing SGX enclave...");
    let enclave_manager = SgxEnclaveManager::new(enclave_path)?;
    println!("SGX enclave initialized successfully!");
    
    // Create a wallet with a password
    println!("\nCreating a new SGX-protected wallet...");
    let password = "my-secure-password";
    let wallet = enclave_manager.create_wallet(password)?;
    println!("Wallet created successfully!");
    
    // Get the wallet's public key
    let public_key = wallet.get_public_key();
    println!("\nWallet public key: {:?}", public_key);
    
    // Create a transaction to sign
    println!("\nCreating a sample transaction...");
    let transaction_data = b"Sample transaction data";
    
    // Sign the transaction using the SGX-protected private key
    println!("Signing transaction with SGX-protected private key...");
    let signature = wallet.sign_transaction(transaction_data)?;
    println!("Transaction signed successfully!");
    println!("Signature: {:?}", signature);
    
    // Create a crypto instance for verification
    println!("\nVerifying signature...");
    let crypto = enclave_manager.create_crypto();
    let is_valid = crypto.verify_signature(public_key, transaction_data, &signature)?;
    
    if is_valid {
        println!("Signature verification successful!");
    } else {
        println!("Signature verification failed!");
    }
    
    println!("\nSGX Wallet example completed successfully!");
    
    Ok(())
}
