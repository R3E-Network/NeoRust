use neo::prelude::{Address, Bytes};
use primitive_types::{H256, U256};
use serde::{Deserialize, Serialize};

/// A log produced by a transaction.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Log {
	/// H160. the contract that emitted the log
	pub address: Address,

	/// topics: Array of 0 to 4 32 Bytes of indexed log arguments.
	/// (In solidity: The first topic is the hash of the signature of the event
	/// (e.g. `Deposit(address,bytes32,uint256)`), except you declared the event
	/// with the anonymous specifier.)
	#[serde(serialize_with = "serialize_vec_h256")]
	#[serde(deserialize_with = "deserialize_vec_h256")]
	pub topics: Vec<H256>,

	/// Data
	pub data: Bytes,

	/// Block Hash
	#[serde(rename = "blockHash")]
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(serialize_with = "serialize_h256_option")]
	#[serde(deserialize_with = "deserialize_h256_option")]
	pub block_hash: Option<H256>,

	/// Block Number
	#[serde(rename = "blockNumber")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub block_number: Option<u64>,

	/// Transaction Hash
	#[serde(rename = "transactionHash")]
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(serialize_with = "serialize_h256_option")]
	#[serde(deserialize_with = "deserialize_h256_option")]
	pub transaction_hash: Option<H256>,

	/// Transaction Index
	#[serde(rename = "transactionIndex")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub transaction_index: Option<u64>,

	/// Integer of the log index position in the block. None if it's a pending log.
	#[serde(rename = "logIndex")]
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(serialize_with = "serialize_u256_option")]
	#[serde(deserialize_with = "deserialize_u256_option")]
	pub log_index: Option<U256>,

	/// Integer of the transactions index position log was created from.
	/// None when it's a pending log.
	#[serde(rename = "transactionLogIndex")]
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(serialize_with = "serialize_u256_option")]
	#[serde(deserialize_with = "deserialize_u256_option")]
	pub transaction_log_index: Option<U256>,

	/// Log Type
	#[serde(rename = "logType")]
	#[serde(skip_serializing_if = "Option::is_none")]
	pub log_type: Option<String>,

	/// True when the log was removed, due to a chain reorganization.
	/// false if it's a valid log.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub removed: Option<bool>,
}
