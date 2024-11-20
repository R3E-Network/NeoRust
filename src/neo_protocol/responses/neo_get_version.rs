use crate::prelude::deserialize_hardforks;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
pub struct NeoVersion {
	#[serde(rename = "tcpport", default = "default_tcp_port")]
	pub tcp_port: Option<u16>,
	#[serde(rename = "wsport", default = "default_ws_port")]
	pub ws_port: Option<u16>,
	#[serde(default = "default_nonce")]
	pub nonce: u32,
	#[serde(rename = "useragent", default = "default_user_agent")]
	pub user_agent: String,
	#[serde(default = "default_protocol")]
	pub protocol: Option<NeoProtocol>,
}

impl Default for NeoVersion {
	fn default() -> Self {
		NeoVersion {
			tcp_port: Some(10333),
			ws_port: Some(10334),
			nonce: 1234567890,
			user_agent: "/Neo:3.5.0/".to_string(),
			protocol: Some(NeoProtocol::default()),
		}
	}
}

fn default_tcp_port() -> Option<u16> {
	Some(10333)
}

fn default_ws_port() -> Option<u16> {
	Some(10334)
}

fn default_nonce() -> u32 {
	1234567890
}

fn default_user_agent() -> String {
	"/Neo:3.5.0/".to_string()
}

fn default_protocol() -> Option<NeoProtocol> {
	Some(NeoProtocol::default())
}

impl PartialEq for NeoVersion {
	fn eq(&self, other: &Self) -> bool {
		self.tcp_port == other.tcp_port
			&& self.ws_port == other.ws_port
			&& self.nonce == other.nonce
			&& self.user_agent == other.user_agent
			&& self.protocol == other.protocol
	}
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct NeoProtocol {
	#[serde(default = "default_network")]
	pub network: u32,
	#[serde(rename = "validatorscount", default = "default_validators_count")]
	pub validators_count: Option<u32>,
	#[serde(rename = "msperblock", default = "default_ms_per_block")]
	pub ms_per_block: u32,
	#[serde(
		rename = "maxvaliduntilblockincrement",
		default = "default_max_valid_until_block_increment"
	)]
	pub max_valid_until_block_increment: u32,
	#[serde(rename = "maxtraceableblocks", default = "default_max_traceable_blocks")]
	pub max_traceable_blocks: u32,
	#[serde(rename = "addressversion", default = "default_address_version")]
	pub address_version: u32,
	#[serde(rename = "maxtransactionsperblock", default = "default_max_transactions_per_block")]
	pub max_transactions_per_block: u32,
	#[serde(
		rename = "memorypoolmaxtransactions",
		default = "default_memory_pool_max_transactions"
	)]
	pub memory_pool_max_transactions: u32,
	#[serde(rename = "initialgasdistribution", default = "default_initial_gas_distribution")]
	pub initial_gas_distribution: u64,
	#[serde(rename = "hardforks", default, deserialize_with = "deserialize_hardforks")]
	pub hard_forks: Vec<HardForks>,
}

impl Default for NeoProtocol {
	fn default() -> Self {
		NeoProtocol {
			network: 860833102,
			validators_count: Some(7),
			ms_per_block: 15000,
			max_valid_until_block_increment: 5760,
			max_traceable_blocks: 2102400,
			address_version: 53,
			max_transactions_per_block: 512,
			memory_pool_max_transactions: 50000,
			initial_gas_distribution: 5200000000000000,
			hard_forks: Vec::new(),
		}
	}
}

fn default_network() -> u32 {
	860833102
}
fn default_validators_count() -> Option<u32> {
	Some(7)
}
fn default_ms_per_block() -> u32 {
	15000
}
fn default_max_valid_until_block_increment() -> u32 {
	5760
}
fn default_max_traceable_blocks() -> u32 {
	2102400
}
fn default_address_version() -> u32 {
	53
}
fn default_max_transactions_per_block() -> u32 {
	512
}
fn default_memory_pool_max_transactions() -> u32 {
	50000
}
fn default_initial_gas_distribution() -> u64 {
	5200000000000000
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct HardForks {
	pub name: String,
	#[serde(rename = "blockheight")]
	pub block_height: u32,
}
