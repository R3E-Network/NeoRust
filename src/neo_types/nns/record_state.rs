use std::hash::{Hash, Hasher};

use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};

#[cfg(feature = "contract")]
use crate::neo_types::stack_item::StackItem;

#[cfg(not(feature = "contract"))]
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct StackItem {
	#[serde(rename = "type")]
	pub type_: String,
	pub value: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RecordState {
	pub name: String,
	pub record_type: RecordType,
	pub data: String,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum RecordType {
	A = 0x01,
	AAAA = 0x02,
	CNAME = 0x04,
	Delete = 0x08,
}

impl RecordState {
	pub fn new(name: String, record_type: RecordType, data: String) -> Self {
		Self { name, record_type, data }
	}

	pub fn from_stack_item(item: &StackItem) -> Result<Self, &'static str> {
		match item {
			#[cfg(feature = "contract")]
			crate::neo_types::stack_item::StackItem::Array { value: vec } if vec.len() == 3 => {
				if let Some(name) = vec[0].as_string() {
					if let Some(byte) = vec[1].as_int() {
						if let Some(record_type) = RecordType::try_from(byte as u8).ok() {
							if let Some(data) = vec[2].as_string() {
								return Ok(Self::new(name, record_type, data));
							}
						}
					}
				}
				Err("Could not deserialize RecordState")
			},
			#[cfg(not(feature = "contract"))]
			_ => {
				// Handle the non-contract feature case with our simplified StackItem
				Err("StackItem Array support requires the contract feature")
			},
			_ => Err("Expected a StackItem array of length 3"),
		}
	}
}
