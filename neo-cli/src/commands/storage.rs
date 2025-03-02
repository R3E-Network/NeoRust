use clap::{Args, Subcommand};
use colored::Colorize;
use prettytable::{Table, row, cell};
use std::path::PathBuf;
use neo::prelude::*;
use crate::utils::output::{print_error, print_success, CliOutput};
use crate::utils::wallet::get_wallet_account;
use crate::config::AppConfig;
use anyhow::{Result, Context};
use neo::neo_fs::{
    AccessRule, Container, ContainerBuilder, ContainerId, Object, ObjectBuilder, 
    NeoFsClient, NeoFsConfig, PublicKey, ReliabilityTier, types::PlacementPolicy
};
use crate::utils::error::{CliError, CliResult};
use std::str::FromStr;
use crate::utils;

/// Output structure for CLI commands
#[derive(Debug)]
pub struct CliOutput {
    message: Option<String>,
    table: Option<Table>,
}

impl CliOutput {
    pub fn new() -> Self {
        Self {
            message: None,
            table: None,
        }
    }
    
    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }
    
    pub fn with_table(mut self, table: Table) -> Self {
        self.table = Some(table);
        self
    }
}

/// Arguments for the storage command
#[derive(Debug, Args)]
pub struct StorageArgs {
    #[command(subcommand)]
    pub command: StorageCommands,
}

/// Storage subcommands for NeoFS operations
#[derive(Debug, Subcommand)]
pub enum StorageCommands {
    /// Display information about NeoFS connection
    Info,
    
    /// Manage containers in NeoFS
    Container {
        #[command(subcommand)]
        command: ContainerCommands,
    },
    
    /// Manage objects in NeoFS
    Object {
        #[command(subcommand)]
        command: ObjectCommands,
    },
}

/// Commands for container operations
#[derive(Debug, Subcommand)]
pub enum ContainerCommands {
    /// List containers owned by the current account
    List,
    /// Create a new container
    Create {
        /// Name of the container
        #[arg(long)]
        name: String,
        /// Basic ACL permissions as a decimal or hex number
        #[arg(long, default_value = "0644")]
        basic_acl: String,
        /// Placement policy for the container
        #[arg(long)]
        placement_policy: Option<String>,
    },
    /// Get container info
    Info {
        /// Container ID in hexadecimal format
        #[arg(long)]
        id: String,
    },
    /// Delete a container
    Delete {
        /// Container ID in hexadecimal format
        #[arg(long)]
        id: String,
    },
}

/// Commands for object operations
#[derive(Debug, Subcommand)]
pub enum ObjectCommands {
    /// Upload a file to a container
    Upload {
        /// Container ID in hexadecimal format
        #[arg(long)]
        container: String,
        /// Path to the file to upload
        #[arg(long)]
        file: PathBuf,
        /// Key-value attributes for the object (format: key=value)
        #[arg(long)]
        attributes: Vec<String>,
    },
    /// Download an object from a container
    Download {
        /// Container ID in hexadecimal format
        #[arg(long)]
        container: String,
        /// Object ID in hexadecimal format
        #[arg(long)]
        id: String,
        /// Path to save the downloaded file
        #[arg(long)]
        output: PathBuf,
    },
    /// List objects in a container
    List {
        /// Container ID in hexadecimal format
        #[arg(long)]
        container: String,
    },
    /// Get object info
    Info {
        /// Container ID in hexadecimal format
        #[arg(long)]
        container: String,
        /// Object ID in hexadecimal format
        #[arg(long)]
        id: String,
    },
    /// Delete an object
    Delete {
        /// Container ID in hexadecimal format
        #[arg(long)]
        container: String,
        /// Object ID in hexadecimal format
        #[arg(long)]
        id: String,
    },
}

/// Handle the storage command
pub fn handle_storage_command(
    args: &StorageArgs,
    state: &mut crate::commands::wallet::CliState,
) -> CliResult<CliOutput> {
    match &args.command {
        StorageCommands::Info => handle_info(state),
        StorageCommands::Container { command } => handle_container_command(command, state),
        StorageCommands::Object { command } => handle_object_command(command, state),
    }
}

/// Display info about NeoFS connection
fn handle_info(state: &mut crate::commands::wallet::CliState) -> CliResult<CliOutput> {
    // Create a NeoFS client
    let client = match create_neofs_client(state) {
        Ok(client) => client,
        Err(e) => return Err(e),
    };

    // Create a table for the output
    let mut table = Table::new();
    table.add_row(row!["Endpoint", "Status", "Version"]);

    // Check the connection by getting the network info
    let status = match client.network().get_network_info() {
        Ok(info) => {
            table.add_row(row![
                "http://localhost:8080", 
                "Connected".green(), 
                info.version
            ]);
            CliOutput::new().with_table(table)
        },
        Err(e) => {
            table.add_row(row![
                "http://localhost:8080", 
                "Failed".red(), 
                format!("Error: {}", e)
            ]);
            CliOutput::new().with_table(table)
        }
    };

    Ok(status)
}

/// Handle container commands
fn handle_container_command(
    command: &ContainerCommands,
    state: &mut crate::commands::wallet::CliState,
) -> CliResult<CliOutput> {
    match command {
        ContainerCommands::List => handle_container_list(state),
        ContainerCommands::Create { name, basic_acl, placement_policy } => {
            handle_container_create(name, basic_acl, placement_policy.clone(), state)
        },
        ContainerCommands::Info { id } => handle_container_info(id, state),
        ContainerCommands::Delete { id } => handle_container_delete(id, state),
    }
}

/// Handle object commands
fn handle_object_command(
    command: &ObjectCommands,
    state: &mut crate::commands::wallet::CliState,
) -> CliResult<CliOutput> {
    match command {
        ObjectCommands::Upload { container, file, attributes } => {
            handle_object_upload(container, file, attributes, state)
        },
        ObjectCommands::Download { container, id, output } => {
            handle_object_download(container, id, output, state)
        },
        ObjectCommands::List { container } => handle_object_list(container, state),
        ObjectCommands::Info { container, id } => handle_object_info(container, id, state),
        ObjectCommands::Delete { container, id } => handle_object_delete(container, id, state),
    }
}

/// Handle container list command
fn handle_container_list(state: &mut crate::commands::wallet::CliState) -> CliResult<CliOutput> {
    // Create a NeoFS client
    let client = create_neofs_client_with_account(state)?;

    // Create a table for the output
    let mut table = Table::new();
    table.add_row(row!["Container ID", "Name", "Created", "Size"]);

    // Attempt to list containers
    match client.containers().list() {
        Ok(containers) => {
            if containers.is_empty() {
                return Ok(CliOutput::new().with_message("No containers found for this account.".into()));
            }

            for container in containers {
                table.add_row(row![
                    container.id().to_hex(),
                    container.name(),
                    format_timestamp(container.created_at()),
                    "Unknown"
                ]);
            }
            Ok(CliOutput::new().with_table(table))
        },
        Err(e) => Err(CliError::Storage(format!("Failed to list containers: {}", e))),
    }
}

/// Handle container create command
fn handle_container_create(
    name: &str,
    basic_acl: &str,
    placement_policy: Option<String>,
    state: &mut crate::commands::wallet::CliState
) -> CliResult<CliOutput> {
    // Create the container
    match create_container(name, basic_acl, placement_policy, state) {
        Ok(container_id) => {
            Ok(CliOutput::new().with_message(format!("Container created successfully. ID: {}", container_id.to_hex())))
        },
        Err(e) => Err(e),
    }
}

/// Handle container info command
fn handle_container_info(
    container_id: &str,
    state: &mut crate::commands::wallet::CliState
) -> CliResult<CliOutput> {
    // Get the container
    match get_container(container_id.to_string(), state) {
        Ok(container) => {
            // Create a table for the output
            let mut table = Table::new();
            table.add_row(row!["Property", "Value"]);
            table.add_row(row!["ID", container.id().to_hex()]);
            table.add_row(row!["Name", container.name()]);
            table.add_row(row!["Owner", container.owner().to_hex()]);
            table.add_row(row!["Created", format_timestamp(container.created_at())]);
            table.add_row(row!["Basic ACL", format!("{:#08x}", container.basic_acl())]);

            Ok(CliOutput::new().with_table(table))
        },
        Err(e) => Err(e),
    }
}

/// Handle container delete command
fn handle_container_delete(
    container_id: &str,
    state: &mut crate::commands::wallet::CliState
) -> CliResult<CliOutput> {
    // Validate the container ID
    let container_id = match ContainerId::from_hex(container_id) {
        Ok(id) => id,
        Err(_) => return Err(CliError::Storage(format!("Invalid container ID: {}", container_id))),
    };

    // Create a NeoFS client
    let client = create_neofs_client_with_account(state)?;

    // Delete the container
    match client.containers().delete(&container_id) {
        Ok(_) => Ok(CliOutput::new().with_message("Container deleted successfully.".into())),
        Err(e) => Err(CliError::Storage(format!("Failed to delete container: {}", e))),
    }
}

/// Get an object from NeoFS
fn get_object(
    container_id: String,
    object_id: String,
    state: &mut crate::commands::wallet::CliState,
) -> Result<(Vec<u8>, Option<neo::neo_fs::object::Object>), CliError> {
    // Validate the container ID
    let container_id = match ContainerId::from_hex(&container_id) {
        Ok(id) => id,
        Err(_) => return Err(CliError::Storage(format!("Invalid container ID: {}", container_id))),
    };

    // Validate the object ID
    let object_id = match ObjectId::from_hex(&object_id) {
        Ok(id) => id,
        Err(_) => return Err(CliError::Storage(format!("Invalid object ID: {}", object_id))),
    };

    // Create a NeoFS client
    let client = create_neofs_client_with_account(state)?;

    // Download the object
    let data = match client.objects().download(&container_id, &object_id) {
        Ok(data) => data,
        Err(e) => return Err(CliError::Storage(format!("Failed to download object: {}", e))),
    };

    // Try to get object info (this may fail but we still have the data)
    let object_info = match client.objects().get_info(&container_id, &object_id) {
        Ok(info) => {
            // Build the object
            Some(neo::neo_fs::object::Object::with_data(
                object_id,
                container_id,
                data.clone(),
            ))
        },
        Err(_) => None,
    };

    Ok((data, object_info))
}

/// Upload an object to NeoFS
fn upload_object(
    container_id: String,
    data: Vec<u8>,
    attributes: Vec<(String, String)>,
    state: &mut crate::commands::wallet::CliState,
) -> Result<neo::neo_fs::types::ObjectId, CliError> {
    // Validate the container ID
    let container_id = match ContainerId::from_hex(&container_id) {
        Ok(id) => id,
        Err(_) => return Err(CliError::Storage(format!("Invalid container ID: {}", container_id))),
    };

    // Create a NeoFS client
    let client = create_neofs_client_with_account(state)?;

    // Upload the object
    match client.objects().upload(&container_id, data, attributes) {
        Ok(id) => Ok(id),
        Err(e) => Err(CliError::Storage(format!("Failed to upload object: {}", e))),
    }
}

/// List objects in a container
fn list_objects(
    container_id: String,
    state: &mut crate::commands::wallet::CliState,
) -> Result<Vec<neo::neo_fs::types::ObjectId>, CliError> {
    // Validate the container ID
    let container_id = match ContainerId::from_hex(&container_id) {
        Ok(id) => id,
        Err(_) => return Err(CliError::Storage(format!("Invalid container ID: {}", container_id))),
    };

    // Create a NeoFS client
    let client = create_neofs_client_with_account(state)?;

    // List objects in the container
    // For now, using an empty filter for simplicity
    match client.objects().search(&container_id, vec![]) {
        Ok(objects) => Ok(objects),
        Err(e) => Err(CliError::Storage(format!("Failed to list objects: {}", e))),
    }
}

/// Delete an object from NeoFS
fn delete_object(
    container_id: String,
    object_id: String,
    state: &mut crate::commands::wallet::CliState,
) -> Result<(), CliError> {
    // Validate the container ID
    let container_id = match ContainerId::from_hex(&container_id) {
        Ok(id) => id,
        Err(_) => return Err(CliError::Storage(format!("Invalid container ID: {}", container_id))),
    };

    // Validate the object ID
    let object_id = match ObjectId::from_hex(&object_id) {
        Ok(id) => id,
        Err(_) => return Err(CliError::Storage(format!("Invalid object ID: {}", object_id))),
    };

    // Create a NeoFS client
    let client = create_neofs_client_with_account(state)?;

    // Delete the object
    match client.objects().delete(&container_id, &object_id) {
        Ok(_) => Ok(()),
        Err(e) => Err(CliError::Storage(format!("Failed to delete object: {}", e))),
    }
}

/// Handle object upload command
fn handle_object_upload(
    container_id: &str,
    file_path: &PathBuf,
    attributes: &Vec<String>,
    state: &mut crate::commands::wallet::CliState
) -> CliResult<CliOutput> {
    // Read the file
    let data = std::fs::read(file_path)
        .map_err(|e| CliError::Storage(format!("Failed to read file: {}", e)))?;

    // Parse the attributes
    let mut parsed_attributes = Vec::new();
    
    // Add filename attribute
    let filename = file_path.file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    
    parsed_attributes.push(("Name".to_string(), filename));
    
    // Add custom attributes
    for attr in attributes {
        if let Some(pos) = attr.find('=') {
            let key = attr[..pos].trim().to_string();
            let value = attr[pos+1..].trim().to_string();
            parsed_attributes.push((key, value));
        }
    }

    // Upload the object
    match upload_object(container_id.to_string(), data, parsed_attributes, state) {
        Ok(object_id) => {
            Ok(CliOutput::new().with_message(format!("File uploaded successfully. Object ID: {}", object_id.to_hex())))
        },
        Err(e) => Err(e),
    }
}

/// Handle object download command
fn handle_object_download(
    container_id: &str,
    object_id: &str,
    output_path: &PathBuf,
    state: &mut crate::commands::wallet::CliState
) -> CliResult<CliOutput> {
    // Download the object
    match get_object(container_id.to_string(), object_id.to_string(), state) {
        Ok((data, _)) => {
            // Write the data to the output file
            std::fs::write(output_path, data)
                .map_err(|e| CliError::Storage(format!("Failed to write file: {}", e)))?;
            
            Ok(CliOutput::new().with_message(format!("Object downloaded successfully to {}", output_path.display())))
        },
        Err(e) => Err(e),
    }
}

/// Handle object list command
fn handle_object_list(
    container_id: &str,
    state: &mut crate::commands::wallet::CliState
) -> CliResult<CliOutput> {
    // List objects
    match list_objects(container_id.to_string(), state) {
        Ok(objects) => {
            if objects.is_empty() {
                return Ok(CliOutput::new().with_message("No objects found in this container.".into()));
            }

            // Create a table for the output
            let mut table = Table::new();
            table.add_row(row!["Object ID"]);

            for object_id in objects {
                table.add_row(row![object_id.to_hex()]);
            }

            Ok(CliOutput::new().with_table(table))
        },
        Err(e) => Err(e),
    }
}

/// Handle object info command
fn handle_object_info(
    container_id: &str,
    object_id: &str,
    state: &mut crate::commands::wallet::CliState
) -> CliResult<CliOutput> {
    // Get the object
    match get_object(container_id.to_string(), object_id.to_string(), state) {
        Ok((data, Some(object))) => {
            // Create a table for the output
            let mut table = Table::new();
            table.add_row(row!["Property", "Value"]);
            table.add_row(row!["ID", object.id().to_hex()]);
            table.add_row(row!["Container ID", object.container_id().to_hex()]);
            table.add_row(row!["Size", format!("{} bytes", data.len())]);
            
            Ok(CliOutput::new().with_table(table))
        },
        Ok((data, None)) => {
            // Limited info since we only have the data
            let mut table = Table::new();
            table.add_row(row!["Property", "Value"]);
            table.add_row(row!["ID", object_id]);
            table.add_row(row!["Container ID", container_id]);
            table.add_row(row!["Size", format!("{} bytes", data.len())]);
            
            Ok(CliOutput::new().with_table(table))
        },
        Err(e) => Err(e),
    }
}

/// Handle object delete command
fn handle_object_delete(
    container_id: &str,
    object_id: &str,
    state: &mut crate::commands::wallet::CliState
) -> CliResult<CliOutput> {
    // Delete the object
    match delete_object(container_id.to_string(), object_id.to_string(), state) {
        Ok(_) => Ok(CliOutput::new().with_message("Object deleted successfully.".into())),
        Err(e) => Err(e),
    }
}

/// Create a NeoFS client with default configuration
fn create_neofs_client(state: &mut crate::commands::wallet::CliState) -> Result<NeoFsClient, CliError> {
    // Create a default NeoFS configuration using endpoint from state if available
    let endpoint = "http://localhost:8080".to_string();
    let config = NeoFsConfig::new(&endpoint);

    // Create the client without an account
    match NeoFsClient::new(config) {
        Ok(client) => Ok(client),
        Err(e) => Err(CliError::Storage(format!("Failed to create NeoFS client: {}", e))),
    }
}

/// Create a NeoFS client with an account from the state
fn create_neofs_client_with_account(state: &mut crate::commands::wallet::CliState) -> Result<NeoFsClient, CliError> {
    // Get the account from the state
    let account = match &state.wallet {
        Some(wallet) => {
            match wallet.get_default_account() {
                Some(account) => account.clone(),
                None => return Err(CliError::Storage("No default account in the wallet".to_string())),
            }
        },
        None => return Err(CliError::Storage("No wallet loaded".to_string())),
    };

    // Create a default NeoFS configuration
    let endpoint = "http://localhost:8080".to_string();
    let config = NeoFsConfig::new(&endpoint);

    // Create the client with the account
    match NeoFsClient::with_account(config, account) {
        Ok(client) => Ok(client),
        Err(e) => Err(CliError::Storage(format!("Failed to create NeoFS client: {}", e))),
    }
}

/// Format bytes to human-readable size
fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes < 1024 * 1024 * 1024 * 1024 {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else {
        format!("{:.2} TB", bytes as f64 / (1024.0 * 1024.0 * 1024.0 * 1024.0))
    }
}

/// Format a timestamp to a readable date string
fn format_timestamp(timestamp: u64) -> String {
    let datetime = chrono::NaiveDateTime::from_timestamp_opt(timestamp as i64, 0)
        .unwrap_or_else(|| chrono::NaiveDateTime::from_timestamp_opt(0, 0).unwrap());
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Create a container with the specified parameters
fn create_container(
    name: &str,
    basic_acl: &str,
    placement_policy: Option<String>,
    state: &mut crate::commands::wallet::CliState,
) -> Result<neo::neo_fs::types::ContainerId, CliError> {
    // Parse the basic ACL
    let acl = match basic_acl.parse::<u32>() {
        Ok(val) => val,
        Err(_) => return Err(CliError::Storage(format!("Invalid basic ACL value: {}", basic_acl))),
    };

    // Set up container rules
    let mut rules = vec![];
    
    // Add placement policy if specified
    let placement = placement_policy.unwrap_or_else(|| "REP 3 IN X CBF 1 SELECT 2 FROM * AS X".to_string());
    
    let policy = match StoragePolicy::from_string(&placement) {
        Ok(p) => p,
        Err(e) => return Err(CliError::Storage(format!("Invalid placement policy: {}", e))),
    };

    // Add default access rule for demonstration
    let access_rule = AccessRule::new_bearer_token();
    rules.push(access_rule);

    // Create a NeoFS client with an account
    let client = create_neofs_client_with_account(state)?;

    // Create the container
    match client.containers().create(name, acl, rules, policy) {
        Ok(id) => Ok(id),
        Err(e) => Err(CliError::Storage(format!("Failed to create container: {}", e))),
    }
}

/// Get a container by ID
fn get_container(
    container_id: String,
    state: &mut crate::commands::wallet::CliState,
) -> Result<neo::neo_fs::container::Container, CliError> {
    // Validate the container ID
    let container_id = match ContainerId::from_hex(&container_id) {
        Ok(id) => id,
        Err(_) => return Err(CliError::Storage(format!("Invalid container ID: {}", container_id))),
    };

    // Create a NeoFS client
    let client = create_neofs_client_with_account(state)?;

    // Get the container
    match client.containers().get(&container_id) {
        Ok(container) => Ok(container),
        Err(e) => Err(CliError::Storage(format!("Failed to get container: {}", e))),
    }
} 