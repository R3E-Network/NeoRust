use neo::{prelude::*, neo_fs::{client::{NeoFsClient, NeoFsConfig}, errors::NeoFsResult, types::{ContainerId, ObjectId}}};
use neo::neo_utils::constants::{rpc_endpoints, contracts};
use std::env;
use std::error::Error;
use std::path::Path;

/// A simple example demonstrating basic NeoFS operations
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("help");
    
    match command {
        "help" => print_help(),
        "info" => network_info().await?,
        "verify" => verify_compatibility().await?,
        "examples" => print_examples().await?,
        "endpoints" => print_endpoints(),
        "contracts" => print_contracts(),
        "upload" if args.len() >= 4 => {
            let container_id = args[2].clone();
            let file_path = args[3].clone();
            upload_file(&container_id, &file_path).await?;
        },
        "download" if args.len() >= 5 => {
            let container_id = args[2].clone();
            let object_id = args[3].clone();
            let file_path = args[4].clone();
            download_file(&container_id, &object_id, &file_path).await?;
        },
        "create-container" => create_container().await?,
        "list-containers" => list_containers().await?,
        _ => {
            println!("Unknown command or insufficient arguments: {}", command);
            print_help();
        }
    }
    
    Ok(())
}

fn print_help() {
    println!("NeoFS Basic Example");
    println!("==================");
    println!();
    println!("Usage:");
    println!("  cargo run --example neofs_basic [COMMAND] [ARGS]");
    println!();
    println!("Commands:");
    println!("  help                      Show this help message");
    println!("  info                      Display NeoFS network information");
    println!("  verify                    Verify Neo N3 NeoFS compatibility");
    println!("  examples                  Show CLI examples for NeoFS operations");
    println!("  endpoints                 Show Neo N3 RPC endpoints");
    println!("  contracts                 Show Neo N3 contract addresses");
    println!("  create-container          Create a new container");
    println!("  list-containers           List all containers");
    println!("  upload [CID] [FILE]       Upload a file to a container");
    println!("  download [CID] [OID] [FILE] Download object to a file");
    println!();
    println!("Environment Variables:");
    println!("  PRIVATE_KEY               Your private key (WIF format) - used for NeoFS operations");
    println!("  CONTAINER_ID              Container ID to operate on");
    println!("  OBJECT_ID                 Object ID to operate on");
    println!("  PATH                      Path to file for upload/download");
    println!("  NEOFS_ENDPOINT            NeoFS endpoint (default: {})", rpc_endpoints::neofs::MAIN_ENDPOINT);
    println!("  NEOFS_WALLET_PATH         Path to NEP-6 wallet");
    println!("  NEOFS_WALLET_ACCOUNT      Account in wallet to use");
    println!("  NEOFS_WALLET_PASSWORD     Wallet password");
}

fn print_endpoints() {
    println!("Neo N3 Network Endpoints");
    println!("=======================");
    println!();
    
    println!("MainNet RPC Endpoints:");
    println!("-----------------");
    for endpoint in rpc_endpoints::mainnet::ALL {
        println!("  {}", endpoint);
    }
    println!();
    
    println!("TestNet RPC Endpoints:");
    println!("-----------------");
    for endpoint in rpc_endpoints::testnet::ALL {
        println!("  {}", endpoint);
    }
    println!();
    
    println!("NeoFS Endpoints:");
    println!("--------------");
    println!("  MainNet: {}", rpc_endpoints::neofs::MAIN_ENDPOINT);
    println!("  TestNet: {}", rpc_endpoints::neofs::TESTNET_ENDPOINT);
}

fn print_contracts() {
    println!("Neo N3 Contract Addresses");
    println!("========================");
    println!();
    
    println!("MainNet Contracts:");
    println!("----------------");
    println!("  NEO Token:     {}", contracts::mainnet::NEO_TOKEN);
    println!("  GAS Token:     {}", contracts::mainnet::GAS_TOKEN);
    println!("  NeoFS:         {}", contracts::mainnet::NEOFS);
    println!("  Neo Name Service: {}", contracts::mainnet::NEO_NS);
    println!("  Flamingo (FLM): {}", contracts::mainnet::FLM_TOKEN);
    println!("  Oracle:        {}", contracts::mainnet::ORACLE);
    println!();
    
    println!("TestNet Contracts:");
    println!("----------------");
    println!("  NEO Token:     {}", contracts::testnet::NEO_TOKEN);
    println!("  GAS Token:     {}", contracts::testnet::GAS_TOKEN);
    println!("  NeoFS:         {}", contracts::testnet::NEOFS);
    println!("  Neo Name Service: {}", contracts::testnet::NEO_NS);
    println!("  Oracle:        {}", contracts::testnet::ORACLE);
    println!("  CNEO Token:    {}", contracts::testnet::CNEO_TOKEN);
    println!("  CGAS Token:    {}", contracts::testnet::CGAS_TOKEN);
    println!();
    
    println!("Native Contracts (same on all networks):");
    println!("-------------------------------------");
    println!("  Contract Management: {}", contracts::native::CONTRACT_MANAGEMENT);
    println!("  Ledger:         {}", contracts::native::LEDGER);
    println!("  Policy:         {}", contracts::native::POLICY);
    println!("  Role Management: {}", contracts::native::ROLE_MANAGEMENT);
}

async fn get_client() -> Result<NeoFsClient, Box<dyn Error>> {
    // Get configuration from environment variables
    let endpoint = env::var("NEOFS_ENDPOINT").unwrap_or_else(|_| rpc_endpoints::neofs::MAIN_ENDPOINT.to_string());
    
    // Create client configuration
    let mut config = NeoFsConfig::default();
    config.endpoint = endpoint;
    
    // Create the client
    let client = NeoFsClient::with_config(config)?;
    
    // Try to load wallet if environment variables are set
    #[cfg(feature = "transaction")]
    if let (Ok(wallet_path), Ok(account_name), Ok(password)) = (
        env::var("NEOFS_WALLET_PATH"),
        env::var("NEOFS_WALLET_ACCOUNT"),
        env::var("NEOFS_WALLET_PASSWORD"),
    ) {
        println!("Loading wallet from {}", wallet_path);
        let wallet = neo::wallets::Wallet::from_file(&wallet_path)?;
        let account = wallet.get_account(&account_name)?;
        let account_data = account.decrypt_private_key(&password)?;
        
        // Set account for authentication
        return Ok(client.with_account(account_data));
    }
    
    // Return client without authentication
    Ok(client)
}

async fn network_info() -> Result<(), Box<dyn Error>> {
    let client = get_client().await?;
    
    println!("Connecting to NeoFS network...");
    if !client.is_connected().await {
        println!("❌ Failed to connect to NeoFS network");
        return Ok(());
    }
    
    println!("✅ Connected to NeoFS network");
    
    // Get network information
    match client.get_network_info().await {
        Ok(info) => {
            println!("\nNetwork Information:");
            println!("-------------------");
            println!("Version:         {}", info.version);
            println!("Node Count:      {}", info.node_count);
            println!("Storage Capacity: {} bytes", info.storage_capacity);
            println!("Available Space: {} bytes", info.available_space);
        },
        Err(e) => {
            println!("❌ Failed to get network information: {}", e);
        }
    }
    
    Ok(())
}

async fn verify_compatibility() -> Result<(), Box<dyn Error>> {
    println!("Verifying Neo N3 NeoFS compatibility...");
    
    println!("❌ NeoFS compatibility check is currently unavailable");
    println!("The NeoFS module is still under development and has some feature mismatches.");
    println!("The code internally checks for 'http-client' feature but the Cargo.toml defines 'reqwest'.");
    println!();
    println!("To use NeoFS functionality when it's fully implemented, you'll need to:");
    println!("1. Run with the crypto-standard feature: cargo run --features crypto-standard --example neofs_basic");
    println!("2. Or add an http-client feature alias in Cargo.toml that points to reqwest");
    println!();
    println!("Available NeoFS endpoints:");
    println!("  MainNet: {}", rpc_endpoints::neofs::MAIN_ENDPOINT);
    println!("  TestNet: {}", rpc_endpoints::neofs::TESTNET_ENDPOINT);
    println!();
    println!("You can specify a different endpoint using the NEOFS_ENDPOINT environment variable.");
    
    Ok(())
}

async fn print_examples() -> Result<(), Box<dyn Error>> {
    println!("NeoFS CLI Examples");
    println!("=================\n");
    
    println!("## Using neofs-cli\n");
    println!("### Connection and setup");
    let endpoint = env::var("NEOFS_ENDPOINT").unwrap_or_else(|_| rpc_endpoints::neofs::MAIN_ENDPOINT.to_string());
    println!("```\n# Set endpoint\nneofs-cli --endpoint=\"{}\" -v\n", endpoint);
    println!("# Generate a key\nneofs-cli util keygen --show\n");
    println!("# Create wallet\nneofs-cli util create-wallet --wallet ./my-neofs-wallet.json\n```\n");
    
    println!("### Container operations");
    println!("```\n# List containers\nneofs-cli container list\n");
    println!("# Create container\nneofs-cli container create \\\n  --policy=\"REP 2 CBF 3 SELECT 2 FROM 3 IN X\"\n");
    println!("# Get container info\nneofs-cli container get --cid <container_id>\n");
    println!("# Delete container\nneofs-cli container delete --cid <container_id>\n```\n");
    
    println!("### Object operations");
    println!("```\n# Upload object\nneofs-cli object put \\\n  --cid <container_id> \\\n  --file ./myfile.txt \\\n  --attributes filename=myfile.txt,timestamp=$(date -u +\"%Y-%m-%dT%H:%M:%SZ\")\n");
    println!("# List objects in container\nneofs-cli object list --cid <container_id>\n");
    println!("# Get object\nneofs-cli object get \\\n  --cid <container_id> \\\n  --oid <object_id> \\\n  --file ./downloaded_file.txt\n");
    println!("# Delete object\nneofs-cli object delete \\\n  --cid <container_id> \\\n  --oid <object_id>\n```\n");
    
    Ok(())
}

// Helper function to parse container ID
fn parse_container_id(id_str: &str) -> Result<ContainerId, Box<dyn Error>> {
    // This is a simplified version - in a real application
    // you would parse the string into the actual format required by ContainerId
    let mut bytes = [0u8; 32];
    let id_bytes = id_str.as_bytes();
    let len = std::cmp::min(id_bytes.len(), 32);
    bytes[..len].copy_from_slice(&id_bytes[..len]);
    Ok(ContainerId::new(bytes))
}

// Helper function to parse object ID
fn parse_object_id(id_str: &str) -> Result<ObjectId, Box<dyn Error>> {
    // This is a simplified version - in a real application
    // you would parse the string into the actual format required by ObjectId
    let mut bytes = [0u8; 32];
    let id_bytes = id_str.as_bytes();
    let len = std::cmp::min(id_bytes.len(), 32);
    bytes[..len].copy_from_slice(&id_bytes[..len]);
    Ok(ObjectId::new(bytes))
}

async fn create_container() -> Result<(), Box<dyn Error>> {
    let client = get_client().await?;
    
    // Check if account is set
    #[cfg(feature = "transaction")]
    if client.account().is_none() {
        println!("❌ Account is required for container creation");
        println!("Please set NEOFS_WALLET_PATH, NEOFS_WALLET_ACCOUNT, and NEOFS_WALLET_PASSWORD environment variables");
        return Ok(());
    }
    
    #[cfg(not(feature = "transaction"))]
    {
        println!("❌ Transaction feature is required for container creation");
        println!("Please rebuild with the 'transaction' feature enabled");
        return Ok(());
    }
    
    println!("Creating a new container...");
    
    // Create container parameters with basic policy
    #[cfg(feature = "transaction")]
    let policy = neo::neo_fs::types::StoragePolicy {
        replicas: 1,
        placement: neo::neo_fs::types::PlacementPolicy {
            regions: vec![neo::neo_fs::types::RegionSelector {
                region: "EU".to_string(),
                node_count: 1,
            }],
            tier: neo::neo_fs::types::ReliabilityTier::Standard,
            min_nodes_per_region: 1,
        },
        lifetime: 100, // Short lifetime for example
    };
    
    #[cfg(feature = "transaction")]
    let params = neo::neo_fs::container::CreateContainerParams {
        rules: vec![neo::neo_fs::types::AccessRule::Public],
        policy,
        attributes: vec![
            ("name".to_string(), "example-container".to_string()),
            ("created".to_string(), chrono::Utc::now().to_rfc3339()),
            ("purpose".to_string(), "NeoFS basic example".to_string()),
        ],
    };
    
    // Create the container
    #[cfg(feature = "transaction")]
    match client.containers().create(params).await {
        Ok(container_id) => {
            println!("✅ Container created successfully");
            println!("Container ID: {}", container_id);
            println!();
            println!("You can use this container ID for upload/download operations:");
            println!("  cargo run --example neofs_basic upload {} path/to/file.txt", container_id);
        },
        Err(e) => {
            println!("❌ Failed to create container: {}", e);
        }
    }
    
    Ok(())
}

async fn list_containers() -> Result<(), Box<dyn Error>> {
    let client = get_client().await?;
    
    // Check if account is set
    #[cfg(feature = "transaction")]
    if client.account().is_none() {
        println!("❌ Account is required for listing containers");
        println!("Please set NEOFS_WALLET_PATH, NEOFS_WALLET_ACCOUNT, and NEOFS_WALLET_PASSWORD environment variables");
        return Ok(());
    }
    
    #[cfg(not(feature = "transaction"))]
    {
        println!("❌ Transaction feature is required for listing containers");
        println!("Please rebuild with the 'transaction' feature enabled");
        return Ok(());
    }
    
    println!("Listing containers...");
    
    #[cfg(feature = "transaction")]
    match client.containers().list().await {
        Ok(containers) => {
            if containers.is_empty() {
                println!("No containers found");
                return Ok(());
            }
            
            println!("Found {} containers:", containers.len());
            println!();
            
            for (i, container) in containers.iter().enumerate() {
                println!("Container #{}", i + 1);
                println!("  ID: {}", container.id);
                println!("  Version: {}", container.version);
                println!("  Created: {}", container.created_at);
                
                if let Some(owner) = &container.owner {
                    println!("  Owner: {}", owner);
                }
                
                if !container.attributes.is_empty() {
                    println!("  Attributes:");
                    for (key, value) in &container.attributes {
                        println!("    {}: {}", key, value);
                    }
                }
                
                println!();
            }
        },
        Err(e) => {
            println!("❌ Failed to list containers: {}", e);
        }
    }
    
    Ok(())
}

async fn upload_file(container_id_str: &str, file_path: &str) -> Result<(), Box<dyn Error>> {
    let client = get_client().await?;
    let container_id = parse_container_id(container_id_str)?;
    
    // Check if account is set
    #[cfg(feature = "transaction")]
    if client.account().is_none() {
        println!("❌ Account is required for uploading files");
        println!("Please set NEOFS_WALLET_PATH, NEOFS_WALLET_ACCOUNT, and NEOFS_WALLET_PASSWORD environment variables");
        return Ok(());
    }
    
    #[cfg(not(feature = "transaction"))]
    {
        println!("❌ Transaction feature is required for uploading files");
        println!("Please rebuild with the 'transaction' feature enabled");
        return Ok(());
    }
    
    println!("Uploading file: {}", file_path);
    println!("To container:   {}", container_id_str);
    
    #[cfg(feature = "transaction")]
    match client.objects().upload_file(&container_id, file_path).await {
        Ok(object_id) => {
            println!("✅ File uploaded successfully");
            println!("Object ID: {}", object_id);
            println!();
            println!("You can download this object with:");
            println!("  cargo run --example neofs_basic download {} {} downloaded_file.txt", 
                container_id_str, object_id);
        },
        Err(e) => {
            println!("❌ Failed to upload file: {}", e);
        }
    }
    
    Ok(())
}

async fn download_file(container_id_str: &str, object_id_str: &str, file_path_str: &str) -> Result<(), Box<dyn Error>> {
    let client = get_client().await?;
    let container_id = parse_container_id(container_id_str)?;
    let object_id = parse_object_id(object_id_str)?;
    let file_path = Path::new(file_path_str);
    
    println!("Downloading object: {}", object_id_str);
    println!("From container:    {}", container_id_str);
    println!("To file:           {}", file_path_str);
    
    match client.objects().download_to_file(&container_id, &object_id, file_path).await {
        Ok(_) => {
            println!("✅ File downloaded successfully to: {}", file_path_str);
        },
        Err(e) => {
            println!("❌ Failed to download file: {}", e);
        }
    }
    
    Ok(())
} 