use std::collections::HashMap;

#[cfg(feature = "getset-macros")]
use getset::{Getters, Setters};
#[cfg(feature = "serde-support")]
use serde::{Deserialize, Serialize};

use crate::neo_types::contract::ContractState;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "getset-macros", derive(Getters, Setters))]
#[cfg_attr(feature = "serde-support", derive(Serialize, Deserialize))]
pub struct NativeContractState {
    #[cfg_attr(feature = "getset-macros", get = "pub")]
    contract_state: ContractState,
    #[cfg_attr(feature = "getset-macros", get = "pub")]
    service_name: String,
}

impl NativeContractState {
    pub fn new(contract_state: ContractState, service_name: String) -> Self {
        Self { contract_state, service_name }
    }

    pub fn get_contract_state(&self) -> &ContractState {
        &self.contract_state
    }

    pub fn get_service_name(&self) -> &String {
        &self.service_name
    }
}
