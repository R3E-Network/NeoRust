// NeoFS commands for the Neo CLI
//
// This module provides commands for interacting with NeoFS storage
// on both Neo N3 and Neo X networks with a consistent interface.

use std::io::Read;
use std::path::Path;
use std::fs::File;

use neo3::{
    neo_types::h160::H160,
    neo_protocol::account::Account,
};
use neo_fs::{
    NeoFSClient, NeoFSConfig, NeoFSService, Container, ContainerId, Object, ObjectId,
    DEFAULT_MAINNET_ENDPOINT, DEFAULT_TESTNET_ENDPOINT,
    DEFAULT_MAINNET_HTTP_GATEWAY, DEFAULT_TESTNET_HTTP_GATEWAY,
};

use crate::{
    cli::CliState,
    commands::defi::network_validator::NetworkTypeCli,
    error::CliError,
};

// Import helper utilities
use super::utils::{
    print_success, print_info, print_error, 
    prompt_yes_no, prompt_password, ensure_account_loaded,
    network_type_from_state,
};

/// Get the appropriate NeoFS endpoint based on network type
fn get_neofs_endpoint(network_type: NetworkTypeCli) -> &'static str {
    match network_type {
        NetworkTypeCli::NeoN3MainNet | NetworkTypeCli::NeoXMainNet => DEFAULT_MAINNET_ENDPOINT,
        NetworkTypeCli::NeoN3TestNet | NetworkTypeCli::NeoXTestNet => DEFAULT_TESTNET_ENDPOINT,
    }
}

/// Get the appropriate NeoFS HTTP gateway based on network type
fn get_neofs_http_gateway(network_type: NetworkTypeCli) -> &'static str {
    match network_type {
        NetworkTypeCli::NeoN3MainNet | NetworkTypeCli::NeoXMainNet => DEFAULT_MAINNET_HTTP_GATEWAY,
        NetworkTypeCli::NeoN3TestNet | NetworkTypeCli::NeoXTestNet => DEFAULT_TESTNET_HTTP_GATEWAY,
    }
}

/// Create a NeoFS client configured for the current network
fn create_neofs_client(state: &CliState) -> NeoFSClient {
    let network_type = network_type_from_state(state);
    let endpoint = get_neofs_endpoint(network_type);
    
    let config = NeoFSConfig {
        endpoint: endpoint.to_string(),
        ..Default::default()
    };
    
    NeoFSClient::new(config)
}

/// Handle NeoFS commands
///
/// This function serves as the main entry point for NeoFS operations
/// in the modular DeFi architecture.
///
/// # Arguments
/// * `cmd` - NeoFS subcommand to execute
/// * `args` - Command arguments
/// * `state` - CLI state containing wallet and RPC client
///
/// # Returns
/// * `Result<(), CliError>` - Success or error
pub async fn handle_neofs_command(
    cmd: &str,
    args: &[String],
    state: &mut CliState,
) -> Result<(), CliError> {
    match cmd {
        "list-containers" | "ls" => {
            list_containers(state).await
        },
        "create-container" | "create" => {
            if args.len() < 1 {
                return Err(CliError::MissingArgument(
                    "container name required".to_string(),
                ));
            }
            let description = if args.len() > 1 { &args[1] } else { "" };
            create_container(&args[0], description, state).await
        },
        "upload" | "put" => {
            if args.len() < 3 {
                return Err(CliError::MissingArgument(
                    "container-id, local file path, and object name required".to_string(),
                ));
            }
            upload_file(&args[0], &args[1], &args[2], state).await
        },
        "download" | "get" => {
            if args.len() < 3 {
                return Err(CliError::MissingArgument(
                    "container-id, object-id, and local file path required".to_string(),
                ));
            }
            download_file(&args[0], &args[1], &args[2], state).await
        },
        "list-objects" | "ls-objects" => {
            if args.is_empty() {
                return Err(CliError::MissingArgument(
                    "container-id required".to_string(),
                ));
            }
            list_objects(&args[0], state).await
        },
        "delete-object" | "rm" => {
            if args.len() < 2 {
                return Err(CliError::MissingArgument(
                    "container-id and object-id required".to_string(),
                ));
            }
            delete_object(&args[0], &args[1], state).await
        },
        "info" => {
            show_neofs_info(state).await
        },
        _ => Err(CliError::InvalidCommand(format!("Unknown NeoFS command: {}", cmd))),
    }
}

/// List all containers in NeoFS
///
/// # Arguments
/// * `state` - CLI state
///
/// # Returns
/// * `Result<(), CliError>` - Success or error
pub async fn list_containers(state: &CliState) -> Result<(), CliError> {
    // Ensure account is loaded
    let account = ensure_account_loaded(state)?;
    
    print_info("Connecting to NeoFS...");
    
    let client = create_neofs_client(state);
    let client = client.with_account(account.clone());
    
    print_info("Listing containers...");
    
    let result = client.list_containers()
        .map_err(|e| CliError::NeoFS(format!("Failed to list containers: {}", e)))?;
    
    if result.is_empty() {
        print_info("No containers found.");
    } else {
        print_success(&format!("Found {} containers:", result.len()));
        for (i, container_id) in result.iter().enumerate() {
            print_info(&format!("{}. {}", i + 1, container_id));
        }
    }
    
    Ok(())
}

/// Create a new container in NeoFS
///
/// # Arguments
/// * `name` - Container name
/// * `description` - Container description
/// * `state` - CLI state
///
/// # Returns
/// * `Result<(), CliError>` - Success or error
pub async fn create_container(
    name: &str,
    description: &str,
    state: &mut CliState,
) -> Result<(), CliError> {
    // Ensure account is loaded
    let account = ensure_account_loaded(state)?;
    
    // Get password and decrypt account
    let password = prompt_password("Enter wallet password: ")?;
    let decrypted_account = account.decrypt(&password)
        .map_err(|_| CliError::Wallet("Failed to decrypt account".to_string()))?;
    
    print_info("Connecting to NeoFS...");
    
    let client = create_neofs_client(state);
    let client = client.with_account(decrypted_account);
    
    // Create a new container
    let container = Container::new(name.to_string(), description.to_string());
    
    print_info(&format!("Creating container '{}' with description '{}'...", name, description));
    
    let container_id = client.create_container(&container)
        .map_err(|e| CliError::NeoFS(format!("Failed to create container: {}", e)))?;
    
    print_success(&format!("Container created successfully with ID: {}", container_id));
    
    Ok(())
}

/// Upload a file to NeoFS
///
/// # Arguments
/// * `container_id` - Container ID
/// * `local_path` - Path to local file
/// * `object_name` - Name to give the object in NeoFS
/// * `state` - CLI state
///
/// # Returns
/// * `Result<(), CliError>` - Success or error
pub async fn upload_file(
    container_id: &str,
    local_path: &str,
    object_name: &str,
    state: &mut CliState,
) -> Result<(), CliError> {
    // Ensure account is loaded
    let account = ensure_account_loaded(state)?;
    
    // Validate that the file exists
    let path = Path::new(local_path);
    if !path.exists() || !path.is_file() {
        return Err(CliError::NeoFS(format!("File not found: {}", local_path)));
    }
    
    // Get password and decrypt account
    let password = prompt_password("Enter wallet password: ")?;
    let decrypted_account = account.decrypt(&password)
        .map_err(|_| CliError::Wallet("Failed to decrypt account".to_string()))?;
    
    print_info("Connecting to NeoFS...");
    
    let client = create_neofs_client(state);
    let client = client.with_account(decrypted_account);
    
    // Parse container ID
    let container_id = ContainerId::from_string(container_id)
        .map_err(|_| CliError::NeoFS(format!("Invalid container ID: {}", container_id)))?;
    
    // Read file content
    let mut file = File::open(path)
        .map_err(|e| CliError::Io(format!("Failed to open file: {}", e)))?;
    
    let mut data = Vec::new();
    file.read_to_end(&mut data)
        .map_err(|e| CliError::Io(format!("Failed to read file: {}", e)))?;
    
    print_info(&format!("Uploading '{}' to container '{}'...", local_path, container_id));
    
    // Create object
    let mut object = Object::new();
    object.set_name(object_name.to_string());
    object.set_data(data);
    
    // Upload object
    let object_id = client.put_object(&container_id, &object)
        .map_err(|e| CliError::NeoFS(format!("Failed to upload file: {}", e)))?;
    
    print_success(&format!(
        "File uploaded successfully!\nContainer ID: {}\nObject ID: {}",
        container_id, object_id
    ));
    
    Ok(())
}

/// Download a file from NeoFS
///
/// # Arguments
/// * `container_id` - Container ID
/// * `object_id` - Object ID
/// * `local_path` - Path to save the file
/// * `state` - CLI state
///
/// # Returns
/// * `Result<(), CliError>` - Success or error
pub async fn download_file(
    container_id: &str,
    object_id: &str,
    local_path: &str,
    state: &mut CliState,
) -> Result<(), CliError> {
    // Ensure account is loaded
    let account = ensure_account_loaded(state)?;
    
    print_info("Connecting to NeoFS...");
    
    let client = create_neofs_client(state);
    let client = client.with_account(account.clone());
    
    // Parse container ID and object ID
    let container_id = ContainerId::from_string(container_id)
        .map_err(|_| CliError::NeoFS(format!("Invalid container ID: {}", container_id)))?;
    
    let object_id = ObjectId::from_string(object_id)
        .map_err(|_| CliError::NeoFS(format!("Invalid object ID: {}", object_id)))?;
    
    print_info(&format!(
        "Downloading object '{}' from container '{}'...",
        object_id, container_id
    ));
    
    // Get object
    let object = client.get_object(&container_id, &object_id)
        .map_err(|e| CliError::NeoFS(format!("Failed to download file: {}", e)))?;
    
    // Save to file
    std::fs::write(local_path, object.data())
        .map_err(|e| CliError::Io(format!("Failed to write file: {}", e)))?;
    
    print_success(&format!("File downloaded successfully to '{}'", local_path));
    
    Ok(())
}

/// List objects in a container
///
/// # Arguments
/// * `container_id` - Container ID
/// * `state` - CLI state
///
/// # Returns
/// * `Result<(), CliError>` - Success or error
pub async fn list_objects(
    container_id: &str,
    state: &CliState,
) -> Result<(), CliError> {
    // Ensure account is loaded
    let account = ensure_account_loaded(state)?;
    
    print_info("Connecting to NeoFS...");
    
    let client = create_neofs_client(state);
    let client = client.with_account(account.clone());
    
    // Parse container ID
    let container_id = ContainerId::from_string(container_id)
        .map_err(|_| CliError::NeoFS(format!("Invalid container ID: {}", container_id)))?;
    
    print_info(&format!("Listing objects in container '{}'...", container_id));
    
    let result = client.list_objects(&container_id)
        .map_err(|e| CliError::NeoFS(format!("Failed to list objects: {}", e)))?;
    
    if result.is_empty() {
        print_info("No objects found in the container.");
    } else {
        print_success(&format!("Found {} objects:", result.len()));
        for (i, object_id) in result.iter().enumerate() {
            print_info(&format!("{}. {}", i + 1, object_id));
        }
    }
    
    Ok(())
}

/// Delete an object from NeoFS
///
/// # Arguments
/// * `container_id` - Container ID
/// * `object_id` - Object ID
/// * `state` - CLI state
///
/// # Returns
/// * `Result<(), CliError>` - Success or error
pub async fn delete_object(
    container_id: &str,
    object_id: &str,
    state: &mut CliState,
) -> Result<(), CliError> {
    // Ensure account is loaded
    let account = ensure_account_loaded(state)?;
    
    // Get password and decrypt account
    let password = prompt_password("Enter wallet password: ")?;
    let decrypted_account = account.decrypt(&password)
        .map_err(|_| CliError::Wallet("Failed to decrypt account".to_string()))?;
    
    print_info("Connecting to NeoFS...");
    
    let client = create_neofs_client(state);
    let client = client.with_account(decrypted_account);
    
    // Parse container ID and object ID
    let container_id = ContainerId::from_string(container_id)
        .map_err(|_| CliError::NeoFS(format!("Invalid container ID: {}", container_id)))?;
    
    let object_id = ObjectId::from_string(object_id)
        .map_err(|_| CliError::NeoFS(format!("Invalid object ID: {}", object_id)))?;
    
    // Confirm deletion
    if !prompt_yes_no(&format!("Are you sure you want to delete object '{}' from container '{}'?", object_id, container_id)) {
        print_info("Object deletion cancelled.");
        return Ok(());
    }
    
    print_info(&format!(
        "Deleting object '{}' from container '{}'...",
        object_id, container_id
    ));
    
    // Delete object
    let result = client.delete_object(&container_id, &object_id)
        .map_err(|e| CliError::NeoFS(format!("Failed to delete object: {}", e)))?;
    
    if result {
        print_success("Object deleted successfully.");
    } else {
        print_error("Object deletion failed or object not found.");
    }
    
    Ok(())
}

/// Show NeoFS information for current network
///
/// # Arguments
/// * `state` - CLI state
///
/// # Returns
/// * `Result<(), CliError>` - Success or error
pub async fn show_neofs_info(state: &CliState) -> Result<(), CliError> {
    let network_type = network_type_from_state(state);
    let endpoint = get_neofs_endpoint(network_type);
    let http_gateway = get_neofs_http_gateway(network_type);
    
    print_info("\n==== NeoFS Information ====");
    
    // Display current network
    if network_type.is_neo_n3() {
        print_info("Current Network: Neo N3");
    } else if network_type.is_neox() {
        print_info("Current Network: Neo X");
    }
    
    if network_type.is_testnet() {
        print_info("Network: TestNet");
    } else {
        print_info("Network: MainNet");
    }
    
    // Display endpoint info
    print_info("\nNeoFS Endpoints:");
    print_info(&format!("  gRPC Endpoint: {}", endpoint));
    print_info(&format!("  HTTP Gateway: {}", http_gateway));
    
    // Display capabilities
    print_info("\nCapabilities:");
    print_info("  - Container management");
    print_info("  - Object storage and retrieval");
    print_info("  - Access control");
    print_info("  - Multipart uploads");
    
    // Display usage examples
    print_info("\nCommand Examples:");
    print_info("  neofs list-containers");
    print_info("  neofs create-container my-container \"My container description\"");
    print_info("  neofs upload container-id ./local-file.txt my-object");
    print_info("  neofs download container-id object-id ./downloaded-file.txt");
    
    Ok(())
}
