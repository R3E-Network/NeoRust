use std::collections::HashMap;

use primitive_types::H160;
use serde::{Deserialize, Serialize};

use crate::{
	contract::{contract_manifest::ContractManifest, contract_nef::ContractNef, nef_file::NefFile},
	serde_with_utils::{deserialize_h160, serialize_h160},
};

use crate::{
	deserialize_script_hash, serialize_script_hash, InvocationResult, StackItem,
};

// Define ContractIdentifiers here instead of importing it
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ContractIdentifiers {
	pub id: i32,
	#[serde(deserialize_with = "deserialize_script_hash")]
	#[serde(serialize_with = "serialize_script_hash")]
	pub hash: H160,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContractState {
	#[serde(rename = "id")]
	pub id: i32,
	#[serde(rename = "updatecounter")]
	pub update_counter: i32,
	#[serde(rename = "hash")]
	#[serde(serialize_with = "serialize_h160", deserialize_with = "deserialize_h160", default)]
	pub hash: H160,
	#[serde(rename = "nef")]
	pub nef: NefFile,
	#[serde(rename = "manifest")]
	pub manifest: ContractManifest,
	#[serde(rename = "updatehistory", default)]
	pub update_history: Vec<i32>,
}

impl ContractState {
	pub fn new(
		id: i32,
		update_counter: i32,
		hash: H160,
		nef: NefFile,
		manifest: ContractManifest,
	) -> Self {
		Self { id, update_counter, hash, nef, manifest, update_history: Vec::new() }
	}

	pub fn contract_identifiers(
		stack_item: &StackItem,
	) -> Result<ContractIdentifiers, &'static str> {
		match stack_item {
			StackItem::Struct { value } if value.len() >= 2 => {
				let id = value[0]
					.as_int()
					.ok_or("Failed to get contract ID as integer from stack item")?;

				let mut v = value[1]
					.as_bytes()
					.ok_or("Failed to get contract hash as bytes from stack item")?;

				v.reverse();
				let hash = H160::from_slice(&v);
				Ok(ContractIdentifiers { id: id as i32, hash })
			},
			_ => Err("Could not deserialize ContractIdentifiers from stack item"),
		}
	}
}

impl From<InvocationResult> for ContractIdentifiers {
	fn from(result: InvocationResult) -> Self {
		let stack_item = &result.stack[0];
		ContractState::contract_identifiers(stack_item).unwrap_or_else(|e| {
			panic!("Failed to convert InvocationResult to ContractIdentifiers: {}", e)
		})
	}
}
