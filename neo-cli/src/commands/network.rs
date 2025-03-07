use crate::{
	errors::CliError,
	utils::{
		config::{load_config, save_config, NetworkConfig},
		print_error, print_info, print_success,
	},
};
use clap::{Args, Subcommand};

/// Arguments for the network command
#[derive(Args, Debug)]
pub struct NetworkArgs {
	#[clap(subcommand)]
	pub command: NetworkCommands,
}

/// Network-related commands
#[derive(Subcommand, Debug)]
pub enum NetworkCommands {
	/// Connect to a Neo N3 node
	#[clap(name = "connect")]
	Connect {
		/// The address of the node (e.g. localhost:10332)
		address: String,

		/// A name for the connection
		#[clap(long, short)]
		name: Option<String>,
	},

	/// List connected nodes
	#[clap(name = "list")]
	List,
}

/// Define a state struct that can be used for keeping track of CLI state
pub struct CliState {
	/// The current network type
	pub network_type: Option<String>,
	/// Whether we're connected to a node
	pub connected: bool,
	/// The current connection details
	pub current_connection: Option<String>,
}

impl Default for CliState {
	fn default() -> Self {
		Self {
			network_type: Some("MainNet".to_string()),
			connected: false,
			current_connection: None,
		}
	}
}

impl CliState {
	/// Get the network type string
	pub fn get_network_type_string(&self) -> String {
		self.network_type.clone().unwrap_or_else(|| "MainNet".to_string())
	}

	/// Check if the current network is a Neo X network
	pub fn is_neo_x(&self) -> bool {
		let network_str = self.get_network_type_string().to_lowercase();
		network_str.contains("neox") || network_str.contains("neo-x")
	}

	/// Check if the current network is a testnet
	pub fn is_testnet(&self) -> bool {
		let network_str = self.get_network_type_string().to_lowercase();
		network_str.contains("test")
	}

	/// Check if the current network is a mainnet
	pub fn is_mainnet(&self) -> bool {
		!self.is_testnet()
	}
}

/// Handle the network command
pub async fn handle_network_command(
	args: NetworkArgs,
	state: &mut CliState,
) -> Result<(), CliError> {
	match args.command {
		NetworkCommands::Connect { address, name } => connect_to_node(&address, name, state).await,
		NetworkCommands::List => list_nodes(state).await,
	}
}

async fn connect_to_node(
	address: &str,
	name: Option<String>,
	state: &mut CliState,
) -> Result<(), CliError> {
	// Parse the address to get the IP and port
	let parts: Vec<&str> = address.split(':').collect();
	if parts.len() != 2 {
		return Err(CliError::InvalidArgument(
			"address".to_string(),
			"Address must be in format 'ip:port'".to_string(),
		));
	}

	let ip = parts[0];
	let port = parts[1].parse::<u16>().map_err(|_| {
		CliError::InvalidArgument("port".to_string(), "Port must be a valid number".to_string())
	})?;

	// Determine the network name
	let network_name = match name {
		Some(n) => n,
		None => format!("{}:{}", ip, port),
	};

	// Load the config
	let mut config = load_config()?;

	// Check if the network name already exists
	let network_exists = config.networks.iter().any(|n| n.name == network_name);

	// Add the network to the config if it doesn't exist
	let rpc_url = format!("http://{}:{}", ip, port);
	if !network_exists {
		config
			.networks
			.push(NetworkConfig { name: network_name.clone(), rpc_url: rpc_url.clone() });
	} else {
		// Update the existing network
		for network in &mut config.networks {
			if network.name == network_name {
				network.rpc_url = rpc_url.clone();
				break;
			}
		}
	}

	// Save the config
	save_config(&config)?;

	// Simulate connecting to the node
	print_info(&format!("Connecting to {}", address));

	// For now, just assume we connected successfully
	state.connected = true;
	state.current_connection = Some(address.to_string());

	print_success(&format!("Connected to {}", address));
	Ok(())
}

async fn list_nodes(state: &CliState) -> Result<(), CliError> {
	// Load the config to show all saved networks
	let config = load_config()?;

	if config.networks.is_empty() {
		print_info("No networks configured.");
		return Ok(());
	}

	print_info("Configured networks:");
	for network in &config.networks {
		let current = match &state.current_connection {
			Some(conn) if network.rpc_url.contains(conn) => " (current)",
			_ => "",
		};
		print_info(&format!("- {} ({}){}", network.name, network.rpc_url, current));
	}

	Ok(())
}
