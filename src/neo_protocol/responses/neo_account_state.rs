use std::hash::{Hash, Hasher};

use crate::{
	crypto::{PublicKeyExtension, Secp256r1PublicKey},
	deserialize_public_key_option, serialize_public_key_option,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct AccountState {
	pub balance: i64,
	pub balance_height: Option<i64>,
	#[serde(deserialize_with = "deserialize_public_key_option")]
	#[serde(serialize_with = "serialize_public_key_option")]
	pub public_key: Option<Secp256r1PublicKey>,
}

impl Hash for AccountState {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.balance.hash(state);
		self.balance_height.hash(state);

		// Only hash the public key if it exists
		if let Some(public_key) = &self.public_key {
			public_key.to_vec().hash(state);
		}
	}
}

impl AccountState {
	pub fn with_no_vote(balance: i64, update_height: i64) -> Self {
		Self { balance, balance_height: Some(update_height), public_key: None }
	}

	pub fn with_no_balance() -> Self {
		Self { balance: 0, balance_height: None, public_key: None }
	}
}
