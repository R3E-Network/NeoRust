use primitive_types::H160;
use serde::{Deserialize, Serialize};

use neo_common::{deserialize_script_hash, serialize_script_hash};
use neo_types::{ContractManifest, ScriptHash};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Hash)]
pub struct ExpressContractState {
	#[serde(serialize_with = "serialize_script_hash")]
	#[serde(deserialize_with = "deserialize_script_hash")]
	pub hash: ScriptHash,
	pub manifest: ContractManifest,
}

impl ExpressContractState {
	pub fn new(hash: H160, manifest: ContractManifest) -> Self {
		Self { hash, manifest }
	}
}

// PartialEq is now derived
