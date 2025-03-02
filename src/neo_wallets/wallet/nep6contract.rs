use serde_derive::{Deserialize, Serialize};

/// Represents a NEP-6 contract in a wallet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NEP6Contract {
	/// The script content, usually base64 encoded
	pub script: Option<String>,
	/// Parameters for the contract
	#[serde(rename = "parameters")]
	pub nep6_parameters: Vec<NEP6Parameter>,
	/// Whether this contract is deployed
	#[serde(rename = "deployed")]
	pub is_deployed: bool,
}

/// Represents a parameter in a NEP-6 contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NEP6Parameter {
	/// Parameter name
	pub param_name: String,
	/// Parameter type
	#[serde(rename = "type")]
	pub param_type: String,
}

impl NEP6Contract {
	/// Creates a new NEP-6 contract.
	pub fn new(script: Option<String>, parameters: Vec<NEP6Parameter>, is_deployed: bool) -> Self {
		Self {
			script,
			nep6_parameters: parameters,
			is_deployed,
		}
	}

	/// Returns a reference to the script.
	pub fn script(&self) -> &Option<String> {
		&self.script
	}

	/// Returns a reference to the parameters.
	pub fn parameters(&self) -> &Vec<NEP6Parameter> {
		&self.nep6_parameters
	}

	/// Checks if the contract is deployed.
	pub fn is_deployed(&self) -> bool {
		self.is_deployed
	}
}

impl NEP6Parameter {
	/// Creates a new NEP-6 parameter.
	pub fn new(name: String, param_type: String) -> Self {
		Self {
			param_name: name,
			param_type,
		}
	}

	/// Returns a reference to the parameter name.
	pub fn param_name(&self) -> &String {
		&self.param_name
	}

	/// Returns a reference to the parameter type.
	pub fn param_type(&self) -> &String {
		&self.param_type
	}
}

impl Default for NEP6Contract {
	fn default() -> Self {
		Self {
			script: None,
			nep6_parameters: Vec::new(),
			is_deployed: false,
		}
	}
}
