use std::hash::Hash;

use primitive_types::H160;
use serde::{Deserialize, Serialize};

pub trait TokenBalances<'a>: Serialize + Deserialize<'a> + Clone + PartialEq + Eq + Hash {
	type Balance: TokenBalance<'a>;
	fn address(&self) -> String;
	fn balances(&self) -> &Vec<Self::Balance>;
}

pub trait TokenBalance<'a>: Serialize + Deserialize<'a> + Clone {
	fn asset_hash(&self) -> H160;
}
