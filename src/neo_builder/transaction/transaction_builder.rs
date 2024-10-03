/// A builder for constructing and configuring NEO blockchain transactions.
///
/// The `TransactionBuilder` provides a fluent interface for setting various transaction parameters
/// such as version, nonce, validity period, signers, fees, and script. Once configured, it can
/// generate an unsigned transaction.
///
/// # Fields
///
/// - `client`: An optional reference to an RPC client for network operations.
/// - `version`: The transaction version.
/// - `nonce`: A random number to prevent transaction duplication.
/// - `valid_until_block`: The block height until which the transaction is valid.
/// - `signers`: A list of transaction signers.
/// - `additional_network_fee`: Additional network fee for the transaction.
/// - `additional_system_fee`: Additional system fee for the transaction.
/// - `attributes`: Transaction attributes.
/// - `script`: The transaction script.
/// - `fee_consumer`: An optional closure for fee-related operations.
/// - `fee_error`: An optional error related to fee calculations.
///
/// # Example
///
/// ```rust
/// use NeoRust::prelude::TransactionBuilder;
///
/// let mut tx_builder = TransactionBuilder::new();
/// tx_builder.version(0)
///           .nonce(1)
///           .valid_until_block(100)
///           .extend_script(vec![0x01, 0x02, 0x03]);
///
/// let unsigned_tx = tx_builder.get_unsigned_tx().await.unwrap();
/// ```
///
/// # Note
///
/// This builder implements `Debug`, `Clone`, `Eq`, `PartialEq`, and `Hash` traits.
/// It uses generics to allow for different types of JSON-RPC providers.
use futures_util::TryFutureExt;
use std::{
	cell::RefCell,
	collections::HashSet,
	fmt::Debug,
	hash::{Hash, Hasher},
	iter::Iterator,
	str::FromStr,
};

use getset::{CopyGetters, Getters, MutGetters, Setters};
use once_cell::sync::Lazy;
use primitive_types::H160;
use rustc_serialize::hex::ToHex;

use neo::{neo_types::ScriptHashExtension, prelude::*};

#[derive(Getters, Setters, MutGetters, CopyGetters, Default)]
pub struct TransactionBuilder<'a, P: JsonRpcProvider + 'static> {
	pub(crate) client: Option<&'a RpcClient<P>>,
	version: u8,
	nonce: u32,
	valid_until_block: Option<u32>,
	// setter and getter
	#[getset(get = "pub")]
	pub(crate) signers: Vec<Signer>,
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
	pub async fn get_unsigned_tx(&mut self) -> Result<Transaction<P>, TransactionError> {
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
		// let script = self.script.as_ref().unwrap();
		// let response = self
		// 	.client
		// 	.unwrap()
		// 	.invoke_script(script.to_hex(), vec![self.signers[0].clone()])
		// 	.await
		// 	.map_err(|e| TransactionError::ProviderError(e))?;

		let system_fee = self.get_system_fee().await.unwrap() + self.additional_system_fee as i64;

		// Check sender balance if needed
		let mut tx = Transaction {
			network: Some(self.client.unwrap()),
			version: self.version,
			nonce: self.nonce,
			valid_until_block: self.valid_until_block.unwrap_or(100),
			size: 0,
			sys_fee: system_fee,
			net_fee: 0,
			signers: self.signers.clone(),
			attributes: self.attributes.clone(),
			script: self.script.clone().unwrap(), // We've already checked for None case above
			witnesses: vec![],
			// block_time: None,
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

	pub(crate) async fn get_system_fee(&self) -> Result<i64, TransactionError> {
		let script = self.script.as_ref().unwrap();

		let response = self
			.client
			.unwrap()
			.invoke_script(script.to_hex(), vec![self.signers[0].clone()])
			.await
			.map_err(|e| TransactionError::ProviderError(e))?;
		if response.has_state_fault() && !self.client.unwrap().allow_transmission_on_fault() {
			return Err(TransactionError::TransactionConfiguration(format!("The vm exited due to the following exception: {}", response.exception.unwrap())));
		}
		Ok(i64::from_str(response.gas_consumed.as_str()).unwrap()) // example
	}

	pub(crate) async fn get_network_fee(&mut self) -> Result<i64, TransactionError> {
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
		if signer.get_type() == SignerType::AccountSigner {
			return true;
		}
		return false;
	}

	// Sign transaction
	pub async fn sign(&mut self) -> Result<Transaction<P>, BuilderError> {
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

	pub fn set_signers(&mut self, signers: Vec<Signer>) -> Result<&mut Self, TransactionError> {
        if self.contains_duplicate_signers(&signers) {
            return Err(TransactionError::TransactionConfiguration("Cannot add multiple signers concerning the same account.".to_string()));
        }
        
        self.check_and_throw_if_max_attributes_exceeded(signers.len(), self.attributes.len())?;
        
        self.signers = signers;
        Ok(self)
    }

	fn contains_duplicate_signers(&self, signers: &Vec<Signer>) -> bool {
		let signer_list: Vec<H160> = signers.iter().map(|s| s.get_signer_hash().clone()).collect();
		let signer_set: HashSet<_> = signer_list.iter().collect();
		signer_list.len() != signer_set.len()
	}

	fn check_and_throw_if_max_attributes_exceeded(&self, total_signers: usize, total_attributes: usize) -> Result<(), TransactionError> {
		if total_signers + total_attributes > NeoConstants::MAX_TRANSACTION_ATTRIBUTES.try_into().unwrap() {
			return Err(TransactionError::TransactionConfiguration(format!(
				"A transaction cannot have more than {} attributes (including signers).",
				NeoConstants::MAX_TRANSACTION_ATTRIBUTES
			)));
		}
		Ok(())
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
