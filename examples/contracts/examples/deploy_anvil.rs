use neo3::prelude::*;
use neo3::neo_utils::network::NeoNetwork;
use std::str::FromStr;
use std::fs;

/// This example demonstrates how to deploy a smart contract to the Neo N3 blockchain.
/// It shows how to read NEF and manifest files and deploy a contract.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Neo N3 Smart Contract Deployment Example");
    println!("======================================");

    // Connect to Neo N3 TestNet
    println!("\nConnecting to Neo N3 TestNet...");
    let client = NeoNetwork::TestNet.create_client()?;
    
    // Create deployer account
    // WARNING: In a real application, you should never hard-code private keys
    // This is just for demonstration purposes
    println!("\nCreating deployer account...");
    // Replace with your own WIF from a TestNet wallet with enough GAS
    let deployer_wif = "L1gjWGtyDUmrsP3oaJCkHFLL1N6jJbbbsbRmgkgMJJ7RAGhVsMf3"; // Example WIF - replace with your own
    let deployer = Account::from_wif(deployer_wif)?;
    println!("Deployer address: {}", deployer.address());
    
    // Check deployer's GAS balance
    let gas_token = NetworkToken::new(NeoNetwork::TestNet, "gas")?;
    let (balance, symbol, decimals) = gas_token.balance_of(deployer.address()).await?;
    let formatted_balance = gas_token.format_balance(balance, decimals);
    println!("Deployer's GAS balance: {} {}", formatted_balance, symbol);
    
    // In a real application, you would load your compiled NEF and manifest files
    println!("\nPreparing contract files...");
    
    // For this example, we'll use placeholder paths
    // In a real application, you would replace these with your actual files
    let nef_path = "./path/to/your/contract.nef"; // Replace with your NEF file path
    let manifest_path = "./path/to/your/contract.manifest.json"; // Replace with your manifest file path
    
    // Note: The below code would read and deploy your contract
    // We're commenting it out since this is an example and the files don't exist
    /*
    // Read NEF file (binary)
    let nef_bytes = fs::read(nef_path)?;
    
    // Read manifest file (JSON)
    let manifest_json = fs::read_to_string(manifest_path)?;
    
    // Get contract management script hash
    let contract_management_hash = ScriptHash::from_str(
        &NeoNetwork::TestNet.get_contract_hash("contract_management").unwrap()
    )?;
    
    // Build deploy script
    println!("\nBuilding deploy script...");
    let script = ScriptBuilder::new()
        .push_data(nef_bytes) // NEF file
        .push_data(manifest_json.as_bytes().to_vec()) // Manifest JSON
        .push_null() // Optional data
        .contract_call(
            &contract_management_hash,
            "deploy",
            &[],
            None
        )?
        .to_bytes();
    
    // Create signer
    let signer = AccountSigner::from_account(deployer.clone())
        .with_scopes(vec![WitnessScope::CalledByEntry]);
    
    // Build transaction
    let mut tx_builder = TransactionBuilder::new();
    tx_builder
        .script(script)
        .add_signer(signer)
        .valid_until_block(client.get_block_count().await? + 5760)?; // Valid for ~1 day
    
    // Get system fee
    let invoke_result = client.invoke_script(&tx_builder.get_script(), None).await?;
    println!("System fee: {} GAS", invoke_result.gas_consumed);
    tx_builder.system_fee(invoke_result.gas_consumed.parse::<i64>()?);
    
    // Sign transaction
    println!("\nSigning transaction...");
    let tx = deployer.sign_transaction(tx_builder)?;
    
    // Send transaction
    println!("\nSending deployment transaction...");
    let response = client.send_raw_transaction(&tx).await?;
    
    println!("\nDeployment transaction sent successfully!");
    println!("Transaction hash: {}", response.hash);
    println!("\nYou can check the deployment status at:");
    println!("https://testnet.neo3.neotube.io/transaction/{}", response.hash);
    
    // Wait for transaction to be confirmed
    println!("\nWaiting for deployment to be confirmed...");
    let confirmation = client.wait_for_transaction(response.hash, 60).await?;
    println!("Deployment confirmed in block: {}", confirmation);
    
    // Calculate the contract hash
    let contract_hash = calculate_contract_hash(
        &deployer.script_hash(),
        nef_bytes,
        manifest_json
    );
    println!("\nContract deployed successfully!");
    println!("Contract Hash: {}", contract_hash);
    */
    
    // Since this is an example and we don't have actual contract files,
    // we'll provide guidance on how to deploy a contract
    println!("\nTo deploy a real Neo N3 smart contract:");
    println!("1. Compile your contract using neo-one, neow3j, neo-devpack-dotnet, or neon");
    println!("2. Get the NEF and manifest.json files");
    println!("3. Load these files in your code");
    println!("4. Use the ContractManagement native contract to deploy");
    println!("5. Ensure your deployer account has enough GAS (at least 10 GAS recommended)");
    println!("\nAfter deployment, you can interact with your contract using its hash");
    
    Ok(())
}

// Function to calculate contract hash - simplified example
fn calculate_contract_hash(
    sender_hash: &ScriptHash,
    nef_bytes: Vec<u8>,
    manifest_json: String
) -> ScriptHash {
    // This is a placeholder implementation
    // In a real application, you would calculate the hash based on Neo's rules
    // See: https://docs.neo.org/docs/en-us/basic/concept/contracts.html
    ScriptHash::from_str("0x0123456789abcdef0123456789abcdef01234567").unwrap()
}
