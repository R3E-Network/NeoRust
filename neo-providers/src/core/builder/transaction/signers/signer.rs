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

impl SignerTrait for Signer {
	fn get_type(&self) -> SignerType {
		match self {
			Signer::Account(account_signer) => account_signer.get_type(),
			Signer::Contract(contract_signer) => contract_signer.get_type(),
			Signer::Transaction(transaction_signer) => transaction_signer.get_type(),
		}
	}

	fn get_signer_hash(&self) -> &H160 {
		match self {
			Signer::Account(account_signer) => account_signer.get_signer_hash(),
			Signer::Contract(contract_signer) => contract_signer.get_signer_hash(),
			Signer::Transaction(transaction_signer) => transaction_signer.get_signer_hash(),
		}
	}

	fn set_signer_hash(&mut self, signer_hash: H160) {
		match self {
			Signer::Account(account_signer) => account_signer.set_signer_hash(signer_hash),
			Signer::Contract(contract_signer) => contract_signer.set_signer_hash(signer_hash),
			Signer::Transaction(transaction_signer) =>
				transaction_signer.set_signer_hash(signer_hash),
		}
	}

	fn get_scopes(&self) -> &Vec<WitnessScope> {
		match self {
			Signer::Account(account_signer) => account_signer.get_scopes(),
			Signer::Contract(contract_signer) => contract_signer.get_scopes(),
			Signer::Transaction(transaction_signer) => transaction_signer.get_scopes(),
		}
	}

	fn get_scopes_mut(&mut self) -> &mut Vec<WitnessScope> {
		match self {
			Signer::Account(account_signer) => account_signer.get_scopes_mut(),
			Signer::Contract(contract_signer) => contract_signer.get_scopes_mut(),
			Signer::Transaction(transaction_signer) => transaction_signer.get_scopes_mut(),
		}
	}

	fn set_scopes(&mut self, scopes: Vec<WitnessScope>) {
		match self {
			Signer::Account(account_signer) => account_signer.set_scopes(scopes),
			Signer::Contract(contract_signer) => contract_signer.set_scopes(scopes),
			Signer::Transaction(transaction_signer) => transaction_signer.set_scopes(scopes),
		}
	}

	fn get_allowed_contracts(&self) -> &Vec<H160> {
		match self {
			Signer::Account(account_signer) => account_signer.get_allowed_contracts(),
			Signer::Contract(contract_signer) => contract_signer.get_allowed_contracts(),
			Signer::Transaction(transaction_signer) => transaction_signer.get_allowed_contracts(),
		}
	}

	fn get_allowed_contracts_mut(&mut self) -> &mut Vec<H160> {
		match self {
			Signer::Account(account_signer) => account_signer.get_allowed_contracts_mut(),
			Signer::Contract(contract_signer) => contract_signer.get_allowed_contracts_mut(),
			Signer::Transaction(transaction_signer) =>
				transaction_signer.get_allowed_contracts_mut(),
		}
	}

	fn get_allowed_groups(&self) -> &Vec<Secp256r1PublicKey> {
		match self {
			Signer::Account(account_signer) => account_signer.get_allowed_groups(),
			Signer::Contract(contract_signer) => contract_signer.get_allowed_groups(),
			Signer::Transaction(transaction_signer) => transaction_signer.get_allowed_groups(),
		}
	}

	fn get_allowed_groups_mut(&mut self) -> &mut Vec<Secp256r1PublicKey> {
		match self {
			Signer::Account(account_signer) => account_signer.get_allowed_groups_mut(),
			Signer::Contract(contract_signer) => contract_signer.get_allowed_groups_mut(),
			Signer::Transaction(transaction_signer) => transaction_signer.get_allowed_groups_mut(),
		}
	}

	fn get_rules(&self) -> &Vec<WitnessRule> {
		match self {
			Signer::Account(account_signer) => account_signer.get_rules(),
			Signer::Contract(contract_signer) => contract_signer.get_rules(),
			Signer::Transaction(transaction_signer) => transaction_signer.get_rules(),
		}
	}

	fn get_rules_mut(&mut self) -> &mut Vec<WitnessRule> {
		match self {
			Signer::Account(account_signer) => account_signer.get_rules_mut(),
			Signer::Contract(contract_signer) => contract_signer.get_rules_mut(),
			Signer::Transaction(transaction_signer) => transaction_signer.get_rules_mut(),
		}
	}
}

impl Signer {
	pub fn from_bytes(data: &[u8]) -> Result<Signer, TransactionError> {
		let mut reader = Decoder::new(data);
		Signer::decode(&mut reader)
	}

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

#[cfg(test)]
mod tests {
	use crate::core::{
		account::{Account, AccountTrait},
		error::BuilderError,
		transaction::{
			signers::{
				account_signer,
				account_signer::AccountSigner,
				signer::{Signer, SignerTrait},
			},
			witness_rule::{
				witness_action::WitnessAction, witness_condition::WitnessCondition,
				witness_rule::WitnessRule,
			},
			witness_scope::WitnessScope,
		},
	};
	use neo_codec::{encode::NeoSerializable, Encoder};
	use neo_crypto::keys::Secp256r1PublicKey;
	use neo_types::script_hash::ScriptHashExtension;
	use primitive_types::H160;
	use rustc_serialize::hex::FromHex;
	use serde::Serialize;
	use std::{io::SeekFrom::End, str::FromStr};

	#[test]
	fn test_create_signer_with_call_by_entry_scope() {
		let account =
			Account::from_wif("Kxt94tAAiZSgH7Yt4i25DW6jJFprZFPSqTgLr5dWmWgKDKCjXMfZ").unwrap();
		let script_hash = H160::from_slice("d802a401".from_hex().unwrap().as_slice());

		let signer = AccountSigner::called_by_entry(script_hash.into()).unwrap();

		assert_eq!(signer.signer_hash, script_hash);
		assert_eq!(signer.scopes, vec![WitnessScope::CalledByEntry]);
		assert!(signer.allowed_contracts.is_empty());
	}

	#[test]
	fn test_fail_depth_check() {
		let script_hash = H160::from_hex("a94a94a942a8a8a9429a9").unwrap();

		let condition = WitnessCondition::And(vec![WitnessCondition::Not(Box::new(
			WitnessCondition::ScriptHash(script_hash),
		))]);

		let rule = WitnessRule::new(WitnessAction::Allow, condition);

		let mut signer = AccountSigner::none(script_hash.into()).unwrap();

		let err = signer.set_rules(vec![rule]).unwrap_err();

		assert_eq!(err, BuilderError::TooManySigners("".to_string()));
	}

	#[test]
	fn test_serialize_and_deserialize() {
		let script_hash = H160::from_hex("a94a94a942a8a8a9429a9").unwrap();

		let serialized = "9429a9a942a8a8a9429a917102d802a401c503b112\
        02a877f3c907cc6c2b66c295d1fcc76ff8f702958ab88e4cea7ae1848047daeb8883daf5fdf5c1301dbbfe973f0a29fe75de6001010128d802a401"
			.from_hex().unwrap();

		let signer = Signer::from_bytes(&serialized).unwrap();

		assert_eq!(signer.get_signer_hash(), &script_hash);

		assert_eq!(
			signer.get_scopes(),
			&vec![
				WitnessScope::CalledByEntry,
				WitnessScope::CustomContracts,
				WitnessScope::CustomGroups,
				WitnessScope::WitnessRules,
			]
		);

		let signer = Signer::from_bytes(&serialized).unwrap();

		// Assert hash
		assert_eq!(signer.get_signer_hash(), &script_hash);

		// Assert other properties
		assert_eq!(signer.get_allowed_contracts().len(), 2);

		let contract1 = H160::from_hex("d802a401").unwrap();
		let contract2 = H160::from_hex("c503b112").unwrap();
		assert_eq!(signer.get_allowed_contracts(), &vec![contract1, contract2]);

		assert_eq!(signer.get_allowed_groups().len(), 2);

		let group1 = Secp256r1PublicKey::from_encoded(
			"030306d3e7f18e6dd477d34ce3cfeca172a877f3c907cc6c2b66c295d1fcc76ff8f7",
		)
		.unwrap();
		let group2 = Secp256r1PublicKey::from_encoded(
			"02958ab88e4cea7ae1848047daeb8883daf5fdf5c1301dbbfe973f0a29fe75de60",
		)
		.unwrap();
		assert_eq!(signer.get_allowed_groups(), &vec![group1, group2]);

		assert_eq!(signer.get_rules().len(), 1);

		let rule = &signer.get_rules()[0];
		assert_eq!(rule.action, WitnessAction::Allow);
		assert_eq!(rule.condition, WitnessCondition::CalledByContract(contract1));
	}

	#[test]
	fn test_build_valid_signer() {
		let script_hash1 = H160::from_str("d802a401").unwrap();
		let script_hash2 = H160::from_str("c503b112").unwrap();

		let mut signer = AccountSigner::none(script_hash1.into()).unwrap();
		signer.set_allowed_contracts(vec![script_hash1, script_hash2]).expect("");

		assert_eq!(signer.signer_hash, script_hash1);
		assert_eq!(signer.scopes, vec![WitnessScope::CustomContracts]);
		assert_eq!(signer.allowed_contracts, vec![script_hash1, script_hash2]);
	}

	#[test]
	fn test_fail_building_signer_too_many_contracts() {
		let script_hash = H160::from_hex("a94a94a942a8a8a9429a9").unwrap();
		let contracts = (0..17).map(|_| script_hash).collect::<Vec<_>>();

		let err = AccountSigner::called_by_entry(script_hash.into())
			.unwrap()
			.set_allowed_contracts(contracts)
			.unwrap_err();

		assert_eq!(err, BuilderError::TooManySigners("".to_string()));
	}

	#[test]
	fn test_serialize_global_scope() {
		let script_hash = H160::zero();
		let mut buffer = Encoder::new();

		AccountSigner::global(script_hash.into()).unwrap().encode(&mut buffer);

		let expected = hex::decode("000000000000000000000000000000000000000000").unwrap();
		assert_eq!(buffer.to_bytes(), expected);
	}
	#[test]
	fn test_serialize_custom_contracts() {
		let script_hash1 = H160::from_str("d802a401").unwrap();
		let script_hash2 = H160::from_str("c503b112").unwrap();

		let mut buffer = Encoder::new();

		let mut signer = AccountSigner::none(script_hash1.into()).unwrap();
		signer.set_allowed_contracts(vec![script_hash1, script_hash2]).unwrap();

		signer.encode(&mut buffer);

		let expected = hex::decode("d8024011111d802a401c503b112").unwrap();
		assert_eq!(buffer.to_bytes(), expected);
	}

	#[test]
	fn test_deserialize_too_many_contracts() {
		let data = hex::decode("111118d802a401d802a401d802a401d802a401d802a401d802a401d802a401d802a401d802a401d802a401d802a401").unwrap();

		let err = Signer::from_bytes(&data).unwrap_err();

		assert!(err.to_string().contains("too many allowed contracts"));
	}

	#[test]
	fn test_serialize_deserialize_max_nested_rules() {
		let script_hash = H160::zero();

		let rule = WitnessRule::new(
			WitnessAction::Allow,
			WitnessCondition::And(vec![WitnessCondition::Boolean(true)]),
		);

		let mut buffer = Encoder::new();

		let mut account_signer = AccountSigner::none(script_hash.into()).unwrap();
		account_signer.set_rules(vec![rule]).unwrap();
		account_signer.encode(&mut buffer);

		let expected =
			hex::decode("0000000000000000000000000000000000000000400101020102010001").unwrap();
		assert_eq!(buffer.to_bytes(), expected);
	}
	#[test]
	fn test_fail_adding_rules_to_global_scope() {
		let script_hash = H160::zero();

		let rule =
			WitnessRule::new(WitnessAction::Allow, WitnessCondition::ScriptHash(script_hash));

		let mut signer = AccountSigner::global(script_hash.into()).unwrap();

		let err = signer.set_rules(vec![rule]).unwrap_err();

		assert_eq!(err, BuilderError::TooManySigners("".to_string()));
	}

	#[test]
	fn test_too_many_rules() {
		let script_hash = H160::zero();

		let rule =
			WitnessRule::new(WitnessAction::Allow, WitnessCondition::ScriptHash(script_hash));

		let mut signer = AccountSigner::none(script_hash.into()).unwrap();

		for _ in 0..16 {
			signer.set_rules(vec![rule]).unwrap();
		}

		let err = signer.set_rules(vec![rule]).unwrap_err();

		assert_eq!(err, BuilderError::TooManySigners("".to_string()));
	}

	#[test]
	fn test_signer_equals() {
		let script_hash = H160::zero();

		let signer1 = AccountSigner::global(script_hash.into()).unwrap();
		let signer2 = AccountSigner::global(script_hash.into()).unwrap();

		assert_eq!(signer1, signer2);

		let signer3 = AccountSigner::called_by_entry(script_hash.into()).unwrap();
		let signer4 = AccountSigner::called_by_entry(script_hash.into()).unwrap();

		assert_eq!(signer3, signer4);
	}
}
