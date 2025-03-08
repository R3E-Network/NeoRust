use std::{hash::Hash, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::{serde_with_utils::{deserialize_script_hash, serialize_script_hash}, ScriptHash};

use primitive_types::H160;

use neo_codec::{Decoder, Encoder, NeoSerializable, CodecError};
use neo_error::TypeError;

#[derive(Serialize, Deserialize, Debug, Hash, Clone, PartialEq)]
pub struct ContractMethodToken {
	#[serde(deserialize_with = "deserialize_script_hash")]
	#[serde(serialize_with = "serialize_script_hash")]
	hash: ScriptHash,
	method: String,
	#[serde(rename = "paramcount")]
	param_count: u32,
	#[serde(rename = "hasreturnvalue")]
	has_return_value: bool,
	#[serde(rename = "callflags")]
	call_flags: String,
}

impl ContractMethodToken {
	pub fn new(
		hash: H160,
		method: String,
		param_count: u32,
		has_return_value: bool,
		call_flags: String,
	) -> Self {
		Self { hash, method, param_count, has_return_value, call_flags }
	}
}

impl NeoSerializable for ContractMethodToken {
	type Error = TypeError;

	fn size(&self) -> usize {
		let mut size = H160::len_bytes();
		size += self.method.len();
		size += 4; // param_count size (u32)
		size += 1; // has_return_value size (bool)
		size += self.call_flags.len();

		size
	}

	fn encode(&self, writer: &mut Encoder) {
		writer.write_serializable_fixed(&self.hash);
		writer.write_var_string(&self.method);
		writer.write_u32(self.param_count);
		writer.write_bool(self.has_return_value);
		writer.write_var_string(&self.call_flags);
	}

	fn decode(reader: &mut Decoder<'_>) -> Result<Self, Self::Error> {
		let hash = reader.read_serializable::<H160>().map_err(|e| {
			TypeError::InvalidEncoding(format!("Failed to decode hash: {}", e))
		})?;
		
		let method = reader.read_var_string().map_err(|e| {
			TypeError::InvalidEncoding(format!("Failed to decode method: {}", e))
		})?;
		
		let param_count = reader.read_u32().map_err(|e| {
			TypeError::InvalidEncoding(format!("Failed to decode param_count: {}", e))
		})?;
		
		let has_return_value = reader.read_bool().map_err(|e| {
			TypeError::InvalidEncoding(format!("Failed to decode has_return_value: {}", e))
		})?;
		
		let call_flags = reader.read_var_string().map_err(|e| {
			TypeError::InvalidEncoding(format!("Failed to decode call_flags: {}", e))
		})?;

		Ok(ContractMethodToken {
			hash,
			method,
			param_count,
			has_return_value,
			call_flags,
		})
	}

	fn to_array(&self) -> Vec<u8> {
		let mut writer = Encoder::new();
		self.encode(&mut writer);
		writer.to_bytes()
	}
}
