use crate::core::{
	account::AccountTrait,
	error::BuilderError,
	transaction::{
		signers::{
			account_signer::AccountSigner, contract_signer::ContractSigner,
			transaction_signer::TransactionSigner,
		},
		transaction_error::TransactionError,
		witness_rule::{witness_condition::WitnessCondition, witness_rule::WitnessRule},
		witness_scope::WitnessScope,
	},
};
use neo_codec::{encode::NeoSerializable, Decoder, Encoder};
use neo_config::NeoConstants;
use neo_crypto::keys::Secp256r1PublicKey;
use primitive_types::H160;
use serde::{Deserialize, Serialize, Serializer};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SignerType {
	Account,
	Contract,
	Transaction,
}

pub trait SignerTrait {
	fn get_type(&self) -> SignerType;

	fn get_signer_hash(&self) -> &H160;

	fn set_signer_hash(&mut self, signer_hash: H160);

	fn get_scopes(&self) -> &Vec<WitnessScope>;
	fn get_scopes_mut(&mut self) -> &mut Vec<WitnessScope>;

	fn set_scopes(&mut self, scopes: Vec<WitnessScope>);

	fn get_allowed_contracts(&self) -> &Vec<H160>;

	fn get_allowed_contracts_mut(&mut self) -> &mut Vec<H160>;

	// fn set_allowed_contracts(&mut self, allowed_contracts: Vec<H160>);

	fn get_allowed_groups(&self) -> &Vec<Secp256r1PublicKey>;
	fn get_allowed_groups_mut(&mut self) -> &mut Vec<Secp256r1PublicKey>;

	fn get_rules(&self) -> &Vec<WitnessRule>;
	fn get_rules_mut(&mut self) -> &mut Vec<WitnessRule>;

	// Set allowed contracts
	fn set_allowed_contracts(&mut self, contracts: Vec<H160>) -> Result<(), BuilderError> {
		// Validate
		if self.get_scopes().contains(&WitnessScope::Global) {
			return Err(BuilderError::TransactionConfiguration(
				"Cannot set contracts for global scope".to_string(),
			))
		}

		if self.get_allowed_contracts().len() + contracts.len()
			> NeoConstants::MAX_SIGNER_SUBITEMS as usize
		{
			return Err(BuilderError::TransactionConfiguration(
				"Too many allowed contracts".to_string(),
			))
		}

		// Update state
		if !self.get_scopes().contains(&WitnessScope::CustomContracts) {
			self.get_scopes_mut().push(WitnessScope::CustomContracts);
		}

		self.get_allowed_contracts_mut().extend(contracts);

		Ok(())
	}

	// Set allowed groups
	fn set_allowed_groups(&mut self, groups: Vec<Secp256r1PublicKey>) -> Result<(), BuilderError> {
		if self.get_scopes().contains(&WitnessScope::Global) {
			return Err(BuilderError::TransactionConfiguration(
				"Cannot set groups for global scope".to_string(),
			))
		}

		if self.get_allowed_groups().len() + groups.len()
			> NeoConstants::MAX_SIGNER_SUBITEMS as usize
		{
			return Err(BuilderError::TransactionConfiguration(
				"Too many allowed groups".to_string(),
			))
		}

		if !self.get_scopes().contains(&WitnessScope::CustomGroups) {
			self.get_scopes_mut().push(WitnessScope::CustomGroups);
		}

		self.get_allowed_groups_mut().extend(groups);

		Ok(())
	}

	// Set rules
	fn set_rules(&mut self, rules: Vec<WitnessRule>) -> Result<(), BuilderError> {
		if self.get_scopes().contains(&WitnessScope::Global) {
			return Err(BuilderError::TransactionConfiguration(
				"Cannot set rules for global scope".to_string(),
			))
		}

		if self.get_rules().len() + rules.len() > NeoConstants::MAX_SIGNER_SUBITEMS as usize {
			return Err(BuilderError::TransactionConfiguration("Too many rules".to_string()))
		}

		// Validate nesting depth
		for rule in &rules {
			self.validate_depth(&rule.condition, NeoConstants::MAX_NESTING_DEPTH).unwrap();
		}

		if !self.get_scopes().contains(&WitnessScope::WitnessRules) {
			self.get_scopes_mut().push(WitnessScope::WitnessRules);
		}

		self.get_rules_mut().extend(rules);

		Ok(())
	}

	// Check depth recursively
	fn validate_depth(&self, rule: &WitnessCondition, depth: u8) -> Result<(), BuilderError> {
		// Depth exceeded
		if depth == 0 {
			return Err(BuilderError::TransactionConfiguration(
				"Max nesting depth exceeded".to_string(),
			))
		}

		match &rule {
			WitnessCondition::And(conditions) | WitnessCondition::Or(conditions) => {
				for inner_rule in conditions {
					self.validate_depth(inner_rule, depth - 1).unwrap();
				}
			},
			_ => (),
		}

		Ok(())
	}
	fn validate_subitems(&self, count: usize, name: &str) -> Result<(), BuilderError> {
		if count > NeoConstants::MAX_SIGNER_SUBITEMS as usize {
			return Err(BuilderError::TooManySigners("".to_string()))
		}
		Ok(())
	}
}

#[derive(Debug, Clone, Deserialize)]
pub enum Signer {
	Account(AccountSigner),
	Contract(ContractSigner),
	Transaction(TransactionSigner),
}

impl PartialEq for Signer {
	fn eq(&self, other: &Self) -> bool {
		match self {
			Signer::Account(account_signer) => match other {
				Signer::Account(other_account_signer) =>
					account_signer.get_signer_hash() == other_account_signer.get_signer_hash(),
				_ => false,
			},
			Signer::Contract(contract_signer) => match other {
				Signer::Contract(other_contract_signer) =>
					contract_signer.get_signer_hash() == other_contract_signer.get_signer_hash(),
				_ => false,
			},
			Signer::Transaction(transaction_signer) => match other {
				Signer::Transaction(other_transaction_signer) =>
					transaction_signer.get_signer_hash()
						== other_transaction_signer.get_signer_hash(),
				_ => false,
			},
		}
	}
}

impl Signer {
	pub fn get_type(&self) -> SignerType {
		match self {
			Signer::Account(account_signer) => account_signer.get_type(),
			Signer::Contract(contract_signer) => contract_signer.get_type(),
			Signer::Transaction(transaction_signer) => transaction_signer.get_type(),
		}
	}
	pub fn get_signer_hash(&self) -> &H160 {
		match self {
			Signer::Account(account_signer) => account_signer.get_signer_hash(),
			Signer::Contract(contract_signer) => contract_signer.get_signer_hash(),
			Signer::Transaction(transaction_signer) => transaction_signer.get_signer_hash(),
		}
	}

	pub fn as_account_signer(&self) -> Option<&AccountSigner> {
		match self {
			Signer::Account(account_signer) => Some(account_signer),
			_ => None,
		}
	}

	pub fn as_contract_signer(&self) -> Option<&ContractSigner> {
		match self {
			Signer::Contract(contract_signer) => Some(contract_signer),
			_ => None,
		}
	}

	pub fn as_transaction_signer(&self) -> Option<&TransactionSigner> {
		match self {
			Signer::Transaction(transaction_signer) => Some(transaction_signer),
			_ => None,
		}
	}
}

impl Hash for Signer {
	fn hash<H: Hasher>(&self, state: &mut H) {
		match self {
			Signer::Account(account_signer) => account_signer.hash(state),
			Signer::Contract(contract_signer) => contract_signer.hash(state),
			Signer::Transaction(transaction_signer) => transaction_signer.hash(state),
		}
	}
}

impl From<AccountSigner> for Signer {
	fn from(account_signer: AccountSigner) -> Self {
		Signer::Account(account_signer)
	}
}

impl From<ContractSigner> for Signer {
	fn from(contract_signer: ContractSigner) -> Self {
		Signer::Contract(contract_signer)
	}
}

impl Into<AccountSigner> for Signer {
	fn into(self) -> AccountSigner {
		match self {
			Signer::Account(account_signer) => account_signer,
			_ => panic!("Cannot convert ContractSigner into AccountSigner"),
		}
	}
}

impl Into<TransactionSigner> for Signer {
	fn into(self) -> TransactionSigner {
		match self {
			Signer::Account(account_signer) =>
				panic!("Cannot convert AccountSigner into TransactionSigner"),
			Signer::Contract(contract_signer) =>
				panic!("Cannot convert ContractSigner into AccountSigner"),
			Signer::Transaction(transaction_signer) => transaction_signer,
		}
	}
}

impl Into<TransactionSigner> for &Signer {
	fn into(self) -> TransactionSigner {
		match self {
			Signer::Account(account_signer) =>
				panic!("Cannot convert AccountSigner into TransactionSigner"),
			Signer::Contract(contract_signer) =>
				panic!("Cannot convert ContractSigner into AccountSigner"),
			Signer::Transaction(transaction_signer) => transaction_signer.clone(),
		}
	}
}

impl Into<TransactionSigner> for &mut Signer {
	fn into(self) -> TransactionSigner {
		match self {
			Signer::Account(account_signer) =>
				panic!("Cannot convert AccountSigner into TransactionSigner"),
			Signer::Contract(contract_signer) =>
				panic!("Cannot convert ContractSigner into AccountSigner"),
			Signer::Transaction(transaction_signer) => transaction_signer.clone(),
		}
	}
}

impl Into<AccountSigner> for &mut Signer {
	fn into(self) -> AccountSigner {
		match self {
			Signer::Account(account_signer) => account_signer.clone(),
			Signer::Contract(contract_signer) =>
				panic!("Cannot convert ContractSigner into AccountSigner"),
			Signer::Transaction(transaction_signer) =>
				panic!("Cannot convert TransactionSigner into AccountSigner"),
		}
	}
}

impl Into<ContractSigner> for &mut Signer {
	fn into(self) -> ContractSigner {
		match self {
			Signer::Account(account_signer) =>
				panic!("Cannot convert AccountSigner into ContractSigner"),
			Signer::Contract(contract_signer) => contract_signer.clone(),
			Signer::Transaction(transaction_signer) =>
				panic!("Cannot convert TransactionSigner into ContractSigner"),
		}
	}
}

impl Into<ContractSigner> for Signer {
	fn into(self) -> ContractSigner {
		match self {
			Signer::Account(account_signer) =>
				panic!("Cannot convert AccountSigner into ContractSigner"),
			Signer::Contract(contract_signer) => contract_signer,
			Signer::Transaction(transaction_signer) =>
				panic!("Cannot convert TransactionSigner into ContractSigner"),
		}
	}
}

impl Serialize for Signer {
	fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
	where
		S: Serializer,
	{
		match self {
			Signer::Account(account_signer) => account_signer.serialize(serializer),
			Signer::Contract(contract_signer) => contract_signer.serialize(serializer),
			Signer::Transaction(transaction_signer) => transaction_signer.serialize(serializer),
		}
	}
}

impl NeoSerializable for Signer {
	type Error = TransactionError;

	fn size(&self) -> usize {
		match self {
			Signer::Account(account_signer) => account_signer.size(),
			Signer::Contract(contract_signer) => contract_signer.size(),
			// Signer::Transaction(transaction_signer) => transaction_signer.size(),
			_ => panic!("Unsupported signer type"),
		}
	}

	fn encode(&self, writer: &mut Encoder) {
		match self {
			Signer::Account(account_signer) => account_signer.encode(writer),
			Signer::Contract(contract_signer) => contract_signer.encode(writer),
			// Signer::Transaction(transaction_signer) => transaction_signer.encode(writer),
			_ => panic!("Unsupported signer type"),
		}
	}

	fn decode(reader: &mut Decoder) -> Result<Self, Self::Error>
	where
		Self: Sized,
	{
		match reader.read_u8() {
			0 => Ok(Signer::Account(AccountSigner::decode(reader)?)),
			1 => Ok(Signer::Contract(ContractSigner::decode(reader)?)),
			// 2 => Ok(Signer::Transaction(TransactionSigner::decode(reader)?)),
			_ => Err(TransactionError::InvalidTransaction),
		}
	}

	fn to_array(&self) -> Vec<u8> {
		match self {
			Signer::Account(account_signer) => account_signer.to_array(),
			Signer::Contract(contract_signer) => contract_signer.to_array(),
			// Signer::Transaction(transaction_signer) => transaction_signer.to_array(),
			_ => panic!("Unsupported signer type"),
		}
	}
}
