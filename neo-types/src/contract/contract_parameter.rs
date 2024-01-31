use crate::{
	contract_parameter_type::ContractParameterType, nef_file::NefFile, nns_name::NNSName,
	role::Role, serde_value::ValueExtension,
};
use base64::encode;
use elliptic_curve::sec1::ToEncodedPoint;
use std::collections::HashMap;

use neo_codec::encode::NeoSerializable;
use neo_crypto::keys::Secp256r1PublicKey;
use primitive_types::{H160, H256};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha3::Digest;
use std::hash::{Hash, Hasher};
use strum_macros::{Display, EnumString};

#[derive(Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Clone)]
pub struct ContractParameter {
	#[serde(skip_serializing_if = "Option::is_none")]
	name: Option<String>,
	#[serde(rename = "type")]
	typ: ContractParameterType,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub value: Option<ParameterValue>,
}

impl From<&H160> for ContractParameter {
	fn from(value: &H160) -> Self {
		Self::hash160(value)
	}
}

impl From<H160> for ContractParameter {
	fn from(value: H160) -> Self {
		Self::hash160(&value)
	}
}

impl From<u8> for ContractParameter {
	fn from(value: u8) -> Self {
		Self::integer(value as i64)
	}
}

impl From<i32> for ContractParameter {
	fn from(value: i32) -> Self {
		Self::integer(value as i64)
	}
}

impl From<u32> for ContractParameter {
	fn from(value: u32) -> Self {
		Self::integer(value as i64)
	}
}

impl From<u64> for ContractParameter {
	fn from(value: u64) -> Self {
		Self::integer(value as i64)
	}
}

impl From<&Role> for ContractParameter {
	fn from(value: &Role) -> Self {
		Self::integer(value.clone() as i64)
	}
}

impl From<&str> for ContractParameter {
	fn from(value: &str) -> Self {
		Self::string(value.to_string())
	}
}

impl From<usize> for ContractParameter {
	fn from(value: usize) -> Self {
		Self::integer(value as i64)
	}
}

impl From<&[u8]> for ContractParameter {
	fn from(value: &[u8]) -> Self {
		Self::byte_array(value.to_vec())
	}
}

impl From<Vec<u8>> for ContractParameter {
	fn from(value: Vec<u8>) -> Self {
		Self::byte_array(value)
	}
}

impl From<&Secp256r1PublicKey> for ContractParameter {
	fn from(value: &Secp256r1PublicKey) -> Self {
		Self::public_key(value)
	}
}

impl From<&H256> for ContractParameter {
	fn from(value: &H256) -> Self {
		Self::hash256(value)
	}
}

impl From<&Vec<ContractParameter>> for ContractParameter {
	fn from(value: &Vec<ContractParameter>) -> Self {
		Self::array(value.clone())
	}
}

// impl From<&[(ContractParameter, ContractParameter)]> for ContractParameter {
// 	fn from(value: &[(ContractParameter, ContractParameter)]) -> Self {
// 		Self::map(value.to_vec())
// 	}
// }

impl From<&NefFile> for ContractParameter {
	fn from(value: &NefFile) -> Self {
		Self::byte_array(value.to_array())
	}
}

impl From<String> for ContractParameter {
	fn from(value: String) -> Self {
		Self::string(value)
	}
}

impl From<bool> for ContractParameter {
	fn from(value: bool) -> Self {
		Self::bool(value)
	}
}
impl From<&String> for ContractParameter {
	fn from(value: &String) -> Self {
		Self::string(value.to_string())
	}
}

impl From<NNSName> for ContractParameter {
	fn from(value: NNSName) -> Self {
		Self::string(value.to_string())
	}
}

impl From<Value> for ContractParameter {
	fn from(value: Value) -> Self {
		match value {
			Value::Null => Self::new(ContractParameterType::Any),
			Value::Bool(b) => Self::bool(b),
			Value::Number(n) => Self::integer(n.as_i64().unwrap()),
			Value::String(s) => Self::string(s),
			Value::Array(a) =>
				Self::array(a.into_iter().map(|v| ContractParameter::from(v)).collect()),
			Value::Object(o) => Self::map(
				o.into_iter()
					.map(|(k, v)| (ContractParameter::from(k), ContractParameter::from(v)))
					.collect(),
			),
		}
	}
}

impl Into<Value> for ContractParameter {
	fn into(self) -> Value {
		match self.value.unwrap() {
			ParameterValue::Boolean(b) => Value::Bool(b),
			ParameterValue::Integer(i) => Value::Number(serde_json::Number::from(i)),
			ParameterValue::ByteArray(b) => Value::String(b),
			ParameterValue::String(s) => Value::String(s),
			ParameterValue::Hash160(h) => Value::String(h),
			ParameterValue::Hash256(h) => Value::String(h),
			ParameterValue::PublicKey(p) => Value::String(p),
			ParameterValue::Signature(s) => Value::String(s),
			ParameterValue::Array(a) => Value::Array(a.into_iter().map(|v| v.into()).collect()),
			ParameterValue::Map(m) => Value::Array(
				m.iter()
					.flat_map(|(key, value)| vec![key.clone().into(), value.clone().into()])
					.collect(),
			),
			ParameterValue::Any => Value::Null,
		}
	}
}

impl From<Vec<Value>> for ContractParameter {
	fn from(value: Vec<Value>) -> Self {
		Self::array(value.into_iter().map(|v| ContractParameter::from(v)).collect())
	}
}

// impl Into<Vec<Value>> for ContractParameter{
// 	fn into(self) -> Vec<Value> {
// 		match self.value.clone().unwrap() {
// 			ParameterValue::Array(a) => a.into_iter().map(|v| v.into()).collect(),
// 			ParameterValue::Map(m) => m.into_iter().map(|v| v.into()).collect(),
// 			_ => panic!("Cannot convert {:?} to Vec<Value>", self.clone()),
// 		}
// 	}
// }

impl ValueExtension for ContractParameter {
	fn to_value(&self) -> Value {
		Value::String(serde_json::to_string(self).unwrap())
	}
}

impl ValueExtension for Vec<ContractParameter> {
	fn to_value(&self) -> Value {
		self.iter().map(|x| x.to_value()).collect()
	}
}

#[derive(Display, EnumString, Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ParameterValue {
	Boolean(bool),
	Integer(i64),
	ByteArray(String),
	String(String),
	Hash160(String),
	Hash256(String),
	PublicKey(String),
	Signature(String),
	Array(Vec<ContractParameter>),
	Map(HashMap<ContractParameter, ContractParameter>),
	Any,
}

impl Hash for ParameterValue {
	fn hash<H: Hasher>(&self, state: &mut H) {
		match self {
			ParameterValue::Boolean(b) => b.hash(state),
			ParameterValue::Integer(i) => i.hash(state),
			ParameterValue::ByteArray(b) => b.hash(state),
			ParameterValue::String(s) => s.hash(state),
			ParameterValue::Hash160(h) => h.hash(state),
			ParameterValue::Hash256(h) => h.hash(state),
			ParameterValue::PublicKey(p) => p.hash(state),
			ParameterValue::Signature(s) => s.hash(state),
			ParameterValue::Array(a) => a.hash(state),
			ParameterValue::Map(m) =>
				for (k, v) in m {
					k.hash(state);
					v.hash(state);
				},
			ParameterValue::Any => "Any".hash(state),
		}
	}
}

impl ContractParameter {
	pub fn new(typ: ContractParameterType) -> Self {
		Self { name: None, typ, value: None }
	}

	pub fn get_type(&self) -> ContractParameterType {
		self.typ.clone()
	}

	pub fn with_value(typ: ContractParameterType, value: ParameterValue) -> Self {
		Self { name: None, typ, value: Some(value) }
	}

	pub fn bool(value: bool) -> Self {
		Self::with_value(ContractParameterType::Boolean, ParameterValue::Boolean(value))
	}

	pub fn integer(value: i64) -> Self {
		Self::with_value(ContractParameterType::Integer, ParameterValue::Integer(value))
	}

	pub fn byte_array(value: Vec<u8>) -> Self {
		let encoded = encode(value);
		Self::with_value(ContractParameterType::ByteArray, ParameterValue::ByteArray(encoded))
	}

	pub fn string(value: String) -> Self {
		Self::with_value(ContractParameterType::String, ParameterValue::String(value))
	}

	// Other helper methods
	pub fn hash160(value: &H160) -> Self {
		Self::with_value(ContractParameterType::H160, ParameterValue::Hash160(value.to_string()))
	}

	pub fn hash256(value: &H256) -> Self {
		Self::with_value(ContractParameterType::H256, ParameterValue::Hash256(value.to_string()))
	}

	pub fn public_key(value: &Secp256r1PublicKey) -> Self {
		Self::with_value(
			ContractParameterType::PublicKey,
			ParameterValue::PublicKey(hex::encode(value.get_encoded(true))),
		)
	}

	pub fn signature(value: &str) -> Self {
		Self::with_value(
			ContractParameterType::Signature,
			ParameterValue::Signature(value.to_string()),
		)
	}

	pub fn array(values: Vec<Self>) -> Self {
		Self::with_value(ContractParameterType::Array, ParameterValue::Array(values))
	}

	pub fn map(values: HashMap<Self, Self>) -> Self {
		// let json = values.into_iter().map(|(k, v)| json!({"key": k, "value": v})).collect();
		Self::with_value(ContractParameterType::Map, ParameterValue::Map(values))
	}

	pub fn hash(self) -> Vec<u8> {
		let mut hasher = std::collections::hash_map::DefaultHasher::new();
		Hash::hash(&self, &mut hasher);
		hasher.finish().to_be_bytes().to_vec()
	}
}
