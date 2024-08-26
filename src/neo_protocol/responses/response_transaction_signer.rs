use std::hash::{Hash, Hasher};

use primitive_types::H160;
use serde::{Deserialize, Serialize};

use neo::prelude::{
	deserialize_scopes, deserialize_script_hash, deserialize_vec_public_key_option,
	deserialize_vec_script_hash_option, deserialize_vec_script_hash, serialize_scopes, serialize_script_hash,
	serialize_vec_public_key_option, serialize_vec_script_hash_option, serialize_vec_script_hash, Decoder, Encoder,
	NeoConstants, NeoSerializable, Secp256r1PublicKey, SignerTrait, SignerType, TransactionError,
	VarSizeTrait, WitnessRule, WitnessScope,
};

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RTransactionSigner {
	#[serde(rename = "account")]
	#[serde(serialize_with = "serialize_script_hash")]
	#[serde(deserialize_with = "deserialize_script_hash")]
	pub account: H160,

	#[serde(rename = "scopes", default)]
	#[serde(serialize_with = "serialize_scopes")]
	#[serde(deserialize_with = "deserialize_scopes")]
	pub scopes: Vec<WitnessScope>,

	#[serde(rename = "allowedcontracts", default)]
	#[serde(serialize_with = "serialize_vec_script_hash")]
	#[serde(deserialize_with = "deserialize_vec_script_hash")]
	pub allowed_contracts: Vec<H160>,

	#[serde(rename = "allowedgroups", default)]
	// #[serde(serialize_with = "serialize_vec_public_key_option")]
	// #[serde(deserialize_with = "deserialize_vec_public_key_option")]
	// #[serde(skip_serializing_if = "Option::is_none")]
	// #[serde(default)]
	pub allowed_groups: Vec<String>,

	#[serde(rename = "rules", default)]
	// #[serde(default)]
	pub rules: Vec<WitnessRule>,
}

impl Hash for RTransactionSigner {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.account.hash(state);
		self.scopes.hash(state);
		self.allowed_contracts.hash(state);
		self.allowed_groups.hash(state);
		self.rules.hash(state);
	}
}

impl RTransactionSigner {
	pub fn new(account: H160, scopes: Vec<WitnessScope>) -> Self {
		Self { account, scopes, allowed_contracts: vec![], allowed_groups: vec![], rules: vec![] }
	}

	pub fn new_full(
		account: H160,
		scopes: Vec<WitnessScope>,
		allowed_contracts: Vec<H160>,
		allowed_groups: Vec<String>,
		rules: Vec<WitnessRule>,
	) -> Self {
		Self {
			account,
			scopes,
			allowed_contracts: allowed_contracts,
			allowed_groups: allowed_groups,
			rules: rules,
		}
	}
}

impl SignerTrait for RTransactionSigner {
	fn get_type(&self) -> SignerType {
		SignerType::Transaction
	}

	fn get_signer_hash(&self) -> &H160 {
		&self.account
	}

	fn set_signer_hash(&mut self, signer_hash: H160) {
		self.account = signer_hash;
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
		panic!("Not implemented")
	}

	fn get_allowed_contracts_mut(&mut self) -> &mut Vec<H160> {
		panic!("Not implemented")
	}

	fn get_allowed_groups(&self) -> &Vec<Secp256r1PublicKey> {
		panic!("Not implemented")
		// &self.allowed_groups
	}

	fn get_allowed_groups_mut(&mut self) -> &mut Vec<Secp256r1PublicKey> {
		panic!("Not implemented")
	}

	fn get_rules(&self) -> &Vec<WitnessRule> {
		panic!("Not implemented")
	}

	fn get_rules_mut(&mut self) -> &mut Vec<WitnessRule> {
		panic!("Not implemented")
	}
}

// impl NeoSerializable for TransactionSigner {
// 	type Error = TransactionError;

// 	fn size(&self) -> usize {
// 		let mut size = (NeoConstants::HASH160_SIZE + 1) as usize;
// 		if self.scopes.contains(&WitnessScope::CustomContracts) {
// 			size += &self.allowed_contracts.clone().unwrap().var_size();
// 		}
// 		if self.scopes.contains(&WitnessScope::CustomGroups) {
// 			size += &self.allowed_groups.clone().unwrap().var_size();
// 		}

// 		if self.scopes.contains(&WitnessScope::WitnessRules) {
// 			size += &self.rules.clone().unwrap().var_size();
// 		}

// 		size
// 	}

// 	fn encode(&self, writer: &mut Encoder) {
// 		writer.write_serializable_fixed(self.get_signer_hash());
// 		writer.write_u8(WitnessScope::combine(self.scopes.as_slice()));
// 		if self.scopes.contains(&WitnessScope::CustomContracts) {
// 			writer.write_serializable_variable_list(self.allowed_contracts.as_ref().unwrap());
// 		}
// 		if self.scopes.contains(&WitnessScope::CustomGroups) {
// 			writer.write_serializable_variable_list(self.allowed_groups.as_ref().unwrap());
// 		}
// 		if self.scopes.contains(&WitnessScope::WitnessRules) {
// 			writer.write_serializable_variable_list(self.rules.as_ref().unwrap());
// 		}
// 	}

// 	fn decode(reader: &mut Decoder) -> Result<Self, Self::Error>
// 	where
// 		Self: Sized,
// 	{
// 		let mut signer = TransactionSigner::default();
// 		signer.set_signer_hash(reader.read_serializable().unwrap());
// 		let scopes = WitnessScope::split(reader.read_u8());
// 		signer.set_scopes(scopes);
// 		if signer.get_scopes().contains(&WitnessScope::CustomContracts) {
// 			signer.allowed_contracts = Some(reader.read_serializable_list().unwrap());
// 		}
// 		if signer.get_scopes().contains(&WitnessScope::CustomGroups) {
// 			signer.allowed_groups = Some(reader.read_serializable_list().unwrap());
// 		}
// 		if signer.get_scopes().contains(&WitnessScope::WitnessRules) {
// 			signer.rules = Some(reader.read_serializable_list().unwrap());
// 		}
// 		Ok(signer)
// 	}

// 	fn to_array(&self) -> Vec<u8> {
// 		let writer = &mut Encoder::new();
// 		self.encode(writer);
// 		writer.to_bytes()
// 	}
// }
