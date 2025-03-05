use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct States {
	#[serde(rename = "firstProof")]
	pub first_proof: Option<String>,
	#[serde(rename = "lastProof")]
	pub last_proof: Option<String>,
	pub truncated: bool,
	#[serde(default)]
	pub results: Vec<StateResult>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct StateResult {
	pub key: String,
	pub value: String,
}
