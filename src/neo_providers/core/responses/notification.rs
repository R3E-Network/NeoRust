use neo_types::{script_hash::ScriptHash, stack_item::StackItem, *};
use primitive_types::H160;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct Notification {
	#[serde(deserialize_with = "deserialize_script_hash")]
	#[serde(serialize_with = "serialize_script_hash")]
	pub contract: ScriptHash,
	#[serde(rename = "eventname")]
	pub event_name: String,
	pub state: StackItem,
}

impl Notification {
	pub fn new(contract: H160, event_name: String, state: StackItem) -> Self {
		Self { contract, event_name, state }
	}
}
