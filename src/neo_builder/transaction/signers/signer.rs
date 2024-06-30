use std::hash::{Hash, Hasher};

use primitive_types::H160;
use serde::{Deserialize, Serialize, Serializer};

use neo::prelude::{
	AccountSigner, BuilderError, ContractSigner, Decoder, Encoder, NeoConstants, NeoSerializable,
	Secp256r1PublicKey, TransactionError, TransactionSigner, WitnessCondition, WitnessRule,
	WitnessScope,
};

use crate::neo_protocol::AccountTrait;

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
			return Err(BuilderError::SignerConfiguration(
				"Trying to set allowed contracts on a Signer with global scope.".to_string(),
			))
		}

		if self.get_allowed_contracts().len() + contracts.len()
			> NeoConstants::MAX_SIGNER_SUBITEMS as usize
		{
			return Err(BuilderError::SignerConfiguration(format!(
				"Trying to set more than {} allowed contracts on a signer.",
				NeoConstants::MAX_SIGNER_SUBITEMS
			)))
		}

		// Update state
		// if !self.get_scopes().contains(&WitnessScope::CustomContracts) {
		// 	if self.get_scopes().contains(&WitnessScope::None) {
		// 		self.set_scopes(vec![WitnessScope::CustomContracts]);
		// 	} else {
		// 		self.get_scopes_mut().push(WitnessScope::CustomContracts);
		// 	}
		// }
		// Remove WitnessScope::None if it is present
		self.get_scopes_mut().retain(|scope| *scope != WitnessScope::None);

		// Add WitnessScope::CustomContracts if it is not already present
		if !self.get_scopes().contains(&WitnessScope::CustomContracts) {
			self.get_scopes_mut().push(WitnessScope::CustomContracts);
		}

		self.get_allowed_contracts_mut().extend(contracts);

		Ok(())
	}

	// Set allowed groups
	fn set_allowed_groups(&mut self, groups: Vec<Secp256r1PublicKey>) -> Result<(), BuilderError> {
		if self.get_scopes().contains(&WitnessScope::Global) {
			return Err(BuilderError::SignerConfiguration(
				"Trying to set allowed contract groups on a Signer with global scope.".to_string(),
			))
		}

		if self.get_allowed_groups().len() + groups.len()
			> NeoConstants::MAX_SIGNER_SUBITEMS as usize
		{
			return Err(BuilderError::SignerConfiguration(format!(
				"Trying to set more than {} allowed contract groups on a signer.",
				NeoConstants::MAX_SIGNER_SUBITEMS
			)))
		}

		self.get_scopes_mut().retain(|scope| *scope != WitnessScope::None);

		if !self.get_scopes().contains(&WitnessScope::CustomGroups) {
			self.get_scopes_mut().push(WitnessScope::CustomGroups);
		}

		self.get_allowed_groups_mut().extend(groups);

		Ok(())
	}

	fn set_rules(&mut self, mut rules: Vec<WitnessRule>) -> Result<&mut Self, BuilderError> {
		if !rules.is_empty() {
			if self.get_scopes().contains(&WitnessScope::Global) {
				return Err(BuilderError::SignerConfiguration(
					"Trying to set witness rules on a Signer with global scope.".to_string(),
				))
			}

			if self.get_rules().len() + rules.len() > NeoConstants::MAX_SIGNER_SUBITEMS as usize {
				return Err(BuilderError::SignerConfiguration(format!(
					"Trying to set more than {} allowed witness rules on a signer.",
					NeoConstants::MAX_SIGNER_SUBITEMS
				)))
			}

			for rule in &rules {
				self.check_depth(&rule.condition, WitnessCondition::MAX_NESTING_DEPTH as i8)?;
			}

			if !self.get_scopes().contains(&WitnessScope::WitnessRules) {
				self.get_scopes_mut().push(WitnessScope::WitnessRules);
			}

			self.get_rules_mut().append(&mut rules);
		}

		Ok(self)
	}

	fn check_depth(&self, condition: &WitnessCondition, depth: i8) -> Result<(), BuilderError> {
		if depth < 0 {
			return Err(BuilderError::IllegalState(format!(
				"A maximum nesting depth of {} is allowed for witness conditions",
				WitnessCondition::MAX_NESTING_DEPTH
			))) // ::)
		}

		match condition {
			WitnessCondition::And(conditions) | WitnessCondition::Or(conditions) => {
				for c in conditions {
					self.check_depth(c, depth - 1)?
				}
			},
			_ => (),
		};

		Ok(())
	}

	fn validate_subitems(&self, count: usize, _name: &str) -> Result<(), BuilderError> {
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
			Signer::Account(account_signer) => TransactionSigner::new(
				account_signer.account.get_script_hash(),
				account_signer.scopes,
			),
			Signer::Contract(contract_signer) => TransactionSigner::new(
				*contract_signer.get_signer_hash(),
				contract_signer.get_scopes().to_vec(),
			),
			Signer::Transaction(transaction_signer) => transaction_signer,
		}
	}
}

impl Into<TransactionSigner> for &Signer {
	fn into(self) -> TransactionSigner {
		match self {
			Signer::Account(account_signer) =>
				TransactionSigner::new_full(account_signer.account.get_script_hash(), 
											 account_signer.get_scopes().to_vec(), 
								  account_signer.get_allowed_contracts().to_vec(), 
								  	 account_signer.get_allowed_groups().to_vec(), 
									          account_signer.get_rules().to_vec()),
			Signer::Contract(contract_signer) =>
				TransactionSigner::new(*contract_signer.get_signer_hash(), contract_signer.get_scopes().to_vec()),
			// Signer::Account(_account_signer) =>
			// 	panic!("Cannot convert AccountSigner into TransactionSigner"),
			// Signer::Contract(_contract_signer) =>
			// 	panic!("Cannot convert ContractSigner into AccountSigner"),
			Signer::Transaction(transaction_signer) => transaction_signer.clone(),
		}
	}
}

impl Into<TransactionSigner> for &mut Signer {
	fn into(self) -> TransactionSigner {
		match self {
			Signer::Account(_account_signer) =>
				panic!("Cannot convert AccountSigner into TransactionSigner"),
			Signer::Contract(_contract_signer) =>
				panic!("Cannot convert ContractSigner into AccountSigner"),
			Signer::Transaction(transaction_signer) => transaction_signer.clone(),
		}
	}
}

impl Into<AccountSigner> for &mut Signer {
	fn into(self) -> AccountSigner {
		match self {
			Signer::Account(account_signer) => account_signer.clone(),
			Signer::Contract(_contract_signer) =>
				panic!("Cannot convert ContractSigner into AccountSigner"),
			Signer::Transaction(_transaction_signer) =>
				panic!("Cannot convert TransactionSigner into AccountSigner"),
		}
	}
}

impl Into<ContractSigner> for &mut Signer {
	fn into(self) -> ContractSigner {
		match self {
			Signer::Account(_account_signer) =>
				panic!("Cannot convert AccountSigner into ContractSigner"),
			Signer::Contract(contract_signer) => contract_signer.clone(),
			Signer::Transaction(_transaction_signer) =>
				panic!("Cannot convert TransactionSigner into ContractSigner"),
		}
	}
}

impl Into<ContractSigner> for Signer {
	fn into(self) -> ContractSigner {
		match self {
			Signer::Account(_account_signer) =>
				panic!("Cannot convert AccountSigner into ContractSigner"),
			Signer::Contract(contract_signer) => contract_signer,
			Signer::Transaction(_transaction_signer) =>
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
			//_ => Ok(Signer::Transaction(TransactionSigner::decode(reader)?)),
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
	use std::ops::Deref;

	use lazy_static::lazy_static;
	use neo::builder::Signer;
	use primitive_types::H160;
	use rustc_serialize::hex::{FromHex, ToHex};
	use std::collections::HashSet;

	use neo::prelude::{
		Account, AccountSigner, AccountTrait, BuilderError, ContractSigner, Encoder, NeoConstants,
		NeoSerializable, ScriptHash, ScriptHashExtension, Secp256r1PublicKey, SignerTrait,
		WitnessAction, WitnessCondition, WitnessRule, WitnessScope,
	};

	// const script_hash:ScriptHash = Account::from_wif("Kzt94tAAiZSgH7Yt4i25DW6jJFprZFPSqTgLr5dWmWgKDKCjXMfZ").unwrap().get_script_hash();

	lazy_static! {
		pub static ref SCRIPT_HASH: ScriptHash = {
			Account::from_wif("Kzt94tAAiZSgH7Yt4i25DW6jJFprZFPSqTgLr5dWmWgKDKCjXMfZ")
				.unwrap()
				.get_script_hash()
		};
		pub static ref SCRIPT_HASH1: H160 = H160::from_script(&"d802a401".from_hex().unwrap());
		pub static ref SCRIPT_HASH2: H160 = H160::from_script(&"c503b112".from_hex().unwrap());
		pub static ref GROUP_PUB_KEY1: Secp256r1PublicKey = Secp256r1PublicKey::from_encoded(
			"0306d3e7f18e6dd477d34ce3cfeca172a877f3c907cc6c2b66c295d1fcc76ff8f7",
		)
		.unwrap();
		pub static ref GROUP_PUB_KEY2: Secp256r1PublicKey = Secp256r1PublicKey::from_encoded(
			"02958ab88e4cea7ae1848047daeb8883daf5fdf5c1301dbbfe973f0a29fe75de60",
		)
		.unwrap();
	}

	#[test]
	fn test_create_signer_with_call_by_entry_scope() {
		let signer = AccountSigner::called_by_entry(&SCRIPT_HASH.deref().into()).unwrap();

		assert_eq!(signer.signer_hash, *SCRIPT_HASH);
		assert_eq!(signer.scopes, vec![WitnessScope::CalledByEntry]);
		assert!(signer.get_allowed_contracts().is_empty());
		assert!(signer.get_allowed_groups().is_empty());
	}

	#[test]
	fn test_create_signer_with_global_scope() {
		let signer = AccountSigner::global(&SCRIPT_HASH.deref().into()).unwrap();

		assert_eq!(signer.signer_hash, *SCRIPT_HASH);
		assert_eq!(signer.scopes, vec![WitnessScope::Global]);
		assert!(signer.get_allowed_contracts().is_empty());
		assert!(signer.get_allowed_groups().is_empty());
	}

	#[test]
	fn test_fail_on_stepping_over_max_condition_nesting_depth() {
		let condition =
			WitnessCondition::And(vec![WitnessCondition::And(vec![WitnessCondition::And(vec![
				WitnessCondition::Not(Box::new(WitnessCondition::ScriptHash(*SCRIPT_HASH))),
			])])]);

		let rule = WitnessRule { action: WitnessAction::Allow, condition };

		let mut signer = AccountSigner::none(&SCRIPT_HASH.deref().into()).unwrap();

		let err = signer.set_rules(vec![rule]).unwrap_err();

		assert_eq!(
			err.to_string(),
			format!(
				"Illegal state: A maximum nesting depth of {} is allowed for witness conditions",
				WitnessCondition::MAX_NESTING_DEPTH
			)
		);
	}

	// #[test]
	// fn test_serialize_and_deserialize() {
	// 	let serialized = "9429a9a942a8a8a9429a917102d802a401c503b112\
	//     02a877f3c907cc6c2b66c295d1fcc76ff8f702958ab88e4cea7ae1848047daeb8883daf5fdf5c1301dbbfe973f0a29fe75de6001010128d802a401"
	// 		.from_hex().unwrap();

	// 	let signer = Signer::from_bytes(&serialized).unwrap();

	// 	assert_eq!(signer.get_signer_hash(), SCRIPT_HASH.deref());

	// 	assert_eq!(
	// 		signer.get_scopes(),
	// 		&vec![
	// 			WitnessScope::CalledByEntry,
	// 			WitnessScope::CustomContracts,
	// 			WitnessScope::CustomGroups,
	// 			WitnessScope::WitnessRules,
	// 		]
	// 	);

	// 	let signer = Signer::from_bytes(&serialized).unwrap();

	// 	// Assert hash
	// 	assert_eq!(signer.get_signer_hash(), SCRIPT_HASH.deref());

	// 	// Assert other properties
	// 	assert_eq!(signer.get_allowed_contracts().len(), 2);

	// 	let contract1 = H160::from_hex("d802a401").unwrap();
	// 	let contract2 = H160::from_hex("c503b112").unwrap();
	// 	assert_eq!(signer.get_allowed_contracts(), &vec![contract1, contract2]);

	// 	assert_eq!(signer.get_allowed_groups().len(), 2);

	// 	let group1 = Secp256r1PublicKey::from_encoded(
	// 		"030306d3e7f18e6dd477d34ce3cfeca172a877f3c907cc6c2b66c295d1fcc76ff8f7",
	// 	)
	// 	.unwrap();
	// 	let group2 = Secp256r1PublicKey::from_encoded(
	// 		"02958ab88e4cea7ae1848047daeb8883daf5fdf5c1301dbbfe973f0a29fe75de60",
	// 	)
	// 	.unwrap();
	// 	assert_eq!(signer.get_allowed_groups(), &vec![group1, group2]);

	// 	assert_eq!(signer.get_rules().len(), 1);

	// 	let rule = &signer.get_rules()[0];
	// 	assert_eq!(rule.action, WitnessAction::Allow);
	// 	assert_eq!(rule.condition, WitnessCondition::CalledByContract(contract1));
	// }

	#[test]
	fn test_deserialize() {
		let data_str = format!(
			"{}{}{}{}{}{}{}{}{}{}{}{}",
			SCRIPT_HASH.as_bytes().to_hex(),
			"71",
			"02",
			SCRIPT_HASH1.as_bytes().to_hex(),
			SCRIPT_HASH2.as_bytes().to_hex(),
			"02",
			GROUP_PUB_KEY1.get_encoded_compressed_hex().trim_start_matches("0x").to_string(),
			GROUP_PUB_KEY2.get_encoded_compressed_hex().trim_start_matches("0x").to_string(),
			"01",
			"01",
			"28",
			SCRIPT_HASH1.as_bytes().to_hex()
		);
		// let mut serialized = "9429a9a942a8a8a9429a917102d802a401c503b112\
		// 02a877f3c907cc6c2b66c295d1fcc76ff8f702958ab88e4cea7ae1848047daeb8883daf5fdf5c1301dbbfe973f0a29fe75de6001010128d802a401"
		// 	.from_hex().unwrap();
		let mut serialized = data_str.from_hex().unwrap();

		serialized.insert(0, 1);

		let signer = Signer::from_bytes(&serialized).unwrap();

		assert_eq!(signer.get_signer_hash(), SCRIPT_HASH.deref());

		let expected_scopes: HashSet<WitnessScope> = vec![
			WitnessScope::CalledByEntry,
			WitnessScope::CustomContracts,
			WitnessScope::CustomGroups,
			WitnessScope::WitnessRules,
		]
		.into_iter()
		.collect();
		// Compare in set so the order does not matter
		assert_eq!(signer.get_scopes().iter().cloned().collect::<HashSet<_>>(), expected_scopes);

		// let signer = Signer::from_bytes(&serialized).unwrap();

		// // Assert hash
		// assert_eq!(signer.get_signer_hash(), SCRIPT_HASH.deref());

		// // Assert other properties
		// assert_eq!(signer.get_allowed_contracts().len(), 2);

		// let contract1 = H160::from_hex("d802a401").unwrap();
		// let contract2 = H160::from_hex("c503b112").unwrap();
		assert_eq!(
			signer.get_allowed_contracts().iter().cloned().collect::<HashSet<_>>(),
			vec![*SCRIPT_HASH1, *SCRIPT_HASH2].into_iter().collect()
		);

		// assert_eq!(signer.get_allowed_groups().len(), 2);

		// let group1 = Secp256r1PublicKey::from_encoded(
		// 	"030306d3e7f18e6dd477d34ce3cfeca172a877f3c907cc6c2b66c295d1fcc76ff8f7",
		// )
		// .unwrap();
		// let group2 = Secp256r1PublicKey::from_encoded(
		// 	"02958ab88e4cea7ae1848047daeb8883daf5fdf5c1301dbbfe973f0a29fe75de60",
		// )
		// .unwrap();
		assert_eq!(
			signer.get_allowed_groups().iter().cloned().collect::<HashSet<_>>(),
			vec![GROUP_PUB_KEY1.clone(), GROUP_PUB_KEY2.clone()].into_iter().collect()
		);

		// assert_eq!(signer.get_rules().len(), 1);

		let rule = &signer.get_rules()[0];
		assert_eq!(rule.action, WitnessAction::Allow);
		assert_eq!(rule.condition, WitnessCondition::CalledByContract(*SCRIPT_HASH1));
	}

	#[test]
	fn test_build_valid_signer1() {
		let mut signer = AccountSigner::called_by_entry(&SCRIPT_HASH.deref().into()).unwrap();
		signer.set_allowed_contracts(vec![*SCRIPT_HASH1, *SCRIPT_HASH2]).expect("");

		assert_eq!(signer.signer_hash, *SCRIPT_HASH);
		assert_eq!(
			signer.get_scopes().iter().cloned().collect::<HashSet<_>>(),
			vec![WitnessScope::CalledByEntry, WitnessScope::CustomContracts]
				.into_iter()
				.collect()
		);
		assert_eq!(
			signer.get_allowed_contracts().iter().cloned().collect::<HashSet<_>>(),
			vec![*SCRIPT_HASH1, *SCRIPT_HASH2].into_iter().collect()
		);
		assert!(signer.get_allowed_groups().is_empty());
	}

	#[test]
	fn test_build_valid_signer2() {
		let mut signer = AccountSigner::none(&SCRIPT_HASH.deref().into()).unwrap();
		signer.set_allowed_contracts(vec![*SCRIPT_HASH1, *SCRIPT_HASH2]).expect("");

		assert_eq!(signer.signer_hash, *SCRIPT_HASH);
		assert_eq!(signer.get_scopes(), &vec![WitnessScope::CustomContracts]);
		assert_eq!(
			signer.get_allowed_contracts().iter().cloned().collect::<HashSet<_>>(),
			vec![*SCRIPT_HASH1, *SCRIPT_HASH2].into_iter().collect()
		);
		assert!(signer.get_allowed_groups().is_empty());
		// XCTAssertEqual(signer.scopes, [.customContracts])
		// XCTAssertEqual(signer.allowedContracts, [contract1, contract2])
		// XCTAssert(signer.allowedGroups.isEmpty)
	}

	#[test]
	fn test_build_valid_signer3() {
		let mut signer = AccountSigner::none(&SCRIPT_HASH.deref().into()).unwrap();
		signer
			.set_allowed_groups(vec![GROUP_PUB_KEY1.clone(), GROUP_PUB_KEY2.clone()])
			.expect("");

		assert_eq!(signer.signer_hash, *SCRIPT_HASH);
		assert_eq!(signer.get_scopes(), &vec![WitnessScope::CustomGroups]);
		assert_eq!(
			signer.get_allowed_groups().iter().cloned().collect::<HashSet<_>>(),
			vec![GROUP_PUB_KEY1.clone(), GROUP_PUB_KEY2.clone()].into_iter().collect()
		);
		assert!(signer.get_allowed_contracts().is_empty());
	}

	#[test]
	fn test_fail_building_signer_with_global_ccope_and_custom_contracts() {
		let mut signer = AccountSigner::global(&SCRIPT_HASH.deref().into()).unwrap();
		let err = signer.set_allowed_contracts(vec![*SCRIPT_HASH1, *SCRIPT_HASH2]).unwrap_err();

		assert_eq!(
			err,
			BuilderError::SignerConfiguration(
				"Trying to set allowed contracts on a Signer with global scope.".to_string()
			)
		);
		// XCTAssertEqual(signer.scopes, [.customContracts])
		// XCTAssertEqual(signer.allowedContracts, [contract1, contract2])
		// XCTAssert(signer.allowedGroups.isEmpty)
	}

	#[test]
	fn test_fail_building_signer_with_global_ccope_and_custom_groups() {
		let mut signer = AccountSigner::global(&SCRIPT_HASH.deref().into()).unwrap();
		let err = signer
			.set_allowed_groups(vec![GROUP_PUB_KEY1.clone(), GROUP_PUB_KEY2.clone()])
			.unwrap_err();

		assert_eq!(
			err,
			BuilderError::SignerConfiguration(
				"Trying to set allowed contract groups on a Signer with global scope.".to_string()
			)
		);
		// XCTAssertEqual(signer.scopes, [.customContracts])
		// XCTAssertEqual(signer.allowedContracts, [contract1, contract2])
		// XCTAssert(signer.allowedGroups.isEmpty)
	}

	#[test]
	fn test_fail_building_signer_too_many_contracts() {
		let script = H160::from_hex("3ab0be8672e25cf475219d018ded961ec684ca88").unwrap();
		let contracts = (0..=16).map(|_| script.clone()).collect::<Vec<_>>();

		let err = AccountSigner::called_by_entry(&SCRIPT_HASH.deref().into())
			.unwrap()
			.set_allowed_contracts(contracts)
			.unwrap_err();

		assert_eq!(
			err,
			BuilderError::SignerConfiguration(format!(
				"Trying to set more than {} allowed contracts on a signer.",
				NeoConstants::MAX_SIGNER_SUBITEMS
			))
		);
	}

	#[test]
	fn test_fail_building_signer_too_many_contracts_added_separately() {
		let script = H160::from_hex("3ab0be8672e25cf475219d018ded961ec684ca88").unwrap();
		let contracts = (0..=15).map(|_| script.clone()).collect::<Vec<_>>();

		let mut signer = AccountSigner::none(&SCRIPT_HASH.deref().into()).unwrap();
		signer.set_allowed_contracts(vec![script]).expect("");

		let err = signer.set_allowed_contracts(contracts).unwrap_err();

		assert_eq!(
			err,
			BuilderError::SignerConfiguration(format!(
				"Trying to set more than {} allowed contracts on a signer.",
				NeoConstants::MAX_SIGNER_SUBITEMS
			))
		);
	}

	#[test]
	fn test_fail_building_signer_too_many_groups() {
		let public_key = Secp256r1PublicKey::from_encoded(
			"0306d3e7f18e6dd477d34ce3cfeca172a877f3c907cc6c2b66c295d1fcc76ff8f7",
		)
		.unwrap();
		let groups = (0..=16).map(|_| public_key.clone()).collect::<Vec<_>>();

		let err = AccountSigner::called_by_entry(&SCRIPT_HASH.deref().into())
			.unwrap()
			.set_allowed_groups(groups)
			.unwrap_err();

		assert_eq!(
			err,
			BuilderError::SignerConfiguration(format!(
				"Trying to set more than {} allowed contract groups on a signer.",
				NeoConstants::MAX_SIGNER_SUBITEMS
			))
		);
	}

	#[test]
	fn test_fail_building_signer_too_many_groups_added_separately() {
		let public_key = Secp256r1PublicKey::from_encoded(
			"0306d3e7f18e6dd477d34ce3cfeca172a877f3c907cc6c2b66c295d1fcc76ff8f7",
		)
		.unwrap();
		let groups = (0..=15).map(|_| public_key.clone()).collect::<Vec<_>>();

		let mut signer = AccountSigner::none(&SCRIPT_HASH.deref().into()).unwrap();
		signer.set_allowed_groups(vec![public_key]).expect("");

		let err = signer.set_allowed_groups(groups).unwrap_err();

		assert_eq!(
			err,
			BuilderError::SignerConfiguration(format!(
				"Trying to set more than {} allowed contract groups on a signer.",
				NeoConstants::MAX_SIGNER_SUBITEMS
			))
		);
	}

	#[test]
	fn test_serialize_global_scope() {
		let mut buffer = Encoder::new();

		AccountSigner::global(&SCRIPT_HASH.deref().into()).unwrap().encode(&mut buffer);

		// let mut script_hash_vec = SCRIPT_HASH.deref().as_bytes().to_vec();
		// SCRIPT_HASH.reverse();

		// let expected = hex::decode(format!(
		// 	"{}{:02x}",
		// 	script_hash_vec.to_hex(),
		// 	WitnessScope::Global.byte_repr()
		// ))
		// .unwrap();
		// let expected = hex::decode(format!(
		// 	"{}{:02x}",
		// 	SCRIPT_HASH.as_bytes().to_hex(),
		// 	WitnessScope::Global.byte_repr()
		// ))
		// .unwrap();
		let expected =
			format!("{}{:02x}", SCRIPT_HASH.as_bytes().to_hex(), WitnessScope::Global.byte_repr());
		assert_eq!(buffer.to_bytes().to_hex(), expected);
	}

	#[test]
	fn test_serialize_custom_contracts_scope_produces_correct_byte_array() {
		//let mut buffer = Encoder::new();
		let mut signer = AccountSigner::none(&SCRIPT_HASH.deref().into()).unwrap(); //stupid mistake using SCRIPT_HASH1
		signer.set_allowed_contracts(vec![*SCRIPT_HASH1, *SCRIPT_HASH2]).unwrap();

		//signer.encode(&mut buffer);

		let expected = format!(
			"{}{:02x}02{}{}",
			SCRIPT_HASH.as_bytes().to_hex(),
			WitnessScope::CustomContracts.byte_repr(),
			SCRIPT_HASH1.as_bytes().to_hex(),
			SCRIPT_HASH2.as_bytes().to_hex()
		);

		assert_eq!(signer.to_array(), expected.from_hex().unwrap());
	}

	#[test]
	fn test_serialize_custom_group_scope() {
		let mut signer = AccountSigner::none(&SCRIPT_HASH.deref().into()).unwrap();
		signer
			.set_allowed_groups(vec![GROUP_PUB_KEY1.clone(), GROUP_PUB_KEY2.clone()])
			.expect("");

		let expected = format!(
			"{}{:02x}02{}{}",
			SCRIPT_HASH.as_bytes().to_hex(),
			WitnessScope::CustomGroups.byte_repr(),
			GROUP_PUB_KEY1.get_encoded_compressed_hex().trim_start_matches("0x").to_string(),
			GROUP_PUB_KEY2.get_encoded_compressed_hex().trim_start_matches("0x").to_string(),
		);

		assert_eq!(signer.to_array(), expected.from_hex().unwrap());
	}

	#[test]
	fn test_serialize_multiple_scopes_contracts_groups_and_rules() {
		let rule = WitnessRule::new(
			WitnessAction::Allow,
			WitnessCondition::CalledByContract(*SCRIPT_HASH1),
		);
		let mut signer = AccountSigner::called_by_entry(&SCRIPT_HASH.deref().into()).unwrap();
		signer
			.set_allowed_groups(vec![GROUP_PUB_KEY1.clone(), GROUP_PUB_KEY2.clone()])
			.expect("");
		signer.set_allowed_contracts(vec![*SCRIPT_HASH1, *SCRIPT_HASH2]).expect("");
		signer.set_rules(vec![rule]).expect("");

		let expected = format!(
			"{}{}{}{}{}{}{}{}{}{}{}{}",
			SCRIPT_HASH.as_bytes().to_hex(),
			"71",
			"02",
			SCRIPT_HASH1.as_bytes().to_hex(),
			SCRIPT_HASH2.as_bytes().to_hex(),
			"02",
			GROUP_PUB_KEY1.get_encoded_compressed_hex().trim_start_matches("0x").to_string(),
			GROUP_PUB_KEY2.get_encoded_compressed_hex().trim_start_matches("0x").to_string(),
			"01",
			"01",
			"28",
			SCRIPT_HASH1.as_bytes().to_hex()
		);

		assert_eq!(signer.to_array(), expected.from_hex().unwrap());
	}

	#[test]
	fn test_fail_deserialize_too_many_contracts() {
		let mut serialized = format!("{}1111", SCRIPT_HASH.as_bytes().to_hex());
		//println!("serialized: {}", serialized.len());
		for _ in 0..=17 {
			serialized.push_str(&SCRIPT_HASH1.as_bytes().to_hex());
		}
		let mut data = hex::decode(&serialized).unwrap();
		data.insert(0, 1);
		//let data = hex::decode("111118d802a401d802a401d802a401d802a401d802a401d802a401d802a401d802a401d802a401d802a401d802a401").unwrap();
		//Always trigger the last arm in decode!!!

		let err = Signer::from_bytes(&data).unwrap_err();
		//println!("Error: {}", err);

		assert!(err.to_string().contains(&format!(
			"A signer's scope can only contain {} allowed contracts.",
			NeoConstants::MAX_SIGNER_SUBITEMS
		)));
	}

	#[test]
	fn test_fail_deserialize_too_many_contract_groups() {
		let mut serialized = format!("{}2111", SCRIPT_HASH.as_bytes().to_hex());
		//println!("serialized: {}", serialized.len());
		for _ in 0..=17 {
			serialized.push_str(
				&GROUP_PUB_KEY1.get_encoded_compressed_hex().trim_start_matches("0x").to_string(),
			);
		}
		let mut data = hex::decode(&serialized).unwrap();
		data.insert(0, 1);
		//let data = hex::decode("111118d802a401d802a401d802a401d802a401d802a401d802a401d802a401d802a401d802a401d802a401d802a401").unwrap();
		//Always trigger the last arm in decode!!!

		let err = Signer::from_bytes(&data).unwrap_err();
		//println!("Error: {}", err);

		assert!(err.to_string().contains(&format!(
			"A signer's scope can only contain {} allowed contract groups.",
			NeoConstants::MAX_SIGNER_SUBITEMS
		)));
	}

	#[test]
	fn test_fail_deserialize_too_many_rules() {
		let mut serialized = format!("{}4111", SCRIPT_HASH.as_bytes().to_hex());
		//println!("serialized: {}", serialized.len());
		for _ in 0..=17 {
			serialized.push_str("01");
			serialized.push_str("28");
			serialized.push_str(&SCRIPT_HASH1.as_bytes().to_hex());
		}
		let mut data = hex::decode(&serialized).unwrap();
		data.insert(0, 1);
		//let data = hex::decode("111118d802a401d802a401d802a401d802a401d802a401d802a401d802a401d802a401d802a401d802a401d802a401").unwrap();
		//Always trigger the last arm in decode!!!

		let err = Signer::from_bytes(&data).unwrap_err();
		//println!("Error: {}", err);

		assert!(err.to_string().contains(&format!(
			"A signer's scope can only contain {} rules.",
			NeoConstants::MAX_SIGNER_SUBITEMS
		)));
	}

	#[test]
	fn test_get_size() {
		let rule = WitnessRule::new(
			WitnessAction::Allow,
			WitnessCondition::And(vec![
				WitnessCondition::Boolean(true),
				WitnessCondition::Boolean(false),
			]),
		);
		let mut signer = AccountSigner::called_by_entry(&SCRIPT_HASH.deref().into()).unwrap();
		signer
			.set_allowed_groups(vec![GROUP_PUB_KEY1.clone(), GROUP_PUB_KEY2.clone()])
			.expect("");
		signer.set_allowed_contracts(vec![*SCRIPT_HASH1, *SCRIPT_HASH2]).expect("");
		signer.set_rules(vec![rule.clone(), rule]).expect("");

		let expected_size = 20 // Account script hash
    							+ 1 // Scope byte
    							+ 1 // length byte of allowed contracts list
    							+ 20 + 20 // Script hashes of two allowed contracts
    							+ 1 // length byte of allowed groups list
    							+ 33 + 33 // Public keys of two allowed groups
    							+ 1 // length byte of rules list
    							+ 1 // byte for WitnessRuleAction Allow
    							+ 1 // byte for WitnessCondition type (AndCondition)
    							+ 1 // length of AND condition list
    							+ 1 // byte for WitnessCondition type (BooleanCondition)
    							+ 1 // byte for value of BooleanCondition
   								+ 1 // byte for WitnessCondition type (BooleanCondition)
    							+ 1 // byte for value of BooleanCondition
    							+ 1 // byte for WitnessRuleAction Allow
    							+ 1 // byte for WitnessCondition type (AndCondition)
    							+ 1 // length of AND condition list
    							+ 1 // byte for WitnessCondition type (BooleanCondition)
    							+ 1 // byte for value of BooleanCondition
   								+ 1 // byte for WitnessCondition type (BooleanCondition)
    							+ 1; // byte for value of BooleanCondition

		assert_eq!(signer.size(), expected_size);
	}

	#[test]
	fn test_serialize_deserialize_max_nested_rules() {
		let rule = WitnessRule::new(
			WitnessAction::Allow,
			//WitnessCondition::And(vec![WitnessCondition::Boolean(true)]),
			WitnessCondition::And(vec![WitnessCondition::And(vec![WitnessCondition::Boolean(
				true,
			)])]),
		);

		let mut buffer = Encoder::new();

		//let mut account_signer = AccountSigner::none(&SCRIPT_HASH.deref().into()).unwrap();
		let mut account_signer = AccountSigner::none(&ScriptHash::zero().into()).unwrap();

		account_signer.set_rules(vec![rule]).unwrap();
		account_signer.encode(&mut buffer);

		let expected =
			hex::decode("0000000000000000000000000000000000000000400101020102010001").unwrap();
		assert_eq!(buffer.to_bytes(), expected);
	}

	#[test]
	fn test_fail_adding_rules_to_global_signer() {
		let rule =
			WitnessRule::new(WitnessAction::Allow, WitnessCondition::ScriptHash(*SCRIPT_HASH));

		let mut signer = AccountSigner::global(&SCRIPT_HASH.deref().into()).unwrap();

		let err = signer.set_rules(vec![rule]).unwrap_err();

		assert_eq!(
			err,
			BuilderError::SignerConfiguration(
				"Trying to set witness rules on a Signer with global scope.".to_string()
			)
		);
	}

	#[test]
	fn test_fail_adding_too_many_rules() {
		let rule =
			WitnessRule::new(WitnessAction::Allow, WitnessCondition::ScriptHash(*SCRIPT_HASH));

		let mut signer = AccountSigner::none(&SCRIPT_HASH.deref().into()).unwrap();

		for _ in 0..NeoConstants::MAX_SIGNER_SUBITEMS {
			signer.set_rules(vec![rule.clone()]).unwrap();
		}

		let err = signer.set_rules(vec![rule]).unwrap_err();

		assert_eq!(
			err,
			BuilderError::SignerConfiguration(format!(
				"Trying to set more than {} allowed witness rules on a signer.",
				NeoConstants::MAX_SIGNER_SUBITEMS
			))
		);
	}

	#[test]
	fn test_signer_equals() {
		let signer1 = AccountSigner::global(&SCRIPT_HASH.deref().into()).unwrap();
		let signer2 = AccountSigner::global(&SCRIPT_HASH.deref().into()).unwrap();

		assert_eq!(signer1, signer2);

		let signer3 = ContractSigner::called_by_entry(*SCRIPT_HASH, &[]);
		let signer4 = ContractSigner::called_by_entry(*SCRIPT_HASH, &[]);

		assert_eq!(signer3, signer4);

		let mut signer5 = AccountSigner::called_by_entry(&SCRIPT_HASH.deref().into()).unwrap();
		signer5
			.set_allowed_groups(vec![GROUP_PUB_KEY1.clone(), GROUP_PUB_KEY2.clone()])
			.expect("");
		signer5.set_allowed_contracts(vec![*SCRIPT_HASH1, *SCRIPT_HASH2]).expect("");

		let mut signer6 = AccountSigner::called_by_entry(&SCRIPT_HASH.deref().into()).unwrap();
		signer6
			.set_allowed_groups(vec![GROUP_PUB_KEY1.clone(), GROUP_PUB_KEY2.clone()])
			.expect("");
		signer6.set_allowed_contracts(vec![*SCRIPT_HASH1, *SCRIPT_HASH2]).expect("");

		assert_eq!(signer5, signer6);
	}
}
