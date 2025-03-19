/// This example demonstrates Neo X integration with NeoRust SDK.
use neo3::prelude::*;
use neo3::neo_x::evm::*;
use neo3::neo_x::bridge::*;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("Neo X Integration Example");
    println!("========================");
    
    // Connect to Neo N3 and Neo X nodes
    println!("\nStep 1: Connecting to Neo N3 and Neo X nodes");
    let neo_provider = HttpProvider::new("https://testnet1.neo.org:443")?;
    let neo_client = RpcClient::new(neo_provider);
    
    // Initialize the Neo X EVM provider
    let neo_x_provider = NeoXProvider::new("https://testnet.rpc.neo-x.org", Some(&neo_client));
    
    // Get basic information
    println!("\nStep 2: Getting basic chain information");
    
    // Neo N3 information
    let neo_block_count = neo_client.get_block_count().await?;
    println!("Neo N3 block height: {}", neo_block_count);
    
    // Neo X information
    let neo_x_chain_id = neo_x_provider.chain_id().await?;
    let neo_x_block_number = neo_x_provider.block_number().await?;
    println!("Neo X chain ID: {}", neo_x_chain_id);
    println!("Neo X block height: {}", neo_x_block_number);
    
    println!("\nStep 3: Working with the Neo X bridge (examples)");
    println!("Note: These operations require actual accounts with funds.");
    println!("In a real application, you would:");
    
    // Bridge operation examples (code shown but not executed)
    println!("\n1. Create a bridge contract instance:");
    println!("   let bridge = NeoXBridgeContract::new(neo_client.clone(), neo_x_provider.clone());");
    
    println!("\n2. Bridge GAS from Neo N3 to Neo X:");
    println!("   let account = Account::from_wif(\"your-wif-here\")?;");
    println!("   let amount = 1_00000000; // 1 GAS");
    println!("   let neo_x_address = \"0x1234567890123456789012345678901234567890\";");
    println!("   let txid = bridge.bridge_to_neox(");
    println!("       &account,");
    println!("       BridgeToken::Gas,");
    println!("       amount,");
    println!("       neo_x_address,");
    println!("   ).await?;");
    
    println!("\n3. Bridge GAS from Neo X back to Neo N3:");
    println!("   let account = Account::from_private_key(\"0xprivate-key-hex\")?;");
    println!("   let amount = 1_000_000_000_000_000_000u128; // 1 GAS on Neo X (18 decimals)");
    println!("   let neo_address = \"Neo1AbcDefGhiJklMnoPqrsTuvWxYz12345\";");
    println!("   let txid = bridge.bridge_to_neo(");
    println!("       &account,");
    println!("       BridgeToken::Gas,");
    println!("       amount,");
    println!("       neo_address,");
    println!("   ).await?;");
    
    println!("\nStep 4: Interacting with EVM contracts on Neo X (examples)");
    println!("Note: These operations require actual accounts with funds.");
    println!("In a real application, you would:");
    
    println!("\n1. Create a contract instance:");
    println!("   let contract_address = \"0x1234567890123456789012345678901234567890\";");
    println!("   let contract = NeoXContract::new(contract_address, neo_x_provider.clone());");
    
    println!("\n2. Call a read-only method (e.g., ERC-20 balanceOf):");
    println!("   let address = \"0x9876543210987654321098765432109876543210\";");
    println!("   let balance = contract.call_read(\"balanceOf\", &[address]).await?;");
    println!("   println!(\"Token balance: {}\", balance.as_u256()?);");
    
    println!("\n3. Call a state-changing method (e.g., ERC-20 transfer):");
    println!("   let account = Account::from_private_key(\"0xprivate-key-hex\")?;");
    println!("   let recipient = \"0x9876543210987654321098765432109876543210\";");
    println!("   let amount = 1_000_000_000_000_000_000u128; // 1 token (18 decimals)");
    println!("   let options = CallOptions {");
    println!("       gas_limit: Some(100_000),");
    println!("       gas_price: Some(20_000_000_000u64), // 20 Gwei");
    println!("       value: None,");
    println!("   };");
    println!("   let tx_hash = contract.call_write(");
    println!("       &account,");
    println!("       \"transfer\",");
    println!("       &[recipient, amount.to_string()],");
    println!("       Some(options),");
    println!("   ).await?;");
    
    println!("\nNeo X integration example completed!");
    println!("For more information, see the Neo X documentation at:");
    println!("https://github.com/your-username/NeoRust/tree/main/docs/neo-x");
    
    Ok(())
}