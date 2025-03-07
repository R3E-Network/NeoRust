use primitive_types::H160;
use serde::{Deserialize, Serialize};

use crate::ContractNef;
use getset::{CopyGetters, Getters, MutGetters, Setters};
use crate::ContractManifest;

#[derive(Serialize, Deserialize, Getters, Setters, Default, Debug, Clone)]
pub struct NativeContractState {
	pub id: i32,
	pub nef: ContractNef,
#[serde(serialize_with = "crate::serde_utils::serialize_h160")]
	#[serde(deserialize_with = "crate::serde_utils::deserialize_h160")]
	#[getset(get = "pub")]
	hash: H160,
	#[getset(get = "pub")]
	manifest: ContractManifest,
	// #[serde(rename = "updatehistory")]
	// pub update_history: Vec<i32>,
}

impl NativeContractState {
	pub fn new(
		id: i32,
		hash: H160,
		nef: ContractNef,
		manifest: ContractManifest,
		// update_history: Vec<i32>,
	) -> Self {
		Self { id, nef, hash, manifest }
	}
}
