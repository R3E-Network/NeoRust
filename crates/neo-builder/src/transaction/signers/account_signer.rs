use std::hash::{Hash, Hasher};

// Local imports
use crate::{BuilderError, TransactionError};
use crate::transaction::signers::signer::{SignerTrait, SignerType};
use crate::transaction::witness_rule::witness_rule::WitnessRule;

// External crate imports
use neo_codec::{Decoder, Encoder, NeoSerializable, VarSizeTrait};
use neo_config::NeoConstants;
use neo_crypto::{PublicKeyExtension, Secp256r1PublicKey};
#[cfg(feature = "protocol")]
use neo_protocol::AccountTrait;
#[cfg(feature = "protocol")]
use neo_protocol::account::Account;
use neo_common::Wallet;
use neo_common::WitnessScope;
use neo_types::ScriptHashExtension;
use neo_common::h160_utils::{serialize_h160, deserialize_h160, serialize_vec_h160, deserialize_vec_h160};
use getset::{Getters, Setters};
use primitive_types::H160;
use serde::{Deserialize, Serialize};

/// Represents an account signer in the NEO blockchain.
///
/// This struct contains information about the account signer, including
/// the signer hash, scopes, allowed contracts, allowed groups, and witness rules.
#[derive(Debug, Clone, Serialize, Deserialize, Getters, Setters)]
pub struct AccountSigner {
#[serde(
		serialize_with = "serialize_h160",
		deserialize_with = "deserialize_h160"
	)]
	pub(crate) signer_hash: H160,
	pub(crate) scopes: Vec<WitnessScope>,
#[serde(
		serialize_with = "serialize_vec_h160",
		deserialize_with = "deserialize_vec_h160"
	)]
	pub(crate) allowed_contracts: Vec<H160>,
	pub(crate) allowed_groups: Vec<Secp256r1PublicKey>,
	rules: Vec<WitnessRule>,
	#[getset(get = "pub")]
	#[cfg(feature = "protocol")]
	pub account: Account<Wallet>,
	#[cfg(not(feature = "protocol"))]
	pub account: Wallet,
}

impl AccountSigner {
	/// Creates a new `AccountSigner` with the specified scope.
	///
	/// # Arguments
	///
	/// * `account` - The account to create the signer for.
	/// * `scope` - The witness scope to use.
	#[cfg(feature = "protocol")]
	pub fn new(account: &Account<Wallet>, scope: WitnessScope) -> Self {
		Self {
			signer_hash: account.get_script_hash().clone(),
			scopes: vec![scope],
			allowed_contracts: vec![],
			allowed_groups: vec![],
			rules: vec![],
			account: account.clone(),
		}
	}

	/// Creates a new `AccountSigner` with no scope.
	///
	/// # Arguments
	///
	/// * `account` - The account to create the signer for.
	#[cfg(feature = "protocol")]
	pub fn none(account: &Account<Wallet>) -> Result<Self, TransactionError> {
		Ok(Self::new(account, WitnessScope::None))
	}

	/// Creates a new `AccountSigner` with the "Called By Entry" scope.
	///
	/// # Arguments
	///
	/// * `account` - The account to create the signer for.
	#[cfg(feature = "protocol")]
	pub fn called_by_entry(account: &Account<Wallet>) -> Result<Self, TransactionError> {
		Ok(Self::new(account, WitnessScope::CalledByEntry))
	}

	/// Creates a new `AccountSigner` with the "Global" scope.
	///
	/// # Arguments
	///
	/// * `account` - The account to create the signer for.
	#[cfg(feature = "protocol")]
	pub fn global(account: &Account<Wallet>) -> Result<Self, TransactionError> {
		Ok(Self::new(account, WitnessScope::Global))
	}

	/// Checks if the account is a multi-signature account.
	#[cfg(feature = "protocol")]
	pub fn is_multi_sig(&self) -> bool {
		matches!(&self.account.verification_script(), Some(script) if script.is_multi_sig())
	}

	#[cfg(not(feature = "protocol"))]
	pub fn is_multi_sig(&self) -> bool {
		false // Default implementation when protocol feature is not enabled
	}

	/// Returns the script hash of the account.
	#[cfg(feature = "protocol")]
	pub fn get_script_hash(&self) -> H160 {
		self.account.get_script_hash().clone()
	}

	#[cfg(not(feature = "protocol"))]
	pub fn get_script_hash(&self) -> H160 {
		self.signer_hash.clone()
	}
}

impl NeoSerializable for AccountSigner {
	type Error = TransactionError;

	fn size(&self) -> usize {
		let mut size: usize = NeoConstants::HASH160_SIZE as usize + 1;
		if self.scopes.contains(&WitnessScope::CustomContracts) {
			size += self.allowed_contracts.var_size();
		}
		if self.scopes.contains(&WitnessScope::CustomGroups) {
			size += self.allowed_groups.var_size();
		}
		if self.scopes.contains(&WitnessScope::WitnessRules) {
			size += self.rules.var_size();
		}
		size
	}

	fn encode(&self, writer: &mut Encoder) {
		writer.write_serializable_fixed(&self.signer_hash);
		writer.write_u8(WitnessScope::combine(&self.scopes));
		if self.scopes.contains(&WitnessScope::CustomContracts) {
			writer.write_serializable_variable_list(&self.allowed_contracts);
		}
		if self.scopes.contains(&WitnessScope::CustomGroups) {
			writer.write_serializable_variable_list(&self.allowed_groups);
		}
		if self.scopes.contains(&WitnessScope::WitnessRules) {
			writer.write_serializable_variable_list(&self.rules);
		}
	}

	fn decode(reader: &mut Decoder<'_>) -> Result<Self, Self::Error>
	where
		Self: Sized,
	{
		let signer_hash = reader.read_serializable::<H160>().unwrap();
		let scopes = WitnessScope::split(reader.read_u8());
		let mut allowed_contracts = vec![];
		let mut allowed_groups = vec![];
		let mut rules = vec![];
		if scopes.contains(&WitnessScope::CustomContracts) {
			allowed_contracts = reader.read_serializable_list::<H160>().unwrap();
			if allowed_contracts.len() > NeoConstants::MAX_SIGNER_SUBITEMS as usize {
				return Err(BuilderError::SignerConfiguration(format!(
                    "A signer's scope can only contain {} allowed contracts. The input data contained {} contracts.",
                    NeoConstants::MAX_SIGNER_SUBITEMS,
                    allowed_contracts.len()
                ))
                    .into());
			}
		}
		if scopes.contains(&WitnessScope::CustomGroups) {
			allowed_groups = reader.read_serializable_list::<Secp256r1PublicKey>().unwrap();
			if allowed_groups.len() > NeoConstants::MAX_SIGNER_SUBITEMS as usize {
				return Err(BuilderError::SignerConfiguration(format!(
                    "A signer's scope can only contain {} allowed contract groups. The input data contained {} groups.",
                    NeoConstants::MAX_SIGNER_SUBITEMS,
                    allowed_groups.len()
                ))
                    .into());
			}
		}
		if scopes.contains(&WitnessScope::WitnessRules) {
			rules = reader.read_serializable_list::<WitnessRule>().unwrap();
			if rules.len() > NeoConstants::MAX_SIGNER_SUBITEMS as usize {
				return Err(BuilderError::SignerConfiguration(format!(
                    "A signer's scope can only contain {} rules. The input data contained {} rules.",
                    NeoConstants::MAX_SIGNER_SUBITEMS,
                    rules.len()
                ))
                    .into());
			}
		}

		#[cfg(feature = "protocol")]
		{
			let account = Account::from_address(signer_hash.to_address().as_str()).unwrap();
			Ok(Self {
				signer_hash,
				scopes,
				allowed_contracts,
				allowed_groups,
				rules,
				account,
			})
		}

		#[cfg(not(feature = "protocol"))]
		{
			let wallet = Wallet::new("NeoRustDefaultAddress".to_string());
			Ok(Self {
				signer_hash,
				scopes,
				allowed_contracts,
				allowed_groups,
				rules,
				account: wallet,
			})
		}
	}

	fn to_array(&self) -> Vec<u8> {
		let mut writer = Encoder::new();
		self.encode(&mut writer);
		writer.to_bytes()
	}
}

impl PartialEq for AccountSigner {
	fn eq(&self, other: &Self) -> bool {
		self.signer_hash == other.signer_hash
			&& self.scopes == other.scopes
			&& self.allowed_contracts == other.allowed_contracts
			&& self.allowed_groups == other.allowed_groups
			&& self.rules == other.rules
		// && self.account == other.account
	}
}

impl Hash for AccountSigner {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.signer_hash.hash(state);
		self.scopes.hash(state);
		self.allowed_contracts.hash(state);
		for group in self.allowed_groups.iter() {
			group.to_vec().hash(state);
		}
		// self.allowed_groups.to_vec().hash(state);
		self.rules.hash(state);
		// self.account.hash(state);
		// self.scope.hash(state);
	}
}

impl SignerTrait for AccountSigner {
	fn get_type(&self) -> SignerType {
		SignerType::AccountSigner
	}

	fn get_signer_hash(&self) -> &H160 {
		&self.signer_hash
	}

	fn set_signer_hash(&mut self, signer_hash: H160) {
		self.signer_hash = signer_hash;
	}

	fn get_scopes(&self) -> &Vec<WitnessScope> {
		&self.scopes
	}

	fn get_scopes_mut(&mut self) -> &mut Vec<WitnessScope> {
		&mut self.scopes
	}

	fn set_scopes(&mut self, scopes: Vec<WitnessScope>) {
		self.scopes = scopes;
	}

	fn get_allowed_contracts(&self) -> &Vec<H160> {
		&self.allowed_contracts
	}

	fn get_allowed_contracts_mut(&mut self) -> &mut Vec<H160> {
		&mut self.allowed_contracts
	}

	fn get_allowed_groups(&self) -> &Vec<Secp256r1PublicKey> {
		&self.allowed_groups
	}

	fn get_allowed_groups_mut(&mut self) -> &mut Vec<Secp256r1PublicKey> {
		&mut self.allowed_groups
	}

	fn get_rules(&self) -> &Vec<crate::transaction::witness_rule::witness_rule::WitnessRule> {
		// This is a temporary solution until we fully migrate to neo-common
		unsafe { std::mem::transmute(&self.rules) }
	}

	fn get_rules_mut(&mut self) -> &mut Vec<crate::transaction::witness_rule::witness_rule::WitnessRule> {
		// This is a temporary solution until we fully migrate to neo-common
		unsafe { std::mem::transmute(&mut self.rules) }
	}
}

impl AccountSigner {
	pub fn none_hash160(account_hash: H160) -> Result<Self, TransactionError> {
		#[cfg(feature = "protocol")]
		{
			let account = neo_protocol::Account::from_address(account_hash.to_address().as_str()).unwrap();
			Ok(Self::none(&account)?)
		}
		
		#[cfg(not(feature = "protocol"))]
		{
			let wallet = Wallet::new(account_hash.to_address());
			Ok(Self {
				signer_hash: account_hash,
				scopes: vec![WitnessScope::None],
				allowed_contracts: vec![],
				allowed_groups: vec![],
				rules: vec![],
				account: wallet,
			})
		}
	}

	pub fn called_by_entry_hash160(account_hash: H160) -> Result<Self, TransactionError> {
		#[cfg(feature = "protocol")]
		{
			let account = neo_protocol::Account::from_address(account_hash.to_address().as_str()).unwrap();
			Ok(Self::called_by_entry(&account)?)
		}

		#[cfg(not(feature = "protocol"))]
		{
			let wallet = Wallet::new(account_hash.to_address());
			Ok(Self {
				signer_hash: account_hash,
				scopes: vec![WitnessScope::CalledByEntry],
				allowed_contracts: vec![],
				allowed_groups: vec![],
				rules: vec![],
				account: wallet,
			})
		}
	}

	pub fn global_hash160(account_hash: H160) -> Result<Self, TransactionError> {
		#[cfg(feature = "protocol")]
		{
			let account = neo_protocol::Account::from_address(account_hash.to_address().as_str()).unwrap();
			Ok(Self::global(&account)?)
		}

		#[cfg(not(feature = "protocol"))]
		{
			let wallet = Wallet::new(account_hash.to_address());
			Ok(Self {
				signer_hash: account_hash,
				scopes: vec![WitnessScope::Global],
				allowed_contracts: vec![],
				allowed_groups: vec![],
				rules: vec![],
				account: wallet,
			})
		}
	}
}
