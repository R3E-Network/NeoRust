use std::hash::{Hash, Hasher};
use std::str::FromStr;

use futures_util::TryFutureExt;
use getset::{CopyGetters, Getters, MutGetters, Setters};
use primitive_types::{U256, H256};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use serde_with::__private__::DeError;

use crate::{
	neo_clients::JsonRpcProvider,
	prelude::{NeoConstants, RawTransaction},
};
use neo::{
	prelude::{
		APITrait, ApplicationLog, Bytes, Decoder, Encoder, HashableForVec, NameOrAddress,
		NeoSerializable, NeoWitness, VMState, RpcClient, Signer, TransactionAttribute, TransactionError, TransactionSigner ,VarSizeTrait,
		Witness,
	},
	// types::ContractParameterType::H256,
};

#[derive(Serialize, Deserialize, Getters, Setters, MutGetters, CopyGetters, Debug, Clone, Hash)]
pub struct RTransaction {

	#[serde(rename = "hash")]
	#[getset(get = "pub", set = "pub")]
	pub hash: H256,

	#[serde(rename = "size")]
	#[getset(get = "pub", set = "pub")]
	pub size: u64,

	#[serde(rename = "version")]
	#[getset(get = "pub", set = "pub")]
	pub version: u8,

	#[serde(rename = "nonce")]
	#[getset(get = "pub", set = "pub")]
	pub nonce: u64,

	#[serde(rename = "sender")]
	#[getset(get = "pub", set = "pub")]
	pub sender: String,

	#[serde(rename = "sysfee")]
	#[getset(get = "pub", set = "pub")]
	pub sys_fee: String,

	#[serde(rename = "netfee")]
	#[getset(get = "pub", set = "pub")]
	pub net_fee: String,

	#[serde(rename = "validuntilblock")]
	#[getset(get = "pub", set = "pub")]
	pub valid_until_block: u64,

	#[serde(rename = "signers", default)]
	#[getset(get = "pub", set = "pub")]
	pub signers: Vec<TransactionSigner>,

	#[serde(rename = "attributes", default)]
	#[getset(get = "pub", set = "pub")]
	pub attributes: Vec<TransactionAttribute>,

	#[serde(rename = "script")]
	#[getset(get = "pub", set = "pub")]
	pub script: String,

	#[serde(rename = "witnesses", default)]
	#[getset(get = "pub", set = "pub")]
	pub witnesses: Vec<NeoWitness>,

	#[serde(rename = "blockhash", default)]
	#[getset(get = "pub", set = "pub")]
	pub block_hash: H256,

	#[serde(rename = "confirmations", default)]
	#[getset(get = "pub", set = "pub")]
	pub confirmations: i32,

	#[serde(rename = "blocktime", default)]
	#[getset(get = "pub", set = "pub")]
	pub block_time: i64,
	
	#[serde(rename = "vmstate", default)]
	#[getset(get = "pub", set = "pub")]
	pub vmstate: VMState,
}

impl RTransaction {
	pub fn new(hash: H256, size: u64, version:u8, nonce:u64, sender: String ,sys_fee: String, net_fee: String, valid_until_block: u64, signers: Vec<TransactionSigner>, attributes: Vec<TransactionAttribute>, script: String, witnesses: Vec<NeoWitness>) -> Self {
		Self {
			hash: hash,
			size: size,
			version: version,
			nonce: nonce,
			sender: sender,
			sys_fee: sys_fee,
			net_fee: net_fee,
			valid_until_block: valid_until_block,
			signers: signers,
			attributes: attributes,
			script: script,
			witnesses: witnesses,
			block_hash: Default::default(),
			confirmations: Default::default(),
			block_time: Default::default(),
			vmstate: Default::default(),
		}
	}
}

// impl Default for RTransaction {
// 	fn default() -> Self {
// 		Transaction {
// 			network: None,
// 			version: Default::default(),
// 			nonce: Default::default(),
// 			valid_until_block: Default::default(),
// 			signers: Default::default(),
// 			size: Default::default(),
// 			sys_fee: Default::default(),
// 			net_fee: Default::default(),
// 			attributes: Default::default(),
// 			script: Default::default(),
// 			witnesses: Default::default(),
// 			block_time: Default::default(),
// 			block_count_when_sent: None,
// 		}
// 	}
// }

// impl<'de> Deserialize<'de> for RTransaction {
// 	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
// 	where
// 		D: Deserializer<'de>,
// 	{
// 		let value = Value::deserialize(deserializer)?;

// 		// Example for version, apply similar logic for other fields
// 		let hash = H256::from_str(value["hash"].as_str().unwrap_or_default()).unwrap();
// 		let version = value
// 			.get("version")
// 			.ok_or(DeError::missing_field("version"))?
// 			.as_u64()
// 			.ok_or(DeError::custom("invalid type for version"))? as u8;

// 		// Deserialize other fields similarly...
// 		let nonce = value["nonce"].as_i64().unwrap() as u32; // Simplified for brevity
// 		let valid_until_block = value["validuntilblock"].as_i64().unwrap() as u32;
// 		let sender = value["sender"].as_str().unwrap().to_string();
// 		// Continue for other fields...

// 		// For Vec<T> fields like signers, attributes, witnesses, you might deserialize them like this:
// 		// This assumes that Signer, TransactionAttribute, Witness can be deserialized directly from serde_json::Value
// 		let signers: Vec<Signer> =
// 			serde_json::from_value(value["signers"].clone()).map_err(DeError::custom)?;
// 		let attributes: Vec<TransactionAttribute> =
// 			serde_json::from_value(value["attributes"].clone()).map_err(DeError::custom)?;
// 		let witnesses: Vec<NeoWitness> =
// 			serde_json::from_value(value["witnesses"].clone()).map_err(DeError::custom)?;

// 		// For bytes, assuming it's a Vec<u8> and stored as a base64 string in JSON
// 		let script: Bytes = base64::decode(value["script"].as_str().unwrap_or_default())
// 			.map_err(DeError::custom)?;

// 		// For optional fields
// 		let block_time = value["blocktime"].as_i64().map(|v| v as i32);

// 		Ok(RTransaction {
// 			network: None,
// 			hash,
// 			version,
// 			nonce,
// 			valid_until_block,
// 			signers,
// 			size: value["size"].as_i64().unwrap() as i32, // Simplified for brevity
// 			sender: sender,
// 			sys_fee: value["sysfee"].as_i64().unwrap(),
// 			net_fee: value["netfee"].as_i64().unwrap(),
// 			attributes,
// 			script,
// 			witnesses,
// 			block_time,
// 			// Fill in other fields as necessary
// 			block_count_when_sent: None,
// 		})
// 	}
// }

// impl<P: JsonRpcClient + 'static> DeserializeOwned for Transaction<P> {}

// impl Hash for Transaction {
// 	fn hash<H: Hasher>(&self, state: &mut H) {
// 		self.to_array().hash(state);
// 	}
// }





impl Eq for RTransaction {}

impl PartialEq for RTransaction {
    fn eq(&self, other: &Self) -> bool {
        self.size == other.size &&
        self.version == other.version &&
        self.hash == other.hash &&
        self.nonce == other.nonce &&
        self.sender == other.sender &&
        self.sys_fee == other.sys_fee &&
        self.net_fee == other.net_fee &&
        self.valid_until_block == other.valid_until_block &&
        self.signers == other.signers &&
        self.attributes == other.attributes &&
        self.script == other.script &&
        self.witnesses == other.witnesses &&
        self.block_hash == other.block_hash &&
        self.confirmations == other.confirmations &&
        self.block_time == other.block_time &&
        self.vmstate == other.vmstate
    }
}

// impl PartialEq for Transaction {
// 	fn eq(&self, other: &Self) -> bool {
// 		self.to_array() == other.to_array()
// 	}
// }