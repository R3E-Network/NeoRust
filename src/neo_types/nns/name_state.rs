use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct NameState {
	pub name: String,
	pub expiration: Option<i64>,
	pub admin: Option<[u8; 20]>,
}

impl NameState {
	pub fn new(name: String, expiration: Option<i64>, admin: Option<[u8; 20]>) -> Self {
		Self { name, expiration, admin }
	}
}
