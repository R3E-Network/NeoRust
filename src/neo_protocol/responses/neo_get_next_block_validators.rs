use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Validator {
	#[serde(rename = "publickey")]
	pub public_key: String,
	pub votes: String,
	pub active: bool,
}

impl Validator {
	pub fn new(public_key: String, votes: String, active: bool) -> Self {
		Self {public_key, votes, active }	
	}
}
