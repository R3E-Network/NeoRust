// DeFi utilities for Neo CLI
//
// This module contains utility functions for DeFi operations

use crate::{commands::wallet::CliState, errors::CliError};
use neo3::{
	neo_clients::{HttpProvider, ProviderError, RpcClient},
	neo_types::AddressExtension,
	neo_wallets::Wallet,
	prelude::*,
	providers::APITrait,
};
use num_traits::cast::ToPrimitive;
use primitive_types::H160;
use std::{path::PathBuf, str::FromStr};

/// Network type enum for CLI operations
///
/// This represents the different Neo N3 networks that can be used with the CLI
pub enum NetworkType {
	MainNet,
	TestNet,
	PrivateNet,
}

impl NetworkType {
	/// Convert a network string to NetworkType
	///
	/// # Arguments
	/// * `network` - Network name string ("MainNet", "TestNet", etc.)
	pub fn from_network(network: &str) -> Self {
		match network.to_lowercase().as_str() {
			"mainnet" => NetworkType::MainNet,
			"testnet" => NetworkType::TestNet,
			_ => NetworkType::PrivateNet,
		}
	}
}

/// Network type for CLI operations compatible with wallet module
///
/// This provides compatibility with the network types used in the wallet module
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetworkTypeCli {
	MainNet,  // Neo N3 MainNet
	TestNet,  // Neo N3 TestNet
	NeoXMain, // Neo X MainNet
	NeoXTest, // Neo X TestNet
}

impl NetworkTypeCli {
	/// Create a NetworkTypeCli from a magic number
	///
	/// # Arguments
	/// * `magic` - The network magic number
	pub fn from_magic(magic: u32) -> Self {
		match magic {
			769 => NetworkTypeCli::MainNet,  // Neo N3 MainNet
			894 => NetworkTypeCli::TestNet,  // Neo N3 TestNet
			245 => NetworkTypeCli::NeoXMain, // Neo X MainNet magic number
			422 => NetworkTypeCli::NeoXTest, // Neo X TestNet magic number
			_ => NetworkTypeCli::TestNet,    // Default to TestNet for safety
		}
	}

	/// Create NetworkTypeCli from a network string
	///
	/// # Arguments
	/// * `network` - Network name string ("MainNet", "TestNet", "NeoX-MainNet", etc.)
	pub fn from_network_string(network: &str) -> Self {
		match network.to_lowercase().as_str() {
			"mainnet" => NetworkTypeCli::MainNet,
			"testnet" => NetworkTypeCli::TestNet,
			"neox" | "neo-x" | "neox-mainnet" => NetworkTypeCli::NeoXMain,
			"neox-testnet" => NetworkTypeCli::NeoXTest,
			_ => NetworkTypeCli::TestNet, // Default to TestNet for safety
		}
	}

	/// Convert this NetworkTypeCli to wallet Network enum string
	pub fn to_network_string(&self) -> String {
		match self {
			NetworkTypeCli::MainNet => "MainNet".to_string(),
			NetworkTypeCli::TestNet => "TestNet".to_string(),
			NetworkTypeCli::NeoXMain => "NeoX-MainNet".to_string(),
			NetworkTypeCli::NeoXTest => "NeoX-TestNet".to_string(),
		}
	}
	
	/// Check if this network type is a Neo X network
	pub fn is_neox(&self) -> bool {
		matches!(self, NetworkTypeCli::NeoXMain | NetworkTypeCli::NeoXTest)
	}
	
	/// Check if this network type is a Neo N3 network
	pub fn is_neo_n3(&self) -> bool {
		matches!(self, NetworkTypeCli::MainNet | NetworkTypeCli::TestNet)
	}
	
	/// Check if this network type is a mainnet
	pub fn is_mainnet(&self) -> bool {
		matches!(self, NetworkTypeCli::MainNet | NetworkTypeCli::NeoXMain)
	}
	
	/// Check if this network type is a testnet
	pub fn is_testnet(&self) -> bool {
		matches!(self, NetworkTypeCli::TestNet | NetworkTypeCli::NeoXTest)
	}
}

/// Load wallet from file
pub async fn load_wallet(
	wallet_path: &PathBuf,
	password: Option<&str>,
) -> Result<Wallet, CliError> {
	// Check if the wallet file exists
	if !wallet_path.exists() {
		return Err(CliError::Wallet(format!("Wallet file not found: {}", wallet_path.display())));
	}

	// Open wallet with or without password
	let wallet = match password {
		Some(pwd) => Wallet::open_wallet(wallet_path, pwd)
			.map_err(|e| CliError::Wallet(format!("Failed to open wallet: {}", e)))?,
		None => {
			// Read wallet file without password
			let wallet_json = std::fs::read_to_string(wallet_path)
				.map_err(|e| CliError::Wallet(format!("Failed to read wallet file: {}", e)))?;

			// Parse wallet JSON
			serde_json::from_str(&wallet_json)
				.map_err(|e| CliError::Wallet(format!("Failed to parse wallet file: {}", e)))?
		},
	};

	Ok(wallet)
}

/// Prepare a CLI state from an existing state
pub fn prepare_state_from_existing(existing_state: &CliState) -> CliState {
	let mut new_state = CliState::default();

	// Copy over relevant state
	if let Some(wallet) = &existing_state.wallet {
		new_state.wallet = Some(wallet.clone());
	}

	if let Some(rpc_client) = &existing_state.rpc_client {
		new_state.rpc_client = Some(rpc_client.clone());
	}

	new_state.network_type = existing_state.network_type.clone();

	new_state
}

/// Get token hash for a token symbol based on network type
pub fn get_token_address_for_network(
	token_symbol: &str,
	network_type: NetworkTypeCli,
) -> Option<ScriptHash> {
	// Uppercase the token symbol for consistent matching
	let token_symbol = token_symbol.to_uppercase();

	// Use centralized constants for token addresses
	match network_type {
		NetworkTypeCli::MainNet => {
			// Token addresses for Neo N3 Mainnet
			constants::neo_n3_mainnet::get_script_hash(&token_symbol)
		},
		NetworkTypeCli::TestNet => {
			// Token addresses for Neo N3 Testnet
			constants::neo_n3_testnet::get_script_hash(&token_symbol)
		},
		NetworkTypeCli::NeoXMain => {
			// Token addresses for Neo X Mainnet
			constants::neo_x_mainnet::get_script_hash(&token_symbol)
		},
		NetworkTypeCli::NeoXTest => {
			// Token addresses for Neo X Testnet
			constants::neo_x_testnet::get_script_hash(&token_symbol)
		},
	}
}

/// Parse amount string to raw integer value based on token decimals
pub async fn parse_amount(
	amount: &str,
	token_hash: &ScriptHash,
	rpc_client: &RpcClient<HttpProvider>,
	network_type: NetworkTypeCli,
) -> Result<i64, CliError> {
	// Try to parse as a simple float first
	let amount_float = f64::from_str(amount).map_err(|_| {
		CliError::InvalidArgument(
			format!("Invalid amount: {}", amount),
			"Please provide a valid number".to_string(),
		)
	})?;

	// Get token decimals
	let token_decimals = get_token_decimals(token_hash, rpc_client, network_type).await?;

	// Calculate raw amount (amount * 10^decimals)
	let multiplier = 10_f64.powi(token_decimals as i32);
	let raw_amount = (amount_float * multiplier).round() as i64;

	Ok(raw_amount)
}

/// Get decimals for a token
pub async fn get_token_decimals(
	token_hash: &ScriptHash,
	rpc_client: &RpcClient<HttpProvider>,
	network_type: NetworkTypeCli,
) -> Result<u8, CliError> {
	// Get token hash as string for comparison
	let token_hash_str = token_hash.to_string();

	// Check for well-known tokens first based on network type
	if network_type.is_neo_n3() {
		// Neo N3 tokens (MainNet and TestNet share same decimals)
		if token_hash_str == constants::neo_n3_mainnet::NEO 
			|| token_hash_str == format!("0x{}", constants::neo_n3_mainnet::NEO) {
			return Ok(0); // NEO has 0 decimals
		} else if token_hash_str == constants::neo_n3_mainnet::GAS 
			|| token_hash_str == format!("0x{}", constants::neo_n3_mainnet::GAS) {
			return Ok(8); // GAS has 8 decimals
		} else if token_hash_str == constants::neo_n3_mainnet::FLM 
			|| token_hash_str == format!("0x{}", constants::neo_n3_mainnet::FLM) {
			return Ok(8); // FLM has 8 decimals
		} else if token_hash_str == constants::neo_n3_mainnet::BNEO 
			|| token_hash_str == format!("0x{}", constants::neo_n3_mainnet::BNEO) {
			return Ok(8); // bNEO has 8 decimals
		}
		// If on testnet, also check testnet-specific tokens
		if network_type.is_testnet() {
			if token_hash_str == constants::neo_n3_testnet::TEST 
				|| token_hash_str == format!("0x{}", constants::neo_n3_testnet::TEST) {
				return Ok(8); // TEST token has 8 decimals
			} else if token_hash_str == constants::neo_n3_testnet::USDT 
				|| token_hash_str == format!("0x{}", constants::neo_n3_testnet::USDT) {
				return Ok(6); // USDT has 6 decimals
			}
		}
	} else if network_type.is_neox() {
		// Neo X tokens
		if network_type == NetworkTypeCli::NeoXMain {
			// NeoX MainNet tokens
			if token_hash_str == constants::neo_x_mainnet::NEO {
				return Ok(0); // NEO has 0 decimals
			} else if token_hash_str == constants::neo_x_mainnet::GAS 
				|| token_hash_str == constants::neo_x_mainnet::NEOX {
				return Ok(18); // GAS/NEOX on Neo X has 18 decimals
			}
		} else {
			// NeoX TestNet tokens
			if token_hash_str == constants::neo_x_testnet::NEO {
				return Ok(0); // NEO has 0 decimals
			} else if token_hash_str == constants::neo_x_testnet::GAS 
				|| token_hash_str == constants::neo_x_testnet::NEOX {
				return Ok(18); // GAS/NEOX on Neo X has 18 decimals
			}
		}
	}

	// Call the decimals method on the token contract
	match rpc_client
		.invoke_function(token_hash, "decimals".to_string(), vec![], None)
		.await
	{
		Ok(result) => {
			if let Some(item) = result.stack.first() {
				match item {
					StackItem::Integer { value } => {
						// Convert to u8
						value.to_u8().ok_or_else(|| {
							CliError::InvalidInput(format!(
								"Invalid decimals value: {}. Expected a small integer for decimals",
								value
							))
						})
					},
					_ => Err(CliError::InvalidInput(format!(
						"Unexpected stack item type for decimals: {:?}. Expected an integer.",
						item
					))),
				}
			} else {
				Err(CliError::InvalidInput(
					"Empty stack response for decimals call. Token contract may not be valid."
						.to_string(),
				))
			}
		},
		Err(e) => Err(CliError::Rpc(format!("Failed to get token decimals: {}", e))),
	}
}

/// Format token amount with proper decimal places
pub fn format_token_amount(raw_amount: i64, decimals: u8) -> String {
	let divisor = 10_f64.powi(decimals as i32);
	let formatted_amount = (raw_amount as f64) / divisor;

	if decimals == 0 {
		return format!("{}", raw_amount);
	} else {
		return format!("{:.1$}", formatted_amount, decimals as usize);
	}
}

/// Resolve token symbol or address to a script hash
pub async fn resolve_token_to_scripthash_with_network(
	token: &str,
	rpc_client: &RpcClient<HttpProvider>,
	network_type: NetworkTypeCli,
) -> Result<ScriptHash, CliError> {
	// Check if the input is a valid script hash
	if let Ok(script_hash) = ScriptHash::from_str(token) {
		return Ok(script_hash);
	}

	// Check if it's a valid address
	match Address::from_str(token) {
		Ok(address) => {
			return address.address_to_script_hash().map_err(|e| {
				CliError::InvalidArgument(
					format!("Invalid address: {}", e),
					"Please provide a valid NEO address".to_string(),
				)
			});
		},
		Err(_) => {},
	}

	// Check if it's a well-known token symbol using our centralized constants
	if let Some(script_hash) = get_token_address_for_network(token, network_type) {
		return Ok(script_hash);
	}

	// For Neo X networks, use appropriate contract registry based on network type
	if network_type.is_neox() {
		// Get the appropriate registry contract hash for the network
		let contract_registry = if network_type == NetworkTypeCli::NeoXMain {
			// MainNet registry
			ScriptHash::from_str(&constants::contracts::neo_x_mainnet::TOKEN_REGISTRY)
		} else {
			// TestNet registry
			ScriptHash::from_str(&constants::contracts::neo_x_testnet::TOKEN_REGISTRY)
		};

		// If we have a valid registry contract, try to resolve the token
		if let Ok(registry) = contract_registry {
			let token_upper = token.to_uppercase();
			let params = vec![ContractParameter::string(token_upper)];
			
			match rpc_client.invoke_function_diagnostics(registry, "getTokenAddress".to_string(), params, vec![]).await {
				Ok(result) => {
					if let Some(item) = result.stack.first() {
						if let Some(bytes) = item.as_bytes() {
							if !bytes.is_empty() {
								let address = String::from_utf8_lossy(&bytes);
								if let Ok(hash) = ScriptHash::from_str(&address) {
									return Ok(hash);
								}
							}
							}
						}
					}
				},
				Err(_) => {}
			}
		}
	}

	// If we get here, we couldn't resolve the token
	Err(CliError::InvalidArgument(
		format!("Could not resolve token: {}", token),
		"Please provide a valid token address, symbol, or contract hash".to_string(),
	))
}

/// Resolve token symbol or address to a string hash
pub async fn resolve_token_hash(
	token: &str,
	rpc_client: &RpcClient<HttpProvider>,
	network_type: NetworkTypeCli,
) -> Result<String, CliError> {
	let script_hash =
		resolve_token_to_scripthash_with_network(token, rpc_client, network_type).await?;
	Ok(script_hash.to_string())
}

/// Helper function to load a wallet from state
pub fn load_wallet_from_state(
	state: &mut CliState,
) -> Result<&mut crate::commands::wallet::Wallet, CliError> {
	if state.wallet.is_none() {
		return Err(CliError::NoWallet);
	}
	Ok(state.wallet.as_mut().unwrap())
}

/// Get the bridge contract hash for a specific network type
pub fn get_bridge_contract_hash(network_type: NetworkTypeCli) -> Result<ScriptHash, CliError> {
	// Use centralized constants for contract addresses
	if network_type.is_neox() {
		return Err(CliError::InvalidArgument(
			"Cannot get bridge contract for Neo X network".to_string(),
			"Bridge contract must be called from Neo N3 network".to_string(),
		));
	}

	// Get the bridge contract hash from constants
	let is_testnet = network_type.is_testnet();
	constants::bridge::get_contract_hash(is_testnet)
		.ok_or_else(|| CliError::InvalidInput("Failed to parse bridge contract hash".to_string()))
}
}
