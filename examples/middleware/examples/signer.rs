use neo3::prelude::*;
use neo3::neo_utils::network::NeoNetwork;
use std::str::FromStr;

/// This example demonstrates different approaches to signing transactions in Neo N3.
/// It shows various account types and signing strategies.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Neo N3 Transaction Signing Strategies Example");
    println!("============================================");

    // Connect to Neo N3 TestNet
    println!("\nConnecting to Neo N3 TestNet...");
    let client = NeoNetwork::TestNet.create_client()?;
    
    // Approach 1: Direct Account signing
    println!("\n1. Direct Account Signing");
    println!("------------------------");
    
    // Create account from WIF
    let wif = "L1gjWGtyDUmrsP3oaJCkHFLL1N6jJbbbsbRmgkgMJJ7RAGhVsMf3"; // Example WIF - replace with your own
    let account = Account::from_wif(wif)?;
    println!("Account created from WIF: {}", account.address());
    
    // Build a simple transaction (just a script invocation)
    println!("Building transaction...");
    let script = ScriptBuilder::new()
        .emit_app_call(&H160::default(), "test", &[])
        .to_bytes();
    
    let mut tx_builder = TransactionBuilder::new()
        .script(script.clone())
        .add_signer(AccountSigner::from_account(account.clone())
            .with_scopes(vec![WitnessScope::CalledByEntry]))
        .valid_until_block(client.get_block_count().await? + 5760)?;
    
    // Sign transaction directly with the account
    println!("Signing with account directly...");
    let tx = account.sign_transaction(tx_builder)?;
    println!("Transaction signed! Hash: {}", tx.hash);
    
    // Approach 2: Multi-signature account
    println!("\n2. Multi-signature Signing");
    println!("------------------------");
    
    // Create individual key pairs
    println!("Creating key pairs...");
    let key1 = KeyPair::new_random()?;
    let key2 = KeyPair::new_random()?;
    let key3 = KeyPair::new_random()?;
    
    // Create a multi-signature account (2 of 3)
    println!("Creating 2-of-3 multi-signature account...");
    let keys = vec![key1.public_key(), key2.public_key(), key3.public_key()];
    let multi_sig_account = Account::create_multi_sig(keys, 2)?;
    println!("Multi-signature address: {}", multi_sig_account.address());
    
    // In a real scenario, you'd need access to at least 2 of the private keys to sign
    println!("To sign a multi-signature transaction:");
    println!("1. Create transaction with the multi-sig account as signer");
    println!("2. Collect signatures from required signers (2 in this case)");
    println!("3. Combine signatures and broadcast transaction");
    
    // Approach 3: Contract-based signing
    println!("\n3. Contract-Based Signing");
    println!("-----------------------");
    
    println!("In a contract-based signing scenario:");
    println!("1. The transaction includes a contract witness");
    println!("2. Verification is done through a contract's verify method");
    println!("3. This enables complex authorization logic beyond simple signatures");
    println!("4. Examples include:");
    println!("   - Time-locked transactions");
    println!("   - Multi-factor authorization");
    println!("   - Role-based permissions");
    
    // Approach 4: Using witness scope to control permissions
    println!("\n4. Controlling Permissions with Witness Scopes");
    println!("-------------------------------------------");
    
    println!("Neo N3 supports different witness scopes:");
    println!("- CalledByEntry: Only allow direct invocation");
    println!("- CustomContracts: Only specified contracts can use this signature");
    println!("- CustomGroups: Only contracts in specified groups can use this signature");
    println!("- Global: Allow any contract to use this signature (dangerous!)");
    
    let restricted_signer = AccountSigner::from_account(account.clone())
        .with_scopes(vec![WitnessScope::CalledByEntry]);
    
    let contract_hash = ScriptHash::from_str("ef4073a0f2b305a38ec4050e4d3d28bc40ea63f5")?; // NEO token as example
    let allowed_contracts_signer = AccountSigner::from_account(account.clone())
        .with_scopes(vec![WitnessScope::CustomContracts])
        .add_allowed_contract(contract_hash)?;
    
    println!("\nExample of account with CalledByEntry scope created");
    println!("Example of account with CustomContracts scope created with NEO token allowed");
    
    // Security best practices
    println!("\nSecurity Best Practices for Transaction Signing:");
    println!("1. Never expose private keys in code");
    println!("2. Store private keys securely");
    println!("3. Use the most restrictive witness scope possible");
    println!("4. Consider using hardware wallets for high-value accounts");
    println!("5. Use multi-signature accounts for added security");
    
    Ok(())
}
