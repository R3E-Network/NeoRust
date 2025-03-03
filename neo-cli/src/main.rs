use clap::{Parser, Subcommand};
use commands::{
	neofs::{handle_neofs_command, NeoFSArgs},
	network::{handle_network_command, CliState, NetworkArgs},
	defi::{handle_defi_command, DefiArgs},
};
use errors::CliError;
use std::path::PathBuf;
use tokio;
use utils::{
	config::{get_config_path, save_config, Config},
	print_success,
};

mod commands;
mod config;
mod errors;
mod utils;

/// Neo CLI is a command-line application for interacting with the Neo blockchain
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
	/// Path to config file
	#[arg(short, long)]
	config: Option<PathBuf>,

	#[command(subcommand)]
	command: Commands,
}

/// Available commands
#[derive(Subcommand, Debug)]
enum Commands {
	/// Initialize a new configuration file
	Init {
		/// Path to save the configuration file
		#[arg(short, long)]
		path: Option<PathBuf>,
	},

	/// Network commands
	Network(NetworkArgs),

	/// File storage commands
	Files {
		/// NeoFS endpoint URL
		#[arg(short, long)]
		endpoint: Option<String>,
	},

	/// Basic storage command
	Storage,

	/// NeoFS commands for file storage on the Neo blockchain
	NeoFS(NeoFSArgs),
	
	/// DeFi commands for interacting with Neo DeFi protocols
	DeFi(DefiArgs),
}

/// Initialize a new configuration file
async fn handle_init_command(path: Option<PathBuf>) -> Result<(), CliError> {
	// Create default config
	let config = Config::default();

	if let Some(custom_path) = path {
		// Create parent directories if they don't exist
		if let Some(parent) = custom_path.parent() {
			std::fs::create_dir_all(parent).map_err(|e| CliError::FileSystem(e.to_string()))?;
		}

		// Save config to custom path
		let config_str = serde_json::to_string_pretty(&config)
			.map_err(|e| CliError::Config(format!("Failed to serialize config: {}", e)))?;
		std::fs::write(&custom_path, config_str).map_err(|e| CliError::Io(e))?;

		print_success(&format!("Configuration file initialized at {}", custom_path.display()));
	} else {
		// Save to default location
		save_config(&config)?;
		let config_path = get_config_path()?;
		print_success(&format!("Configuration file initialized at {}", config_path.display()));
	}

	Ok(())
}

#[tokio::main]
async fn main() -> Result<(), CliError> {
	// Initialize logger
	env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

	// Parse command line arguments
	let cli = Cli::parse();

	// Initialize CLI state
	let mut state = CliState::default();

	// Handle commands
	match cli.command {
		Commands::Init { path } => {
			handle_init_command(path).await?;
			Ok(())
		},
		Commands::Network(args) => handle_network_command(args, &mut state).await,
		Commands::Files { endpoint } => {
			println!("Files command selected with endpoint: {:?}", endpoint);
			Ok(())
		},
		Commands::Storage => {
			println!("Storage command selected");
			Ok(())
		},
		Commands::NeoFS(args) => handle_neofs_command(args, &mut state).await,
		Commands::DeFi(args) => handle_defi_command(args, &mut state).await,
	}
}
