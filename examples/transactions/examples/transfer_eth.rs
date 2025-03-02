use neo3::prelude::*;
use neo3::neo_utils::network::NeoNetwork;
use std::str::FromStr;

/// This example demonstrates how to transfer GAS tokens on the Neo N3 blockchain.
/// It shows how to create, sign, and send a transaction to transfer GAS between accounts.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Neo N3 GAS Token Transfer Example");
    println!("===============================");

    // Connect to Neo N3 TestNet
    println!("\nConnecting to Neo N3 TestNet...");
    let client = NeoNetwork::TestNet.create_client()?;
    
    // Create sender account
    // WARNING: In a real application, you should never hard-code private keys
    // This is just for demonstration purposes
    println!("\nCreating sender account...");
    // Replace with your own WIF from a TestNet wallet
    let sender_wif = "L1gjWGtyDUmrsP3oaJCkHFLL1N6jJbbbsbRmgkgMJJ7RAGhVsMf3"; // Example WIF - replace with your own
    let sender = Account::from_wif(sender_wif)?;
    println!("Sender address: {}", sender.address());
    
    // Create recipient address
    let recipient_address = "NVkg1yRMrTyY6QFnEkpP4WUFaviE1gFa3g"; // Example address - replace with your recipient
    println!("Recipient address: {}", recipient_address);
    
    // Get sender's current GAS balance
    let gas_token = NetworkToken::new(NeoNetwork::TestNet, "gas")?;
    let (balance, symbol, decimals) = gas_token.balance_of(sender.address()).await?;
    let formatted_balance = gas_token.format_balance(balance, decimals);
    println!("Sender's current balance: {} {}", formatted_balance, symbol);
    
    // Amount to transfer (0.1 GAS)
    let amount_to_transfer = 0.1;
    println!("Amount to transfer: {} GAS", amount_to_transfer);
    
    // Convert to raw value (GAS has 8 decimals)
    let raw_amount = (amount_to_transfer * 10_f64.powi(decimals as i32)) as i64;
    
    // Get recipient script hash
    let recipient = Address::from_str(recipient_address)?;
    let recipient_script_hash = recipient.script_hash();
    
    // Create transaction parameters
    let params = vec![
        ContractParameter::hash160(&sender.script_hash()), // from
        ContractParameter::hash160(&recipient_script_hash), // to
        ContractParameter::integer(raw_amount), // amount
        ContractParameter::any(), // data
    ];
    
    // Get GAS token contract hash
    let gas_contract_hash = ScriptHash::from_str(&NeoNetwork::TestNet.get_contract_hash("gas").unwrap())?;
    
    // Build the transaction
    println!("\nBuilding transaction...");
    let script = ScriptBuilder::new()
        .contract_call(
            &gas_contract_hash,
            "transfer", 
            &params, 
            None
        )?
        .to_bytes();
    
    let signer = AccountSigner::from_account(sender.clone())
        .with_scopes(vec![WitnessScope::CalledByEntry]);
    
    let mut tx_builder = TransactionBuilder::new();
    tx_builder
        .script(script)
        .add_signer(signer)
        .valid_until_block(client.get_block_count().await? + 5760)?; // Valid for ~1 day
    
    // Get system fee from test invoke
    let invoke_result = client.invoke_script(&tx_builder.get_script(), None).await?;
    println!("System fee: {} GAS", invoke_result.gas_consumed);
    tx_builder.system_fee(invoke_result.gas_consumed.parse::<i64>()?);
    
    // Sign the transaction
    println!("\nSigning transaction...");
    let tx = sender.sign_transaction(tx_builder)?;
    
    // Send the transaction
    println!("\nSending transaction...");
    let response = client.send_raw_transaction(&tx).await?;
    
    println!("\nTransaction sent successfully!");
    println!("Transaction hash: {}", response.hash);
    println!("\nYou can check the transaction status at:");
    println!("https://testnet.neo3.neotube.io/transaction/{}", response.hash);
    
    Ok(())
}
