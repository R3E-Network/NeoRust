// NeoFS type definitions
//
// This module defines the command line argument types and structures
// specifically for NeoFS operations.

use clap::{Args, Subcommand};
use std::fmt;

/// Arguments for the NeoFS command
#[derive(Args)]
pub struct NeoFSArgs {
    /// NeoFS subcommand to execute
    #[clap(subcommand)]
    pub command: NeoFSCommands,
}

/// Available NeoFS subcommands
#[derive(Subcommand)]
pub enum NeoFSCommands {
    /// List containers owned by the current account
    #[clap(name = "list-containers", alias = "ls")]
    ListContainers,
    
    /// Create a new container
    #[clap(name = "create-container", alias = "create")]
    CreateContainer {
        /// Container name
        #[clap(value_name = "NAME")]
        name: String,
        
        /// Container description (optional)
        #[clap(value_name = "DESCRIPTION")]
        description: Option<String>,
    },
    
    /// Upload a file to NeoFS
    #[clap(name = "upload", alias = "put")]
    Upload {
        /// Container ID
        #[clap(value_name = "CONTAINER_ID")]
        container_id: String,
        
        /// Local file path
        #[clap(value_name = "LOCAL_PATH")]
        local_path: String,
        
        /// Object name in NeoFS
        #[clap(value_name = "OBJECT_NAME")]
        object_name: String,
    },
    
    /// Download a file from NeoFS
    #[clap(name = "download", alias = "get")]
    Download {
        /// Container ID
        #[clap(value_name = "CONTAINER_ID")]
        container_id: String,
        
        /// Object ID
        #[clap(value_name = "OBJECT_ID")]
        object_id: String,
        
        /// Local file path to save the object
        #[clap(value_name = "LOCAL_PATH")]
        local_path: String,
    },
    
    /// List objects in a container
    #[clap(name = "list-objects", alias = "ls-objects")]
    ListObjects {
        /// Container ID
        #[clap(value_name = "CONTAINER_ID")]
        container_id: String,
    },
    
    /// Delete an object from NeoFS
    #[clap(name = "delete-object", alias = "rm")]
    DeleteObject {
        /// Container ID
        #[clap(value_name = "CONTAINER_ID")]
        container_id: String,
        
        /// Object ID
        #[clap(value_name = "OBJECT_ID")]
        object_id: String,
    },
    
    /// Show NeoFS information and endpoints for the current network
    #[clap(name = "info")]
    Info,
}

/// Helper for converting NeoFS command enum variants to strings for display
impl fmt::Display for NeoFSCommands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NeoFSCommands::ListContainers => write!(f, "list-containers"),
            NeoFSCommands::CreateContainer { .. } => write!(f, "create-container"),
            NeoFSCommands::Upload { .. } => write!(f, "upload"),
            NeoFSCommands::Download { .. } => write!(f, "download"),
            NeoFSCommands::ListObjects { .. } => write!(f, "list-objects"),
            NeoFSCommands::DeleteObject { .. } => write!(f, "delete-object"),
            NeoFSCommands::Info => write!(f, "info"),
        }
    }
}
