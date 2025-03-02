//! Basic example of using AWS Nitro TEE with Neo Rust SDK
//!
//! Run with:
//! cargo run --example aws_nitro/basic_usage --features="aws-nitro-tee"

use neo3::neo_aws_nitro::{AwsNitroConfig, AwsNitroKeyManager};

fn main() {
    println!("AWS Nitro TEE Example");
    println!("====================");

    // Create a new AWS Nitro TEE configuration
    let config = AwsNitroConfig {
        region: "us-east-1".to_string(),
        key_id: "my-key-id".to_string(),
    };

    // Create a new AWS Nitro TEE key manager
    let key_manager = AwsNitroKeyManager::new(config);

    // Try to generate a key pair
    match key_manager.generate_key_pair() {
        Ok(key_pair) => println!("Generated key pair: {:?}", key_pair),
        Err(e) => println!("Failed to generate key pair: {}", e),
    }

    // Try to get a public key
    match key_manager.get_public_key("my-key-id") {
        Ok(public_key) => println!("Retrieved public key: {:?}", public_key),
        Err(e) => println!("Failed to retrieve public key: {}", e),
    }

    println!("Note: This example only demonstrates the API structure.");
    println!("A real implementation would require AWS Nitro TEE integration.");
}
