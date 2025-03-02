use serde::{Deserialize, Serialize};
use std::fmt;

/// A public key type for NeoFS
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PublicKey(Vec<u8>);

impl PublicKey {
	/// Create a new public key from bytes
	pub fn new(bytes: Vec<u8>) -> Self {
		Self(bytes)
	}

	/// Get the underlying bytes
	pub fn as_bytes(&self) -> &[u8] {
		&self.0
	}
}

/// A script hash type for NeoFS
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScriptHash([u8; 20]);

impl ScriptHash {
	/// Create a new script hash from bytes
	pub fn new(bytes: [u8; 20]) -> Self {
		Self(bytes)
	}

	/// Get the underlying bytes
	pub fn as_bytes(&self) -> &[u8; 20] {
		&self.0
	}
}

/// Represents a NeoFS container ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContainerId([u8; 32]);

impl ContainerId {
	/// Creates a new ContainerId from a byte array
	pub fn new(bytes: [u8; 32]) -> Self {
		Self(bytes)
	}

	/// Returns the underlying bytes
	pub fn as_bytes(&self) -> &[u8; 32] {
		&self.0
	}

	/// Creates a container ID from a hex string
	pub fn from_hex(hex_str: &str) -> Result<Self, hex::FromHexError> {
		let mut bytes = [0u8; 32];
		hex::decode_to_slice(hex_str.trim_start_matches("0x"), &mut bytes)?;
		Ok(Self(bytes))
	}
}

impl fmt::Display for ContainerId {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "0x{}", hex::encode(&self.0))
	}
}

/// Represents a NeoFS object ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectId([u8; 32]);

impl ObjectId {
	/// Creates a new ObjectId from a byte array
	pub fn new(bytes: [u8; 32]) -> Self {
		Self(bytes)
	}

	/// Returns the underlying bytes
	pub fn as_bytes(&self) -> &[u8; 32] {
		&self.0
	}

	/// Creates an object ID from a hex string
	pub fn from_hex(hex_str: &str) -> Result<Self, hex::FromHexError> {
		let mut bytes = [0u8; 32];
		hex::decode_to_slice(hex_str.trim_start_matches("0x"), &mut bytes)?;
		Ok(Self(bytes))
	}
}

impl fmt::Display for ObjectId {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "0x{}", hex::encode(&self.0))
	}
}

/// Storage policy for NeoFS objects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoragePolicy {
	/// Number of replicas to maintain
	pub replicas: u8,
	/// Data placement rules (regions, tiers, etc.)
	pub placement: PlacementPolicy,
	/// Container lifespan in epochs (0 for infinite)
	pub lifetime: u64,
}

/// Defines how data should be placed in the NeoFS network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacementPolicy {
	/// Geographic region selectors
	pub regions: Vec<RegionSelector>,
	/// Reliability tier
	pub tier: ReliabilityTier,
	/// Minimum number of nodes per region
	pub min_nodes_per_region: u8,
}

/// Geographic region placement selector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionSelector {
	/// Region identifier (e.g., "EU", "US", "APAC")
	pub region: String,
	/// Number of nodes to use in this region
	pub node_count: u8,
}

/// Reliability tier for data storage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReliabilityTier {
	/// Standard reliability
	Standard = 0,
	/// Premium reliability with better guarantees
	Premium = 1,
	/// Maximum reliability for critical data
	Maximum = 2,
}

impl Default for ReliabilityTier {
	fn default() -> Self {
		Self::Standard
	}
}

/// Access control rule type for containers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessRule {
	/// Public access (anyone can access)
	Public,
	/// Only specified users can access
	Private(Vec<PublicKey>),
	/// Access controlled by a bearer token
	BearerToken,
	/// Custom smart contract-based access control
	SmartContract(ScriptHash),
}

/// NeoFS API response status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResponseStatus {
	/// Operation completed successfully
	Success = 0,
	/// Failed to process request
	Failed = 1,
	/// Access denied
	AccessDenied = 2,
	/// Resource not found
	NotFound = 3,
	/// Container already exists
	AlreadyExists = 4,
	/// Invalid request format or parameters
	InvalidRequest = 5,
}
