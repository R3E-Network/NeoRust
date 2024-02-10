use crate::{
	core::{
		account::AccountTrait,
		transaction::{
			signers::{signer::Signer, transaction_signer::TransactionSigner},
			transaction_attribute::TransactionAttribute,
			transaction_error::TransactionError,
			witness::Witness,
		},
	},
	JsonRpcClient,
};
use neo_codec::{
	encode::{NeoSerializable, VarSizeTrait},
	Decoder, Encoder,
};
use neo_crypto::hash::HashableForVec;
use neo_types::{address::NameOrAddress, vm_state::VMState, *};
use primitive_types::{H160, H256, U256};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Default, Serialize, Deserialize, Hash, Debug, Clone)]
pub struct Transaction {
	#[serde(rename = "version")]
	pub version: u8,

	#[serde(rename = "nonce")]
	pub nonce: i32,

	#[serde(rename = "validuntilblock")]
	pub valid_until_block: i32,

	#[serde(rename = "hash")]
	#[serde(serialize_with = "serialize_h256")]
	#[serde(deserialize_with = "deserialize_h256")]
	pub hash: H256,

	#[serde(rename = "size")]
	pub size: i32,

	#[serde(rename = "sender")]
	#[serde(deserialize_with = "deserialize_script_hash")]
	#[serde(serialize_with = "serialize_script_hash")]
	pub sender: H160,

	#[serde(rename = "sysfee")]
	pub sys_fee: i64,

	#[serde(rename = "netfee")]
	pub net_fee: i64,

	#[serde(rename = "signers")]
	pub signers: Vec<Signer>,

	#[serde(rename = "attributes")]
	pub attributes: Vec<TransactionAttribute>,

	#[serde(rename = "script")]
	pub script: Bytes,

	#[serde(rename = "witnesses")]
	pub witnesses: Vec<Witness>,

	#[serde(rename = "blockhash")]
	#[serde(serialize_with = "serialize_h256_option")]
	#[serde(deserialize_with = "deserialize_h256_option")]
	pub block_hash: Option<H256>,

	#[serde(rename = "confirmations")]
	pub confirmations: Option<i32>,

	#[serde(rename = "blocktime")]
	pub block_time: Option<i32>,

	#[serde(rename = "vmstate")]
	pub vm_state: Option<VMState>,

	#[serde(rename = "network")]
	pub network_magic: Option<u32>,
}

impl Transaction {
	const HEADER_SIZE: usize = 25;
	pub fn new() -> Self {
		Self::default()
	}

	/// Convenience function for sending a new payment transaction to the receiver.
	pub fn pay<K: Into<NameOrAddress>, V: Into<U256>>(to: K, value: V) -> Self {
		Transaction { ..Default::default() }
	}

	pub fn network_magic(&self) -> Option<u32> {
		self.network_magic
	}

	pub fn set_network_magic(&mut self, network_magic: u32) {
		self.network_magic = Some(network_magic);
	}

	pub fn add_witness(&mut self, witness: Witness) {
		self.witnesses.push(witness);
	}

	pub fn get_hash_data(&self) -> Result<Bytes, TransactionError> {
		if self.network_magic().is_none() {
			panic!("Transaction network magic is not set");
		}
		let mut encoder = Encoder::new();
		self.serialize_without_witnesses(&mut encoder);
		let mut data = encoder.to_bytes().hash256();
		data.splice(0..0, self.network_magic.unwrap().to_be_bytes());

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

impl Eq for Transaction {}

impl PartialEq for Transaction {
	fn eq(&self, other: &Self) -> bool {
		self.hash == other.hash
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
		if (reader.available() > 0) {
			witnesses.append(&mut reader.read_serializable_list::<Witness>().unwrap());
		}

		Ok(Self {
			version,
			nonce: nonce as i32,
			valid_until_block: valid_until_block as i32,
			hash: Default::default(),
			size: 0,
			sender: Default::default(),
			sys_fee: system_fee,
			net_fee: network_fee,
			signers,
			attributes,
			script,
			witnesses,
			block_hash: None,
			confirmations: None,
			block_time: None,
			vm_state: None,
			network_magic: None,
		})
	}

	fn to_array(&self) -> Vec<u8> {
		let mut writer = Encoder::new();
		self.encode(&mut writer);
		writer.to_bytes()
	}
}
