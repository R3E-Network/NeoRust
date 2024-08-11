use std::hash::{Hash, Hasher};

use futures_util::TryFutureExt;
use getset::{CopyGetters, Getters, MutGetters, Setters};
use primitive_types::U256;
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
		NeoSerializable, RpcClient, Signer, TransactionAttribute, TransactionError, VarSizeTrait,
		Witness,
	},
	types::ContractParameterType::H256,
};

#[derive(Serialize, Getters, Setters, MutGetters, CopyGetters, Debug, Clone)]
pub struct Transaction {
	#[serde(skip)]
	#[getset(get = "pub", set = "pub")]
	pub network: Option<u32>,

	#[serde(rename = "version")]
	#[getset(get = "pub", set = "pub")]
	pub version: u8,

	#[serde(rename = "nonce")]
	#[getset(get = "pub", set = "pub")]
	pub nonce: u32,

	#[serde(rename = "validuntilblock")]
	#[getset(get = "pub", set = "pub")]
	pub valid_until_block: u32,

	#[serde(rename = "signers")]
	#[getset(get = "pub", set = "pub")]
	pub signers: Vec<Signer>,

	#[serde(rename = "size")]
	#[getset(get = "pub", set = "pub")]
	pub size: i32,

	#[serde(rename = "sysfee")]
	#[getset(get = "pub", set = "pub")]
	pub sys_fee: i64,

	#[serde(rename = "netfee")]
	#[getset(get = "pub", set = "pub")]
	pub net_fee: i64,

	#[serde(rename = "attributes")]
	#[getset(get = "pub", set = "pub")]
	pub attributes: Vec<TransactionAttribute>,

	#[serde(rename = "script")]
	#[getset(get = "pub", set = "pub")]
	pub script: Bytes,

	#[serde(rename = "witnesses")]
	#[getset(get = "pub", set = "pub")]
	pub witnesses: Vec<Witness>,

	#[serde(rename = "blocktime")]
	#[getset(get = "pub", set = "pub")]
	pub block_time: Option<i32>,
	#[serde(skip)]
	pub(crate) block_count_when_sent: Option<u32>,
}

impl Default for Transaction {
	fn default() -> Self {
		Transaction {
			network: None,
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
			block_count_when_sent: None,
		}
	}
}

impl<'de> Deserialize<'de> for Transaction {
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
		let nonce = value["nonce"].as_i64().unwrap() as u32; // Simplified for brevity
		let valid_until_block = value["validuntilblock"].as_i64().unwrap() as u32;
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
			network: None,
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
			block_count_when_sent: None,
		})
	}
}

// impl<P: JsonRpcClient + 'static> DeserializeOwned for Transaction<P> {}

impl Hash for Transaction {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.to_array().hash(state);
	}
}

impl Transaction {
	const HEADER_SIZE: usize = 25;
	pub fn new() -> Self {
		Self::default()
	}

	/// Convenience function for sending a new payment transaction to the receiver.
	pub fn pay<K: Into<NameOrAddress>, V: Into<U256>>(_to: K, _value: V) -> Self {
		Transaction { ..Default::default() }
	}

	pub fn add_witness(&mut self, witness: Witness) {
		self.witnesses.push(witness);
	}

	pub async fn get_hash_data(&self) -> Result<Bytes, TransactionError> {
		if self.network.is_none() {
			panic!("Transaction network magic is not set");
		}
		let mut encoder = Encoder::new();
		self.serialize_without_witnesses(&mut encoder);
		let mut data = encoder.to_bytes().hash256();
		data.splice(0..0, self.network.unwrap().to_be_bytes());

		Ok(data)
	}

	fn serialize_without_witnesses(&self, writer: &mut Encoder) {
		writer.write_u8(self.version);
		writer.write_u32(self.nonce);
		writer.write_i64(self.sys_fee);
		writer.write_i64(self.net_fee);
		writer.write_u32(self.valid_until_block);
		writer.write_serializable_variable_list(&self.signers);
		writer.write_serializable_variable_list(&self.attributes);
		writer.write_var_bytes(&self.script);
	}

	pub async fn send_tx<P>(&mut self, client: &P) -> Result<RawTransaction, TransactionError>
	where
		P: APITrait,
	{
		if self.signers.len() != self.witnesses.len() {
			return Err(TransactionError::TransactionConfiguration(
				"The transaction does not have the same number of signers and witnesses."
					.to_string(),
			));
		}
		if self.size() > &(NeoConstants::MAX_TRANSACTION_SIZE as i32) {
			return Err(TransactionError::TransactionConfiguration(
				"The transaction exceeds the maximum transaction size.".to_string(),
			));
		}
		let hex = hex::encode(self.to_array());
		// self.throw()?;
		self.block_count_when_sent = Some(client.get_block_count().await.unwrap());
		client
			.send_raw_transaction(hex)
			.await
			.map_err(|e| TransactionError::IllegalState(e.to_string()))
	}

	// TODO: implement this
	// pub async fn track_tx(
	// 	&self,
	// 	provider: Box<
	// 		dyn APITrait<
	// 			Error = TransactionError,
	// 			Provider = dyn JsonRpcProvider<Error = TransactionError>,
	// 		>,
	// 	>,
	// ) -> Result<(), TransactionError> {
	// 	let block_count_when_sent =
	// 		self.block_count_when_sent.ok_or(TransactionError::IllegalState(
	// 			"Cannot subscribe before transaction has been sent.".to_string(),
	// 		))?;
	// 	// self.throw_i
	// 	let predicate = |get_block: GetBlock| {
	// 		get_block.block.as_ref().map_or(true, |block| {
	// 			!block.transactions.iter().any(|tx| tx.hash == self.tx_id().unwrap())
	// 		})
	// 	};
	// 	provider
	// 		.as_ref()
	// 		.catch_up_to_latest_and_subscribe_to_new_blocks_publisher(block_count_when_sent, true)
	// 		.await?
	// 		.take_while(predicate)
	// 		.map(|block| block.index)
	// 		.collect::<Vec<_>>()
	// 		.await;
	// 	Ok(())
	// }

	pub async fn get_application_log<P>(
		&self,
		provider: &P,
	) -> Result<ApplicationLog, TransactionError>
	where
		P: APITrait,
	{
		if self.block_count_when_sent.is_none() {
			return Err(TransactionError::IllegalState(
				"Cannot get the application log before transaction has been sent.".to_string(),
			));
		}
		// self.thro
		provider
			.get_application_log(primitive_types::H256::from_slice(&self.get_hash_data().await?))
			.await
			.map_err(|e| TransactionError::IllegalState(e.to_string()))
	}
}

// impl<P: JsonRpcClient + 'static> Transaction<P> {
//
// pub(crate) async fn send(&self) -> Result<RawTransaction, TransactionError> {
// 	if self.signers.len() != self.witnesses.len() {
// 		return Err(TransactionError::TransactionConfiguration("The transaction does not have the same number of signers and witnesses. For every signer there has to be one witness, even if that witness is empty.".to_string()));
// 	}
// 	if self.size > NeoConstants::MAX_TRANSACTION_SIZE as i32 {
// 		return Err(TransactionError::TransactionConfiguration(format!("The transaction exceeds the maximum transaction size. The maximum size is {} bytes while the transaction has size {}.", NeoConstants::MAX_TRANSACTION_SIZE, self.size)));
// 	}
//
// 	let hex = self.to_array().to_hex();
// 	let block_count_when_sent = self.provider.unwrap().get_block_count().await?;
// 	let result = self.provider.unwrap().send_raw_transaction(hex).await?;
// 	Ok(result)
// }
// }

impl Eq for Transaction {}

impl PartialEq for Transaction {
	fn eq(&self, other: &Self) -> bool {
		self.to_array() == other.to_array()
	}
}

impl NeoSerializable for Transaction {
	type Error = TransactionError;

	fn size(&self) -> usize {
		Transaction::HEADER_SIZE
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
			network: None,
			version,
			nonce,
			valid_until_block,
			size: 0,
			sys_fee: system_fee,
			net_fee: network_fee,
			signers,
			attributes,
			script,
			witnesses,
			block_time: None,
			block_count_when_sent: None,
		})
	}

	fn to_array(&self) -> Vec<u8> {
		let mut writer = Encoder::new();
		self.encode(&mut writer);
		writer.to_bytes()
	}
}
