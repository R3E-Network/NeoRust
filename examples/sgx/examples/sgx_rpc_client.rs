//! # SGX RPC Client Example
//!
//! This example demonstrates how to use the SGX-enabled RPC client
//! to securely interact with the Neo blockchain from within an Intel SGX enclave.

use neo::prelude::*;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Neo SGX RPC Client Example");
    println!("==========================");
    
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
    
    // Create an SGX RPC client
    println!("\nCreating SGX RPC client...");
    let rpc_client = SgxRpcClient::new(
        enclave_manager.get_enclave().clone(),
        "https://mainnet1.neo.org:443".to_string(),
    );
    println!("SGX RPC client created successfully!");
    
    // Get the current block count
    println!("\nGetting current block count...");
    match rpc_client.get_block_count() {
        Ok(count) => println!("Current block count: {}", count),
        Err(e) => println!("Error getting block count: {:?}", e),
    }
    
    // Get a block by index
    println!("\nGetting block at index 1...");
    match rpc_client.get_block("1") {
        Ok(block) => println!("Block: {}", block),
        Err(e) => println!("Error getting block: {:?}", e),
    }
    
    // Create a wallet with a password
    println!("\nCreating a new SGX-protected wallet...");
    let password = "my-secure-password";
    let wallet = enclave_manager.create_wallet(password)?;
    println!("Wallet created successfully!");
    
    // Get the wallet's public key
    let public_key = wallet.get_public_key();
    println!("\nWallet public key: {:?}", public_key);
    
    println!("\nSGX RPC Client example completed successfully!");
    
    Ok(())
}
