use async_trait::async_trait;
use primitive_types::H160;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use neo_clients::{JsonRpcProvider, RpcClient};
use neo_contract::ContractError;

/// Neo X EVM provider for interacting with the Neo X EVM-compatible chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeoXProvider<'a, P: JsonRpcProvider> {
	rpc_url: String,
	#[serde(skip)]
	provider: Option<&'a RpcClient<P>>,
	is_testnet: bool,
}

impl<'a, P: JsonRpcProvider + 'static> NeoXProvider<'a, P> {
	/// Creates a new NeoXProvider instance with the specified RPC URL
	///
	/// # Arguments
	///
	/// * `rpc_url` - The RPC URL for the Neo X chain
	/// * `provider` - An optional reference to an RPC client
	/// * `is_testnet` - Flag indicating whether to use testnet configuration
	///
	/// # Returns
	///
	/// A new NeoXProvider instance
	pub fn new(rpc_url: &str, provider: Option<&'a RpcClient<P>>, is_testnet: bool) -> Self {
		Self { 
			rpc_url: rpc_url.to_string(), 
			provider,
			is_testnet,
		}
	}

	/// Gets the RPC URL for the Neo X chain
	///
	/// # Returns
	///
	/// The RPC URL as a string
	pub fn rpc_url(&self) -> &str {
		&self.rpc_url
	}

	/// Sets the RPC URL for the Neo X chain
	///
	/// # Arguments
	///
	/// * `rpc_url` - The new RPC URL
	pub fn set_rpc_url(&mut self, rpc_url: &str) {
		self.rpc_url = rpc_url.to_string();
	}

	/// Gets the chain ID for the Neo X chain
	///
	/// # Returns
	///
	/// The chain ID as a u64
	pub async fn chain_id(&self) -> Result<u64, ContractError> {
		// Return appropriate chain ID based on network
		if self.is_testnet {
			Ok(11571) // Neo X Testnet Chain ID
		} else {
			Ok(11570) // Neo X Mainnet Chain ID 
		}
	}

	/// Gets whether this provider is connected to testnet
	///
	/// # Returns
	///
	/// Boolean indicating if this is a testnet connection
	pub fn is_testnet(&self) -> bool {
		self.is_testnet
	}
}
