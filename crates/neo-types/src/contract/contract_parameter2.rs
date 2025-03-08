use serde::{Deserialize, Serialize};

use crate::contract::contract_parameter_type::ContractParameterType;

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Debug)]
pub struct ContractParameter2 {
    pub name: String,
    #[serde(rename = "type")]
    pub typ: ContractParameterType,
}

impl ContractParameter2 {
    pub fn new(name: String, typ: ContractParameterType) -> Self {
        Self { name, typ }
    }
} 