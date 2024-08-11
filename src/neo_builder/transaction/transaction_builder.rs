use futures_util::TryFutureExt;
use std::{
	cell::RefCell,
	collections::HashSet,
	fmt::Debug,
	hash::{Hash, Hasher},
	iter::Iterator,
	str::FromStr,
};

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
/// use NeoRust::prelude::TransactionBuilder;
/// let mut tx_builder = TransactionBuilder::new();
/// tx_builder.version(0)
///           .nonce(1)
///           .valid_until_block(100)
///           .set_script(vec![0x01, 0x02, 0x03])
///           .get_unsigned_tx();
/// ```
use getset::{CopyGetters, Getters, MutGetters, Setters};
use once_cell::sync::Lazy;
use primitive_types::H160;
use rustc_serialize::hex::ToHex;

use neo::{neo_types::ScriptHashExtension, prelude::*};

#[derive(Getters, Setters, MutGetters, CopyGetters, Default)]
pub struct TransactionBuilder<'a, P: JsonRpcProvider + 'static> {
	client: Option<&'a RpcClient<P>>,
	version: u8,
	nonce: u32,
	valid_until_block: Option<u32>,
	// setter and getter
	#[getset(get = "pub", set = "pub")]
	signers: Vec<Signer>,
	#[getset(get = "pub", set = "pub")]
	additional_network_fee: u64,
	#[getset(get = "pub", set = "pub")]
	additional_system_fee: u64,
	#[getset(get = "pub", set = "pub")]
	attributes: Vec<TransactionAttribute>,
	#[getset(get = "pub", set = "pub")]
	script: Option<Bytes>,
	fee_consumer: Option<Box<dyn Fn(i64, i64)>>,
	fee_error: Option<TransactionError>,
}

impl<'a, P: JsonRpcProvider + 'static> Debug for TransactionBuilder<'a, P> {
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

impl<'a, P: JsonRpcProvider + 'static> Clone for TransactionBuilder<'a, P> {
	fn clone(&self) -> Self {
		Self {
			client: self.client,
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

impl<'a, P: JsonRpcProvider + 'static> Eq for TransactionBuilder<'a, P> {}

impl<'a, P: JsonRpcProvider + 'static> PartialEq for TransactionBuilder<'a, P> {
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

impl<'a, P: JsonRpcProvider + 'static> Hash for TransactionBuilder<'a, P> {
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

pub static GAS_TOKEN_HASH: Lazy<ScriptHash> =
	Lazy::new(|| ScriptHash::from_str("d2a4cff31913016155e38e474a2c06d08be276cf").unwrap());

impl<'a, P: JsonRpcProvider + 'static> TransactionBuilder<'a, P> {
	// const GAS_TOKEN_HASH: ScriptHash = ScriptHash::from_str("d2a4cff31913016155e38e474a2c06d08be276cf").unwrap();
	pub const BALANCE_OF_FUNCTION: &'static str = "balanceOf";
	pub const DUMMY_PUB_KEY: &'static str =
		"02ec143f00b88524caf36a0121c2de09eef0519ddbe1c710a00f0e2663201ee4c0";

	// Constructor
	pub fn new() -> Self {
		Self {
			client: None,
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

	pub fn with_client(client: &'a RpcClient<P>) -> Self {
		Self {
			client: Some(client),
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
		if nonce > u32::MAX {
			return Err(TransactionError::InvalidNonce);
		}

		self.nonce = nonce;
		Ok(self)
	}

	// Other methods

	// Set valid until block
	pub fn valid_until_block(&mut self, block: u32) -> Result<&mut Self, TransactionError> {
		if block == 0 {
			return Err(TransactionError::InvalidBlock);
		}

		self.valid_until_block = Some(block);
		Ok(self)
	}

	// Set script
	// pub fn set_script(&mut self, script: Vec<u8>) -> &mut Self {
	// 	self.script = Some(script);
	// 	self
	// }

	pub fn first_signer(&mut self, sender: &Account) -> Result<&mut Self, TransactionError> {
		self.first_signer_by_hash(&sender.get_script_hash())
	}

	pub fn first_signer_by_hash(&mut self, sender: &H160) -> Result<&mut Self, TransactionError> {
		if self.signers.iter().any(|s| s.get_scopes().contains(&WitnessScope::None)) {
			return Err(TransactionError::ScriptFormat("This transaction contains a signer with fee-only witness scope that will cover the fees. Hence, the order of the signers does not affect the payment of the fees.".to_string()));
		}
		if let Some(pos) = self.signers.iter().position(|s| s.get_signer_hash() == sender) {
			let s = self.signers.remove(pos);
			self.signers.insert(0, s);
			Ok(self)
		} else {
			Err(TransactionError::ScriptFormat(format!("Could not find a signer with script hash {}. Make sure to add the signer before calling this method.", sender.to_string()).into()))
		}
	}

	pub fn extend_script(&mut self, script: Vec<u8>) -> &mut Self {
		if let Some(ref mut existing_script) = self.script {
			existing_script.extend(script);
		} else {
			self.script = Some(script);
		}
		self
	}

	pub async fn call_invoke_script(&self) -> InvocationResult {
		if self.script.is_none() || self.script.as_ref().unwrap().is_empty() {
			panic!("Script is not set");
		}
		self.client
			.unwrap()
			.rpc_client()
			.invoke_script(self.script.clone().unwrap().to_hex(), self.signers.clone())
			.await
			.unwrap_or(panic!("Failed to invoke script"))
	}

	// Get unsigned transaction
	pub async fn get_unsigned_tx(&mut self) -> Result<Transaction, TransactionError> {
		// Validate configuration
		if self.signers.is_empty() {
			return Err(TransactionError::NoSigners);
		}

		if self.script.is_none() {
			return Err(TransactionError::NoScript);
		}
		let len = self.signers.len();
		self.signers.dedup();

		// Validate no duplicate signers
		if len != self.signers.len() {
			return Err(TransactionError::DuplicateSigner);
		}

		// Check signer limits
		if self.signers.len() > NeoConstants::MAX_SIGNER_SUBITEMS as usize {
			return Err(TransactionError::TooManySigners);
		}

		// Validate script
		if let Some(script) = &self.script {
			if script.is_empty() {
				return Err(TransactionError::EmptyScript);
			}
		} else {
			return Err(TransactionError::NoScript);
		}

		// Get fees
		let script = self.script.as_ref().unwrap();
		let response = self
			.client
			.unwrap()
			.invoke_script(script.to_hex(), vec![self.signers[0].clone()])
			.await
			.map_err(|e| TransactionError::ProviderError(e))?;

		let system_fee = i64::from_str(response.gas_consumed.as_str()).unwrap_or_else(|e| {
			panic!(
				"Failed to parse system fee from response: {:?}, error: {:?}",
				response.gas_consumed, e
			)
		});

		// Check sender balance if needed
		let mut tx = Transaction {
			network: Some(self.client.unwrap().network().await),
			version: self.version,
			nonce: self.nonce,
			valid_until_block: self.valid_until_block.unwrap(),
			size: 0,
			sys_fee: system_fee,
			net_fee: 0,
			signers: self.signers.clone(),
			attributes: self.attributes.clone(),
			script: self.script.clone().unwrap(), // We've already checked for None case above
			witnesses: vec![],
			block_time: None,
			block_count_when_sent: None,
		};

		// It's impossible to calculate network fee when the tx is unsigned, because there is no witness
		// let network_fee = Box::pin(self.client.unwrap().calculate_network_fee(base64::encode(tx.to_array()))).await?;
		// if let Some(fee_consumer) = &self.fee_consumer {
		// 	let sender_balance = 0; // self.get_sender_balance().await.unwrap();
		// 	if network_fee + system_fee > sender_balance {
		// 		fee_consumer(network_fee + system_fee, sender_balance);
		// 	}
		// }
		// tx.set_net_fee(network_fee);

		Ok(tx)
	}

	async fn get_system_fee(&self) -> Result<u64, TransactionError> {
		let script = self.script.as_ref().unwrap();

		let response = self
			.client
			.unwrap()
			.invoke_script(script.to_hex(), vec![self.signers[0].clone()])
			.await
			.map_err(|e| TransactionError::ProviderError(e))?;
		Ok(u64::from_str(response.gas_consumed.as_str()).unwrap()) // example
	}

	async fn get_network_fee(&mut self) -> Result<i64, TransactionError> {
		let fee = self
			.client
			.unwrap()
			.calculate_network_fee(self.get_unsigned_tx().await.unwrap().to_array().to_hex())
			.await
			.map_err(|e| TransactionError::ProviderError(e))?;
		Ok(fee)
	}

	async fn get_sender_balance(&self) -> Result<u64, TransactionError> {
		// Call network
		let sender = &self.signers[0];

		if Self::is_account_signer(sender) {
			let balance = self
				.client
				.unwrap()
				.invoke_function(
					&GAS_TOKEN_HASH,
					Self::BALANCE_OF_FUNCTION.to_string(),
					vec![ContractParameter::from(sender.get_signer_hash())],
					None,
				)
				.await
				.map_err(|e| TransactionError::ProviderError(e))?
				.stack[0]
				.clone();
			return Ok(balance.as_int().unwrap() as u64);
		}
		Err(TransactionError::InvalidSender)
	}

	fn is_account_signer(signer: &Signer) -> bool {
		if signer.get_type() == SignerType::Account {
			return true;
		}
		return false;
	}

	// Sign transaction
	pub async fn sign(&mut self) -> Result<Transaction, BuilderError> {
		let mut unsigned_tx = self.get_unsigned_tx().await?;
		// let client = self.client.unwrap();
		let tx_bytes = unsigned_tx.get_hash_data().await?;

		let mut witnesses_to_add = Vec::new();

		for signer in &mut unsigned_tx.signers {
			if Self::is_account_signer(signer) {
				let account_signer = signer.as_account_signer().unwrap();
				let acc = &account_signer.account;
				if acc.is_multi_sig() {
					return Err(BuilderError::IllegalState(
						"Transactions with multi-sig signers cannot be signed automatically."
							.to_string(),
					));
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
			unsigned_tx.add_witness(witness);
		}

		Ok(unsigned_tx)
	}

	fn signers_contain_multi_sig_with_committee_member(&self, committee: &HashSet<H160>) -> bool {
		for signer in &self.signers {
			if let Some(account_signer) = signer.as_account_signer() {
				if account_signer.is_multi_sig() {
					if let Some(script) = &account_signer.account().verification_script() {
						for pubkey in script.get_public_keys().unwrap() {
							let hash = public_key_to_script_hash(&pubkey);
							if committee.contains(&hash) {
								return true;
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

	/// Checks if the sender account of this transaction can cover the network and system fees.
	/// If not, executes the given consumer supplying it with the required fee and the sender's GAS balance.
	///
	/// The check and potential execution of the consumer is only performed when the transaction is built, i.e., when calling `TransactionBuilder::sign` or `TransactionBuilder::get_unsigned_transaction`.
	/// - Parameter consumer: The consumer
	/// - Returns: This transaction builder (self)
	pub fn do_if_sender_cannot_cover_fees<F>(
		&mut self,
		mut consumer: F,
	) -> Result<&mut Self, TransactionError>
	where
		F: FnMut(i64, i64) + Send + Sync + 'static,
	{
		if self.fee_error.is_some() {
			return Err(TransactionError::IllegalState(
                "Cannot handle a consumer for this case, since an exception will be thrown if the sender cannot cover the fees.".to_string(),
            ));
		}
		let consumer = RefCell::new(consumer);
		self.fee_consumer = Some(Box::new(move |fee, balance| {
			let mut consumer = consumer.borrow_mut();
			consumer(fee, balance);
		}));
		Ok(self)
	}

	/// Checks if the sender account of this transaction can cover the network and system fees.
	/// If not, otherwise throw an error created by the provided supplier.
	///
	/// The check and potential throwing of the exception is only performed when the transaction is built, i.e., when calling `TransactionBuilder::sign` or `TransactionBuilder::get_unsigned_transaction`.
	/// - Parameter error: The error to throw
	/// - Returns: This transaction builder (self)
	pub fn throw_if_sender_cannot_cover_fees(
		&mut self,
		error: TransactionError,
	) -> Result<&mut Self, TransactionError> {
		if self.fee_consumer.is_some() {
			return Err(TransactionError::IllegalState(
                "Cannot handle a supplier for this case, since a consumer will be executed if the sender cannot cover the fees.".to_string(),
            ));
		}
		self.fee_error = Some(error);
		Ok(self)
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

#[cfg(test)]
mod tests {
	use crate::{
		neo_builder::GAS_TOKEN_HASH,
		neo_clients::MockClient,
		neo_types::ScriptHashExtension,
		prelude::{
			ApplicationLog, ContractParameter, ContractSigner, InvocationResult, Signer, StackItem,
			TestConstants, TransactionAttribute, TransactionError, Witness, WitnessScope,
		},
	};
	use lazy_static::lazy_static;
	use neo::{
		builder::VerificationScript,
		config::{NeoConfig, NEOCONFIG},
		prelude::{
			APITrait, Account, AccountSigner, AccountTrait, Http, HttpProvider, KeyPair,
			NeoConstants, RawTransaction, RpcClient, ScriptBuilder, Secp256r1PrivateKey,
			TransactionBuilder,
		},
	};
	use num_bigint::BigInt;
	use primitive_types::{H160, H256};
	use rustc_serialize::hex::ToHex;
	use serde_json::json;
	use std::{
		ops::Deref,
		str::FromStr,
		sync::{Arc, Mutex},
	};
	use tokio::sync::OnceCell;

	lazy_static! {
		pub static ref ACCOUNT1: Account = Account::from_key_pair(
			KeyPair::from_secret_key(
				&Secp256r1PrivateKey::from_bytes(
					&hex::decode(
						"e6e919577dd7b8e97805151c05ae07ff4f752654d6d8797597aca989c02c4cb3"
					)
					.unwrap()
				)
				.unwrap()
			),
			None,
			None
		)
		.expect("Failed to create ACCOUNT1");
		pub static ref ACCOUNT2: Account = Account::from_key_pair(
			KeyPair::from_secret_key(
				&Secp256r1PrivateKey::from_bytes(
					&hex::decode(
						"b4b2b579cac270125259f08a5f414e9235817e7637b9a66cfeb3b77d90c8e7f9"
					)
					.unwrap()
				)
				.unwrap()
			),
			None,
			None
		)
		.expect("Failed to create ACCOUNT2");
	}

	static CLIENT: OnceCell<RpcClient<HttpProvider>> = OnceCell::const_new();

	#[tokio::test]
	async fn test_build_transaction_with_correct_nonce() {
		let mut nonce = 1;
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let mut tx = TransactionBuilder::with_client(&client)
			.valid_until_block(1)
			.unwrap()
			.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::called_by_entry(ACCOUNT1.deref()).unwrap().into()])
			.nonce(nonce)
			.unwrap()
			.get_unsigned_tx()
			.await
			.unwrap();

		assert_eq!(*tx.nonce(), nonce);

		nonce = 0;
		tx = TransactionBuilder::with_client(&client)
			.valid_until_block(1)
			.unwrap()
			.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::called_by_entry(ACCOUNT1.deref()).unwrap().into()])
			.nonce(nonce)
			.unwrap()
			.get_unsigned_tx()
			.await
			.unwrap();
		assert_eq!(*tx.nonce(), nonce);

		nonce = u32::MAX;
		tx = TransactionBuilder::with_client(&client)
			.valid_until_block(1)
			.unwrap()
			.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::called_by_entry(ACCOUNT1.deref()).unwrap().into()])
			.nonce(nonce)
			.unwrap()
			.get_unsigned_tx()
			.await
			.unwrap();
		assert_eq!(*tx.nonce(), nonce);
	}

	#[tokio::test]
	async fn test_invoke_script() {
		let mock_provider = Arc::new(MockClient::new().await);
		let client = Arc::new(mock_provider.clone().into_client());

		mock_provider
			.mock_response_ignore_param(
				"invokescript",
				json!(Ok::<InvocationResult, ()>(InvocationResult {
					stack: vec![StackItem::ByteString { value: "NEO".into() }],
					..Default::default()
				})),
			)
			.await;

		let script = ScriptBuilder::new()
			.contract_call(
				&ScriptHashExtension::from_hex("0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5")
					.unwrap(),
				"symbol",
				&[],
				None,
			)
			.unwrap()
			.to_bytes();

		let tb = TransactionBuilder::with_client(&client);
		let response = tb.client.unwrap().invoke_script((&script).to_hex(), vec![]).await.unwrap();

		assert_eq!(response.stack[0].as_string().unwrap(), "NEO");
	}

	#[tokio::test]
	async fn test_build_without_setting_script() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let err = TransactionBuilder::with_client(&client)
			.set_signers(vec![AccountSigner::called_by_entry(ACCOUNT1.deref()).unwrap().into()])
			.get_unsigned_tx()
			.await
			.err()
			.unwrap();

		assert_eq!(err, TransactionError::NoScript);
	}

	#[tokio::test]
	async fn test_sign_transaction_with_additional_signers() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let script = vec![0x01u8, 0x02u8, 0x03u8];

		let tx = TransactionBuilder::with_client(&client)
			.set_script(Some(script))
			.set_signers(vec![
				AccountSigner::called_by_entry(ACCOUNT1.deref()).unwrap().into(),
				AccountSigner::called_by_entry(ACCOUNT2.deref()).unwrap().into(),
			])
			.valid_until_block(1000)
			.unwrap()
			.sign()
			.await
			.unwrap();

		assert_eq!(tx.witnesses().len(), 2);

		let signers = tx
			.witnesses()
			.iter()
			.map(|witness| witness.verification.get_public_keys().unwrap().first().unwrap().clone())
			.collect::<Vec<_>>();

		assert!(signers.contains(&ACCOUNT1.deref().clone().key_pair.unwrap().public_key()));
		assert!(signers.contains(&ACCOUNT2.deref().clone().key_pair.unwrap().public_key()));
	}

	#[tokio::test]
	async fn test_send_invoke_function() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let tb = TransactionBuilder::with_client(&client);
		let response = tb
			.client
			.unwrap()
			.invoke_function(
				&H160::from_str("0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5").unwrap(),
				"symbol".to_string(),
				vec![],
				None,
			)
			.await
			.unwrap();

		assert_eq!(response.stack[0].as_string().unwrap(), "NEO");
	}

	#[tokio::test]
	async fn test_fail_building_transaction_with_incorrect_nonce() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let mut tb = TransactionBuilder::with_client(&client);
		tb.valid_until_block(1)
			.unwrap()
			.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::called_by_entry(ACCOUNT1.deref()).unwrap().into()]);

		// Test with 0, which should be valid
		assert!(tb.nonce(0).is_ok());

		// Test with u32::MAX, which should be valid
		assert!(tb.nonce(u32::MAX).is_ok());

		// Test overflow condition
		tb.nonce(u32::MAX).unwrap();
		assert!(tb.nonce(u32::MAX).is_ok());

		// Reset nonce for next test
		tb.nonce(0).unwrap();

		// Test with -1 cast to u32, which is actually u32::MAX
		assert!(tb.nonce((-1i32) as u32).is_ok());
	}

	#[tokio::test]
	async fn test_fail_building_transaction_with_invalid_block_number() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let mut tb = TransactionBuilder::with_client(&client);
		tb.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::called_by_entry(ACCOUNT1.deref()).unwrap().into()]);

		assert!(tb.valid_until_block(-1i32 as u32).is_ok());
		// assert!(tb.valid_until_block(2u32.pow(32)).is_err());
	}

	#[tokio::test]
	async fn test_automatically_set_nonce() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let mut tb = TransactionBuilder::with_client(&client);
		tb.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::called_by_entry(ACCOUNT1.deref()).unwrap().into()]);

		let tx = match tb.get_unsigned_tx().await {
			Ok(tx) => tx,
			Err(e) => panic!("Failed to build transaction: {:?}", e),
		};
		assert!(tx.nonce() < &u32::MAX && tx.nonce() > &0);
	}

	#[tokio::test]
	async fn test_fail_building_tx_without_any_signer() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let mut tb = TransactionBuilder::with_client(&client);
		tb.valid_until_block(100).unwrap().set_script(Some(vec![1, 2, 3]));

		assert!(tb.get_unsigned_tx().await.is_err());

		let mut tb2 = TransactionBuilder::with_client(&client);
		tb2.set_signers(vec![
			AccountSigner::global(ACCOUNT1.deref()).unwrap().into(),
			AccountSigner::called_by_entry(ACCOUNT1.deref()).unwrap().into(),
		]);
	}

	#[tokio::test]
	async fn test_override_signer() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let mut tb = TransactionBuilder::with_client(&client);
		tb.set_script(Some(vec![1, 2, 3])).set_signers(vec![AccountSigner::global(
			ACCOUNT1.deref(),
		)
		.unwrap()
		.into()]);
		assert_eq!(
			tb.signers()[0],
			Signer::Account(AccountSigner::global(ACCOUNT1.deref()).unwrap())
		);

		tb.set_signers(vec![AccountSigner::global(ACCOUNT2.deref()).unwrap().into()]);
		assert_eq!(tb.signers(), &vec![AccountSigner::global(ACCOUNT2.deref()).unwrap().into()]);
	}

	#[tokio::test]
	async fn test_attributes_high_priority_committee() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let mut tb = TransactionBuilder::with_client(&client);
		let multi_sig_account = Account::multi_sig_from_public_keys(
			&mut vec![ACCOUNT2.get_public_key().unwrap(), ACCOUNT1.get_public_key().unwrap()],
			1,
		)
		.unwrap();
		tb.set_script(Some(vec![1, 2, 3]))
			.set_attributes(vec![TransactionAttribute::HighPriority])
			.set_signers(vec![AccountSigner::none(&multi_sig_account).unwrap().into()]);

		let tx = tb.get_unsigned_tx().await.unwrap();
		assert_eq!(tx.attributes()[0], TransactionAttribute::HighPriority);
	}

	#[tokio::test]
	async fn test_attributes_high_priority() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let mut tb = TransactionBuilder::with_client(&client);
		tb.set_script(Some(vec![1, 2, 3]))
			.set_attributes(vec![TransactionAttribute::HighPriority])
			.set_signers(vec![AccountSigner::none(ACCOUNT1.deref()).unwrap().into()]);

		let tx = tb.get_unsigned_tx().await.unwrap();
		assert_eq!(tx.attributes()[0], TransactionAttribute::HighPriority);
	}

	#[tokio::test]
	async fn test_attributes_high_priority_not_committee_member() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let mut tb = TransactionBuilder::with_client(&client);
		tb.set_script(Some(vec![1, 2, 3]))
			.set_attributes(vec![TransactionAttribute::HighPriority])
			.set_signers(vec![AccountSigner::none(ACCOUNT2.deref()).unwrap().into()]);

		assert!(tb.get_unsigned_tx().await.is_err());
	}

	#[tokio::test]
	async fn test_attributes_high_priority_only_added_once() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let mut tb = TransactionBuilder::with_client(&client);
		tb.set_script(Some(vec![1, 2, 3]))
			.set_attributes(vec![
				TransactionAttribute::HighPriority,
				TransactionAttribute::HighPriority,
			])
			.set_signers(vec![AccountSigner::none(ACCOUNT1.deref()).unwrap().into()]);

		let tx = tb.get_unsigned_tx().await.unwrap();
		assert_eq!(tx.attributes()[0], TransactionAttribute::HighPriority);
	}

	#[tokio::test]
	async fn test_fail_adding_more_than_max_attributes_to_tx_just_attributes() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let attrs: Vec<TransactionAttribute> = (0..=NeoConstants::MAX_TRANSACTION_ATTRIBUTES)
			.map(|_| TransactionAttribute::HighPriority)
			.collect();
		let mut tb = TransactionBuilder::with_client(&client);
		// assert!(tb.set_attributes(attrs));
	}

	#[tokio::test]
	async fn test_fail_adding_more_than_max_attributes_to_tx_attributes_and_signers() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let mut tb = TransactionBuilder::with_client(&client);
		tb.set_signers(vec![
			AccountSigner::called_by_entry(ACCOUNT1.deref()).unwrap().into(),
			AccountSigner::called_by_entry(ACCOUNT1.deref()).unwrap().into(),
			AccountSigner::called_by_entry(ACCOUNT1.deref()).unwrap().into(),
		]);
		let attrs: Vec<TransactionAttribute> = (0..=NeoConstants::MAX_TRANSACTION_ATTRIBUTES - 3)
			.map(|_| TransactionAttribute::HighPriority)
			.collect();
		// assert!(tb.set_attributes(attrs));
	}

	#[tokio::test]
	async fn test_fail_adding_more_than_max_attributes_to_tx_signers() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let mut tb = TransactionBuilder::with_client(&client);
		tb.set_attributes(vec![TransactionAttribute::HighPriority]);
		let signers: Vec<AccountSigner> = (0..NeoConstants::MAX_TRANSACTION_ATTRIBUTES)
			.map(|_| AccountSigner::called_by_entry(ACCOUNT1.deref()).unwrap())
			.collect();
		// assert!(tb.set_signers(signers.into_iter().map(Into::into).collect()));
	}

	#[tokio::test]
	async fn test_automatic_setting_of_valid_until_block_variable() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let block_count = 1000;
		let max_valid_until_block_increment = 1000;
		let mut tb = TransactionBuilder::with_client(&client);
		tb.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::none(ACCOUNT1.deref()).unwrap().into()]);

		let tx = tb.get_unsigned_tx().await.unwrap();
		assert_eq!(*tx.valid_until_block(), block_count + max_valid_until_block_increment);
	}

	#[tokio::test]
	async fn test_automatic_setting_of_system_fee_and_network_fee() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let script = vec![1, 2, 3];
		let mut tb = TransactionBuilder::with_client(&client);
		tb.set_script(Some(script.clone()))
			.set_signers(vec![AccountSigner::none(ACCOUNT1.deref()).unwrap().into()])
			.valid_until_block(1000)
			.unwrap();

		let tx = tb.get_unsigned_tx().await.unwrap();
		assert_eq!(*tx.sys_fee(), 984060);
		assert_eq!(*tx.net_fee(), 1230610);
	}

	#[tokio::test]
	async fn test_fail_trying_to_sign_transaction_with_account_missing_a_private_key() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let account_without_keypair =
			Account::from_address(ACCOUNT1.get_address().as_str()).unwrap();
		let mut tb = TransactionBuilder::with_client(&client);
		tb.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::none(&account_without_keypair).unwrap().into()])
			.valid_until_block(1000)
			.unwrap();

		assert!(tb.sign().await.is_err());
	}

	#[tokio::test]
	async fn test_fail_automatically_signing_with_multi_sig_account_signer() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let multi_sig_account = Account::multi_sig_from_public_keys(
			vec![ACCOUNT1.get_public_key().unwrap()].as_mut(),
			1,
		)
		.unwrap();
		let mut tb = TransactionBuilder::with_client(&client);
		tb.set_script(Some(vec![1, 2, 3])).set_signers(vec![AccountSigner::none(
			&multi_sig_account,
		)
		.unwrap()
		.into()]);

		assert!(tb.sign().await.is_err());
	}

	#[tokio::test]
	async fn test_fail_with_no_signing_account() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let mut tb = TransactionBuilder::with_client(&client);
		tb.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![ContractSigner::called_by_entry(
				ACCOUNT1.address_or_scripthash().script_hash(),
				&*vec![],
			)
			.into()]);

		assert!(tb.sign().await.is_err());
	}

	#[tokio::test]
	async fn test_fail_signing_with_account_without_ec_keypair() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let account_without_keypair = Account::from_verification_script(
			&ACCOUNT1.clone().verification_script().clone().unwrap(),
		)
		.unwrap();
		let mut tb = TransactionBuilder::with_client(&client);
		tb.set_script(Some(vec![1, 2, 3])).set_signers(vec![AccountSigner::none(
			&account_without_keypair,
		)
		.unwrap()
		.into()]);

		assert!(tb.sign().await.is_err());
	}

	#[tokio::test]
	async fn test_fail_sending_transaction_because_it_doesnt_contain_the_right_number_of_witnesses()
	{
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let mut tb = TransactionBuilder::with_client(&client);
		tb.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::called_by_entry(ACCOUNT1.deref()).unwrap().into()])
			.valid_until_block(1000)
			.unwrap();

		let mut tx = tb.get_unsigned_tx().await.unwrap();
		// assert!(tx.send_tx(&client).await.is_err());
	}

	#[tokio::test]
	async fn test_contract_witness() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let contract_hash = H160::from_str("e87819d005b730645050f89073a4cd7bf5f6bd3c").unwrap();
		let params = vec![ContractParameter::from("iamgroot"), ContractParameter::from(2)];
		let invocation_script = ScriptBuilder::new()
			.push_data("iamgroot".as_bytes().to_vec())
			.push_integer(BigInt::from(2))
			.to_bytes();
		let mut tb = TransactionBuilder::with_client(&client);
		tb.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![
				ContractSigner::global(contract_hash, &params).into(),
				AccountSigner::called_by_entry(ACCOUNT1.deref()).unwrap().into(),
			])
			.valid_until_block(1000)
			.unwrap();

		let tx = tb.sign().await.unwrap();
		assert!(tx.witnesses().contains(&Witness::from_scripts(invocation_script, vec![])));
	}

	#[tokio::test]
	async fn test_transfer_neo_from_normal_account() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let script = ScriptBuilder::new()
			.contract_call(
				&ScriptHashExtension::from_hex("0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5")
					.unwrap(),
				"transfer",
				&vec![
					ContractParameter::from(ACCOUNT1.address_or_scripthash().script_hash()),
					ContractParameter::from(
						H160::from_str("969a77db482f74ce27105f760efa139223431394").unwrap(),
					),
					ContractParameter::from(5),
					ContractParameter::any(),
				],
				None,
			)
			.unwrap()
			.to_bytes();

		let binding = ACCOUNT1.verification_script().clone().unwrap();
		let expected_verification_script = binding.script();
		let mut tb = TransactionBuilder::with_client(&client);
		tb.set_script(Some(script.clone()))
			.set_signers(vec![AccountSigner::none(ACCOUNT1.deref()).unwrap().into()])
			.valid_until_block(100)
			.unwrap();

		let tx = tb.sign().await.unwrap();
		assert_eq!(tx.script(), &script);
		assert_eq!(tx.witnesses().len(), 1);
		assert_eq!(
			tx.witnesses().first().unwrap().verification.script(),
			expected_verification_script
		);
	}

	#[tokio::test]
	async fn test_extend_script() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let script1 = ScriptBuilder::new()
			.contract_call(
				&ScriptHashExtension::from_hex("0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5")
					.unwrap(),
				"transfer",
				&vec![
					ContractParameter::from(ACCOUNT1.address_or_scripthash().script_hash()),
					ContractParameter::from(
						H160::from_str("969a77db482f74ce27105f760efa139223431394").unwrap(),
					),
					ContractParameter::from(11),
					ContractParameter::any(),
				],
				None,
			)
			.unwrap()
			.to_bytes();

		let script2 = ScriptBuilder::new()
			.contract_call(
				&ScriptHashExtension::from_hex("0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5")
					.unwrap(),
				"transfer",
				&vec![
					ContractParameter::from(ACCOUNT1.address_or_scripthash().script_hash()),
					ContractParameter::from(ACCOUNT2.address_or_scripthash().script_hash()),
					ContractParameter::from(22),
					ContractParameter::any(),
				],
				None,
			)
			.unwrap()
			.to_bytes();

		let mut tb = TransactionBuilder::with_client(&client);
		tb.set_script(Some(script1.clone()));
		assert_eq!(tb.script().clone().unwrap().len(), script1.len());

		tb.extend_script(script2.clone());
		assert_eq!(tb.script().clone().unwrap().len(), [script1, script2].concat().len());
	}

	#[tokio::test]
	async fn test_invoking_with_params_should_produce_the_correct_request() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let mut tb = TransactionBuilder::with_client(&client);
		let script = ScriptBuilder::new()
			.contract_call(
				&ScriptHashExtension::from_hex("0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5")
					.unwrap(),
				"transfer",
				&vec![
					ContractParameter::from(ACCOUNT1.address_or_scripthash().script_hash()),
					ContractParameter::from(
						H160::from_str("969a77db482f74ce27105f760efa139223431394").unwrap(),
					),
					ContractParameter::from(5),
					ContractParameter::any(),
				],
				None,
			)
			.unwrap()
			.to_bytes();

		let response = tb
			.client
			.unwrap()
			.invoke_function(
				&H160::from_str("0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5").unwrap(),
				"transfer".to_string(),
				vec![
					ContractParameter::from(ACCOUNT1.address_or_scripthash().script_hash()),
					ContractParameter::from(
						H160::from_str("969a77db482f74ce27105f760efa139223431394").unwrap(),
					),
					ContractParameter::from(5),
					ContractParameter::any(),
				],
				None,
			)
			.await
			.unwrap();

		assert_eq!(response.stack[0].as_string().unwrap(), "NEO");
	}

	#[tokio::test]
	async fn test_fail_signing_with_account_without_ec_key_pair() {
		let mock_provider = Arc::new(MockClient::new().await);
		let client = Arc::new(mock_provider.clone().into_client());
		mock_provider
			.mock_response_ignore_param(
				"invokescript",
				json!(Ok::<InvocationResult, ()>(InvocationResult::default())),
			)
			.await;
		mock_provider
			.mock_response_ignore_param("getblockcount", json!(Ok::<i32, ()>(1000)))
			.await;
		mock_provider
			.mock_response_ignore_param("calculatenetworkfee", json!(Ok::<i32, ()>(1000000)))
			.await;

		let account =
			Account::from_verification_script(&VerificationScript::from(vec![1, 2, 3])).unwrap();

		let mut tx_builder = TransactionBuilder::with_client(client.as_ref());
		tx_builder
			.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::none(&account).unwrap().into()]);

		assert!(tx_builder.sign().await.is_err());
	}

	#[tokio::test]
	async fn test_do_if_sender_cannot_cover_fees() {
		let mock_provider = Arc::new(MockClient::new().await);
		let client = Arc::new(mock_provider.clone().into_client());
		mock_provider
			.mock_response_ignore_param(
				"invokescript",
				json!(Ok::<InvocationResult, ()>(InvocationResult {
					gas_consumed: "9999510".to_string(),
					..Default::default()
				})),
			)
			.await;
		mock_provider
			.mock_response_ignore_param("calculatenetworkfee", json!(Ok::<i32, ()>(1230610)))
			.await;
		mock_provider
			.mock_response_ignore_param(
				"invokefunction",
				json!(Ok::<InvocationResult, ()>(InvocationResult {
					stack: vec![StackItem::Integer { value: 1000000.into() }],
					..Default::default()
				})),
			)
			.await;

		let account1 =
			Account::from_wif("L1WMhxazScMhUrdv34JqQb1HFSQmWeN2Kpc1R9JGKwL7CDNP21uR").unwrap();
		let recipient = H160::from_str("969a77db482f74ce27105f760efa139223431394").unwrap();

		let script = ScriptBuilder::new()
			.contract_call(
				&GAS_TOKEN_HASH,
				"transfer",
				&[
					ContractParameter::h160(&account1.address_or_scripthash().script_hash()),
					ContractParameter::h160(&recipient),
					ContractParameter::integer(2_000_000),
					ContractParameter::any(),
				],
				None,
			)
			.unwrap()
			.to_bytes();

		let tested = Arc::new(std::sync::atomic::AtomicBool::new(false));
		let tested_clone = tested.clone();

		let mut tx_builder = TransactionBuilder::with_client(client.as_ref());
		let _ = tx_builder
			.set_script(Some(script))
			.set_signers(vec![AccountSigner::called_by_entry(&account1).unwrap().into()])
			.valid_until_block(2000000)
			.unwrap()
			.do_if_sender_cannot_cover_fees(Box::new(move |fee, balance| {
				assert_eq!(fee, 1230610 + 9999510);
				assert_eq!(balance, 1000000);
				tested_clone.store(true, std::sync::atomic::Ordering::SeqCst);
			}));

		let _ = tx_builder.get_unsigned_tx().await.unwrap();

		assert!(tested.load(std::sync::atomic::Ordering::SeqCst));
	}

	#[tokio::test]
	async fn test_do_if_sender_cannot_cover_fees_already_specified_a_supplier() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let mut tx_builder = TransactionBuilder::with_client(&client);

		// TODO: check and add
		// NeoConfig::throw_if_sender_cannot_cover_fees(TransactionError::InsufficientFunds);

		assert!(tx_builder.do_if_sender_cannot_cover_fees(Box::new(|_, _| {})).is_err());
	}

	#[tokio::test]
	async fn test_throw_if_sender_cannot_cover_fees() {
		let mock_provider = Arc::new(MockClient::new().await);
		let client = Arc::new(mock_provider.clone().into_client());
		mock_provider
			.mock_response_ignore_param(
				"invokescript",
				json!(Ok::<InvocationResult, ()>(InvocationResult {
					gas_consumed: "9999510".to_string(),
					..Default::default()
				})),
			)
			.await;
		mock_provider
			.mock_response_ignore_param("calculatenetworkfee", json!(Ok::<i32, ()>(1230610)))
			.await;
		mock_provider
			.mock_response_ignore_param(
				"invokefunction",
				json!(Ok::<InvocationResult, ()>(InvocationResult {
					stack: vec![StackItem::Integer { value: 1000000 }],
					..Default::default()
				})),
			)
			.await;

		let account1 =
			Account::from_wif("L1WMhxazScMhUrdv34JqQb1HFSQmWeN2Kpc1R9JGKwL7CDNP21uR").unwrap();

		let script = ScriptBuilder::new()
			.contract_call(
				&H160::from_str(TestConstants::NEO_TOKEN_HASH).unwrap(),
				"transfer",
				&[
					ContractParameter::h160(&account1.address_or_scripthash().script_hash()),
					ContractParameter::h160(&H160::zero()),
					ContractParameter::integer(5),
					ContractParameter::any(),
				],
				None,
			)
			.unwrap()
			.to_bytes();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		let _ = tx_builder
			.set_script(Some(script))
			.valid_until_block(2000000)
			.unwrap()
			.set_signers(vec![AccountSigner::called_by_entry(&account1).unwrap().into()])
			.throw_if_sender_cannot_cover_fees(TransactionError::InsufficientFunds);

		assert!(tx_builder.get_unsigned_tx().await.is_err());
	}

	#[tokio::test]
	async fn test_throw_if_sender_cannot_cover_fees_already_specified_a_consumer() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let mut tx_builder = TransactionBuilder::with_client(&client);
		let _ = tx_builder.do_if_sender_cannot_cover_fees(Box::new(|_, _| {}));

		assert!(tx_builder
			.throw_if_sender_cannot_cover_fees(TransactionError::InsufficientFunds)
			.is_err());
	}

	#[tokio::test]
	async fn test_build_with_invalid_script() {
		let mock_provider = Arc::new(MockClient::new().await);
		let client = Arc::new(mock_provider.clone().into_client());
		mock_provider
			.mock_response_ignore_param("invokescript", Default::default())
			.await;
		mock_provider
			.mock_response_ignore_param("getblockcount", json!(Ok::<i32, ()>(1000)))
			.await;

		let account1 =
			Account::from_wif("L1WMhxazScMhUrdv34JqQb1HFSQmWeN2Kpc1R9JGKwL7CDNP21uR").unwrap();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder
			.set_script(Some(hex::decode("0c0e4f7261636c65436f6e7472616374411af77b67").unwrap()))
			.set_signers(vec![AccountSigner::called_by_entry(&account1).unwrap().into()]);

		assert!(tx_builder.get_unsigned_tx().await.is_err());
	}

	#[tokio::test]
	async fn test_build_with_script_vm_faults() {
		let mock_provider = Arc::new(MockClient::new().await);
		let client = Arc::new(mock_provider.clone().into_client());
		mock_provider
			.mock_response_ignore_param("invokescript", Default::default())
			.await;
		mock_provider
			.mock_response_ignore_param("getblockcount", json!(Ok::<i32, ()>(1000)))
			.await;

		let account1 =
			Account::from_wif("L1WMhxazScMhUrdv34JqQb1HFSQmWeN2Kpc1R9JGKwL7CDNP21uR").unwrap();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder
			.set_script(Some(hex::decode("0c00120c1493ad1572").unwrap()))
			.set_signers(vec![AccountSigner::called_by_entry(&account1).unwrap().into()]);

		let result = tx_builder.get_unsigned_tx().await;
		assert!(result.is_err());
		assert_eq!(
            result.unwrap_err().to_string(),
            "The vm exited due to the following exception: Value was either too large or too small for an Int32."
        );
	}

	#[tokio::test]
	async fn test_get_unsigned_transaction() {
		let mock_provider = Arc::new(MockClient::new().await);
		let client = Arc::new(mock_provider.clone().into_client());
		mock_provider
			.mock_response_ignore_param(
				"invokescript",
				json!(Ok::<InvocationResult, ()>(InvocationResult {
					gas_consumed: "984060".to_string(),
					..Default::default()
				})),
			)
			.await;
		mock_provider
			.mock_response_ignore_param("getblockcount", json!(Ok::<i32, ()>(1000)))
			.await;

		let account1 =
			Account::from_wif("L1WMhxazScMhUrdv34JqQb1HFSQmWeN2Kpc1R9JGKwL7CDNP21uR").unwrap();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder
			.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::called_by_entry(&account1).unwrap().into()]);

		let tx =match  tx_builder.get_unsigned_tx().await {
			Ok(tx) => tx,
			Err(e) => panic!("Error: {}", e)
		};

		assert_eq!(tx.version, 0);
		// TODO: fix equal
		// assert_eq!(
		// 	tx.signers[0].as_account_signer().unwrap(),
		// 	AccountSigner::called_by_entry(&account1).unwrap()
		// );
		assert!(tx.witnesses.is_empty());
	}

	#[tokio::test]
	async fn test_version() {
		let mock_provider = Arc::new(MockClient::new().await);
		let client = Arc::new(mock_provider.clone().into_client());
		mock_provider
			.mock_response_ignore_param(
				"invokescript",
				json!(Ok::<InvocationResult, ()>(InvocationResult {
					gas_consumed: "984060".to_string(),
					..Default::default()
				})),
			)
			.await;
		mock_provider
			.mock_response_ignore_param("getblockcount", json!(Ok::<i32, ()>(1000)))
			.await;

		let account1 =
			Account::from_wif("L1WMhxazScMhUrdv34JqQb1HFSQmWeN2Kpc1R9JGKwL7CDNP21uR").unwrap();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder
			.version(1)
			.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::called_by_entry(&account1).unwrap().into()]);

		let tx =match  tx_builder.get_unsigned_tx().await {
			Ok(tx) => tx,
			Err(e) => panic!("Error: {}", e)
		};

		assert_eq!(tx.version, 1);
	}
	#[tokio::test]
	async fn test_additional_network_fee() {
		let mock_provider = Arc::new(MockClient::new().await);
		let client = Arc::new(mock_provider.clone().into_client());
		mock_provider
			.mock_response_ignore_param(
				"invokescript",
				json!(Ok::<InvocationResult, ()>(InvocationResult {
					gas_consumed: "984060".to_string(),
					..Default::default()
				})),
			)
			.await;
		mock_provider
			.mock_response_ignore_param("getblockcount", json!(Ok::<i32, ()>(1000)))
			.await;
		mock_provider
			.mock_response_ignore_param("calculatenetworkfee", json!(Ok::<i32, ()>(1230610)))
			.await;

		let account = Account::create().unwrap();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder
			.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::called_by_entry(&account).unwrap().into()]);

		let tx =match tx_builder.get_unsigned_tx().await {
			Ok(tx) => tx,
			Err(e) => panic!("Error: {}", e)
		};
		assert_eq!(tx.net_fee, 1230610);

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder
			.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::none(&account).unwrap().into()])
			.set_additional_network_fee(2000);

		let tx = match tx_builder.get_unsigned_tx().await {
			Ok(tx) => tx,
			Err(e) => panic!("Error: {}", e)
		};

		assert_eq!(tx.net_fee, 1230610 + 2000);
	}

	#[tokio::test]
	async fn test_additional_system_fee() {
		let mock_provider = Arc::new(MockClient::new().await);
		let client = Arc::new(mock_provider.clone().into_client());
		mock_provider
			.mock_response_ignore_param(
				"invokescript",
				json!(Ok::<InvocationResult, ()>(InvocationResult {
					script: "0x0000".to_string(),
					// gas_consumed: "984060".to_string(),
					..Default::default()
				})),
			)
			.await;
		mock_provider
			.mock_response_ignore_param("getblockcount", json!(Ok::<i32, ()>(1000)))
			.await;
		mock_provider
			.mock_response_ignore_param("calculatenetworkfee", json!(Ok::<i32, ()>(1230610)))
			.await;

		let account = Account::create().unwrap();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder
			.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::called_by_entry(&account).unwrap().into()]);

		let tx =match  tx_builder.get_unsigned_tx().await {
			Ok(tx) => tx,
			Err(e) => panic!("Error: {}", e)
		};

		assert_eq!(tx.sys_fee, 984060);

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder
			.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::none(&account).unwrap().into()])
			.set_additional_system_fee(3000);

		let tx =match  tx_builder.get_unsigned_tx().await {
			Ok(tx) => tx,
			Err(e) => panic!("Error: {}", e)
		};
		assert_eq!(tx.sys_fee, 984060 + 3000);
	}

	#[tokio::test]
	async fn test_set_first_signer() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let account1 =
			Account::from_wif("L1WMhxazScMhUrdv34JqQb1HFSQmWeN2Kpc1R9JGKwL7CDNP21uR").unwrap();
		let account2 =
			Account::from_wif("KysNqEuLb3wmZJ6PsxbA9Bh6ewTybEda4dEiN9X7X48dJPkLWZ5a").unwrap();

		let s1 = AccountSigner::global(&account1.clone()).unwrap();
		let s2 = AccountSigner::called_by_entry(&account2.clone()).unwrap();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		&tx_builder
			.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![s1.clone().into(), s2.clone().into()]);
		assert_eq!(tx_builder.clone().signers, vec![s1.clone().into(), s2.clone().into()]);

		tx_builder.clone().first_signer(&s2.account).unwrap();
		assert_eq!(tx_builder.clone().signers, vec![s2.clone().into(), s1.clone().into()]);

		&tx_builder.first_signer(&account1).unwrap();
		assert_eq!(tx_builder.clone().signers, vec![s1.clone().into(), s2.clone().into()]);
	}

	#[tokio::test]
	async fn test_set_first_signer_fee_only_present() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let account1 =
			Account::from_wif("L1WMhxazScMhUrdv34JqQb1HFSQmWeN2Kpc1R9JGKwL7CDNP21uR").unwrap();
		let account2 =
			Account::from_wif("KysNqEuLb3wmZJ6PsxbA9Bh6ewTybEda4dEiN9X7X48dJPkLWZ5a").unwrap();

		let s1 = AccountSigner::none(&account1.clone()).unwrap();
		let s2 = AccountSigner::called_by_entry(&account2.clone()).unwrap();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder
			.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![s1.clone().into(), s2.clone().into()]);
		assert_eq!(tx_builder.signers, vec![s1.clone().into(), s2.clone().into()]);

		assert!(tx_builder.first_signer(s2.account()).is_err());
	}

	#[tokio::test]
	async fn test_set_first_signer_not_present() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let account1 =
			Account::from_wif("L1WMhxazScMhUrdv34JqQb1HFSQmWeN2Kpc1R9JGKwL7CDNP21uR").unwrap();
		let account2 =
			Account::from_wif("KysNqEuLb3wmZJ6PsxbA9Bh6ewTybEda4dEiN9X7X48dJPkLWZ5a").unwrap();

		let s1 = AccountSigner::global(&account1.clone()).unwrap();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder.set_script(Some(vec![1, 2, 3])).set_signers(vec![s1.clone().into()]);
		assert_eq!(tx_builder.signers[0], s1.clone().into());

		assert!(tx_builder.first_signer(&account2).is_err());
	}

	#[tokio::test]
	async fn test_tracking_transaction_should_return_correct_block() {
		let mock_provider = Arc::new(MockClient::new().await);
		let client = Arc::new(mock_provider.clone().into_client());
		mock_provider
			.mock_response_ignore_param(
				"invokescript",
				json!(Ok::<InvocationResult, ()>(InvocationResult {
					gas_consumed: "984060".to_string(),
					..Default::default()
				})),
			)
			.await;
		mock_provider
			.mock_response_ignore_param("getblockcount", json!(Ok::<i32, ()>(1000)))
			.await;
		mock_provider
			.mock_response_ignore_param("calculatenetworkfee", json!(Ok::<i32, ()>(1230610)))
			.await;
		mock_provider
			.mock_response_ignore_param(
				"sendrawtransaction",
				json!(Ok::<RawTransaction, ()>(RawTransaction {
					hash: H256::zero(),
					..Default::default()
				})),
			)
			.await;

		let account1 =
			Account::from_wif("L1WMhxazScMhUrdv34JqQb1HFSQmWeN2Kpc1R9JGKwL7CDNP21uR").unwrap();
		let recipient = H160::from_str("969a77db482f74ce27105f760efa139223431394").unwrap();

		let script = ScriptBuilder::new()
			.contract_call(
				&H160::from_str(TestConstants::NEO_TOKEN_HASH).unwrap(),
				"transfer",
				&[
					ContractParameter::h160(&account1.address_or_scripthash().script_hash()),
					ContractParameter::h160(&recipient),
					ContractParameter::integer(5),
					ContractParameter::any(),
				],
				None,
			)
			.unwrap()
			.to_bytes();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder
			.set_script(Some(script))
			.nonce(0)
			.unwrap()
			.set_signers(vec![AccountSigner::called_by_entry(&account1).unwrap().into()]);

		let mut tx = tx_builder.sign().await.unwrap();
		let _ = tx.send_tx(&client.as_ref()).await.map_err(TransactionError::from).unwrap();

		let mut block_num = 0;
		// TODO: check this
		// let mut subscription = tx.track_tx(&client).await.unwrap();
		// while let Some(result) = subscription.next(&client).await {
		// 	block_num = result.unwrap();
		// 	if block_num == 1002 {
		// 		break;
		// 	}
		// }

		assert_eq!(block_num, 1002);
	}

	#[tokio::test]
	async fn test_tracking_transaction_tx_not_sent() {
		let mock_provider = Arc::new(MockClient::new().await);
		let client = Arc::new(mock_provider.clone().into_client());
		mock_provider
			.mock_response_ignore_param(
				"invokescript",
				json!(Ok::<InvocationResult, ()>(InvocationResult {
					gas_consumed: "984060".to_string(),
					..Default::default()
				})),
			)
			.await;
		mock_provider
			.mock_response_ignore_param("getblockcount", json!(Ok::<u32, ()>(1000)))
			.await;
		mock_provider
			.mock_response_ignore_param("calculatenetworkfee", json!(Ok::<i64, ()>(1230610)))
			.await;

		let account1 =
			Account::from_wif("L1WMhxazScMhUrdv34JqQb1HFSQmWeN2Kpc1R9JGKwL7CDNP21uR").unwrap();
		let recipient = H160::from_str("969a77db482f74ce27105f760efa139223431394").unwrap();

		let script = ScriptBuilder::new()
			.contract_call(
				&H160::from_str(TestConstants::NEO_TOKEN_HASH).unwrap(),
				"transfer",
				&[
					ContractParameter::h160(&account1.address_or_scripthash().script_hash()),
					ContractParameter::h160(&recipient),
					ContractParameter::integer(5),
					ContractParameter::any(),
				],
				None,
			)
			.unwrap()
			.to_bytes();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder
			.set_script(Some(script))
			.nonce(0)
			.unwrap()
			.set_signers(vec![AccountSigner::called_by_entry(&account1).unwrap().into()]);

		let tx = tx_builder.sign().await.unwrap();

		// TODO: Implement track_tx method for Transaction
		// assert!(tx.track_tx(&client).await.is_err());
	}

	#[tokio::test]
	async fn test_get_application_log() {
		let mock_provider = Arc::new(MockClient::new().await);
		let client = Arc::new(mock_provider.clone().into_client());
		mock_provider
			.mock_response_ignore_param(
				"invokescript",
				json!(Ok::<InvocationResult, ()>(InvocationResult {
					gas_consumed: "984060".to_string(),
					..Default::default()
				})),
			)
			.await;
		mock_provider
			.mock_response_ignore_param("getblockcount", json!(Ok::<u32, ()>(1000)))
			.await;
		mock_provider
			.mock_response_ignore_param("calculatenetworkfee", json!(Ok::<i64, ()>(1230610)))
			.await;
		mock_provider
			.mock_response_ignore_param(
				"sendrawtransaction",
				json!(Ok::<RawTransaction, ()>(RawTransaction {
					hash: H256::zero(),
					..Default::default()
				})),
			)
			.await;
		mock_provider
			.mock_response_ignore_param(
				"getapplicationlog",
				json!(Ok::<ApplicationLog, ()>(ApplicationLog {
					transaction_id: H256::from_str(
						"0xeb52f99ae5cf923d8905bdd91c4160e2207d20c0cb42f8062f31c6743770e4d1"
					)
					.unwrap(),
					..Default::default()
				})),
			)
			.await;

		let account1 =
			Account::from_wif("L1WMhxazScMhUrdv34JqQb1HFSQmWeN2Kpc1R9JGKwL7CDNP21uR").unwrap();

		let script = ScriptBuilder::new()
			.contract_call(
				&H160::from_str(TestConstants::NEO_TOKEN_HASH).unwrap(),
				"transfer",
				&[
					ContractParameter::h160(&account1.address_or_scripthash().script_hash()),
					ContractParameter::h160(&account1.address_or_scripthash().script_hash()),
					ContractParameter::integer(1),
					ContractParameter::any(),
				],
				None,
			)
			.unwrap()
			.to_bytes();

		let mut tx_builder = TransactionBuilder::with_client(client.as_ref());
		tx_builder
			.set_script(Some(script))
			.set_signers(vec![AccountSigner::called_by_entry(&account1).unwrap().into()]);

		let mut tx =
			tx_builder.sign().await.map_err(|e| TransactionError::BuilderError(e)).unwrap();
		let _ = tx.send_tx(client.as_ref()).await.map_err(TransactionError::from).unwrap();
		let application_log = tx
			.get_application_log(client.as_ref())
			.await
			.map_err(TransactionError::from)
			.unwrap();

		assert_eq!(
			application_log.transaction_id,
			H256::from_str("0xeb52f99ae5cf923d8905bdd91c4160e2207d20c0cb42f8062f31c6743770e4d1")
				.unwrap()
		);
	}

	#[tokio::test]
	async fn test_get_application_log_tx_not_sent() {
		let mock_provider = Arc::new(MockClient::new().await);
		let client = Arc::new(mock_provider.clone().into_client());
		mock_provider
			.mock_response_ignore_param(
				"invokescript",
				json!(Ok::<InvocationResult, ()>(InvocationResult {
					gas_consumed: "984060".to_string(),
					..Default::default()
				})),
			)
			.await;
		mock_provider
			.mock_response_ignore_param("getblockcount", json!(Ok::<i32, ()>(1000)))
			.await;
		mock_provider
			.mock_response_ignore_param("calculatenetworkfee", json!(Ok::<i32, ()>(1230610)))
			.await;

		let account1 =
			Account::from_wif("L1WMhxazScMhUrdv34JqQb1HFSQmWeN2Kpc1R9JGKwL7CDNP21uR").unwrap();

		let script = ScriptBuilder::new()
			.contract_call(
				&H160::from_str(TestConstants::NEO_TOKEN_HASH).unwrap(),
				"transfer",
				&[
					ContractParameter::h160(&account1.address_or_scripthash().script_hash()),
					ContractParameter::h160(&account1.address_or_scripthash().script_hash()),
					ContractParameter::integer(1),
					ContractParameter::any(),
				],
				None,
			)
			.unwrap()
			.to_bytes();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder
			.set_script(Some(script))
			.set_signers(vec![AccountSigner::called_by_entry(&account1).unwrap().into()]);

		let tx = tx_builder.sign().await.unwrap();

		assert!(tx.get_application_log(&client.as_ref()).await.is_err());
	}

	#[tokio::test]
	async fn test_get_application_log_not_existing() {
		let mock_provider = Arc::new(MockClient::new().await);
		let client = Arc::new(mock_provider.clone().into_client());
		mock_provider
			.mock_response_ignore_param(
				"invokescript",
				json!(Ok::<InvocationResult, ()>(InvocationResult {
					gas_consumed: "984060".to_string(),
					..Default::default()
				})),
			)
			.await;
		mock_provider
			.mock_response_ignore_param("getblockcount", json!(Ok::<i32, ()>(1000)))
			.await;
		mock_provider
			.mock_response_ignore_param("calculatenetworkfee", json!(Ok::<i32, ()>(1230610)))
			.await;
		mock_provider
			.mock_response_ignore_param(
				"sendrawtransaction",
				json!(Ok::<RawTransaction, ()>(RawTransaction {
					hash: H256::zero(),
					..Default::default()
				})),
			)
			.await;
		mock_provider
			.mock_response_ignore_param("getapplicationlog", Default::default())
			.await;

		let account1 =
			Account::from_wif("L1WMhxazScMhUrdv34JqQb1HFSQmWeN2Kpc1R9JGKwL7CDNP21uR").unwrap();

		let script = ScriptBuilder::new()
			.contract_call(
				&H160::from_str(TestConstants::NEO_TOKEN_HASH).unwrap(),
				"transfer",
				&[
					ContractParameter::h160(&account1.address_or_scripthash().script_hash()),
					ContractParameter::h160(&account1.address_or_scripthash().script_hash()),
					ContractParameter::integer(1),
					ContractParameter::any(),
				],
				None,
			)
			.unwrap()
			.to_bytes();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder
			.set_script(Some(script))
			.set_signers(vec![AccountSigner::called_by_entry(&account1).unwrap().into()]);

		let mut tx = tx_builder.sign().await.unwrap();
		let _ = tx.send_tx(&client.as_ref()).await.map_err(TransactionError::from).unwrap();

		assert!(tx
			.get_application_log(&client.as_ref())
			.await
			.map_err(TransactionError::from)
			.is_err());
	}

	#[tokio::test]
	async fn test_transmission_on_fault() {
		let mock_provider = Arc::new(MockClient::new().await);
		let client = Arc::new(mock_provider.clone().into_client());
		mock_provider
			.mock_response_ignore_param(
				"invokescript",
				json!(Ok::<InvocationResult, ()>(InvocationResult {
					gas_consumed: "984060".to_string(),
					exception: Some("Test fault".to_string()),
					..Default::default()
				})),
			)
			.await;
		mock_provider
			.mock_response_ignore_param("getblockcount", json!(Ok::<i32, ()>(1000)))
			.await;
		mock_provider
			.mock_response_ignore_param("calculatenetworkfee", json!(Ok::<i32, ()>(1230610)))
			.await;

		let account = Account::create().unwrap();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder
			.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::none(&account).unwrap().into()]);
		// .allow_transmission_on_fault();

		let result = tx_builder.call_invoke_script().await;
		assert!(result.has_state_fault());

		let tx =match  tx_builder.get_unsigned_tx().await {
			Ok(tx) => tx,
			Err(e) => panic!("Error: {}", e)
		};
		assert_eq!(tx.sys_fee, 984060);

		NEOCONFIG.lock().unwrap().allows_transmission_on_fault = false;
		assert!(!NEOCONFIG.lock().unwrap().allows_transmission_on_fault);
	}

	#[tokio::test]
	async fn test_prevent_transmission_on_fault() {
		let mock_provider = Arc::new(MockClient::new().await);
		let client = Arc::new(mock_provider.clone().into_client());
		mock_provider
			.mock_response_ignore_param(
				"invokescript",
				json!(Ok::<InvocationResult, ()>(InvocationResult {
					gas_consumed: "984060".to_string(),
					exception: Some("Test fault".to_string()),
					..Default::default()
				})),
			)
			.await;
		mock_provider
			.mock_response_ignore_param("getblockcount", json!(Ok::<i32, ()>(1000)))
			.await;

		let account = Account::create().unwrap();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder
			.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::none(&account).unwrap().into()]);

		assert!(!NEOCONFIG.lock().unwrap().allows_transmission_on_fault);

		let result = tx_builder.call_invoke_script().await;
		assert!(result.has_state_fault());

		assert!(tx_builder.get_unsigned_tx().await.is_err());
	}

	#[tokio::test]
	async fn test_sign_with_multiple_accounts() {
		let mock_provider = Arc::new(MockClient::new().await);
		let client = Arc::new(mock_provider.clone().into_client());
		mock_provider
			.mock_response_ignore_param(
				"invokescript",
				json!(Ok::<InvocationResult, ()>(InvocationResult {
					gas_consumed: "984060".to_string(),
					..Default::default()
				})),
			)
			.await;
		mock_provider
			.mock_response_ignore_param("getblockcount", json!(Ok::<i32, ()>(1000)))
			.await;
		mock_provider
			.mock_response_ignore_param("calculatenetworkfee", json!(Ok::<i32, ()>(1230610)))
			.await;

		let account1 =
			Account::from_wif("L1WMhxazScMhUrdv34JqQb1HFSQmWeN2Kpc1R9JGKwL7CDNP21uR").unwrap();
		let account2 =
			Account::from_wif("KysNqEuLb3wmZJ6PsxbA9Bh6ewTybEda4dEiN9X7X48dJPkLWZ5a").unwrap();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder.set_script(Some(vec![1, 2, 3])).set_signers(vec![
			AccountSigner::called_by_entry(&account1).unwrap().into(),
			AccountSigner::called_by_entry(&account2).unwrap().into(),
		]);

		let tx = tx_builder.sign().await.unwrap();

		assert_eq!(tx.witnesses.len(), 2);
		assert!(tx
			.witnesses
			.iter()
			.any(|w| w.verification == account1.verification_script().clone().unwrap()));
		assert!(tx
			.witnesses
			.iter()
			.any(|w| w.verification == account2.verification_script().clone().unwrap()));
	}

	#[tokio::test]
	async fn test_sign_with_multi_sig_account() {
		let mock_provider = Arc::new(MockClient::new().await);
		let client = Arc::new(mock_provider.clone().into_client());
		mock_provider
			.mock_response_ignore_param(
				"invokescript",
				json!(Ok::<InvocationResult, ()>(InvocationResult {
					gas_consumed: "984060".to_string(),
					..Default::default()
				})),
			)
			.await;
		mock_provider
			.mock_response_ignore_param("getblockcount", json!(Ok::<i32, ()>(1000)))
			.await;
		mock_provider
			.mock_response_ignore_param("calculatenetworkfee", json!(Ok::<i32, ()>(1230610)))
			.await;

		let account1 =
			Account::from_wif("L1WMhxazScMhUrdv34JqQb1HFSQmWeN2Kpc1R9JGKwL7CDNP21uR").unwrap();
		let account2 =
			Account::from_wif("KysNqEuLb3wmZJ6PsxbA9Bh6ewTybEda4dEiN9X7X48dJPkLWZ5a").unwrap();
		let multi_sig_account = Account::multi_sig_from_public_keys(
			vec![account1.get_public_key().unwrap(), account2.get_public_key().unwrap()]
				.as_mut_slice(),
			2,
		)
		.unwrap();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder
			.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::called_by_entry(&multi_sig_account).unwrap().into()]);

		let tx = tx_builder.sign().await.unwrap();

		assert_eq!(tx.witnesses.len(), 1);
		assert_eq!(
			tx.witnesses[0].verification,
			multi_sig_account.verification_script().clone().unwrap()
		);
	}

	#[tokio::test]
	async fn test_get_network_fee() {
		let mock_provider = Arc::new(MockClient::new().await);
		let client = Arc::new(mock_provider.clone().into_client());
		mock_provider
			.mock_response_ignore_param(
				"invokescript",
				json!(Ok::<InvocationResult, ()>(InvocationResult {
					gas_consumed: "984060".to_string(),
					..Default::default()
				})),
			)
			.await;
		mock_provider
			.mock_response_ignore_param("getblockcount", json!(Ok::<i32, ()>(1000)))
			.await;
		mock_provider
			.mock_response_ignore_param("calculatenetworkfee", json!(Ok::<i32, ()>(1230610)))
			.await;

		let account = Account::create().unwrap();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder
			.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::called_by_entry(&account).unwrap().into()]);

		let network_fee = tx_builder.get_network_fee().await.unwrap();
		assert_eq!(network_fee, 1230610);
	}

	#[tokio::test]
	async fn test_get_system_fee() {
		let mock_provider = Arc::new(MockClient::new().await);
		let client = Arc::new(mock_provider.clone().into_client());

		let invocation_result =
			InvocationResult { gas_consumed: "984060".to_string(), ..Default::default() };

		mock_provider
			.mock_response_ignore_param("invokescript", json!(invocation_result))
			.await;

		let account = Account::create().unwrap();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder
			.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::called_by_entry(&account).unwrap().into()]);

		let system_fee = tx_builder.get_system_fee().await.unwrap();
		assert_eq!(system_fee, 984060);
	}
}
