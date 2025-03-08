use primitive_types::H160;
use serde::{Deserialize, Serialize};

use getset::{Getters, Setters};

use crate::{
	contract::ContractManifest,
	serde_with_utils::{deserialize_h160, serialize_h160},
};

#[derive(Serialize, Deserialize, Debug, Clone, Getters, Setters)]
pub struct NativeContractState {
	#[serde(serialize_with = "serialize_h160")]
	#[serde(deserialize_with = "deserialize_h160")]
	pub hash: H160,
	pub manifest: ContractManifest,
}

impl NativeContractState {
	pub fn new(
		hash: H160,
		manifest: ContractManifest,
	) -> Self {
		Self { hash, manifest }
	}
}
