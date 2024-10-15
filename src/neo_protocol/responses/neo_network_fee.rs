use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
pub struct NeoNetworkFee {
	#[serde(rename = "networkfee")]
	pub network_fee: i64
}
