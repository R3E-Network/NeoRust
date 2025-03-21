use crate::{
	builder::{BuilderError, SignerTrait, SignerType, TransactionError, WitnessRule, WitnessScope},
	codec::{Decoder, Encoder, NeoSerializable, VarSizeTrait},
	config::NeoConstants,
	crypto::Secp256r1PublicKey,
	deserialize_script_hash, deserialize_vec_script_hash,
	neo_types::{deserialize_vec_public_key, serialize_vec_public_key},
	serialize_script_hash, serialize_vec_script_hash, ContractParameter,
};
use getset::{Getters, Setters};
use primitive_types::H160;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

/// Represents a contract signer in the NEO blockchain.
///
/// This struct contains information about the contract signer, including
/// the signer hash, scopes, allowed contracts, allowed groups, and witness rules.
#[derive(Debug, Clone, Serialize, PartialEq, Deserialize, Getters, Setters)]
pub struct ContractSigner {
	#[serde(
		serialize_with = "serialize_script_hash",
		deserialize_with = "deserialize_script_hash"
	)]
	signer_hash: H160,
	scopes: Vec<WitnessScope>,
	#[serde(
		serialize_with = "serialize_vec_script_hash",
		deserialize_with = "deserialize_vec_script_hash"
	)]
	allowed_contracts: Vec<H160>,
	#[serde(
		serialize_with = "serialize_vec_public_key",
		deserialize_with = "deserialize_vec_public_key"
	)]
	allowed_groups: Vec<Secp256r1PublicKey>,
	rules: Vec<WitnessRule>,
	#[getset(get = "pub")]
	verify_params: Vec<ContractParameter>,
	#[serde(
		serialize_with = "serialize_script_hash",
		deserialize_with = "deserialize_script_hash"
	)]
	#[serde(skip_deserializing)]
	contract_hash: H160,
	scope: WitnessScope,
}

impl Hash for ContractSigner {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.signer_hash.hash(state);
		self.scopes.hash(state);
		// self.allowed_contracts.hash(state);
		// self.allowed_groups.hash(state);
		self.rules.hash(state);
		self.verify_params.hash(state);
		self.contract_hash.hash(state);
		self.scope.hash(state);
	}
}

impl SignerTrait for ContractSigner {
	fn get_type(&self) -> SignerType {
		SignerType::ContractSigner
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

	fn get_rules(&self) -> &Vec<WitnessRule> {
		&self.rules
	}

	fn get_rules_mut(&mut self) -> &mut Vec<WitnessRule> {
		&mut self.rules
	}
}

impl ContractSigner {
	fn new(
		contract_hash: H160,
		scope: WitnessScope,
		verify_params: Vec<ContractParameter>,
	) -> Self {
		Self {
			signer_hash: Default::default(),
			scopes: vec![],
			allowed_contracts: vec![],
			allowed_groups: vec![],
			rules: vec![],
			verify_params,
			contract_hash,
			scope,
		}
	}

	/// Creates a new `ContractSigner` with the "Called By Entry" scope.
	///
	/// # Arguments
	///
	/// * `contract_hash` - The hash of the contract.
	/// * `verify_params` - The parameters for contract verification.
	pub fn called_by_entry(contract_hash: H160, verify_params: &[ContractParameter]) -> Self {
		Self::new(contract_hash, WitnessScope::CalledByEntry, verify_params.to_vec())
	}

	/// Creates a new `ContractSigner` with the "Global" scope.
	///
	/// # Arguments
	///
	/// * `contract_hash` - The hash of the contract.
	/// * `verify_params` - The parameters for contract verification.
	pub fn global(contract_hash: H160, verify_params: &[ContractParameter]) -> Self {
		Self::new(contract_hash, WitnessScope::Global, verify_params.to_vec())
	}
}

impl NeoSerializable for ContractSigner {
	type Error = TransactionError;

	fn size(&self) -> usize {
		let mut size: usize = NeoConstants::HASH160_SIZE as usize;
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

	fn decode(reader: &mut Decoder) -> Result<Self, Self::Error>
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
		Ok(Self {
			signer_hash,
			scopes,
			allowed_contracts,
			allowed_groups,
			rules,
			verify_params: vec![],
			contract_hash: Default::default(),
			scope: WitnessScope::None,
		})
	}

	fn to_array(&self) -> Vec<u8> {
		let mut writer = Encoder::new();
		self.encode(&mut writer);
		writer.to_bytes()
	}
}
