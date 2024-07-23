use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use neo::prelude::ContractMethodToken;

#[derive(Serialize, Deserialize, Default, Hash, Clone, Debug)]
#[serde_as]
pub struct ContractNef {
	pub magic: i32,
	pub compiler: String,
	pub source: String,
	#[serde_as(as = "Vec<ContractMethodToken>")]
	pub tokens: Vec<ContractMethodToken>,
	pub script: String,
	pub checksum: i32,
}

impl ContractNef {
	pub fn new(
		magic: i32,
		compiler: String,
		source: Option<String>,
		tokens: Vec<ContractMethodToken>,
		script: String,
		checksum: i32,
	) -> Self {
		Self {
			magic,
			compiler,
			source: source.unwrap_or_else(|| "".to_string()),
			tokens,
			script,
			checksum,
		}
	}
}
