use primitive_types::H160;
use serde::{Deserialize, Serialize};

use crate::neo_types::contract::{
	ContractManifest,
	ContractNef,
	invocation_result::InvocationResult,
	ContractIdentifiers
};
use crate::neo_types::stack_item::StackItem;
use crate::neo_types::serde_with_utils::{serialize_h160, deserialize_h160};

#[derive(Clone, Debug, Hash, Serialize, Deserialize, PartialEq)]
pub struct ContractState {
	#[serde(default)]
	pub id: i32,
	pub nef: ContractNef,
	#[serde(rename = "updatecounter", default)]
	pub update_counter: i32,
	#[serde(serialize_with = "serialize_h160", deserialize_with = "deserialize_h160", default)]
	pub hash: H160,
	pub manifest: ContractManifest,
}

impl ContractState {
	pub fn new(
		id: i32,
		update_counter: i32,
		hash: H160,
		nef: ContractNef,
		manifest: ContractManifest,
	) -> Self {
		Self { id, nef, update_counter, hash, manifest }
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
				Ok(ContractIdentifiers::new(id as u32, hash, String::new()))
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
