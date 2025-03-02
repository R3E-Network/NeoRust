use std::collections::HashMap;

use primitive_types::H160;

use crate::neo_types::contract::{
    contract_manifest::ContractManifest,
    contract_nef::ContractNef,
    invocation_result::InvocationResult,
    ContractIdentifiers
};
#[cfg(feature = "contract")]
use crate::neo_types::stack_item::StackItem;

#[cfg(not(feature = "contract"))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-support", derive(serde::Serialize, serde::Deserialize))]
pub struct StackItem {
    #[serde(default)]
    pub typ: String,
    #[serde(default)]
    pub value: String,
}
#[cfg(feature = "serde-support")]
use crate::neo_types::serde_with_utils::{serialize_h160, deserialize_h160};

#[derive(Clone, Debug, Hash, PartialEq)]
#[cfg_attr(feature = "serde-support", derive(serde::Serialize, serde::Deserialize))]
pub struct ContractState {
    pub id: i32,
    #[cfg_attr(feature = "serde-support", serde(serialize_with = "serialize_h160", deserialize_with = "deserialize_h160"))]
    pub hash: H160,
    pub nef: ContractNef,
    pub manifest: ContractManifest,
    pub update_counter: u32,
}

impl ContractState {
    pub fn new(
        id: i32,
        hash: H160,
        nef: ContractNef,
        manifest: ContractManifest,
        update_counter: u32,
    ) -> Self {
        Self { id, hash, nef, manifest, update_counter }
    }
}

impl ContractIdentifiers for ContractState {
    fn get_id(&self) -> i32 {
        self.id
    }

    fn get_hash(&self) -> H160 {
        self.hash
    }

    fn get_name(&self) -> String {
        self.manifest.name.clone()
    }
}
