use futures_util::TryFutureExt;
use getset::{CopyGetters, Getters, MutGetters, Setters};
use std::{
	error::Error,
	hash::{Hash, Hasher},
};

use neo::config::NeoConstants;
use primitive_types::{H160, H256, U256};
use rustc_serialize::hex::ToHex;
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize};
use serde_json::Value;
use serde_with::__private__::DeError;

use crate::neo_providers::{JsonRpcClient, Provider};
use neo::prelude::{
	deserialize_h256, deserialize_h256_option, deserialize_script_hash, serialize_h256,
	serialize_h256_option, serialize_script_hash, Bytes, Decoder, Encoder, HashableForVec,
	Middleware, NameOrAddress, NeoSerializable, RawTransaction, Signer, TransactionAttribute,
	TransactionError, VMState, VarSizeTrait, Witness,
};

#[derive(Serialize, Getters, Setters, MutGetters, CopyGetters, Debug, Clone)]
pub struct Transaction<P: JsonRpcClient + 'static> {
	#[serde(skip)]
	pub(crate) provider: Option<&'static Provider<P>>,

	#[serde(rename = "version")]
	pub version: u8,

	#[serde(rename = "nonce")]
	pub nonce: i32,

	#[serde(rename = "validuntilblock")]
	pub valid_until_block: i32,

	#[serde(rename = "signers")]
	pub signers: Vec<Signer>,

	#[serde(rename = "size")]
	pub size: i32,

	#[serde(rename = "sysfee")]
	pub sys_fee: i64,

	#[serde(rename = "netfee")]
	pub net_fee: i64,

	#[serde(rename = "attributes")]
	pub attributes: Vec<TransactionAttribute>,

	#[serde(rename = "script")]
	pub script: Bytes,

	#[serde(rename = "witnesses")]
	pub witnesses: Vec<Witness>,

	#[serde(rename = "blocktime")]
	pub block_time: Option<i32>,
}

impl<P: JsonRpcClient + 'static> Default for Transaction<P> {
	fn default() -> Self {
		Transaction {
			provider: None,
			version: Default::default(),
			nonce: Default::default(),
			valid_until_block: Default::default(),
			signers: Default::default(),
			size: Default::default(),
			sys_fee: Default::default(),
			net_fee: Default::default(),
			attributes: Default::default(),
			script: Default::default(),
			witnesses: Default::default(),
			block_time: Default::default(),
		}
	}
}

impl<'de, P: JsonRpcClient + 'static> Deserialize<'de> for Transaction<P> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		let value = Value::deserialize(deserializer)?;

		// Example for version, apply similar logic for other fields
		let version = value
			.get("version")
			.ok_or(DeError::missing_field("version"))?
			.as_u64()
			.ok_or(DeError::custom("invalid type for version"))? as u8;

		// Deserialize other fields similarly...
		let nonce = value["nonce"].as_i64().unwrap() as i32; // Simplified for brevity
		let valid_until_block = value["validuntilblock"].as_i64().unwrap() as i32;
		// Continue for other fields...

		// For Vec<T> fields like signers, attributes, witnesses, you might deserialize them like this:
		// This assumes that Signer, TransactionAttribute, Witness can be deserialized directly from serde_json::Value
		let signers: Vec<Signer> =
			serde_json::from_value(value["signers"].clone()).map_err(DeError::custom)?;
		let attributes: Vec<TransactionAttribute> =
			serde_json::from_value(value["attributes"].clone()).map_err(DeError::custom)?;
		let witnesses: Vec<Witness> =
			serde_json::from_value(value["witnesses"].clone()).map_err(DeError::custom)?;

		// For bytes, assuming it's a Vec<u8> and stored as a base64 string in JSON
		let script: Bytes = base64::decode(value["script"].as_str().unwrap_or_default())
			.map_err(DeError::custom)?;

		// For optional fields
		let block_time = value["blocktime"].as_i64().map(|v| v as i32);

		Ok(Transaction {
			provider: None, // Skipped in serialization, likely needs to be set up outside deserialization
			version,
			nonce,
			valid_until_block,
			signers,
			size: value["size"].as_i64().unwrap() as i32, // Simplified for brevity
			sys_fee: value["sysfee"].as_i64().unwrap(),
			net_fee: value["netfee"].as_i64().unwrap(),
			attributes,
			script,
			witnesses,
			block_time,
			// Fill in other fields as necessary
		})
	}
}

impl<P: JsonRpcClient + 'static> DeserializeOwned for Transaction<P> {}

impl<P: JsonRpcClient + 'static> Hash for Transaction<P> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.to_array().hash(state);
	}
}

impl<P: JsonRpcClient + 'static> Transaction<P> {
	const HEADER_SIZE: usize = 25;
	pub fn new() -> Self {
		Self::default()
	}

	pub fn with_provider(&mut self, provider: &'static Provider<P>) -> &mut Self {
		self.provider = Some(provider);
		self
	}

	/// Convenience function for sending a new payment transaction to the receiver.
	pub fn pay<K: Into<NameOrAddress>, V: Into<U256>>(_to: K, _value: V) -> Self {
		Transaction { ..Default::default() }
	}

	pub fn add_witness(&mut self, witness: Witness) {
		self.witnesses.push(witness);
	}

	pub async fn get_hash_data(&self) -> Result<Bytes, TransactionError> {
		if self.provider.is_none() {
			panic!("Transaction network magic is not set");
		}
		let mut encoder = Encoder::new();
		self.serialize_without_witnesses(&mut encoder);
		let mut data = encoder.to_bytes().hash256();
		data.splice(0..0, self.provider.unwrap().network().await);

		Ok(data)
	}

	fn serialize_without_witnesses(&self, writer: &mut Encoder) {
		writer.write_u8(self.version);
		writer.write_u32(self.nonce as u32);
		writer.write_i64(self.sys_fee);
		writer.write_i64(self.net_fee);
		writer.write_u32(self.valid_until_block as u32);
		writer.write_serializable_variable_list(&self.signers);
		writer.write_serializable_variable_list(&self.attributes);
		writer.write_var_bytes(&self.script);
	}
}

// #[async_trait]
impl<P: JsonRpcClient + 'static> Transaction<P> {
	pub(crate) async fn send(&self) -> Result<RawTransaction, TransactionError> {
		if self.signers.len() != self.witnesses.len() {
			return Err(TransactionError::TransactionConfiguration("The transaction does not have the same number of signers and witnesses. For every signer there has to be one witness, even if that witness is empty.".to_string()));
		}
		if self.size > NeoConstants::MAX_TRANSACTION_SIZE as i32 {
			return Err(TransactionError::TransactionConfiguration(format!("The transaction exceeds the maximum transaction size. The maximum size is {} bytes while the transaction has size {}.", NeoConstants::MAX_TRANSACTION_SIZE, self.size)));
		}

		let hex = self.to_array().to_hex();
		let block_count_when_sent = self.provider.unwrap().get_block_count().await?;
		let result = self.provider.unwrap().send_raw_transaction(hex).await?;
		Ok(result)
	}
}

impl<P: JsonRpcClient + 'static> Eq for Transaction<P> {}

impl<P: JsonRpcClient + 'static> PartialEq for Transaction<P> {
	fn eq(&self, other: &Self) -> bool {
		self.to_array() == other.to_array()
	}
}

impl<P: JsonRpcClient + 'static> NeoSerializable for Transaction<P> {
	type Error = TransactionError;

	fn size(&self) -> usize {
		Transaction::<P>::HEADER_SIZE
			+ self.signers.var_size()
			+ self.attributes.var_size()
			+ self.script.var_size()
			+ self.witnesses.var_size()
	}

	fn encode(&self, writer: &mut Encoder) {
		self.serialize_without_witnesses(writer);
		writer.write_serializable_variable_list(&self.witnesses);
	}

	fn decode(reader: &mut Decoder) -> Result<Self, Self::Error>
	where
		Self: Sized,
	{
		let version = reader.read_u8();
		let nonce = reader.read_u32();
		let system_fee = reader.read_i64();
		let network_fee = reader.read_i64();
		let valid_until_block = reader.read_u32();

		// Read signers
		let signers: Vec<Signer> = reader.read_serializable_list::<Signer>().unwrap();

		// Read attributes
		let attributes: Vec<TransactionAttribute> =
			reader.read_serializable_list::<TransactionAttribute>().unwrap();

		let script = reader.read_var_bytes().unwrap().to_vec();

		let mut witnesses = vec![];
		if reader.available() > 0 {
			witnesses.append(&mut reader.read_serializable_list::<Witness>().unwrap());
		}

		Ok(Self {
			provider: None,
			version,
			nonce: nonce as i32,
			valid_until_block: valid_until_block as i32,
			size: 0,
			sys_fee: system_fee,
			net_fee: network_fee,
			signers,
			attributes,
			script,
			witnesses,
			block_time: None,
		})
	}

	fn to_array(&self) -> Vec<u8> {
		let mut writer = Encoder::new();
		self.encode(&mut writer);
		writer.to_bytes()
	}
}
