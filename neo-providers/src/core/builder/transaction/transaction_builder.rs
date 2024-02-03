/// This module contains the implementation of the `TransactionBuilder` struct, which is used to build and configure transactions.
///
/// The `TransactionBuilder` struct has various fields that can be set using its methods. Once the fields are set, the `get_unsigned_tx` method can be called to obtain an unsigned transaction.
///
/// The `TransactionBuilder` struct implements various traits such as `Debug`, `Clone`, `Eq`, `PartialEq`, and `Hash`.
///
/// # Example
///
/// ```
///
/// use neo_providers::core::transaction::transaction_builder::TransactionBuilder;
/// let mut tx_builder = TransactionBuilder::new();
/// tx_builder.version(0)
///           .nonce(1)
///           .valid_until_block(100)
///           .set_script(vec![0x01, 0x02, 0x03])
///           .get_unsigned_tx();
/// ```
use getset::{CopyGetters, Getters, MutGetters, Setters};
use neo_codec::encode::NeoSerializable;
use neo_config::NeoConstants;
use neo_types::{
	contract_parameter::ContractParameter, public_key_to_script_hash, script_hash::ScriptHash,
	Bytes,
};
use once_cell::sync::Lazy;
use primitive_types::H160;
use rustc_serialize::hex::ToHex;
use serde::Serialize;
use std::{
	collections::HashSet,
	fmt::Debug,
	hash::{Hash, Hasher},
	iter::Iterator,
	str::FromStr,
};

use crate::{
	core::{
		account::{Account, AccountTrait},
		builder::{
			error::BuilderError,
			transaction::{transaction_error::TransactionError, witness::Witness},
		},
		transaction::{
			signers::{
				signer::{Signer, SignerType},
				transaction_signer::TransactionSigner,
			},
			transaction::Transaction,
			transaction_attribute::TransactionAttribute,
		},
	},
	JsonRpcClient, Middleware, Provider,
};

#[derive(Getters, Setters, MutGetters, CopyGetters, Default)]
pub struct TransactionBuilder<P: JsonRpcClient + 'static> {
	provider: Option<&'static Provider<P>>,
	version: u8,
	nonce: u32,
	valid_until_block: Option<u32>,
	// setter and getter
	#[getset(get = "pub", set = "pub")]
	signers: Vec<Signer>,
	additional_network_fee: u64,
	additional_system_fee: u64,
	attributes: Vec<TransactionAttribute>,
	script: Option<Bytes>,
	fee_consumer: Option<Box<dyn Fn(u64, u64)>>,
	fee_error: Option<TransactionError>,
}

impl<P: JsonRpcClient> Debug for TransactionBuilder<P> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("TransactionBuilder")
			.field("version", &self.version)
			.field("nonce", &self.nonce)
			.field("valid_until_block", &self.valid_until_block)
			.field("signers", &self.signers)
			.field("additional_network_fee", &self.additional_network_fee)
			.field("additional_system_fee", &self.additional_system_fee)
			.field("attributes", &self.attributes)
			.field("script", &self.script)
			// .field("fee_consumer", &self.fee_consumer)
			.field("fee_error", &self.fee_error)
			.finish()
	}
}

impl<P: JsonRpcClient> Clone for TransactionBuilder<P> {
	fn clone(&self) -> Self {
		Self {
			provider: self.provider,
			version: self.version,
			nonce: self.nonce,
			valid_until_block: self.valid_until_block,
			signers: self.signers.clone(),
			additional_network_fee: self.additional_network_fee,
			additional_system_fee: self.additional_system_fee,
			attributes: self.attributes.clone(),
			script: self.script.clone(),
			// fee_consumer: self.fee_consumer.clone(),
			fee_consumer: None,
			fee_error: None,
		}
	}
}

impl<P: JsonRpcClient> Eq for TransactionBuilder<P> {}

impl<P: JsonRpcClient> PartialEq for TransactionBuilder<P> {
	fn eq(&self, other: &Self) -> bool {
		self.version == other.version
			&& self.nonce == other.nonce
			&& self.valid_until_block == other.valid_until_block
			&& self.signers == other.signers
			&& self.additional_network_fee == other.additional_network_fee
			&& self.additional_system_fee == other.additional_system_fee
			&& self.attributes == other.attributes
			&& self.script == other.script
	}
}

impl<P: JsonRpcClient> Hash for TransactionBuilder<P> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.version.hash(state);
		self.nonce.hash(state);
		self.valid_until_block.hash(state);
		self.signers.hash(state);
		self.additional_network_fee.hash(state);
		self.additional_system_fee.hash(state);
		self.attributes.hash(state);
		self.script.hash(state);
	}
}

static GAS_TOKEN_HASH: Lazy<ScriptHash> =
	Lazy::new(|| ScriptHash::from_str("d2a4cff31913016155e38e474a2c06d08be276cf").unwrap());

impl<P: JsonRpcClient> TransactionBuilder<P> {
	// const GAS_TOKEN_HASH: ScriptHash = ScriptHash::from_str("d2a4cff31913016155e38e474a2c06d08be276cf").unwrap();
	pub const BALANCE_OF_FUNCTION: &'static str = "balanceOf";
	pub const DUMMY_PUB_KEY: &'static str =
		"02ec143f00b88524caf36a0121c2de09eef0519ddbe1c710a00f0e2663201ee4c0";

	// Constructor
	pub fn new() -> Self {
		Self {
			provider: None,
			version: 0,
			nonce: 0,
			valid_until_block: None,
			signers: Vec::new(),
			additional_network_fee: 0,
			additional_system_fee: 0,
			attributes: Vec::new(),
			script: None,
			fee_consumer: None,
			fee_error: None,
		}
	}

	// Configuration

	pub fn version(&mut self, version: u8) -> &mut Self {
		self.version = version;
		self
	}

	pub fn nonce(&mut self, nonce: u32) -> Result<&mut Self, TransactionError> {
		// Validate
		if nonce >= u32::MAX {
			return Err(TransactionError::InvalidNonce)
		}

		self.nonce = nonce;
		Ok(self)
	}

	// Other methods

	// Set valid until block
	pub fn valid_until_block(&mut self, block: u32) -> Result<&mut Self, TransactionError> {
		if block == 0 {
			return Err(TransactionError::InvalidBlock)
		}

		self.valid_until_block = Some(block);
		Ok(self)
	}

	// Set script
	pub fn set_script(&mut self, script: Bytes) -> &mut Self {
		self.script = Some(script);
		self
	}

	// Get unsigned transaction
	pub async fn get_unsigned_tx(&mut self) -> Result<Transaction, TransactionError> {
		// Validate configuration
		if self.signers.is_empty() {
			return Err(TransactionError::NoSigners)
		}

		if self.script.is_none() {
			return Err(TransactionError::NoScript)
		}
		let len = self.signers.len();
		self.signers.dedup();

		// Validate no duplicate signers
		if len != self.signers.len() {
			return Err(TransactionError::DuplicateSigner)
		}

		// Check signer limits
		if self.signers.len() > NeoConstants::MAX_SIGNER_SUBITEMS as usize {
			return Err(TransactionError::TooManySigners)
		}

		// Validate script
		if let Some(script) = &self.script {
			if script.is_empty() {
				return Err(TransactionError::EmptyScript)
			}
		} else {
			return Err(TransactionError::NoScript)
		}

		let mut tx = Transaction::new();
		// 	self.version,
		// 	self.nonce,
		// 	self.valid_until_block.unwrap(),
		// 	self.clone().signers,
		// 	0,
		// 	0,
		// 	self.clone().attributes,
		// 	self.clone().script.unwrap(),
		// 	vec![],
		// );

		// Get fees
		let system_fee = 0; //self.get_system_fee().await.unwrap();
		let network_fee = 0; //self.get_network_fee(&tx).await.unwrap();

		// Check sender balance if needed
		if let Some(fee_consumer) = &self.fee_consumer {
			let sender_balance = 0; // self.get_sender_balance().await.unwrap();
			if network_fee + system_fee > sender_balance {
				fee_consumer(network_fee + system_fee, sender_balance);
			}
		}

		// tx.set_network_fee(network_fee as i64);
		// tx.set_system_fee(system_fee as i64);
		// Build transaction
		Ok(tx)
	}

	// async fn get_system_fee(&self) -> Result<u64, TransactionError> {
	// 	let script = self.script.as_ref().unwrap();
	//
	// 	let response = NEO_INSTANCE
	// 		.read()
	// 		.unwrap()
	// 		.invoke_script(script.to_hex(), vec![self.signers[0].clone()])
	// 		.request()
	// 		.await
	// 		.unwrap();
	// 	Ok(u64::from_str(response.gas_consumed.as_str()).unwrap()) // example
	// }

	async fn get_network_fee(&mut self, tx: &Transaction) -> Result<u64, TransactionError> {
		let fee = self.provider.unwrap().calculate_network_fee(tx.to_array().to_hex()).await?;
		Ok(fee)
	}

	async fn get_sender_balance(&self) -> Result<u64, TransactionError> {
		// Call network
		let sender = &self.signers[0];

		if Self::is_account_signer(sender) {
			let balance = self
				.provider
				.unwrap()
				.invoke_function(
					&GAS_TOKEN_HASH,
					Self::BALANCE_OF_FUNCTION.to_string(),
					vec![ContractParameter::from(sender.get_signer_hash())],
					None,
				)
				.await?
				.stack[0]
				.clone();
			return Ok(balance.as_int().unwrap() as u64)
		}
		Err(TransactionError::InvalidSender)
	}

	fn is_account_signer(signer: &Signer) -> bool {
		// let sig = <T as Signer>::SignerType;
		if signer.get_type() == SignerType::Account {
			return true
		}
		return false
	}

	// Sign transaction
	pub async fn sign(&mut self) -> Result<Transaction, BuilderError> {
		let mut transaction = self.get_unsigned_tx().await.unwrap();
		let tx_bytes = transaction.get_hash_data()?;

		let mut witnesses_to_add = Vec::new();

		for signer in &mut transaction.signers {
			if Self::is_account_signer(signer) {
				let account_signer = signer.as_account_signer().unwrap();
				let acc = &account_signer.account;
				if acc.is_multi_sig() {
					return Err(BuilderError::IllegalState(
						"Transactions with multi-sig signers cannot be signed automatically."
							.to_string(),
					))
				}

				let key_pair = acc.key_pair().as_ref().ok_or_else(|| {
					BuilderError::InvalidConfiguration(
						"Cannot create transaction signature because account does not hold a private key.".to_string(),
					)
				})?;

				witnesses_to_add.push(Witness::create(tx_bytes.clone(), key_pair)?);
			} else {
				let contract_signer = signer.as_contract_signer().unwrap();
				witnesses_to_add
					.push(Witness::create_contract_witness(contract_signer.verify_params.clone())?);
			}
		}

		for witness in witnesses_to_add {
			transaction.add_witness(witness);
		}

		Ok(transaction)
	}

	fn signers_contain_multi_sig_with_committee_member(&self, committee: &HashSet<H160>) -> bool {
		for signer in &self.signers {
			if let Some(account_signer) = signer.as_account_signer() {
				if account_signer.is_multi_sig() {
					if let Some(script) = &account_signer.account().verification_script() {
						for pubkey in script.get_public_keys().unwrap() {
							let hash = public_key_to_script_hash(&pubkey);
							if committee.contains(&hash) {
								return true
							}
						}
					}
				}
			}
		}

		false
	}

	pub fn is_high_priority(&self) -> bool {
		self.attributes
			.iter()
			.any(|attr| matches!(attr, TransactionAttribute::HighPriority))
	}

	// async fn can_send_cover_fees(&self, fees: u64) -> Result<bool, BuilderError> {
	// 	let balance = self.get_sender_gas_balance().await?;
	// 	Ok(balance >= fees)
	// }

	// async fn get_sender_gas_balance(&self) -> Result<u64, BuilderError> {
	// 	let sender_hash = self.signers[0].get_signer_hash();
	// 	let result = NEO_INSTANCE
	// 		.read()
	// 		.unwrap()
	// 		.invoke_function(
	// 			&H160::from(Self::GAS_TOKEN_HASH),
	// 			Self::BALANCE_OF_FUNCTION.to_string(),
	// 			vec![sender_hash.into()],
	// 			vec![],
	// 		)
	// 		.request()
	// 		.await?;
	//
	// 	Ok(result.stack[0].as_int().unwrap() as u64)
	// }
}
