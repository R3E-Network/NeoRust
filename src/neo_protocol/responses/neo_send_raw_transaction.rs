use primitive_types::H256;

use neo::prelude::{deserialize_h256, serialize_h256};

#[derive(Debug, Hash, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize, Clone)]
pub struct RawTransaction {
	#[serde(serialize_with = "serialize_h256")]
	#[serde(deserialize_with = "deserialize_h256")]
	pub hash: H256,
}
