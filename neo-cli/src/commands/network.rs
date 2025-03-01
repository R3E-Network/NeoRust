use clap::{Args, Subcommand};
// use neo3::prelude::*;
use crate::utils::error::{CliError, CliResult};
use crate::utils::{print_success, print_error, print_info};
use std::path::PathBuf;
use neo3::prelude::*;
use hex;
use neo3::neo_serializers::BinaryDeserializable;
use crate::commands::wallet::CliState;

#[derive(Args, Debug)]
pub struct NetworkArgs {
    #[command(subcommand)]
    pub command: NetworkCommands,
}

#[derive(Subcommand, Debug)]
pub enum NetworkCommands {
    /// Broadcast network address
    BroadcastAddr,
    
    /// Broadcast block
    BroadcastBlock {
        /// Block hash
        #[arg(short, long)]
        hash: String,
    },
    
    /// Request blocks from network
    BroadcastGetBlocks {
        /// Hash of the block to start from
        #[arg(short, long)]
        hash: String,
        
        /// Number of blocks to request
        #[arg(short, long, default_value = "500")]
        count: u32,
    },
    
    /// Broadcast transaction
    BroadcastTransaction {
        /// Transaction hash or path to transaction file
        #[arg(short, long)]
        hash_or_file: String,
    },
    
    /// Relay a signed transaction
    Relay {
        /// Path to the transaction file
        #[arg(short, long)]
        path: PathBuf,
    },
    
    /// Connect to a specific node
    Connect {
        /// Node address in format: ip:port
        #[arg(short, long)]
        address: String,
    },
    
    /// List connected nodes
    ListNodes,
}

/// CLI state is defined in wallet.rs

pub async fn handle_network_command(args: NetworkArgs, state: &mut crate::commands::wallet::CliState) -> CliResult<()> {
    match args.command {
        NetworkCommands::BroadcastAddr => broadcast_addr(state).await,
        NetworkCommands::BroadcastBlock { hash } => broadcast_block(hash, state).await,
        NetworkCommands::BroadcastGetBlocks { hash, count } => broadcast_get_blocks(hash, count, state).await,
        NetworkCommands::BroadcastTransaction { hash_or_file } => broadcast_transaction(hash_or_file, state).await,
        NetworkCommands::Relay { path } => relay_transaction(path, state).await,
        NetworkCommands::Connect { address } => connect_to_node(address, state).await,
        NetworkCommands::ListNodes => list_nodes(state).await,
    }
}

async fn broadcast_addr(state: &mut crate::commands::wallet::CliState) -> CliResult<()> {
    if state.rpc_client.is_none() {
        print_error("No RPC client is connected. Please connect to a node first.");
        return Err(CliError::Network("No RPC client is connected".to_string()));
    }
    
    print_info("Broadcasting network address...");
    
    let rpc_client = state.rpc_client.as_ref().unwrap();
    let result = rpc_client.broadcast_address().await
        .map_err(|e| CliError::Network(format!("Failed to broadcast address: {}", e)))?;
    
    println!("Response: {}", result);
    print_success("Network address broadcasted successfully");
    Ok(())
}

async fn broadcast_block(hash: String, state: &mut crate::commands::wallet::CliState) -> CliResult<()> {
    if state.rpc_client.is_none() {
        print_error("No RPC client is connected. Please connect to a node first.");
        return Err(CliError::Network("No RPC client is connected".to_string()));
    }
    
    print_info(&format!("Broadcasting block: {}", hash));
    
    let rpc_client = state.rpc_client.as_ref().unwrap();
    
    // Get the block first
    let block = rpc_client.get_block_by_hash(&hash).await
        .map_err(|e| CliError::Network(format!("Failed to get block: {}", e)))?;
    
    // Broadcast the block
    let result = rpc_client.broadcast_block(&block).await
        .map_err(|e| CliError::Network(format!("Failed to broadcast block: {}", e)))?;
    
    println!("Response: {}", result);
    print_success("Block broadcasted successfully");
    Ok(())
}

async fn broadcast_get_blocks(hash: String, count: u32, state: &mut crate::commands::wallet::CliState) -> CliResult<()> {
    if state.rpc_client.is_none() {
        print_error("No RPC client is connected. Please connect to a node first.");
        return Err(CliError::Network("No RPC client is connected".to_string()));
    }
    
    print_info(&format!("Requesting {} blocks starting from: {}", count, hash));
    
    let rpc_client = state.rpc_client.as_ref().unwrap();
    let result = rpc_client.broadcast_get_blocks(&hash, count).await
        .map_err(|e| CliError::Network(format!("Failed to broadcast get blocks: {}", e)))?;
    
    println!("Response: {}", result);
    print_success("Block request sent successfully");
    Ok(())
}

async fn broadcast_transaction(hash_or_file: String, state: &mut CliState) -> CliResult<()> {
    if state.rpc_client.is_none() {
        print_error("No RPC client is connected. Please connect to a node first.");
        return Err(CliError::Network("No RPC client is connected".to_string()));
    }
    
    print_info(&format!("Broadcasting transaction: {}", hash_or_file));
    
    let rpc_client = state.rpc_client.as_ref().unwrap();
    let tx = if hash_or_file.starts_with("0x") || (hash_or_file.len() == 64 && hash_or_file.chars().all(|c| c.is_ascii_hexdigit())) {
        // Hash provided
        // Convert to H256
        let hash_str = if hash_or_file.starts_with("0x") {
            &hash_or_file[2..]
        } else {
            &hash_or_file
        };
        
        let hash_bytes = hex::decode(hash_str)
            .map_err(|e| CliError::Input(format!("Invalid transaction hash format: {}", e)))?;
        
        if hash_bytes.len() != 32 {
            return Err(CliError::Input("Transaction hash must be 32 bytes".to_string()));
        }
        
        let hash = primitive_types::H256::from_slice(&hash_bytes);
        
        rpc_client.get_transaction(hash).await
            .map_err(|e| CliError::Network(format!("Failed to get transaction: {}", e)))?
    } else {
        // File path provided
        let tx_data = std::fs::read(&hash_or_file)
            .map_err(|e| CliError::IO(e.to_string()))?;
        
        RTransaction::deserialize(&tx_data)
            .map_err(|e| CliError::Input(format!("Failed to deserialize transaction: {}", e)))?
    };
    
    let result = rpc_client.broadcast_transaction(&tx).await
        .map_err(|e| CliError::Network(format!("Failed to broadcast transaction: {}", e)))?;
    
    println!("Response: {}", result);
    print_success("Transaction broadcasted successfully");
    Ok(())
}

async fn relay_transaction(path: PathBuf, state: &mut CliState) -> CliResult<()> {
    if state.rpc_client.is_none() {
        print_error("No RPC client is connected. Please connect to a node first.");
        return Err(CliError::Network("No RPC client is connected".to_string()));
    }
    
    print_info(&format!("Relaying transaction from file: {:?}", path));
    
    if !path.exists() {
        print_error(&format!("Transaction file not found: {:?}", path));
        return Err(CliError::Input(format!("Transaction file not found: {:?}", path)));
    }
    
    let tx_data = std::fs::read(&path)
        .map_err(|e| CliError::IO(e.to_string()))?;
    
    let tx = RTransaction::deserialize(&tx_data)
        .map_err(|e| CliError::Input(format!("Failed to deserialize transaction: {}", e)))?;
    
    let rpc_client = state.rpc_client.as_ref().unwrap();
    let result = rpc_client.send_raw_transaction(&tx).await
        .map_err(|e| CliError::Network(format!("Failed to send transaction: {}", e)))?;
    
    println!("Transaction hash: {}", result.hash);
    print_success("Transaction relayed successfully");
    Ok(())
}

async fn connect_to_node(address: String, state: &mut CliState) -> CliResult<()> {
    print_info(&format!("Connecting to node: {}", address));
    
    // Format address if needed
    let rpc_url = if address.starts_with("http://") || address.starts_with("https://") {
        address.to_string()
    } else {
        format!("http://{}", address)
    };
    
    // Load current config
    let mut config = crate::config::CliConfig::load()
        .map_err(|e| CliError::Config(format!("Failed to load config: {}", e)))?;
    
    // Create a new HTTP client
    match Http::new(rpc_url.as_str()) {
        Ok(provider) => {
            // Connect to the node and get information
            match provider.get_version().await {
                Ok(result) => {
                    print_success("Connected to node successfully");
                    println!("Node Information:");
                    println!("  RPC Server: {}", rpc_url);
                    println!("  User Agent: {}", result.user_agent);
                    println!("  Network Magic: {}", result.protocol.magic);
                    println!("  Protocol: {}", result.protocol.address_version);
                    
                    if let Some(state_root) = result.protocol.state_root {
                        println!("  State Root Enabled: {}", state_root);
                    } else {
                        println!("  State Root: Not available");
                    }
                    
                    // Create RPC client and update state
                    let rpc_client = RpcClient::new(provider);
                    state.rpc_client = Some(rpc_client);
                    
                    // Update the state with the connected URL
                    state.rpc_url = Some(rpc_url.clone());
                    config.network.rpc_url = rpc_url.clone();
                    config.save()
                        .map_err(|e| CliError::Config(format!("Failed to save config: {}", e)))?;
                    
                    Ok(())
                },
                Err(e) => {
                    print_error(&format!("Failed to connect to node: {}", e));
                    Err(CliError::Network(format!("Failed to connect to node: {}", e)))
                }
            }
        },
        Err(e) => {
            print_error(&format!("Failed to create HTTP client: {}", e));
            Err(CliError::Network(format!("Failed to create HTTP client: {}", e)))
        }
    }
}

async fn list_nodes(state: &mut crate::commands::wallet::CliState) -> CliResult<()> {
    if state.rpc_client.is_none() {
        print_error("No RPC client is connected. Please connect to a node first.");
        return Err(CliError::Network("No RPC client is connected".to_string()));
    }
    
    print_info("Connected nodes:");
    
    let rpc_client = state.rpc_client.as_ref().unwrap();
    
    // Get peers from the node
    match rpc_client.get_peers().await {
        Ok(peers) => {
            if peers.connected.is_empty() {
                println!("  No connected peers found");
            } else {
                println!("  Connected peers:");
                for (i, peer) in peers.connected.iter().enumerate() {
                    println!("  {}. Address: {}:{}", i + 1, peer.address, peer.port);
                }
            }
            
            if !peers.unconnected.is_empty() {
                println!("\n  Unconnected peers:");
                for (i, peer) in peers.unconnected.iter().enumerate() {
                    println!("  {}. Address: {}:{}", i + 1, peer.address, peer.port);
                }
            }
            
            if !peers.bad.is_empty() {
                println!("\n  Bad peers:");
                for (i, peer) in peers.bad.iter().enumerate() {
                    println!("  {}. Address: {}:{}", i + 1, peer.address, peer.port);
                }
            }
        },
        Err(e) => {
            print_error(&format!("Failed to get peers: {}", e));
            return Err(CliError::Network(format!("Failed to get peers: {}", e)));
        }
    }
    
    print_success("Node list retrieved successfully");
    Ok(())
}
