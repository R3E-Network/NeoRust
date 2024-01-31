use crate::core::{
	error::BuilderError, script::interop_service::InteropService,
	transaction::call_flags::CallFlags,
};
use getset::{Getters, Setters};
use neo_codec::Encoder;
use neo_crypto::keys::Secp256r1PublicKey;
use neo_types::{
	contract_parameter::{ContractParameter, ParameterValue},
	contract_parameter_type::ContractParameterType,
	op_code::OpCode,
	script_hash::ScriptHashExtension,
	Bytes,
};
use num_bigint::{BigInt, Sign};
use num_traits::ToPrimitive;
use primitive_types::H160;
use std::collections::HashMap;
use tokio::io::AsyncWriteExt;

#[derive(Debug, PartialEq, Eq, Hash, Getters, Setters)]
pub struct ScriptBuilder {
	#[getset(get = "pub")]
	pub script: Encoder,
}

impl ScriptBuilder {
	pub fn new() -> Self {
		Self { script: Encoder::new() }
	}
	pub fn op_code(&mut self, op_codes: &[OpCode]) -> &mut Self {
		for opcode in op_codes {
			self.script.write_u8(opcode.opcode());
		}
		self
	}

	pub fn op_code_with_arg(&mut self, opcode: OpCode, argument: Bytes) -> &mut Self {
		self.script.write_u8(opcode.opcode());
		let _ = self.script.write_bytes(&argument);
		self
	}

	pub fn contract_call(
		&mut self,
		hash160: &H160,
		method: &str,
		params: &[ContractParameter],
		call_flags: CallFlags,
	) -> Result<&mut Self, BuilderError> {
		if params.is_empty() {
			self.op_code(&[OpCode::NewArray]);
		} else {
			self.push_params(params);
		}

		Ok(self
			.push_integer(BigInt::from(call_flags.value()))
			.unwrap()
			.push_data(method.as_bytes().to_vec())
			.unwrap()
			.push_data(hash160.to_vec())
			.unwrap()
			.sys_call(InteropService::SystemContractCall))
	}

	pub fn sys_call(&mut self, operation: InteropService) -> &mut Self {
		self.op_code(&[OpCode::Syscall])
			.push_data(operation.hash().into_bytes())
			.unwrap()
	}

	pub fn push_params(&mut self, params: &[ContractParameter]) -> &mut Self {
		for param in params {
			self.push_param(param).unwrap();
		}

		self.push_integer(BigInt::from(params.len())).unwrap().op_code(&[OpCode::Pack])
	}

	pub fn push_param(&mut self, param: &ContractParameter) -> Result<&mut Self, BuilderError> {
		if param.get_type() == ContractParameterType::Any {
			self.op_code(&[OpCode::PushNull]);
		}
		match &param.value.clone().unwrap() {
			ParameterValue::Boolean(b) => self.push_bool(*b),
			ParameterValue::Integer(i) => self.push_integer(BigInt::from(i.clone())).unwrap(),
			ParameterValue::ByteArray(b)
			| ParameterValue::Signature(b)
			| ParameterValue::PublicKey(b) => self.push_data(b.as_bytes().to_vec()).unwrap(),
			ParameterValue::Hash160(h) => self.push_data(h.as_bytes().to_vec()).unwrap(),
			ParameterValue::Hash256(h) => self.push_data(h.as_bytes().to_vec()).unwrap(),
			ParameterValue::String(s) => self.push_data(s.as_bytes().to_vec()).unwrap(),
			ParameterValue::Array(arr) => self.push_array(arr).unwrap(),
			ParameterValue::Map(map) => {
				// Create an empty HashMap to hold your ContractParameter key-value pairs
				let mut map_value: HashMap<ContractParameter, ContractParameter> = HashMap::new();

				// Iterate over pairs of elements in the vector
				// (assuming the vector has an even number of elements)
				for i in (0..map.len()).step_by(2) {
					let key: ContractParameter =
						serde_json::from_str(map[i].as_str().unwrap()).unwrap();
					let value: ContractParameter =
						serde_json::from_str(map[i + 1].as_str().unwrap()).unwrap();

					// Insert the key-value pair into the HashMap
					map_value.insert(key, value);
				}

				self.push_map(&map_value).unwrap()
			},
			_ =>
				return Err(BuilderError::IllegalArgument("Unsupported parameter type".to_string())),
		};

		Ok(self)
	}

	// Additional push_* methods
	pub fn push_integer(&mut self, value: BigInt) -> Result<&mut Self, BuilderError> {
		if value >= BigInt::from(-1) && value <= BigInt::from(16) {
			self.op_code(vec![OpCode::try_from(value.to_i32().unwrap() as u8 + OpCode::Push0 as u8).unwrap()].as_slice());
		} else {
			let bytes = value.to_signed_bytes_le();

			let padded = match bytes.as_slice().len() {
				1 => Self::pad_right(&bytes, 1, value.sign() == Sign::Minus),
				2 => Self::pad_right(&bytes, 2, value.sign() == Sign::Minus),
				len if len <= 4 => Self::pad_right(&bytes, 4, value.sign() == Sign::Minus),
				len if len <= 8 => Self::pad_right(&bytes, 8, value.sign() == Sign::Minus),
				len if len <= 16 => Self::pad_right(&bytes, 16, value.sign() == Sign::Minus),
				_ => Self::pad_right(&bytes, 32, value.sign() == Sign::Minus),
			};

			let opcode = match bytes.len() {
				1 => OpCode::PushInt8,
				2 => OpCode::PushInt16,
				len if len <= 4 => OpCode::PushInt32,
				len if len <= 8 => OpCode::PushInt64,
				len if len <= 16 => OpCode::PushInt128,
				_ => OpCode::PushInt256,
			};

			self.op_code_with_arg(opcode, padded);
		}

		Ok(self)
	}

	fn pad_right(bytes: &[u8], size: usize, negative: bool) -> Vec<u8> {
		let pad_value = if negative { 0xFF } else { 0 };

		let mut padded = vec![0; size];
		padded[0..bytes.len()].copy_from_slice(bytes);
		padded[bytes.len()..].fill(pad_value);
		padded
	}

	// Push data handling

	pub fn push_data(&mut self, data: Vec<u8>) -> Result<&mut Self, BuilderError> {
		match data.len() {
			0..=0xff => {
				self.op_code(&[OpCode::PushData1]);
				self.script.write_u8(data.len() as u8);
				let _ = self.script.write_bytes(&data);
			},
			0x100..=0xffff => {
				self.op_code(&[OpCode::PushData2]);
				self.script.write_u16(data.len() as u16);
				let _ = self.script.write_bytes(&data);
			},
			_ => {
				self.op_code(&[OpCode::PushData4]);
				self.script.write_u32(data.len() as u32);
				let _ = self.script.write_bytes(&data);
			},
			// _ => return Err(BuilderError::IllegalArgument("Data too long".to_string())),
		}
		Ok(self)
	}

	pub fn push_bool(&mut self, b: bool) -> &mut Self {
		if b {
			self.op_code(&[OpCode::PushTrue])
		} else {
			self.op_code(&[OpCode::PushFalse])
		};
		self
	}

	pub fn push_array(&mut self, arr: &[ContractParameter]) -> Result<&mut Self, BuilderError> {
		if arr.is_empty() {
			self.op_code(&[OpCode::NewArray0]);
		} else {
			self.push_params(arr);
		};
		Ok(self)
	}

	pub fn push_map(
		&mut self,
		map: &HashMap<ContractParameter, ContractParameter>,
	) -> Result<&mut Self, BuilderError> {
		for (k, v) in map {
			let kk: ContractParameter = k.clone().into();
			let vv: ContractParameter = v.clone().into();
			self.push_param(&vv).unwrap();
			self.push_param(&kk).unwrap();
		}

		Ok(self.push_integer(BigInt::from(map.len())).unwrap().op_code(&[OpCode::PackMap]))
	}

	// Additional helper methods

	pub fn pack(&mut self) -> &mut Self {
		self.op_code(&[OpCode::Pack])
	}

	pub fn to_bytes(&self) -> Bytes {
		self.script.to_bytes()
	}

	pub fn build_verification_script(pub_key: &Secp256r1PublicKey) -> Bytes {
		let mut sb = ScriptBuilder::new();
		sb.push_data(pub_key.get_encoded(true))
			.unwrap()
			.sys_call(InteropService::SystemCryptoCheckSig);
		sb.to_bytes()
	}

	pub fn build_multi_sig_script(
		pubkeys: &mut [Secp256r1PublicKey],
		threshold: u8,
	) -> Result<Bytes, BuilderError> {
		let mut sb = ScriptBuilder::new();
		sb.push_integer(BigInt::from(threshold)).unwrap();
		pubkeys.sort_by(|a, b| a.get_encoded(true).cmp(&b.get_encoded(true)));
		for pk in pubkeys.iter() {
			sb.push_data(pk.get_encoded(true)).unwrap();
		}
		sb.push_integer(BigInt::from(pubkeys.len())).unwrap();
		sb.sys_call(InteropService::SystemCryptoCheckMultiSig);
		Ok(sb.to_bytes())
	}

	pub fn build_contract_script(
		sender: &H160,
		nef_checksum: u32,
		name: &str,
	) -> Result<Bytes, BuilderError> {
		let mut sb = ScriptBuilder::new();
		sb.op_code(&[OpCode::Abort])
			.push_data(sender.to_vec())
			.unwrap()
			.push_integer(BigInt::from(nef_checksum))
			.unwrap()
			.push_data(name.as_bytes().to_vec())
			.unwrap();
		Ok(sb.to_bytes())
	}
	pub fn build_contract_call_and_unwrap_iterator(
		contract_hash: &H160,
		method: &str,
		params: &[ContractParameter],
		max_items: u32,
		call_flags: CallFlags,
	) -> Result<Bytes, BuilderError> {
		let mut sb = Self::new();
		sb.push_integer(BigInt::from(max_items)).unwrap();

		sb.contract_call(contract_hash, method, params, call_flags).unwrap();

		sb.op_code(&[OpCode::NewArray]);

		let cycle_start = sb.len();
		sb.op_code(&[OpCode::Over]);
		sb.sys_call(InteropService::SystemIteratorNext);

		let jmp_if_not = sb.len();
		sb.op_code_with_arg(OpCode::JmpIf, vec![0]);

		sb.op_code(&[OpCode::Dup, OpCode::Push2, OpCode::Pick])
			.sys_call(InteropService::SystemIteratorValue)
			.op_code(&[
				OpCode::Append,
				OpCode::Dup,
				OpCode::Size,
				OpCode::Push3,
				OpCode::Pick,
				OpCode::Ge,
			]);

		let jmp_if_max = sb.len();
		sb.op_code_with_arg(OpCode::JmpIf, vec![0]);

		let jmp_offset = sb.len();
		let jmp_bytes = (cycle_start - jmp_offset) as u8;
		sb.op_code_with_arg(OpCode::Jmp, vec![jmp_bytes]);

		let load_result = sb.len();
		sb.op_code(&[OpCode::Nip, OpCode::Nip]);

		let mut script = sb.to_bytes();
		let jmp_not_bytes = (load_result - jmp_if_not) as i8;
		script[jmp_if_not + 1] = jmp_not_bytes as u8;

		let jmp_max_bytes = (load_result - jmp_if_max) as i8;
		script[jmp_if_max + 1] = jmp_max_bytes as u8;

		Ok(script)
	}

	pub fn len(&self) -> usize {
		self.script().size()
	}
	// Other static helper methods
}

#[cfg(test)]
mod tests {
	use super::*;
	use hex_literal::hex;
	use num_bigint::BigInt;
	use num_traits::FromPrimitive;
	use std::vec;

	#[test]
	fn test_push_empty_array() {
		let mut builder = ScriptBuilder::new();
		builder.push_array(&[]).unwrap();
		assert_eq!(builder.to_bytes(), vec![OpCode::NewArray0 as u8]);
	}

	#[test]
	fn test_push_byte_array() {
		let mut builder = ScriptBuilder::new();

		builder.push_data(vec![0xAAu8; 1]).unwrap();
		assert_eq!(builder.to_bytes()[..2], hex!("0c01"));

		let mut builder = ScriptBuilder::new();
		builder.push_data(vec![0xAAu8; 75]).unwrap();
		assert_eq!(builder.to_bytes()[..2], hex!("0c4b"));

		let mut builder = ScriptBuilder::new();
		builder.push_data(vec![0xAAu8; 256]).unwrap();
		assert_eq!(builder.to_bytes()[..3], hex!("0d0001"));

		let mut builder = ScriptBuilder::new();
		builder.push_data(vec![0xAAu8; 65536]).unwrap();
		assert_eq!(builder.to_bytes()[..5], hex!("0e00000100"));
	}

	#[test]
	fn test_push_string() {
		let mut builder = ScriptBuilder::new();

		builder.push_data("".as_bytes().to_vec()).unwrap();
		assert_eq!(builder.to_bytes()[..2], hex!("0c00"));

		builder.push_data("a".as_bytes().to_vec()).unwrap();
		assert_eq!(builder.to_bytes()[..3], hex!("0c0161"));

		builder.push_data("a".repeat(10000).as_bytes().to_vec()).unwrap();
		assert_eq!(builder.to_bytes()[..3], hex!("0d1027"));
	}

	#[test]
	fn test_push_integer() {
		let mut builder = ScriptBuilder::new();
		builder.push_integer(BigInt::from(0)).unwrap();
		assert_eq!(builder.to_bytes()[..1], vec![OpCode::Push0 as u8]);

		let mut builder = ScriptBuilder::new();
		builder.push_integer(BigInt::from(1)).unwrap();
		assert_eq!(builder.to_bytes()[..1], vec![OpCode::Push1 as u8]);

		let mut builder = ScriptBuilder::new();
		builder.push_integer(BigInt::from(16)).unwrap();
		assert_eq!(builder.to_bytes()[..1], vec![OpCode::Push16 as u8]);

		let mut builder = ScriptBuilder::new();
		builder.push_integer(BigInt::from(17)).unwrap();
		assert_eq!(builder.to_bytes()[..2], hex!("0011"));

		let mut builder = ScriptBuilder::new();
		builder.push_integer(BigInt::from(-100000)).unwrap();
		assert_eq!(builder.to_bytes(), hex!("026079FEFF"));

		// let mut builder = ScriptBuilder::new();
		// builder.push_integer(-100000000000).unwrap();
		// assert_eq!(builder.to_bytes()[builder.len()-8..], hex!("FFE8B78918000000"));
		//
		// let mut builder = ScriptBuilder::new();
		// builder.push_integer(BigInt::from_i64(100000000000).unwrap()).unwrap();
		// assert_eq!(builder.to_bytes()[builder.len()-8..], hex!("001748768E00000000"));
	}

	#[test]
	fn test_verification_script() {
		// let pubkey1 = hex!("035fdb1d1f06759547020891ae97c729327853aeb1256b6fe0473bc2e9fa42ff50");
		// let pubkey2 = hex!("03eda286d19f7ee0b472afd1163d803d620a961e1581a8f2704b52c0285f6e022d");
		// let pubkey3 = hex!("03ac81ec17f2f15fd6d193182f927c5971559c2a32b9408a06fec9e711fb7ca02e");
		//
		// let script = ScriptBuilder::build_multisig_script(&[pubkey1, pubkey2, pubkey3], 2).unwrap();
		//
		// let expected = hex!("5221035fdb1d1f06759547020891ae97c729327853aeb1256b6fe0473bc2e9fa42ff50210"
		//     "03ac81ec17f2f15fd6d193182f927c5971559c2a32b9408a06fec9e711fb7ca02e210"
		//     "03eda286d19f7ee0b472afd1163d803d620a961e1581a8f2704b52c0285f6e022d53ae");

		// assert_eq!(script, expected);
	}

	#[test]
	fn test_map() {
		// test map packing in different orders
		// let mut builder = ScriptBuilder::new();
		// builder.push_map(BTreeMap::from([
		// 	(0u8.into(), "first".into()),
		// 	(b"second".to_vec().into(), true.into())
		// ])).unwrap();
		//
		// let expected1 = hex!("66697273740001737365636f6e6401c2780200");
		// let expected2 = hex!("01c2780200737365636f6e64666972737400010200");
		//
		// assert!(builder.build() == expected1 || builder.build() == expected2);
	}
}
