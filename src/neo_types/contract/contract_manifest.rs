use std::{
	collections::HashMap,
	hash::{Hash, Hasher},
};

use serde::{Deserialize, Serialize};

use neo::prelude::{ContractParameter, ContractParameterType};

use crate::prelude::{ContractParameter2, serialize_wildcard, deserialize_wildcard, TypeError};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ContractManifest {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub name: Option<String>,
	#[serde(default)]
	pub groups: Vec<ContractGroup>,
	#[serde(skip_serializing)]
	pub features: Option<HashMap<String, serde_json::Value>>,
	#[serde(rename = "supportedstandards")]
	pub supported_standards: Vec<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub abi: Option<ContractABI>,
	#[serde(default)]
	pub permissions: Vec<ContractPermission>,
	pub trusts: Vec<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub extra: Option<HashMap<String, serde_json::Value>>,
}

impl ContractManifest{
	pub fn get_supported_standard(&self, index: usize) -> Result<&String, TypeError> {
		if index >= self.supported_standards.len() {
			return Err(TypeError::IndexOutOfBounds(format!(
				"This contract only supports {} standards. Tried to access a supported standard at index {} in the manifest",
				self.supported_standards.len(),
				index
			)));
		}
		Ok(&self.supported_standards[index])
	}

}

// impl Eq for ContractManifest
impl PartialEq for ContractManifest {
	fn eq(&self, other: &Self) -> bool {
		self.name == other.name
			&& self.groups == other.groups
			&& self.features == other.features
			&& self.supported_standards == other.supported_standards
			&& self.abi == other.abi
			&& self.permissions == other.permissions
			&& self.trusts == other.trusts
			&& self.extra == other.extra
	}
}

impl Hash for ContractManifest {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.name.hash(state);
		self.groups.hash(state);
		// self.features.hash(state);
		self.supported_standards.hash(state);
		self.abi.hash(state);
		self.permissions.hash(state);
		self.trusts.hash(state);
		// self.extra.hash(state);
	}
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Debug, Clone)]
pub struct ContractGroup {
	pub pub_key: String,
	pub signature: String,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct ContractABI {
	pub methods: Vec<ContractMethod>,
	// #[serde(skip_serializing_if = "Option::is_none")]
	#[serde(default)]
	pub events: Vec<ContractEvent>,
}

impl  ContractABI {
	pub fn get_first_event(&self) -> Result<&ContractEvent, TypeError> {
		if self.events.is_empty() {
			return Err(TypeError::IndexOutOfBounds(
				"This ABI does not have any events.".to_string(),
			));
		}
		self.get_event(0)
	}
	pub fn get_event(&self, index: usize) -> Result<&ContractEvent, TypeError> {
		if index >= self.events.len() {
			return Err(TypeError::IndexOutOfBounds(format!(
				"This ABI only has {} events. Tried to access index {}.",
				self.events.len(),
				index
			)));
		}
		Ok(&self.events[index])
	}
	
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub struct ContractMethod {
	pub name: String,
	pub parameters: Vec<ContractParameter2>,
	pub offset: usize,
	#[serde(rename = "returntype")]
	pub return_type: ContractParameterType,
	pub safe: bool,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Debug, Clone)]
pub struct ContractEvent {
	pub name: String,
	pub parameters: Vec<ContractParameter>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct ContractPermission {
	pub contract: String,
	#[serde(serialize_with = "serialize_wildcard")]
	#[serde(deserialize_with = "deserialize_wildcard")]
	pub methods: Vec<String>,
}
