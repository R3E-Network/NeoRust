use std::str::FromStr;

use neo::{
    neo_clients::JsonRpcProvider,
    neo_contract::{ContractManagement, SmartContractTrait},
    neo_protocol::account::Account,
    neo_types::{
        contract::{ContractParameter, NefFile},
        script_hash::ScriptHash,
    },
    prelude::{HttpProvider, RpcClient},
};

/// This example demonstrates how to deploy a smart contract to the Neo N3 blockchain.
/// It uses the ContractManagement native contract to deploy a NEF file with a contract manifest.
///
/// Prerequisites:
/// - A NEF file containing the compiled contract
/// - A JSON file containing the contract manifest
/// - An account with sufficient GAS to pay for deployment
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to a Neo N3 TestNet node
    let provider = HttpProvider::new("https://testnet1.neo.org:443");
    let client = RpcClient::new(provider);

    // Load the account that will deploy the contract
    // Replace with your own WIF or use another method to load the account
    let account = Account::from_wif("YOUR_PRIVATE_KEY_WIF_HERE")?;
    println!("Using account: {}", account.get_address());

    // Get the ContractManagement native contract
    let contract_hash = ScriptHash::from_str("fffdc93764dbaddd97c48f252a53ea4643faa3fd")?;
    let contract_management = ContractManagement::new(contract_hash, Some(&client));

    // Load the NEF file
    // In a real application, you would load this from a file
    // let nef_bytes = std::fs::read("path/to/your/contract.nef")?;
    // let nef = NefFile::from_bytes(&nef_bytes)?;
    
    // For this example, we'll create a placeholder NEF file
    // In a real application, you would load the actual NEF file
    let nef = create_placeholder_nef();

    // Load the contract manifest
    // In a real application, you would load this from a file
    // let manifest_bytes = std::fs::read("path/to/your/contract.manifest.json")?;
    let manifest_bytes = r#"{
        "name": "ExampleContract",
        "groups": [],
        "features": {},
        "supportedstandards": [],
        "abi": {
            "methods": [
                {
                    "name": "main",
                    "parameters": [],
                    "returntype": "String",
                    "offset": 0,
                    "safe": false
                }
            ],
            "events": []
        },
        "permissions": [
            {
                "contract": "*",
                "methods": "*"
            }
        ],
        "trusts": [],
        "extra": null
    }"#.as_bytes();

    // Optional data parameter for the deploy method
    let data = Some(ContractParameter::string("Optional deployment data"));

    // Create a transaction to deploy the contract
    let tx_builder = contract_management.deploy(&nef, manifest_bytes, data).await?;

    // Add the account as a signer
    let tx_builder = tx_builder
        .set_signers(vec![account.into()])
        .valid_until_block(client.get_block_count().await? + 5760)?;

    // Sign and send the transaction
    let tx = tx_builder.sign().await?;
    let result = tx.send_tx().await?;

    println!("Contract deployed successfully!");
    println!("Transaction hash: {}", result.hash);

    // Wait for the transaction to be confirmed
    tx.track_tx(10).await?;

    println!("Transaction confirmed!");

    Ok(())
}

/// Creates a placeholder NEF file for demonstration purposes.
/// In a real application, you would load the actual NEF file from disk.
fn create_placeholder_nef() -> NefFile {
    // This is just a placeholder and won't actually work for deployment
    // In a real application, you would load the actual NEF file
    unimplemented!("This is a placeholder. In a real application, you would load the actual NEF file.")
}
