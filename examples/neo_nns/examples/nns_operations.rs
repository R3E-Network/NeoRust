use std::str::FromStr;

use neo::{
    neo_clients::{HttpProvider, JsonRpcProvider},
    neo_contract::NeoNameService,
    neo_protocol::account::Account,
    neo_types::script_hash::ScriptHash,
    prelude::RpcClient,
};

/// This example demonstrates how to work with the Neo Name Service (NNS) on the Neo N3 blockchain.
/// It shows how to check domain availability, register domains, and manage domain records.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Neo N3 Name Service (NNS) Operations Example");
    println!("===========================================");

    // Connect to Neo N3 TestNet
    println!("\nConnecting to Neo N3 TestNet...");
    let provider = HttpProvider::new("https://testnet1.neo.org:443");
    let client = RpcClient::new(provider);
    
    // Load the account that will interact with NNS
    // In a real application, you would load your private key securely
    println!("\nSetting up account...");
    let account = Account::from_wif("YOUR_PRIVATE_KEY_WIF_HERE")?;
    println!("Account address: {}", account.get_address());
    
    // Create a reference to the NNS contract
    println!("\nSetting up NNS contract reference...");
    let nns_service = NeoNameService::new(Some(&client));
    
    // Check domain availability
    println!("\nChecking domain availability...");
    let domain_name = "example.neo";
    let is_available = nns_service.is_available(domain_name).await?;
    
    if is_available {
        println!("Domain '{}' is available for registration", domain_name);
        
        // Register the domain
        println!("\nPreparing domain registration...");
        let tx_builder = nns_service
            .register(domain_name, account.get_script_hash())
            .await?;
        
        // In a real application, you would sign and send this transaction
        // For this example, we'll just print the transaction details
        println!("Transaction builder created for domain registration");
        println!("Registration details:");
        println!("  Domain: {}", domain_name);
        println!("  Owner: {}", account.get_address());
        
        // To actually register the domain, you would:
        /*
        // Add the account as a signer
        let tx_builder = tx_builder
            .set_signers(vec![account.into()])
            .valid_until_block(client.get_block_count().await? + 5760)?;
        
        // Sign and send the transaction
        let tx = tx_builder.sign().await?;
        let result = tx.send_tx().await?;
        
        println!("Registration transaction sent! Hash: {}", result.hash);
        */
        
        // Set domain records
        println!("\nPreparing to set domain records...");
        
        // Set a text record
        let tx_builder = nns_service
            .set_record(domain_name, 1, "Hello, Neo N3!")
            .await?;
        
        println!("Transaction builder created for setting text record");
        println!("Record details:");
        println!("  Domain: {}", domain_name);
        println!("  Record type: TXT (1)");
        println!("  Data: Hello, Neo N3!");
        
        // Set an address record
        let tx_builder = nns_service
            .set_record(domain_name, 2, &account.get_address())
            .await?;
        
        println!("\nTransaction builder created for setting address record");
        println!("Record details:");
        println!("  Domain: {}", domain_name);
        println!("  Record type: A (2)");
        println!("  Data: {}", account.get_address());
    } else {
        println!("Domain '{}' is already registered", domain_name);
        
        // For registered domains, you can renew them
        println!("\nPreparing domain renewal...");
        let renewal_years = 1;
        let tx_builder = nns_service
            .renew(domain_name, renewal_years)
            .await?;
        
        println!("Transaction builder created for domain renewal");
        println!("Renewal details:");
        println!("  Domain: {}", domain_name);
        println!("  Duration: {} year(s)", renewal_years);
        
        // To actually renew the domain, you would sign and send the transaction
        // similar to the registration example above
    }
    
    println!("\nNNS operations example completed successfully!");
    Ok(())
}
