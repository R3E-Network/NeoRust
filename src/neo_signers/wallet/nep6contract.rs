use neo_types::contract_parameter_type::ContractParameterType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NEP6Contract {
	#[serde(rename = "script")]
	pub script: Option<String>,

	#[serde(rename = "deployed")]
	pub is_deployed: bool,

	#[serde(rename = "parameters")]
	pub nep6_parameters: Vec<NEP6Parameter>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NEP6Parameter {
	#[serde(rename = "name")]
	pub param_name: String,
	#[serde(rename = "type")]
	pub param_type: ContractParameterType,
}

impl PartialEq for NEP6Contract {
	fn eq(&self, other: &Self) -> bool {
		self.script == other.script
			&& self.nep6_parameters == other.nep6_parameters
			&& self.is_deployed == other.is_deployed
	}
}
