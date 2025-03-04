// Copyright (c) 2023-2025 R3E Network
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # NeoFS Core Types
//!
//! This module defines the core data types used in the NeoFS system.

use derive_more::{Display, From};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a container in NeoFS
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Display, From)]
pub struct ContainerId(pub String);

/// Unique identifier for an object in NeoFS
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Display, From)]
pub struct ObjectId(pub String);

/// Unique identifier for a user in the NeoFS network
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Display, From)]
pub struct OwnerId(pub String);

/// Represents the placement policy for objects in a container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlacementPolicy {
	/// Number of replicas to store
	pub replicas: u32,
	/// Rules that define the storage nodes selection
	pub selectors: Vec<Selector>,
	/// Filters for storage node properties
	pub filters: Vec<Filter>,
}

impl Default for PlacementPolicy {
	fn default() -> Self {
		Self { replicas: 3, selectors: Vec::new(), filters: Vec::new() }
	}
}

/// Selector for the placement policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Selector {
	/// Name of the selector
	pub name: String,
	/// Number of nodes to select
	pub count: u32,
	/// Attribute to use for selection
	pub attribute: String,
	/// ClauseOperator to apply
	pub clause: ClauseOperator,
	/// Value to compare with
	pub value: String,
}

/// Filter for the placement policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Filter {
	/// Name of the filter
	pub name: String,
	/// Key to filter on
	pub key: String,
	/// Operator to apply
	pub operation: MatchOperator,
	/// Value to compare with
	pub value: String,
}

/// Operators for clause matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClauseOperator {
	/// Equals
	EQ,
	/// Not equals
	NE,
	/// Greater than
	GT,
	/// Greater than or equals
	GE,
	/// Less than
	LT,
	/// Less than or equals
	LE,
}

/// Operators for filter matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchOperator {
	/// Equals
	EQ,
	/// Not equals
	NE,
	/// Regular expression match
	RE,
	/// Regular expression not match
	NRE,
}

/// Type of NeoFS object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObjectType {
	/// Regular file
	Regular,
	/// Tombstone (marks deleted object)
	Tombstone,
	/// Storage group
	StorageGroup,
}

/// Metadata attributes for NeoFS objects
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Attributes {
	/// Key-value pairs of attributes
	pub attributes: HashMap<String, String>,
}

impl Attributes {
	/// Creates a new empty attributes collection
	pub fn new() -> Self {
		Self { attributes: HashMap::new() }
	}

	/// Adds an attribute
	pub fn add(&mut self, key: impl Into<String>, value: impl Into<String>) {
		self.attributes.insert(key.into(), value.into());
	}

	/// Gets an attribute value by key
	pub fn get(&self, key: &str) -> Option<&String> {
		self.attributes.get(key)
	}
}

/// Access permissions for NeoFS containers and objects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessPermission {
	/// Permission to put an object
	PutObject,
	/// Permission to get an object
	GetObject,
	/// Permission to head an object (retrieve metadata)
	HeadObject,
	/// Permission to delete an object
	DeleteObject,
	/// Permission to search objects
	SearchObject,
	/// Permission to get extended ACL
	GetEACL,
	/// Permission to set extended ACL
	SetEACL,
}

/// Session token for authenticating with NeoFS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionToken {
	/// Token ID
	pub token_id: String,
	/// Owner ID
	pub owner_id: OwnerId,
	/// Expires at
	pub expires_at: u64,
	/// Signature
	pub signature: Vec<u8>,
}

// Multipart upload functionality moved to object.rs
