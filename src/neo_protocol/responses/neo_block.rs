use primitive_types::H256;
use serde::{Deserialize, Serialize};

use crate::prelude::Transaction;
use neo::prelude::{
	deserialize_h256, deserialize_h256_option, serialize_h256, serialize_h256_option, NeoWitness,
};

#[derive(Serialize, Deserialize, Hash, Clone, Debug)]
pub struct NeoBlock {
	// Transaction, TransactionResult
	#[serde(serialize_with = "serialize_h256")]
	#[serde(deserialize_with = "deserialize_h256")]
	pub hash: H256,
	pub size: i32,
	pub version: i32,
	#[serde(serialize_with = "serialize_h256")]
	#[serde(deserialize_with = "deserialize_h256")]
	#[serde(rename = "previousblockhash")]
	pub prev_block_hash: H256,
	#[serde(serialize_with = "serialize_h256")]
	#[serde(deserialize_with = "deserialize_h256")]
	#[serde(rename = "merkleroot")]
	pub merkle_root_hash: H256,
	pub time: i32,
	pub index: i32,
	pub primary: Option<i32>,
	#[serde(rename = "nextconsensus")]
	pub next_consensus: String,
	pub witnesses: Option<Vec<NeoWitness>>,
	pub transactions: Option<Vec<Transaction>>,
	pub confirmations: i32,
	#[serde(serialize_with = "serialize_h256_option")]
	#[serde(deserialize_with = "deserialize_h256_option")]
	#[serde(rename = "nextblockhash")]
	pub next_block_hash: Option<H256>,
}
