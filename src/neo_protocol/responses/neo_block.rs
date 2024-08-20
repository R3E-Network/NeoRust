use std::num::ParseIntError;
use primitive_types::H256;
use serde::{Deserialize, Serialize};

use crate::prelude::RTransaction;
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
	pub nonce: String,
	pub index: i32,
	pub primary: Option<i32>,
	#[serde(rename = "nextconsensus")]
	pub next_consensus: String,
	pub witnesses: Option<Vec<NeoWitness>>,
	#[serde(rename = "tx", default = "default_transactions")]
	pub transactions: Option<Vec<RTransaction>>,
	pub confirmations: i32,
	#[serde(serialize_with = "serialize_h256_option")]
	#[serde(deserialize_with = "deserialize_h256_option")]
	#[serde(rename = "nextblockhash")]
	pub next_block_hash: Option<H256>,
}

impl NeoBlock {
    // Method to convert nonce from hexadecimal string to u64
    pub fn get_nonce_as_u64(&self) -> Result<u64, ParseIntError> {
        u64::from_str_radix(&self.nonce, 16)
    }
}

fn default_transactions() -> Option<Vec<RTransaction>> {
    Some(Vec::new())
}