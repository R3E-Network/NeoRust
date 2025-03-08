use std::fmt;

use neo_codec::{Decoder, Encoder, NeoSerializable, CodecError};
use neo_common::HashableForVec;
use neo_error::TypeError;
use primitive_types::H160;
use serde::{Deserialize, Serialize};

use crate::{contract::contract_method_token::ContractMethodToken, StackItem, bytes::Bytes};

/*
┌───────────────────────────────────────────────────────────────────────┐
│                    NEO Executable Format 3 (NEF3)                     │
├──────────┬───────────────┬────────────────────────────────────────────┤
│  Field   │     Type      │                  Comment                   │
├──────────┼───────────────┼────────────────────────────────────────────┤
│ Magic    │ uint32        │ Magic header                               │
│ Compiler │ byte[64]      │ Compiler name and version                  │
├──────────┼───────────────┼────────────────────────────────────────────┤
│ Source   │ byte[]        │ The url of the source files, max 255 bytes │
│ Reserve  │ byte[2]       │ Reserved for future extensions. Must be 0. │
│ Tokens   │ MethodToken[] │ Method tokens                              │
│ Reserve  │ byte[2]       │ Reserved for future extensions. Must be 0. │
│ Script   │ byte[]        │ Var bytes for the payload                  │
├──────────┼───────────────┼────────────────────────────────────────────┤
│ Checksum │ uint32        │ First four bytes of double SHA256 hash     │
└──────────┴───────────────┴────────────────────────────────────────────┘
 */

/// NEF file format for Neo N3 smart contracts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NefFile {
	pub magic: u32,
	pub compiler: String,
	pub source: Option<String>,
	pub tokens: Vec<ContractMethodToken>,
	pub script: Vec<u8>,
	pub checksum: u32,
}

// Forward declare ContractParameter to be implemented later
#[derive(Debug, Clone)]
pub struct ContractParameter {
	pub value: String,
}

impl Into<ContractParameter> for NefFile {
	fn into(self) -> ContractParameter {
		ContractParameter { value: self.to_array().into_iter().map(|b| b as char).collect() }
	}
}

impl NefFile {
	/// Magic number for NEF files: "NEF3" in ASCII
	pub const MAGIC: u32 = 0x3346454E;
	const MAGIC_SIZE: usize = 4;
	const COMPILER_SIZE: usize = 64;
	const MAX_SOURCE_URL_SIZE: usize = 256;
	const MAX_SCRIPT_LENGTH: usize = 512 * 1024;
	const CHECKSUM_SIZE: usize = 4;
	pub const HEADER_SIZE: usize = Self::MAGIC_SIZE + Self::COMPILER_SIZE;

	// Reference to StackItem constants
	const BYTE_STRING_VALUE: &'static str = "ByteString";

	fn get_checksum_as_integer(bytes: &Vec<u8>) -> Result<i32, TypeError> {
		let mut bytes = bytes.clone();
		bytes.reverse();
		bytes.try_into().map(i32::from_be_bytes).map_err(|_| {
			TypeError::InvalidEncoding(String::from("Failed to convert checksum bytes to i32"))
		})
	}

	fn compute_checksum_from_bytes(bytes: Vec<u8>) -> Result<Vec<u8>, TypeError> {
		let mut file_bytes = bytes.clone();
		file_bytes.truncate(bytes.len() - Self::CHECKSUM_SIZE);
		file_bytes.hash256()[..Self::CHECKSUM_SIZE].try_into().map_err(|_| {
			TypeError::InvalidEncoding(String::from("Failed to extract checksum from hash"))
		})
	}

	fn read_from_file(file: &str) -> Result<Self, TypeError> {
		let file_bytes = std::fs::read(file)
			.map_err(|e| TypeError::InvalidArgError(format!("Failed to read NEF file: {}", e)))?;

		if file_bytes.len() > 0x100000 {
			return Err(TypeError::InvalidArgError(String::from("NEF file is too large")));
		}

		let mut reader = Decoder::new(&file_bytes);
		reader.read_serializable().map_err(|e| {
			TypeError::InvalidEncoding(format!("Failed to deserialize NEF file: {}", e))
		})
	}

	/// Deserializes a NEF file from a byte array
	///
	/// # Arguments
	///
	/// * `bytes` - The byte array to deserialize
	///
	/// # Returns
	///
	/// A `Result` containing the deserialized NEF file or a `TypeError`
	pub fn deserialize(bytes: &[u8]) -> Result<Self, TypeError> {
		if bytes.len() > 0x100000 {
			return Err(TypeError::InvalidArgError(String::from("NEF file is too large")));
		}

		let mut reader = Decoder::new(bytes);
		reader.read_serializable().map_err(|e| {
			TypeError::InvalidEncoding(format!("Failed to deserialize NEF file: {}", e))
		})
	}

	fn read_from_stack_item(_item: StackItem) -> Result<Self, TypeError> {
		// This would need to be implemented properly
		Err(TypeError::InvalidEncoding(String::from("Not implemented")))
	}

	// Add a public method to compute checksum
	pub fn compute_checksum(&self) -> Result<Vec<u8>, TypeError> {
		let file_bytes = self.to_array();
		file_bytes.hash256()[..Self::CHECKSUM_SIZE].try_into().map_err(|_| {
			TypeError::InvalidEncoding(String::from("Failed to convert hash to checksum"))
		})
	}
}

impl NeoSerializable for NefFile {
	type Error = TypeError;

	fn size(&self) -> usize {
		let mut size = Self::HEADER_SIZE;
		size += self.source.as_ref().map_or(0, |s| s.len() + 1);
		size += self.tokens.len() + 2;
		size += self.script.len();
		size += Self::CHECKSUM_SIZE;

		size
	}

	fn encode(&self, writer: &mut Encoder) {
		writer.write_u32(Self::MAGIC);
		writer
			.write_fixed_string(&Some(self.compiler.clone()), Self::COMPILER_SIZE)
			.expect("Failed to serialize compiler");
		writer.write_var_string(self.source.as_deref().unwrap_or_default());
		writer.write_u8(0);
		writer.write_serializable_variable_list(&self.tokens);
		writer.write_u16(0);
		writer.write_var_bytes(&self.script);
		writer.write_bytes(&self.checksum.to_be_bytes());
	}

	fn decode(reader: &mut Decoder<'_>) -> Result<Self, Self::Error> {
		let magic = reader
			.read_u32()
			.map_err(|e| TypeError::InvalidEncoding(format!("Failed to read magic: {}", e)))?;

		if magic != Self::MAGIC {
			return Err(TypeError::InvalidEncoding(String::from("Invalid magic")));
		}

		let compiler_bytes = reader.read_bytes(Self::COMPILER_SIZE)?;
		let compiler = String::from_utf8(compiler_bytes.to_vec())
			.map_err(|_| CodecError::InvalidEncoding(String::from("Invalid compiler")))?;

		let source = reader.read_var_string()?;
		if source.len() > Self::MAX_SOURCE_URL_SIZE {
			return Err(TypeError::InvalidEncoding(String::from("Invalid source url")));
		}

		if reader.read_u8() != 0 {
			return Err(TypeError::InvalidEncoding(String::from("Invalid reserve bytes")));
		}

		let tokens = reader.read_serializable_list()?;

		if reader.read_u16().map_err(|e| {
			TypeError::InvalidEncoding(format!("Failed to read reserve bytes: {}", e))
		})? != 0
		{
			return Err(TypeError::InvalidEncoding(String::from("Invalid reserve bytes")));
		}

		let script = reader.read_var_bytes()?;
		if script.is_empty() {
			return Err(TypeError::InvalidEncoding(String::from("Invalid script")));
		}

		let checksum_bytes = reader.read_bytes(Self::CHECKSUM_SIZE)?;
		let checksum = u32::from_be_bytes(checksum_bytes.try_into().map_err(|_| {
			TypeError::InvalidEncoding(String::from("Failed to convert checksum bytes to u32"))
		})?);
		
		// Create a NefFile instance
		let nef_file = NefFile {
			magic,
			compiler,
			source: Some(source),
			tokens,
			script,
			checksum,
		};
		
		// Compute checksum from the encoded data
		let mut encoder = Encoder::new();
		// Encode everything except the checksum
		encoder.write_u32(nef_file.magic);
		encoder.write_fixed_string(&Some(nef_file.compiler.clone()), Self::COMPILER_SIZE)
			.expect("Failed to serialize compiler");
		encoder.write_var_string(nef_file.source.as_deref().unwrap_or_default());
		encoder.write_u8(0);
		encoder.write_serializable_variable_list(&nef_file.tokens);
		encoder.write_u16(0);
		encoder.write_var_bytes(&nef_file.script);
		
		let computed_checksum = Self::compute_checksum_from_bytes(encoder.to_bytes())?;
		let computed_checksum_u32 = u32::from_be_bytes(computed_checksum.try_into().map_err(|_| {
			TypeError::InvalidEncoding(String::from("Failed to convert computed checksum bytes to u32"))
		})?);

		if checksum != computed_checksum_u32 {
			return Err(TypeError::InvalidEncoding(String::from("Invalid checksum")));
		}

		Ok(nef_file)
	}

	fn to_array(&self) -> Vec<u8> {
		let mut writer = Encoder::new();
		self.encode(&mut writer);
		writer.to_bytes()
	}
}
