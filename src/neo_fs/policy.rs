use crate::neo_fs::{
	errors::NeoFsResult,
	types::{AccessRule, ContainerId},
};
use serde::{Deserialize, Serialize};

/// NeoFS access control mechanisms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControl {
	/// Extended ACL table (eACL)
	pub extended_acl: Option<ExtendedAcl>,
	/// Basic ACL bitmask
	pub basic_acl: u32,
}

/// Extended Access Control List (eACL)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedAcl {
	/// Container ID this eACL applies to
	pub container_id: ContainerId,
	/// Version of the eACL
	pub version: u32,
	/// List of access control rules
	pub records: Vec<EaclRecord>,
}

/// Extended ACL record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EaclRecord {
	/// Operation this record applies to
	pub operation: Operation,
	/// Action to take (allow or deny)
	pub action: Action,
	/// Filters to apply this record to specific targets
	pub filters: Vec<EaclFilter>,
	/// Targets of the rule (who it applies to)
	pub targets: Vec<EaclTarget>,
}

/// EACL operation types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation {
	/// Create a new object
	Put = 0,
	/// Get an object
	Get = 1,
	/// Read object headers
	Head = 2,
	/// Search for objects
	Search = 3,
	/// Delete an object
	Delete = 4,
	/// Get access information
	GetACL = 5,
	/// Set access information
	SetACL = 6,
	/// Any operation
	Any = 7,
}

/// EACL action types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
	/// Allow the operation
	Allow = 0,
	/// Deny the operation
	Deny = 1,
}

/// EACL filter types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EaclFilter {
	/// Filter by object attribute
	Attribute {
		/// Attribute key
		key: String,
		/// Attribute value (exact match)
		value: String,
	},
	/// Filter by object header
	Header {
		/// Header type
		header_type: HeaderType,
		/// Match type for the header
		match_type: MatchType,
		/// Value to match
		value: String,
	},
}

/// EACL header types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeaderType {
	/// Object ID
	ObjectID = 0,
	/// Container ID
	ContainerID = 1,
	/// Object owner
	Owner = 2,
	/// Creation epoch
	CreationEpoch = 3,
	/// Payload size
	PayloadSize = 4,
	/// Object type
	ObjectType = 5,
	/// Object version
	Version = 6,
	/// Object hash
	Hash = 7,
}

/// EACL match types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MatchType {
	/// Exact match
	StringEqual = 0,
	/// String starts with
	StringPrefix = 1,
	/// Numeric value greater than
	NumericGt = 2,
	/// Numeric value greater than or equal
	NumericGe = 3,
	/// Numeric value less than
	NumericLt = 4,
	/// Numeric value less than or equal
	NumericLe = 5,
}

/// EACL target (who a rule applies to)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EaclTarget {
	/// Target type
	pub target_type: TargetType,
	/// Target key (e.g. public key for users)
	pub key: Option<Vec<u8>>,
	/// Target role (system roles)
	pub role: Option<Role>,
}

/// EACL target types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetType {
	/// Target specific public key
	PublicKey = 0,
	/// Target system role
	SystemRole = 1,
	/// Target other field
	Other = 2,
}

/// System roles for eACL targeting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Role {
	/// Container owner
	Owner = 0,
	/// Other users
	Others = 1,
	/// Inner ring nodes
	InnerRing = 2,
	/// Any storage node
	StorageNode = 3,
}

/// Builder for creating extended ACL rules
pub struct ExtendedAclBuilder {
	container_id: Option<ContainerId>,
	version: u32,
	records: Vec<EaclRecord>,
}

impl ExtendedAclBuilder {
	/// Create a new extended ACL builder
	pub fn new() -> Self {
		Self { container_id: None, version: 1, records: Vec::new() }
	}

	/// Set the container ID
	pub fn container_id(mut self, container_id: ContainerId) -> Self {
		self.container_id = Some(container_id);
		self
	}

	/// Set the version
	pub fn version(mut self, version: u32) -> Self {
		self.version = version;
		self
	}

	/// Add a rule
	pub fn add_rule(mut self, record: EaclRecord) -> Self {
		self.records.push(record);
		self
	}

	/// Build the extended ACL
	pub fn build(self) -> NeoFsResult<ExtendedAcl> {
		let container_id = self.container_id.ok_or_else(|| {
			crate::neo_fs::errors::NeoFsError::InvalidResponse(
				"Container ID is required for eACL".to_string(),
			)
		})?;

		Ok(ExtendedAcl { container_id, version: self.version, records: self.records })
	}
}

impl Default for ExtendedAclBuilder {
	fn default() -> Self {
		Self::new()
	}
}

/// Builder for creating eACL records
pub struct EaclRecordBuilder {
	operation: Operation,
	action: Action,
	filters: Vec<EaclFilter>,
	targets: Vec<EaclTarget>,
}

impl EaclRecordBuilder {
	/// Create a new eACL record builder
	pub fn new(operation: Operation, action: Action) -> Self {
		Self { operation, action, filters: Vec::new(), targets: Vec::new() }
	}

	/// Add an attribute filter
	pub fn attribute_filter(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
		self.filters
			.push(EaclFilter::Attribute { key: key.into(), value: value.into() });
		self
	}

	/// Add a header filter
	pub fn header_filter(
		mut self,
		header_type: HeaderType,
		match_type: MatchType,
		value: impl Into<String>,
	) -> Self {
		self.filters
			.push(EaclFilter::Header { header_type, match_type, value: value.into() });
		self
	}

	/// Add a target public key
	pub fn target_public_key(mut self, key: Vec<u8>) -> Self {
		self.targets.push(EaclTarget {
			target_type: TargetType::PublicKey,
			key: Some(key),
			role: None,
		});
		self
	}

	/// Add a target role
	pub fn target_role(mut self, role: Role) -> Self {
		self.targets.push(EaclTarget {
			target_type: TargetType::SystemRole,
			key: None,
			role: Some(role),
		});
		self
	}

	/// Build the eACL record
	pub fn build(self) -> EaclRecord {
		EaclRecord {
			operation: self.operation,
			action: self.action,
			filters: self.filters,
			targets: self.targets,
		}
	}
}
