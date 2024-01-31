use neo_types::*;

use crate::core::transaction::{
	signers::signer::SignerType::Transaction, transaction_error::TransactionError,
	witness_scope::WitnessScope::WitnessRules,
};
use neo_codec::{encode::NeoSerializable, Decoder, Encoder};
use neo_crypto::keys::Secp256r1PublicKey;
use primitive_types::H160;
use serde::{Deserialize, Deserializer, Serialize};
use std::hash::{Hash, Hasher};

/// Enum representing the different types of witness conditions that can be used in a smart contract.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum WitnessCondition {
	/// Boolean value.
	Boolean(bool),
	/// Not operator.
	Not(Box<WitnessCondition>),
	/// And operator.
	And(Vec<WitnessCondition>),
	/// Or operator.
	Or(Vec<WitnessCondition>),
	/// Script hash.
	ScriptHash(H160),
	/// Public key group.
	Group(Secp256r1PublicKey),
	/// Called by entry.
	CalledByEntry,
	/// Called by contract.
	CalledByContract(H160),
	/// Called by public key group.
	CalledByGroup(Secp256r1PublicKey),
}

impl Hash for WitnessCondition {
	/// Hashes the witness condition.
	fn hash<H: Hasher>(&self, state: &mut H) {
		match self {
			WitnessCondition::Boolean(b) => b.hash(state),
			WitnessCondition::Not(exp) => exp.hash(state),
			WitnessCondition::And(exp) => exp.hash(state),
			WitnessCondition::Or(exp) => exp.hash(state),
			WitnessCondition::ScriptHash(hash) => hash.hash(state),
			WitnessCondition::Group(group) => group.get_encoded(true).hash(state),
			WitnessCondition::CalledByEntry => WitnessCondition::CalledByEntry.hash(state),
			WitnessCondition::CalledByContract(hash) => hash.hash(state),
			WitnessCondition::CalledByGroup(group) => group.get_encoded(true).hash(state),
		}
	}
}

impl WitnessCondition {
	/// Maximum number of subitems.
	const MAX_SUBITEMS: usize = 16;
	/// Maximum nesting depth.
	const MAX_NESTING_DEPTH: usize = 2;

	/// Boolean value string.
	const BOOLEAN_VALUE: &'static str = "Boolean";
	/// Not operator string.
	const NOT_VALUE: &'static str = "Not";
	/// And operator string.
	const AND_VALUE: &'static str = "And";
	/// Or operator string.
	const OR_VALUE: &'static str = "Or";
	/// Script hash string.
	const SCRIPT_HASH_VALUE: &'static str = "ScriptHash";
	/// Public key group string.
	const GROUP_VALUE: &'static str = "Group";
	/// Called by entry string.
	const CALLED_BY_ENTRY_VALUE: &'static str = "CalledByEntry";
	/// Called by contract string.
	const CALLED_BY_CONTRACT_VALUE: &'static str = "CalledByContract";
	/// Called by public key group string.
	const CALLED_BY_GROUP_VALUE: &'static str = "CalledByGroup";

	/// Boolean byte value.
	const BOOLEAN_BYTE: u8 = 0x00;
	/// Not operator byte value.
	const NOT_BYTE: u8 = 0x01;
	/// And operator byte value.
	const AND_BYTE: u8 = 0x02;
	/// Or operator byte value.
	const OR_BYTE: u8 = 0x03;
	/// Script hash byte value.
	const SCRIPT_HASH_BYTE: u8 = 0x18;
	/// Public key group byte value.
	const GROUP_BYTE: u8 = 0x19;
	/// Called by entry byte value.
	const CALLED_BY_ENTRY_BYTE: u8 = 0x20;
	/// Called by contract byte value.
	const CALLED_BY_CONTRACT_BYTE: u8 = 0x28;
	/// Called by public key group byte value.
	const CALLED_BY_GROUP_BYTE: u8 = 0x29;

	/// Returns the JSON value of the witness condition.
	pub fn json_value(&self) -> &'static str {
		match self {
			WitnessCondition::Boolean(_) => WitnessCondition::BOOLEAN_VALUE,
			WitnessCondition::Not(_) => WitnessCondition::NOT_VALUE,
			WitnessCondition::And(_) => WitnessCondition::AND_VALUE,
			WitnessCondition::Or(_) => WitnessCondition::OR_VALUE,
			WitnessCondition::ScriptHash(_) => WitnessCondition::SCRIPT_HASH_VALUE,
			WitnessCondition::Group(_) => WitnessCondition::GROUP_VALUE,
			WitnessCondition::CalledByEntry => WitnessCondition::CALLED_BY_ENTRY_VALUE,
			WitnessCondition::CalledByContract(_) => WitnessCondition::CALLED_BY_CONTRACT_VALUE,
			WitnessCondition::CalledByGroup(_) => WitnessCondition::CALLED_BY_GROUP_VALUE,
		}
	}

	/// Returns the byte value of the witness condition.
	pub fn byte(&self) -> u8 {
		match self {
			WitnessCondition::Boolean(_) => WitnessCondition::BOOLEAN_BYTE,
			WitnessCondition::Not(_) => WitnessCondition::NOT_BYTE,
			WitnessCondition::And(_) => WitnessCondition::AND_BYTE,
			WitnessCondition::Or(_) => WitnessCondition::OR_BYTE,
			WitnessCondition::ScriptHash(_) => WitnessCondition::SCRIPT_HASH_BYTE,
			WitnessCondition::Group(_) => WitnessCondition::GROUP_BYTE,
			WitnessCondition::CalledByEntry => WitnessCondition::CALLED_BY_ENTRY_BYTE,
			WitnessCondition::CalledByContract(_) => WitnessCondition::CALLED_BY_CONTRACT_BYTE,
			WitnessCondition::CalledByGroup(_) => WitnessCondition::CALLED_BY_GROUP_BYTE,
		}
	}

	/// Returns the boolean expression of the witness condition.
	pub fn boolean_expression(&self) -> Option<bool> {
		match self {
			WitnessCondition::Boolean(b) => Some(*b),
			_ => None,
		}
	}

	/// Returns the expression of the witness condition.
	pub fn expression(&self) -> Option<&WitnessCondition> {
		match self {
			WitnessCondition::Not(exp) => Some(&exp),
			_ => None,
		}
	}

	/// Returns the expression list of the witness condition.
	pub fn expression_list(&self) -> Option<&[WitnessCondition]> {
		match self {
			WitnessCondition::And(exp) | WitnessCondition::Or(exp) => Some(&exp),
			_ => None,
		}
	}

	/// Returns the script hash of the witness condition.
	pub fn script_hash(&self) -> Option<&H160> {
		match self {
			WitnessCondition::ScriptHash(hash) | WitnessCondition::CalledByContract(hash) =>
				Some(hash),
			_ => None,
		}
	}

	/// Returns the public key group of the witness condition.
	pub fn group(&self) -> Option<&Secp256r1PublicKey> {
		match self {
			WitnessCondition::Group(group) | WitnessCondition::CalledByGroup(group) => Some(group),
			_ => None,
		}
	}
}

impl NeoSerializable for WitnessCondition {
	type Error = TransactionError;

	fn size(&self) -> usize {
		match self {
			WitnessCondition::Boolean(_) => 2,
			WitnessCondition::Not(_) => 1 + self.expression().unwrap().size(),
			WitnessCondition::And(_) | WitnessCondition::Or(_) => {
				let exp = self.expression_list().unwrap();
				1 + 1 + exp.len() + exp.iter().map(|e| e.size()).sum::<usize>()
			},
			WitnessCondition::ScriptHash(_) | WitnessCondition::CalledByContract(_) => 1 + 20,
			WitnessCondition::Group(_) | WitnessCondition::CalledByGroup(_) => 1 + 33,
			WitnessCondition::CalledByEntry => 1,
		}
	}

	fn encode(&self, writer: &mut Encoder) {
		match self {
			WitnessCondition::Boolean(b) => {
				writer.write_u8(WitnessCondition::BOOLEAN_BYTE);
				writer.write_bool(*b);
			},
			WitnessCondition::Not(exp) => {
				writer.write_u8(WitnessCondition::NOT_BYTE);
				writer.write_serializable_fixed(exp.expression().unwrap());
				// exp.writeSerializableFixed(writer);
			},
			WitnessCondition::And(exp) => {
				writer.write_u8(WitnessCondition::AND_BYTE);
				writer.write_serializable_variable_list(exp);
			},
			WitnessCondition::Or(exp) => {
				writer.write_u8(WitnessCondition::OR_BYTE);
				writer.write_serializable_variable_list(exp)
			},
			WitnessCondition::ScriptHash(hash) => {
				writer.write_u8(WitnessCondition::SCRIPT_HASH_BYTE);
				writer.write_serializable_fixed(hash);
			},
			WitnessCondition::Group(group) => {
				writer.write_u8(WitnessCondition::GROUP_BYTE);
				writer.write_serializable_fixed(group);
			},
			WitnessCondition::CalledByEntry => {
				writer.write_u8(WitnessCondition::CALLED_BY_ENTRY_BYTE);
			},
			WitnessCondition::CalledByContract(hash) => {
				writer.write_u8(WitnessCondition::CALLED_BY_CONTRACT_BYTE);
				writer.write_serializable_fixed(hash);
			},
			WitnessCondition::CalledByGroup(group) => {
				writer.write_u8(WitnessCondition::CALLED_BY_GROUP_BYTE);
				writer.write_serializable_fixed(group);
			},
		}
	}

	fn decode(reader: &mut Decoder) -> Result<Self, Self::Error> {
		let byte = reader.read_u8();
		match byte {
			WitnessCondition::BOOLEAN_BYTE => {
				let b = reader.read_bool();
				Ok(WitnessCondition::Boolean(b))
			},
			WitnessCondition::NOT_BYTE => {
				let exp = WitnessCondition::decode(reader)?;
				Ok(WitnessCondition::Not(Box::from(exp)))
			},
			WitnessCondition::OR_BYTE | WitnessCondition::AND_BYTE => {
				let len = reader.read_var_int()?;
				if len > WitnessCondition::MAX_SUBITEMS as i64 {
					return Err(TransactionError::InvalidWitnessCondition)
				}
				let exp = WitnessCondition::decode(reader)?;
				if byte == WitnessCondition::OR_BYTE {
					Ok(WitnessCondition::Or(vec![exp]))
				} else {
					Ok(WitnessCondition::And(vec![exp]))
				}
			},
			WitnessCondition::SCRIPT_HASH_BYTE | WitnessCondition::CALLED_BY_CONTRACT_BYTE => {
				let hash = H160::decode(reader)?;
				if byte == WitnessCondition::SCRIPT_HASH_BYTE {
					Ok(WitnessCondition::ScriptHash(hash))
				} else {
					Ok(WitnessCondition::CalledByContract(hash))
				}
			},
			WitnessCondition::GROUP_BYTE | WitnessCondition::CALLED_BY_GROUP_BYTE => {
				let group = Secp256r1PublicKey::decode(reader)?;
				if byte == WitnessCondition::GROUP_BYTE {
					Ok(WitnessCondition::Group(group))
				} else {
					Ok(WitnessCondition::CalledByGroup(group))
				}
			},
			WitnessCondition::CALLED_BY_ENTRY_BYTE => Ok(WitnessCondition::CalledByEntry),
			_ => Err(TransactionError::InvalidTransaction),
		}
	}

	fn to_array(&self) -> Vec<u8> {
		let mut writer = Encoder::new();
		self.encode(&mut writer);
		writer.to_bytes()
	}
}
