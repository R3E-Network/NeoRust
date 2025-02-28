use clap::{Parser, Subcommand};
// use neo3::prelude::*;
use std::path::PathBuf;
use tokio;

use crate::commands::wallet::{WalletArgs, CliState, handle_wallet_command};
use crate::commands::blockchain::{BlockchainArgs, handle_blockchain_command};
use crate::commands::network::{NetworkArgs, handle_network_command};
use crate::commands::contract::{ContractArgs, handle_contract_command};
use crate::config::CliConfig;
use crate::utils::{print_success, print_error};
use crate::utils::error::CliResult;

mod commands;
mod config;
mod utils;

#[derive(Parser)]
#[command(author = "R3E Network", version, about = "Neo Blockchain CLI", long_about = None)]
struct Cli {
    /// Path to config file
    #[arg(short, long)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Wallet management commands
    Wallet(WalletArgs),
    
    /// Blockchain commands
    Blockchain(BlockchainArgs),
    
    /// Network commands
    Network(NetworkArgs),
    
    /// Contract commands
    Contract(ContractArgs),
    
    /// Initialize a new configuration file
    Init {
        /// Path to save the configuration file
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> CliResult<()> {
    // Initialize logger
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Load configuration
    let _config = match &cli.config {
        Some(path) => {
            if !path.exists() {
                print_error(&format!("Config file not found: {:?}", path));
                return Err(utils::error::CliError::Config(format!("Config file not found: {:?}", path)));
            }
            
            let config_str = std::fs::read_to_string(path)
                .map_err(|e| utils::error::CliError::Io(e))?;
            
            serde_json::from_str(&config_str)
                .map_err(|e| utils::error::CliError::Config(format!("Failed to parse config file: {}", e)))?
        },
        None => CliConfig::load()?,
    };
    
    // Initialize CLI state
    let mut state = CliState::default();
    
    // Handle commands
    match cli.command {
        Commands::Wallet(args) => {
            handle_wallet_command(args, &mut state).await?;
        },
        Commands::Blockchain(args) => {
            handle_blockchain_command(args, &mut state).await?;
        },
        Commands::Network(args) => {
            handle_network_command(args, &mut state).await?;
        },
        Commands::Contract(args) => {
            handle_contract_command(args, &mut state).await?;
        },
        Commands::Init { path } => {
            let config_path = path.unwrap_or_else(|| {
                dirs::config_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("neo-cli/config.json")
            });
            
            let config = CliConfig::default();
            let config_str = serde_json::to_string_pretty(&config)
                .map_err(|e| utils::error::CliError::Config(format!("Failed to serialize config: {}", e)))?;
            
            std::fs::create_dir_all(config_path.parent().unwrap())
                .map_err(|e| utils::error::CliError::Io(e))?;
            
            std::fs::write(&config_path, config_str)
                .map_err(|e| utils::error::CliError::Io(e))?;
            
            print_success(&format!("Configuration initialized at: {:?}", config_path));
        },
    }
    
    Ok(())
}
