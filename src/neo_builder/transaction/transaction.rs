use std::hash::{Hash, Hasher};

use futures_util::TryFutureExt;
use getset::{CopyGetters, Getters, MutGetters, Setters};
use primitive_types::U256;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use serde_with::__private__::DeError;
use tracing::info;

use crate::{
	neo_clients::JsonRpcProvider,
	prelude::{init_logger, HttpProvider, NeoConstants, RawTransaction},
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
pub struct Transaction<'a, P: JsonRpcProvider + 'static> {
	#[serde(skip)]
	#[getset(get = "pub", set = "pub")]
	pub network: Option<&'a RpcClient<P>>,

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

	// #[serde(rename = "blocktime")]
	// #[getset(get = "pub", set = "pub")]
	// pub block_time: Option<i32>,
	#[serde(skip)]
	pub(crate) block_count_when_sent: Option<u32>,
}

impl<'a, P: JsonRpcProvider + 'static> Default for Transaction<'a, P> {
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
			// block_time: Default::default(),
			block_count_when_sent: None,
		}
	}
}

impl<'de, 'a, P: JsonRpcProvider + 'static> Deserialize<'de> for Transaction<'a, P> {
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
			// block_time,
			// Fill in other fields as necessary
			block_count_when_sent: None,
		})
	}
}

// impl<P: JsonRpcClient + 'static> DeserializeOwned for Transaction<P> {}

impl<'a, P: JsonRpcProvider + 'static> Hash for Transaction<'a, P> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.to_array().hash(state);
	}
}

impl<'a, T: JsonRpcProvider + 'static> Transaction<'a, T> {
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
		data.splice(0..0, self.network.as_ref().unwrap().network().await.to_be_bytes());

		Ok(data)
	}

	fn get_tx_id(&self) -> Result<primitive_types::H256, TransactionError> {
		let mut encoder = Encoder::new();
		self.serialize_without_witnesses(&mut encoder);
		let data = encoder.to_bytes().hash256();
		let reversed_data = data.iter().rev().cloned().collect::<Vec<u8>>();
		Ok(primitive_types::H256::from_slice(&reversed_data))
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

	pub async fn send_tx(&mut self) -> Result<RawTransaction, TransactionError>
// where
	// 	P: APITrait,
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
		self.block_count_when_sent = Some(self.network().unwrap().get_block_count().await?);
		self.network()
			.unwrap()
			.send_raw_transaction(hex)
			.await
			.map_err(|e| TransactionError::IllegalState(e.to_string()))
	}

	/// Tracks a transaction until it appears in a block.
	///
	/// This method waits for the transaction to be included in a block by monitoring new blocks
	/// as they are added to the blockchain. It returns when the transaction is found in a block.
	///
	/// # Arguments
	///
	/// * `max_blocks` - The maximum number of blocks to wait for the transaction to appear
	///
	/// # Returns
	///
	/// * `Ok(())` - If the transaction is found in a block
	/// * `Err(TransactionError)` - If the transaction is not found after waiting for `max_blocks` blocks
	///
	/// # Errors
	///
	/// Returns an error if:
	/// * The transaction has not been sent yet
	/// * The maximum number of blocks is reached without finding the transaction
	/// * There is an error communicating with the blockchain
	pub async fn track_tx(&self, max_blocks: u32) -> Result<(), TransactionError> {
		let block_count_when_sent =
			self.block_count_when_sent.ok_or(TransactionError::IllegalState(
				"Cannot track transaction before it has been sent.".to_string(),
			))?;
		
		let tx_id = self.get_tx_id()?;
		let mut current_block = block_count_when_sent;
		let max_block = block_count_when_sent + max_blocks;
		
		while current_block <= max_block {
			// Get the current block count
			let latest_block = self.network().unwrap().get_block_count().await?;
			
			// If there are new blocks, check them for our transaction
			if latest_block > current_block {
				for block_index in current_block..latest_block {
					// Get the block hash for this index
					let block_hash = self.network().unwrap().get_block_hash(block_index).await?;
					
					// Get the block with full transaction details
					let block = self.network().unwrap().get_block(block_hash, true).await?;
					
					// Check if our transaction is in this block
					if let Some(transactions) = &block.transactions {
						for tx in transactions.iter() {
							if tx.hash == tx_id {
								return Ok(());
							}
						}
					}
					
					current_block = block_index + 1;
				}
			}
			
			// Wait a bit before checking again
			tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
		}
		
		Err(TransactionError::IllegalState(format!(
			"Transaction {} not found after waiting for {} blocks",
			tx_id, max_blocks
		)))
	}

	pub async fn get_application_log<P>(
		&self,
		provider: &P,
	) -> Result<ApplicationLog, TransactionError>
	where
		P: APITrait,
	{
		init_logger();
		if self.block_count_when_sent.is_none() {
			return Err(TransactionError::IllegalState(
				"Cannot get the application log before transaction has been sent.".to_string(),
			));
		}

		let hash = self.get_tx_id()?;
		info!("hash: {:?}", hash);

		// self.thro
		provider
			.get_application_log(hash)
			.await
			.map_err(|e| TransactionError::IllegalState(e.to_string()))
	}
}

// This commented-out code has been replaced by the send_tx method above


impl<'a, P: JsonRpcProvider + 'static> Eq for Transaction<'a, P> {}

impl<'a, P: JsonRpcProvider + 'static> PartialEq for Transaction<'a, P> {
	fn eq(&self, other: &Self) -> bool {
		self.to_array() == other.to_array()
	}
}

impl<'a, P: JsonRpcProvider + 'static> NeoSerializable for Transaction<'a, P> {
	type Error = TransactionError;

	fn size(&self) -> usize {
		Transaction::<HttpProvider>::HEADER_SIZE
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
			// block_time: None,
			block_count_when_sent: None,
		})
	}

	fn to_array(&self) -> Vec<u8> {
		let mut writer = Encoder::new();
		self.encode(&mut writer);
		writer.to_bytes()
	}
}
