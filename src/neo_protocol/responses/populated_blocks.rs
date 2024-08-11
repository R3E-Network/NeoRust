use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct PopulatedBlocks {
	pub cache_id: String,
	pub blocks: Vec<i32>,
}

impl PopulatedBlocks {
	pub fn new(cache_id: String, blocks: Vec<i32>) -> Self {
		Self { cache_id, blocks }
	}
}
