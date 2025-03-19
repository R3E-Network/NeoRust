use neo3::{
    neo_crypto::KeyPair,
};

/// Example demonstrating simplified key pair generation in Neo N3
fn main() {
    println!("Neo N3 Key Pair Example");
    println!("======================");

    // Generate a random key pair
    println!("\n1. Generating a random key pair...");
    let key_pair = KeyPair::new_random();
    println!("Key pair created successfully");

    // Display key pair details
    println!("\n2. Examining the key pair...");
    println!("Public key: {:?}", key_pair.public_key);
    println!("Private key: {:?}", key_pair.private_key);
    
    // Get the script hash (Neo address)
    println!("\n3. Getting Neo address...");
    println!("Script hash: {:?}", key_pair.get_script_hash());
    println!("Address: {}", key_pair.get_address());
    
    println!("\nKey pair example completed successfully!");
} 