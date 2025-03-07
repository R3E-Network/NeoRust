// NeoFS integration module
//
// This module provides integration between the CLI commands and the NeoFS
// implementation, handling argument parsing and command routing.

use crate::{
    cli::CliState, 
    commands::defi_modules::{
        types_neofs::{NeoFSArgs, NeoFSCommands},
        neo_fs,
    },
    error::CliError,
};

/// Handle NeoFS commands with clap integration
///
/// # Arguments
/// * `args` - NeoFSArgs parsed by clap
/// * `state` - Current CLI state
///
/// # Returns
/// * `Result<(), CliError>` - Success or error
pub async fn handle_neofs_command(
    args: NeoFSArgs,
    state: &mut CliState,
) -> Result<(), CliError> {
    match args.command {
        NeoFSCommands::ListContainers => {
            neo_fs::handle_neofs_command("list-containers", &[], state).await
        },
        NeoFSCommands::CreateContainer { name, description } => {
            let desc = description.unwrap_or_default();
            neo_fs::handle_neofs_command("create-container", &[name, desc], state).await
        },
        NeoFSCommands::Upload { container_id, local_path, object_name } => {
            neo_fs::handle_neofs_command(
                "upload", 
                &[container_id, local_path, object_name], 
                state
            ).await
        },
        NeoFSCommands::Download { container_id, object_id, local_path } => {
            neo_fs::handle_neofs_command(
                "download", 
                &[container_id, object_id, local_path], 
                state
            ).await
        },
        NeoFSCommands::ListObjects { container_id } => {
            neo_fs::handle_neofs_command(
                "list-objects", 
                &[container_id], 
                state
            ).await
        },
        NeoFSCommands::DeleteObject { container_id, object_id } => {
            neo_fs::handle_neofs_command(
                "delete-object", 
                &[container_id, object_id], 
                state
            ).await
        },
        NeoFSCommands::Info => {
            neo_fs::handle_neofs_command("info", &[], state).await
        },
    }
}

/// Example function demonstrating NeoFS usage across different networks
///
/// This function shows how to use NeoFS with both Neo N3 and Neo X networks,
/// automatically selecting the correct endpoints based on network type.
pub async fn neofs_network_example(state: &mut CliState) -> Result<(), CliError> {
    // Show NeoFS info for the current network (which could be Neo N3 or Neo X)
    println!("NeoFS information for current network:");
    neo_fs::show_neofs_info(state).await?;
    
    // You can list containers on any network
    if let Some(network_type) = &state.network_type {
        println!("\nListing containers on {}:", network_type);
        
        // This will automatically use the appropriate NeoFS endpoint
        // based on whether you're connected to Neo N3 or Neo X
        neo_fs::list_containers(state).await?;
    }
    
    // NeoFS operations automatically handle network differences
    // The same code works for both Neo N3 and Neo X
    println!("\nAll NeoFS operations support both Neo N3 and Neo X networks.");
    println!("The SDK automatically selects the correct endpoints based on network type.");
    
    Ok(())
}
