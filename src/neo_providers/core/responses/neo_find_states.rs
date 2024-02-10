use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct States {
	#[serde(rename = "firstproof")]
	pub first_proof: Option<String>,
	#[serde(rename = "lastproof")]
	pub last_proof: Option<String>,
	pub truncated: bool,
	pub results: Vec<Result>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Result {
	pub key: String,
	pub value: String,
}
