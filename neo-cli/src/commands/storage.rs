use clap::{Args, Subcommand};
use colored::Colorize;
use prettytable::{Table, row, cell};
use std::path::PathBuf;
use neo::prelude::*;
use crate::utils::output::{print_error, print_success, CliOutput};
use crate::utils::wallet::get_wallet_account;
use crate::config::AppConfig;
use anyhow::{Result, Context};

/// Commands for interacting with NeoFS decentralized storage
#[derive(Args, Debug)]
pub struct StorageArgs {
    #[command(subcommand)]
    command: StorageCommands,
}

/// Storage subcommands for NeoFS operations
#[derive(Subcommand, Debug)]
pub enum StorageCommands {
    /// Get information about the NeoFS network
    Info,
    
    /// Manage containers in NeoFS
    #[command(subcommand)]
    Container(ContainerCommands),
    
    /// Manage objects in NeoFS
    #[command(subcommand)]
    Object(ObjectCommands),
}

/// Commands for container management
#[derive(Subcommand, Debug)]
pub enum ContainerCommands {
    /// List containers owned by the account
    List,
    
    /// Create a new container
    Create {
        /// Friendly name for the container (stored as attribute)
        #[arg(long)]
        name: String,
        
        /// Whether the container is publicly readable
        #[arg(long)]
        public: bool,
        
        /// Number of replicas to maintain
        #[arg(long, default_value = "3")]
        replicas: u8,
    },
    
    /// Get container information
    Get {
        /// Container ID (hex format)
        container_id: String,
    },
    
    /// Delete a container
    Delete {
        /// Container ID (hex format)
        container_id: String,
        
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
}

/// Commands for object management
#[derive(Subcommand, Debug)]
pub enum ObjectCommands {
    /// Upload a file to NeoFS
    Upload {
        /// Container ID (hex format)
        container_id: String,
        
        /// Path to the file to upload
        file_path: PathBuf,
        
        /// Content type of the file (optional)
        #[arg(long)]
        content_type: Option<String>,
        
        /// Friendly name for the object (optional)
        #[arg(long)]
        name: Option<String>,
    },
    
    /// Download an object from NeoFS
    Download {
        /// Container ID (hex format)
        container_id: String,
        
        /// Object ID (hex format)
        object_id: String,
        
        /// Path to save the downloaded file
        #[arg(long)]
        output: Option<PathBuf>,
    },
    
    /// List objects in a container
    List {
        /// Container ID (hex format)
        container_id: String,
        
        /// Filter objects by attribute (e.g., --filter key=value)
        #[arg(long)]
        filter: Option<String>,
    },
    
    /// Get object information
    Info {
        /// Container ID (hex format)
        container_id: String,
        
        /// Object ID (hex format)
        object_id: String,
    },
    
    /// Delete an object
    Delete {
        /// Container ID (hex format)
        container_id: String,
        
        /// Object ID (hex format)
        object_id: String,
        
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
}

/// Handler for storage commands
pub async fn handle_storage_command(args: StorageArgs, config: AppConfig) -> Result<CliOutput> {
    match args.command {
        StorageCommands::Info => handle_info(config).await,
        StorageCommands::Container(cmd) => handle_container_command(cmd, config).await,
        StorageCommands::Object(cmd) => handle_object_command(cmd, config).await,
    }
}

/// Handler for network information command
async fn handle_info(config: AppConfig) -> Result<CliOutput> {
    // Create NeoFS client
    let client = create_neofs_client(&config).await?;
    
    // Get network information
    let network_info = client.get_network_info().await
        .map_err(|e| anyhow::anyhow!("Failed to get NeoFS network info: {}", e))?;
    
    // Get network statistics
    let network_stats = client.network().get_stats().await
        .map_err(|e| anyhow::anyhow!("Failed to get NeoFS network stats: {}", e))?;
    
    // Create output table
    let mut table = Table::new();
    table.add_row(row!["NeoFS Network Information"]);
    table.add_row(row!["Protocol Version", network_info.version]);
    table.add_row(row!["Active Nodes", network_info.node_count.to_string()]);
    table.add_row(row!["Total Containers", network_stats.total_containers.to_string()]);
    table.add_row(row!["Total Objects", network_stats.total_objects.to_string()]);
    
    // Format storage sizes in human-readable format
    let total_capacity_str = format_bytes(network_stats.total_storage_capacity);
    let used_storage_str = format_bytes(network_stats.total_storage_used);
    let available_storage_str = format_bytes(network_info.available_space);
    
    table.add_row(row!["Total Storage Capacity", total_capacity_str]);
    table.add_row(row!["Used Storage", used_storage_str]);
    table.add_row(row!["Available Storage", available_storage_str]);
    
    Ok(CliOutput::new().with_table(table))
}

/// Handler for container commands
async fn handle_container_command(cmd: ContainerCommands, config: AppConfig) -> Result<CliOutput> {
    match cmd {
        ContainerCommands::List => handle_container_list(config).await,
        ContainerCommands::Create { name, public, replicas } => {
            handle_container_create(name, public, replicas, config).await
        },
        ContainerCommands::Get { container_id } => handle_container_get(container_id, config).await,
        ContainerCommands::Delete { container_id, force } => {
            handle_container_delete(container_id, force, config).await
        },
    }
}

/// Handler for object commands
async fn handle_object_command(cmd: ObjectCommands, config: AppConfig) -> Result<CliOutput> {
    match cmd {
        ObjectCommands::Upload { container_id, file_path, content_type, name } => {
            handle_object_upload(container_id, file_path, content_type, name, config).await
        },
        ObjectCommands::Download { container_id, object_id, output } => {
            handle_object_download(container_id, object_id, output, config).await
        },
        ObjectCommands::List { container_id, filter } => {
            handle_object_list(container_id, filter, config).await
        },
        ObjectCommands::Info { container_id, object_id } => {
            handle_object_info(container_id, object_id, config).await
        },
        ObjectCommands::Delete { container_id, object_id, force } => {
            handle_object_delete(container_id, object_id, force, config).await
        },
    }
}

/// Handler for container list command
async fn handle_container_list(config: AppConfig) -> Result<CliOutput> {
    // Create NeoFS client with account
    let client = create_neofs_client_with_account(&config).await?;
    
    // Get containers
    let containers = client.containers().list().await
        .map_err(|e| anyhow::anyhow!("Failed to list containers: {}", e))?;
    
    if containers.is_empty() {
        return Ok(CliOutput::new().with_message("No containers found for this account.".into()));
    }
    
    // Create output table
    let mut table = Table::new();
    table.add_row(row!["Container ID", "Size", "Objects", "Created At", "Attributes"]);
    
    for container in containers {
        let size_str = format_bytes(container.size);
        let created_at = format_timestamp(container.created_at);
        
        // Format attributes as key-value pairs
        let attributes = container.attributes.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join(", ");
        
        table.add_row(row![
            container.container_id.to_string(),
            size_str,
            container.object_count.to_string(),
            created_at,
            attributes
        ]);
    }
    
    Ok(CliOutput::new().with_table(table))
}

/// Handler for container create command
async fn handle_container_create(name: String, public: bool, replicas: u8, config: AppConfig) -> Result<CliOutput> {
    // Create NeoFS client with account
    let client = create_neofs_client_with_account(&config).await?;
    
    // Create container parameters
    let mut attributes = vec![("Name".to_string(), name.clone())];
    
    // Add timestamp
    let timestamp = chrono::Utc::now().timestamp().to_string();
    attributes.push(("CreatedAt".to_string(), timestamp));
    
    // Create ACL rules
    let mut rules = Vec::new();
    if public {
        rules.push(AccessRule::Public);
    } else {
        // Only the owner can access by default
        if let Some(account) = client.account() {
            let pk = account.get_public_key().to_bytes();
            rules.push(AccessRule::Private(vec![PublicKey::from_bytes(&pk).unwrap()]));
        }
    }
    
    // Create policy
    let policy = StoragePolicy {
        replicas,
        placement: crate::neo_fs::types::PlacementPolicy {
            regions: vec![],
            tier: crate::neo_fs::types::ReliabilityTier::Standard,
            min_nodes_per_region: 1,
        },
        lifetime: 0, // 0 means no expiration
    };
    
    // Create container
    let params = crate::neo_fs::client::CreateContainerParams {
        rules,
        policy,
        attributes,
    };
    
    let container_id = client.containers().create(params).await
        .map_err(|e| anyhow::anyhow!("Failed to create container: {}", e))?;
    
    Ok(CliOutput::new().with_message(format!("Container created successfully. ID: {}", container_id)))
}

/// Handler for container get command
async fn handle_container_get(container_id: String, config: AppConfig) -> Result<CliOutput> {
    // Create NeoFS client
    let client = create_neofs_client(&config).await?;
    
    // Parse container ID
    let container_id = ContainerId::from_hex(&container_id)
        .context("Invalid container ID format. Expected hex string.")?;
    
    // Get container info
    let container_info = client.containers().get(&container_id).await
        .map_err(|e| anyhow::anyhow!("Failed to get container info: {}", e))?;
    
    // Create output table
    let mut table = Table::new();
    table.add_row(row!["Container Information"]);
    table.add_row(row!["ID", container_info.container_id.to_string()]);
    
    // Format owner as hex
    let owner_hex = hex::encode(&container_info.owner);
    table.add_row(row!["Owner", owner_hex]);
    
    // Format other fields
    let size_str = format_bytes(container_info.size);
    let created_at = format_timestamp(container_info.created_at);
    
    table.add_row(row!["Size", size_str]);
    table.add_row(row!["Objects", container_info.object_count.to_string()]);
    table.add_row(row!["Created At", created_at]);
    table.add_row(row!["Basic ACL", format!("0x{:x}", container_info.basic_acl)]);
    
    // Add attributes
    table.add_row(row!["Attributes"]);
    for (key, value) in &container_info.attributes {
        table.add_row(row!["", format!("{}: {}", key, value)]);
    }
    
    Ok(CliOutput::new().with_table(table))
}

/// Handler for container delete command
async fn handle_container_delete(container_id: String, force: bool, config: AppConfig) -> Result<CliOutput> {
    // Create NeoFS client with account
    let client = create_neofs_client_with_account(&config).await?;
    
    // Parse container ID
    let container_id = ContainerId::from_hex(&container_id)
        .context("Invalid container ID format. Expected hex string.")?;
    
    // Ask for confirmation unless force flag is used
    if !force {
        println!("WARNING: Deleting a container is irreversible and will remove all objects inside.");
        println!("Container ID: {}", container_id);
        
        // Use dialoguer to confirm
        let confirm = dialoguer::Confirm::new()
            .with_prompt("Are you sure you want to delete this container?")
            .default(false)
            .interact()?;
        
        if !confirm {
            return Ok(CliOutput::new().with_message("Container deletion cancelled.".into()));
        }
    }
    
    // Delete container
    client.containers().delete(&container_id).await
        .map_err(|e| anyhow::anyhow!("Failed to delete container: {}", e))?;
    
    Ok(CliOutput::new().with_message("Container deleted successfully.".into()))
}

/// Handler for object upload command
async fn handle_object_upload(
    container_id: String,
    file_path: PathBuf,
    content_type: Option<String>,
    name: Option<String>,
    config: AppConfig
) -> Result<CliOutput> {
    // Create NeoFS client with account
    let client = create_neofs_client_with_account(&config).await?;
    
    // Parse container ID
    let container_id = ContainerId::from_hex(&container_id)
        .context("Invalid container ID format. Expected hex string.")?;
    
    // Check if file exists
    if !file_path.exists() {
        return Err(anyhow::anyhow!("File not found: {}", file_path.display()));
    }
    
    // Read file
    let data = std::fs::read(&file_path)
        .context(format!("Failed to read file: {}", file_path.display()))?;
    
    // Create attributes
    let mut attributes = Vec::new();
    
    // Add filename
    if let Some(file_name) = file_path.file_name() {
        if let Some(file_name_str) = file_name.to_str() {
            attributes.push(("Filename".to_string(), file_name_str.to_string()));
        }
    }
    
    // Add content type if provided
    if let Some(ct) = content_type {
        attributes.push(("Content-Type".to_string(), ct));
    } else {
        // Try to guess content type
        if let Some(ext) = file_path.extension() {
            if let Some(ext_str) = ext.to_str() {
                let content_type = match ext_str.to_lowercase().as_str() {
                    "jpg" | "jpeg" => "image/jpeg",
                    "png" => "image/png",
                    "gif" => "image/gif",
                    "pdf" => "application/pdf",
                    "txt" => "text/plain",
                    "html" | "htm" => "text/html",
                    "json" => "application/json",
                    "xml" => "application/xml",
                    "zip" => "application/zip",
                    "doc" | "docx" => "application/msword",
                    "xls" | "xlsx" => "application/vnd.ms-excel",
                    "ppt" | "pptx" => "application/vnd.ms-powerpoint",
                    _ => "application/octet-stream",
                };
                attributes.push(("Content-Type".to_string(), content_type.to_string()));
            }
        }
    }
    
    // Add custom name if provided
    if let Some(custom_name) = name {
        attributes.push(("Name".to_string(), custom_name));
    }
    
    // Add timestamp
    let timestamp = chrono::Utc::now().timestamp().to_string();
    attributes.push(("Timestamp".to_string(), timestamp));
    
    // Upload file
    println!("Uploading file: {}", file_path.display());
    println!("Size: {} bytes", data.len());
    
    let object_id = client.objects().upload(&container_id, data, attributes).await
        .map_err(|e| anyhow::anyhow!("Failed to upload file: {}", e))?;
    
    Ok(CliOutput::new().with_message(format!("File uploaded successfully. Object ID: {}", object_id)))
}

/// Handler for object download command
async fn handle_object_download(
    container_id: String,
    object_id: String,
    output: Option<PathBuf>,
    config: AppConfig
) -> Result<CliOutput> {
    // Create NeoFS client
    let client = create_neofs_client(&config).await?;
    
    // Parse container ID and object ID
    let container_id = ContainerId::from_hex(&container_id)
        .context("Invalid container ID format. Expected hex string.")?;
    
    let object_id = ObjectId::from_hex(&object_id)
        .context("Invalid object ID format. Expected hex string.")?;
    
    // Download object
    println!("Downloading object {} from container {}...", object_id, container_id);
    
    let data = client.objects().download(&container_id, &object_id).await
        .map_err(|e| anyhow::anyhow!("Failed to download object: {}", e))?;
    
    println!("Downloaded {} bytes", data.len());
    
    // Get object info to retrieve filename
    let object_info = client.objects().get_info(&container_id, &object_id).await
        .map_err(|e| anyhow::anyhow!("Failed to get object info: {}", e))?;
    
    // Determine output path
    let output_path = if let Some(path) = output {
        path
    } else {
        // Try to get filename from object attributes
        let filename = object_info.attributes.iter()
            .find(|(k, _)| k == "Filename")
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| format!("object_{}.bin", hex::encode(&object_id.as_bytes()[..8])));
        
        PathBuf::from(&filename)
    };
    
    // Write to file
    std::fs::write(&output_path, data)
        .context(format!("Failed to write to file: {}", output_path.display()))?;
    
    Ok(CliOutput::new().with_message(format!("Object downloaded successfully to {}", output_path.display())))
}

/// Handler for object list command
async fn handle_object_list(
    container_id: String,
    filter: Option<String>,
    config: AppConfig
) -> Result<CliOutput> {
    // Create NeoFS client
    let client = create_neofs_client(&config).await?;
    
    // Parse container ID
    let container_id = ContainerId::from_hex(&container_id)
        .context("Invalid container ID format. Expected hex string.")?;
    
    // Parse filter if provided
    let filters = if let Some(filter_str) = filter {
        let parts: Vec<&str> = filter_str.split('=').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid filter format. Expected key=value"));
        }
        
        vec![(parts[0].to_string(), parts[1].to_string())]
    } else {
        vec![]
    };
    
    // List objects
    let objects = client.objects().search(&container_id, filters).await
        .map_err(|e| anyhow::anyhow!("Failed to list objects: {}", e))?;
    
    if objects.is_empty() {
        return Ok(CliOutput::new().with_message("No objects found in this container.".into()));
    }
    
    // Create output table
    let mut table = Table::new();
    table.add_row(row!["Object ID"]);
    
    for object_id in objects {
        table.add_row(row![object_id.to_string()]);
    }
    
    Ok(CliOutput::new()
        .with_message(format!("Found {} objects in container {}", objects.len(), container_id))
        .with_table(table))
}

/// Handler for object info command
async fn handle_object_info(
    container_id: String,
    object_id: String,
    config: AppConfig
) -> Result<CliOutput> {
    // Create NeoFS client
    let client = create_neofs_client(&config).await?;
    
    // Parse container ID and object ID
    let container_id = ContainerId::from_hex(&container_id)
        .context("Invalid container ID format. Expected hex string.")?;
    
    let object_id = ObjectId::from_hex(&object_id)
        .context("Invalid object ID format. Expected hex string.")?;
    
    // Get object info
    let object_info = client.objects().get_info(&container_id, &object_id).await
        .map_err(|e| anyhow::anyhow!("Failed to get object info: {}", e))?;
    
    // Create output table
    let mut table = Table::new();
    table.add_row(row!["Object Information"]);
    table.add_row(row!["ID", object_info.object_id.to_string()]);
    table.add_row(row!["Container ID", object_info.container_id.to_string()]);
    
    // Format owner as hex
    let owner_hex = hex::encode(&object_info.owner);
    table.add_row(row!["Owner", owner_hex]);
    
    // Format size
    let size_str = format_bytes(object_info.size);
    let created_at = format_timestamp(object_info.created_at);
    
    table.add_row(row!["Size", size_str]);
    table.add_row(row!["Created At", created_at]);
    
    // Add attributes
    table.add_row(row!["Attributes"]);
    for (key, value) in &object_info.attributes {
        table.add_row(row!["", format!("{}: {}", key, value)]);
    }
    
    Ok(CliOutput::new().with_table(table))
}

/// Handler for object delete command
async fn handle_object_delete(
    container_id: String,
    object_id: String,
    force: bool,
    config: AppConfig
) -> Result<CliOutput> {
    // Create NeoFS client with account
    let client = create_neofs_client_with_account(&config).await?;
    
    // Parse container ID and object ID
    let container_id = ContainerId::from_hex(&container_id)
        .context("Invalid container ID format. Expected hex string.")?;
    
    let object_id = ObjectId::from_hex(&object_id)
        .context("Invalid object ID format. Expected hex string.")?;
    
    // Ask for confirmation unless force flag is used
    if !force {
        println!("WARNING: Deleting an object is irreversible.");
        println!("Object ID: {}", object_id);
        
        // Use dialoguer to confirm
        let confirm = dialoguer::Confirm::new()
            .with_prompt("Are you sure you want to delete this object?")
            .default(false)
            .interact()?;
        
        if !confirm {
            return Ok(CliOutput::new().with_message("Object deletion cancelled.".into()));
        }
    }
    
    // Delete object
    client.objects().delete(&container_id, &object_id).await
        .map_err(|e| anyhow::anyhow!("Failed to delete object: {}", e))?;
    
    Ok(CliOutput::new().with_message("Object deleted successfully.".into()))
}

/// Helper function to create NeoFS client
async fn create_neofs_client(config: &AppConfig) -> Result<NeoFsClient> {
    let neofs_config = NeoFsConfig {
        endpoint: config.neofs_endpoint.clone().unwrap_or_else(|| "https://fs.neo.org".to_string()),
        timeout_seconds: 60,
        max_concurrent_requests: 5,
    };
    
    NeoFsClient::with_config(neofs_config)
        .map_err(|e| anyhow::anyhow!("Failed to create NeoFS client: {}", e))
}

/// Helper function to create NeoFS client with account
async fn create_neofs_client_with_account(config: &AppConfig) -> Result<NeoFsClient> {
    let client = create_neofs_client(config).await?;
    
    // Get wallet account
    let (wallet, account) = get_wallet_account(config)?;
    
    Ok(client.with_account(account))
}

/// Format bytes to human-readable format
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;
    
    if bytes < KB {
        format!("{} B", bytes)
    } else if bytes < MB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else if bytes < GB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes < TB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    }
}

/// Format timestamp to human-readable format
fn format_timestamp(timestamp: u64) -> String {
    let dt = chrono::DateTime::<chrono::Utc>::from_timestamp(timestamp as i64, 0)
        .unwrap_or_else(|| chrono::Utc::now());
    
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
} 