use clap::{Args, Subcommand};
use neo3::prelude::*;
use crate::utils::error::{CliError, CliResult};
use crate::utils::{print_success, print_error, print_info};
use std::path::PathBuf;
use crate::utils::extensions::TransactionExtensions;

#[derive(Args, Debug)]
pub struct BlockchainArgs {
    #[command(subcommand)]
    pub command: BlockchainCommands,
}

#[derive(Subcommand, Debug)]
pub enum BlockchainCommands {
    /// Export blockchain data
    Export {
        /// Path to save the exported data
        #[arg(short, long)]
        path: PathBuf,
        
        /// Start block index (inclusive)
        #[arg(short, long, default_value = "0")]
        start: u32,
        
        /// End block index (inclusive, if not specified, exports to the latest block)
        #[arg(short, long)]
        end: Option<u32>,
    },
    
    /// Show block details
    ShowBlock {
        /// Block hash or index
        #[arg(short, long)]
        identifier: String,
    },
    
    /// Show transaction information
    ShowTx {
        /// Transaction hash
        #[arg(short, long)]
        hash: String,
    },
    
    /// Show contract details
    ShowContract {
        /// Contract hash or script hash
        #[arg(short, long)]
        hash: String,
    },
}

/// CLI state is defined in wallet.rs

pub async fn handle_blockchain_command(args: BlockchainArgs, state: &mut crate::commands::wallet::CliState) -> CliResult<()> {
    match args.command {
        BlockchainCommands::Export { path, start, end } => export_blockchain(path, start, end, state).await,
        BlockchainCommands::ShowBlock { identifier } => show_block(identifier, state).await,
        BlockchainCommands::ShowTx { hash } => show_transaction(hash, state).await,
        BlockchainCommands::ShowContract { hash } => show_contract(hash, state).await,
    }
}

async fn export_blockchain(path: PathBuf, start: u32, end: Option<u32>, state: &mut crate::commands::wallet::CliState) -> CliResult<()> {
    if state.rpc_client.is_none() {
        print_error("No RPC client is connected. Please connect to a node first.");
        return Err(CliError::Network("No RPC client is connected".to_string()));
    }
    
    print_info(&format!("Exporting blockchain data from block {} to {}...", 
        start, end.map_or("latest".to_string(), |e| e.to_string())));
    
    let rpc_client = state.rpc_client.as_ref().unwrap();
    let latest_block = rpc_client.get_block_count().await
        .map_err(|e| CliError::Network(format!("Failed to get block count: {}", e)))?;
    let end_block = end.unwrap_or(latest_block - 1);
    
    if start > end_block {
        print_error("Start block index is greater than end block index");
        return Err(CliError::Input("Invalid block range".to_string()));
    }
    
    // Create export directory if it doesn't exist
    std::fs::create_dir_all(&path).map_err(|e| CliError::Io(e))?;
    
    // Export blocks
    let mut exported = 0;
    let total_blocks = end_block - start + 1;
    
    for i in start..=end_block {
        let block = match rpc_client.get_block(i).await {
            Ok(block) => block,
            Err(e) => {
                print_error(&format!("Failed to get block {}: {}", i, e));
                continue;
            }
        };
        
        // Export block to JSON file
        let block_path = path.join(format!("block_{}.json", i));
        let json = serde_json::to_string_pretty(&block)
            .map_err(|e| CliError::Input(format!("Failed to serialize block: {}", e)))?;
        
        std::fs::write(&block_path, json)
            .map_err(|e| CliError::Io(e))?;
        
        exported += 1;
        
        // Show progress
        if exported % 100 == 0 || exported == total_blocks {
            print_info(&format!("Exported {}/{} blocks ({}%)", 
                exported, total_blocks, (exported * 100) / total_blocks));
        }
    }
    
    print_success(&format!("Blockchain data exported to: {:?} ({} blocks)", path, exported));
    Ok(())
}

async fn show_block(identifier: String, state: &mut crate::commands::wallet::CliState) -> CliResult<()> {
    if state.rpc_client.is_none() {
        print_error("No RPC client is connected. Please connect to a node first.");
        return Err(CliError::Network("No RPC client is connected".to_string()));
    }
    
    print_info(&format!("Fetching block information for: {}", identifier));
    
    let rpc_client = state.rpc_client.as_ref().unwrap();
    
    // Determine if identifier is a hash or an index
    let block = if identifier.starts_with("0x") || (identifier.len() == 64 && identifier.chars().all(|c| c.is_ascii_hexdigit())) {
        // Identifier is a hash
        match rpc_client.get_block_by_hash(&identifier).await {
            Ok(block) => block,
            Err(e) => {
                print_error(&format!("Failed to get block by hash: {}", e));
                return Err(CliError::Network(format!("Failed to get block by hash: {}", e)));
            }
        }
    } else {
        // Identifier is an index
        let index = identifier.parse::<u32>().map_err(|_| CliError::Input("Invalid block index".to_string()))?;
        match rpc_client.get_block(index).await {
            Ok(block) => block,
            Err(e) => {
                print_error(&format!("Failed to get block by index: {}", e));
                return Err(CliError::Network(format!("Failed to get block by index: {}", e)));
            }
        }
    };
    
    // Display block information
    println!("Block Hash: {}", block.hash);
    println!("Block Index: {}", block.index);
    println!("Block Time: {}", block.time);
    println!("Block Size: {}", block.size);
    println!("Transaction Count: {}", block.tx.len());
    println!("Merkle Root: {}", block.merkle_root);
    println!("Previous Block: {}", block.prev_hash);
    println!("Next Consensus: {}", block.next_consensus);
    
    // Show transactions if there are any
    if !block.tx.is_empty() {
        println!("\nTransactions:");
        for (i, tx) in block.tx.iter().enumerate() {
            println!("  {}. Hash: {}", i + 1, tx);
        }
    }
    
    print_success("Block information retrieved successfully");
    Ok(())
}

async fn show_transaction(hash: String, state: &mut crate::commands::wallet::CliState) -> CliResult<()> {
    if state.rpc_client.is_none() {
        print_error("No RPC client is connected. Please connect to a node first.");
        return Err(CliError::Network("No RPC client is connected".to_string()));
    }
    
    print_info(&format!("Fetching transaction information for: {}", hash));
    
    let rpc_client = state.rpc_client.as_ref().unwrap();
    let tx = rpc_client.get_transaction(&hash).await
        .map_err(|e| CliError::Network(format!("Failed to get transaction: {}", e)))?;
    
    // Display transaction information
    println!("Transaction Hash: {}", tx.hash);
    println!("Transaction Type: {}", tx.type_name());
    println!("Transaction Size: {}", tx.size);
    println!("Transaction Version: {}", tx.version);
    println!("Transaction Nonce: {}", tx.nonce);
    println!("Transaction Sender: {}", tx.sender);
    println!("Transaction System Fee: {}", tx.sys_fee);
    println!("Transaction Network Fee: {}", tx.net_fee);
    println!("Transaction Valid Until Block: {}", tx.valid_until_block);
    
    // Display signers
    println!("\nTransaction Signers ({}):", tx.signers.len());
    for (i, signer) in tx.signers.iter().enumerate() {
        println!("  {}. Account: {}", i + 1, signer.account);
        println!("     Scopes: {:?}", signer.scopes);
        if !signer.allowed_contracts.is_empty() {
            println!("     Allowed Contracts: {:?}", signer.allowed_contracts);
        }
        if !signer.allowed_groups.is_empty() {
            println!("     Allowed Groups: {:?}", signer.allowed_groups);
        }
    }
    
    // Display witnesses
    println!("\nTransaction Witnesses ({}):", tx.witnesses.len());
    for (i, witness) in tx.witnesses.iter().enumerate() {
        println!("  {}. Invocation Script: 0x{}", i + 1, hex::encode(&witness.invocation_script));
        println!("     Verification Script: 0x{}", hex::encode(&witness.verification_script));
    }
    
    // Display script
    println!("\nTransaction Script: 0x{}", hex::encode(&tx.script));
    
    print_success("Transaction information retrieved successfully");
    Ok(())
}

async fn show_contract(hash: String, state: &mut crate::commands::wallet::CliState) -> CliResult<()> {
    if state.rpc_client.is_none() {
        print_error("No RPC client is connected. Please connect to a node first.");
        return Err(CliError::Network("No RPC client is connected".to_string()));
    }
    
    print_info(&format!("Fetching contract information for: {}", hash));
    
    let rpc_client = state.rpc_client.as_ref().unwrap();
    let contract = rpc_client.get_contract_state(&hash).await
        .map_err(|e| CliError::Network(format!("Failed to get contract: {}", e)))?;
    
    // Display contract information
    println!("Contract Hash: {}", contract.hash);
    println!("Contract ID: {}", contract.id);
    println!("Contract Update Counter: {}", contract.update_counter);
    println!("Contract NEF: {}", contract.nef.checksum);
    
    println!("\nContract Manifest:");
    println!("  Name: {}", contract.manifest.name);
    println!("  Groups: {}", contract.manifest.groups.len());
    println!("  Features:");
    for (feature, value) in contract.manifest.features.iter() {
        println!("    {}: {}", feature, value);
    }
    
    println!("  Supported Standards:");
    for standard in contract.manifest.supported_standards.iter() {
        println!("    {}", standard);
    }
    
    println!("\n  ABI Methods ({}):", contract.manifest.abi.methods.len());
    for (i, method) in contract.manifest.abi.methods.iter().enumerate() {
        println!("    {}. Name: {}", i + 1, method.name);
        println!("       Parameters: {:?}", method.parameters);
        println!("       Return Type: {:?}", method.return_type);
        println!("       Offset: {}", method.offset);
        println!("       Safe: {}", method.safe);
    }
    
    println!("\n  ABI Events ({}):", contract.manifest.abi.events.len());
    for (i, event) in contract.manifest.abi.events.iter().enumerate() {
        println!("    {}. Name: {}", i + 1, event.name);
        println!("       Parameters: {:?}", event.parameters);
    }
    
    println!("\n  Permissions ({}):", contract.manifest.permissions.len());
    for (i, perm) in contract.manifest.permissions.iter().enumerate() {
        println!("    {}. {}", i + 1, perm);
    }
    
    println!("\n  Trusts ({}):", contract.manifest.trusts.len());
    for (i, trust) in contract.manifest.trusts.iter().enumerate() {
        println!("    {}. {}", i + 1, trust);
    }
    
    print_success("Contract information retrieved successfully");
    Ok(())
}
