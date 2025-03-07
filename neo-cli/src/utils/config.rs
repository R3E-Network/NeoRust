use std::{env, fs, path::PathBuf};

use crate::errors::CliError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
	pub networks: Vec<NetworkConfig>,
	pub default_network: String,
	pub wallet_path: Option<String>,
	#[serde(default)]
	pub auto_connect: bool,
	#[serde(default)]
	pub neofs: NeoFSConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkConfig {
	pub name: String,
	pub rpc_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct NeoFSConfig {
	pub endpoints: Vec<NeoFSEndpoint>,
	pub default_endpoint: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NeoFSEndpoint {
	pub name: String,
	pub url: String,
	pub network: String,       // mainnet, testnet, private
	pub endpoint_type: String, // grpc, http, rest
}

impl Default for Config {
	fn default() -> Self {
		Config {
			networks: vec![
				NetworkConfig {
					name: "mainnet".to_string(),
					rpc_url: "https://seed1.neo.org:10331".to_string(),
				},
				NetworkConfig {
					name: "testnet".to_string(),
					rpc_url: "https://testnet1.neo.org:443".to_string(),
				},
				NetworkConfig {
					name: "private-net".to_string(),
					rpc_url: "http://localhost:50012".to_string(),
				},
				NetworkConfig {
					name: "neox-mainnet".to_string(),
					rpc_url: "https://rpc.neo-x.org".to_string(),
				},
				NetworkConfig {
					name: "neox-testnet".to_string(),
					rpc_url: "https://testnet.rpc.neo-x.org".to_string(),
				},
			],
			default_network: "mainnet".to_string(),
			wallet_path: None,
			auto_connect: false,
			neofs: NeoFSConfig {
				endpoints: vec![
					NeoFSEndpoint {
						name: "mainnet-grpc".to_string(),
						url: "https://grpc.fs.neo.org".to_string(),
						network: "mainnet".to_string(),
						endpoint_type: "grpc".to_string(),
					},
					NeoFSEndpoint {
						name: "testnet-grpc".to_string(),
						url: "https://grpc.testnet.fs.neo.org".to_string(),
						network: "testnet".to_string(),
						endpoint_type: "grpc".to_string(),
					},
					NeoFSEndpoint {
						name: "mainnet-http".to_string(),
						url: "https://http.fs.neo.org".to_string(),
						network: "mainnet".to_string(),
						endpoint_type: "http".to_string(),
					},
					NeoFSEndpoint {
						name: "testnet-http".to_string(),
						url: "https://http.testnet.fs.neo.org".to_string(),
						network: "testnet".to_string(),
						endpoint_type: "http".to_string(),
					},
				],
				default_endpoint: Some("mainnet-grpc".to_string()),
			},
		}
	}
}

pub fn get_config_dir() -> Result<PathBuf, CliError> {
	let home = env::var("HOME")
		.map_err(|_| CliError::Config("HOME environment variable not set".to_string()))?;
	let config_dir = PathBuf::from(home).join(".neo-cli");
	fs::create_dir_all(&config_dir).map_err(|e| CliError::Io(e))?;
	Ok(config_dir)
}

pub fn get_config_path() -> Result<PathBuf, CliError> {
	Ok(get_config_dir()?.join("config.json"))
}

pub fn load_config() -> Result<Config, CliError> {
	let config_path = get_config_path()?;
	if config_path.exists() {
		let config_str = fs::read_to_string(config_path).map_err(|e| CliError::Io(e))?;
		serde_json::from_str(&config_str)
			.map_err(|e| CliError::Config(format!("Failed to parse config file: {}", e)))
	} else {
		let config = Config::default();
		save_config(&config)?;
		Ok(config)
	}
}

pub fn save_config(config: &Config) -> Result<(), CliError> {
	let config_path = get_config_path()?;
	let config_str = serde_json::to_string_pretty(config)
		.map_err(|e| CliError::Config(format!("Failed to serialize config: {}", e)))?;
	fs::write(config_path, config_str).map_err(|e| CliError::Io(e))?;
	Ok(())
}
