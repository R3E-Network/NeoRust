//! Builder for NeoVM scripts
//!
//! Contract calls are built by pushing arguments in reverse, packing them into
//! an array, pushing the method name and contract hash, then issuing a
//! `System.Contract.Call` syscall.

use crate::{
	errors::{Error, Result},
	opcode::{interop, CallFlags, OpCode},
	serialization::Writer,
};
use alloc::{format, string::String, vec::Vec};
use neo_primitives::Hash160;

/// Maximum size of a script this builder will produce, in bytes.
pub const MAX_SCRIPT_SIZE: usize = 65_536;

/// An argument to a contract call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractParameter {
	/// A boolean.
	Boolean(bool),
	/// A signed integer.
	Integer(i64),
	/// Raw bytes.
	ByteArray(Vec<u8>),
	/// A UTF-8 string.
	String(String),
	/// A 20-byte script hash, pushed in wire order.
	Hash160(Hash160),
	/// An ordered list of parameters.
	Array(Vec<ContractParameter>),
	/// The `null` value.
	Null,
}

/// Builds a NeoVM script.
#[derive(Debug, Default)]
pub struct ScriptBuilder {
	writer: Writer,
}

impl ScriptBuilder {
	/// Creates an empty builder.
	pub fn new() -> Self {
		Self { writer: Writer::new() }
	}

	/// Appends a raw opcode.
	pub fn emit(&mut self, opcode: OpCode) -> &mut Self {
		self.writer.write_u8(opcode.as_byte());
		self
	}

	/// Appends a raw opcode byte.
	pub fn emit_byte(&mut self, byte: u8) -> &mut Self {
		self.writer.write_u8(byte);
		self
	}

	/// Appends raw bytes with no length prefix.
	pub fn emit_bytes(&mut self, bytes: &[u8]) -> &mut Self {
		self.writer.write_bytes(bytes);
		self
	}

	/// Pushes an integer using the shortest encoding.
	pub fn push_integer(&mut self, value: i64) -> &mut Self {
		if let Some(byte) = OpCode::push_small_int(value) {
			return self.emit_byte(byte);
		}

		// Outside -1..=16, use the narrowest PushInt* that fits.
		if let Ok(v) = i8::try_from(value) {
			self.emit(OpCode::PushInt8).emit_bytes(&v.to_le_bytes())
		} else if let Ok(v) = i16::try_from(value) {
			self.emit(OpCode::PushInt16).emit_bytes(&v.to_le_bytes())
		} else if let Ok(v) = i32::try_from(value) {
			self.emit(OpCode::PushInt32).emit_bytes(&v.to_le_bytes())
		} else {
			self.emit(OpCode::PushInt64).emit_bytes(&value.to_le_bytes())
		}
	}

	/// Pushes a boolean.
	pub fn push_bool(&mut self, value: bool) -> &mut Self {
		self.emit(if value { OpCode::PushTrue } else { OpCode::PushFalse })
	}

	/// Pushes `null`.
	pub fn push_null(&mut self) -> &mut Self {
		self.emit(OpCode::PushNull)
	}

	/// Pushes raw bytes using the shortest `PushData` form.
	///
	/// Fails if `data` exceeds [`MAX_SCRIPT_SIZE`].
	pub fn push_data(&mut self, data: &[u8]) -> Result<&mut Self> {
		if data.len() > MAX_SCRIPT_SIZE {
			return Err(Error::LengthLimitExceeded {
				len: data.len() as u64,
				max: MAX_SCRIPT_SIZE,
			});
		}

		match data.len() {
			len if len <= u8::MAX as usize => {
				self.emit(OpCode::PushData1).emit_byte(len as u8);
			},
			len if len <= u16::MAX as usize => {
				self.emit(OpCode::PushData2).emit_bytes(&(len as u16).to_le_bytes());
			},
			len => {
				self.emit(OpCode::PushData4).emit_bytes(&(len as u32).to_le_bytes());
			},
		}

		Ok(self.emit_bytes(data))
	}

	/// Pushes a UTF-8 string.
	pub fn push_string(&mut self, s: &str) -> Result<&mut Self> {
		self.push_data(s.as_bytes())
	}

	/// Pushes a script hash in wire order.
	pub fn push_hash160(&mut self, hash: &Hash160) -> Result<&mut Self> {
		self.push_data(hash.as_bytes())
	}

	/// Pushes a contract parameter.
	pub fn push_parameter(&mut self, param: &ContractParameter) -> Result<&mut Self> {
		match param {
			ContractParameter::Boolean(value) => Ok(self.push_bool(*value)),
			ContractParameter::Integer(value) => Ok(self.push_integer(*value)),
			ContractParameter::ByteArray(bytes) => self.push_data(bytes),
			ContractParameter::String(s) => self.push_string(s),
			ContractParameter::Hash160(hash) => self.push_hash160(hash),
			ContractParameter::Null => Ok(self.push_null()),
			ContractParameter::Array(items) => self.push_array(items),
		}
	}

	/// Pushes an array of parameters.
	///
	/// Items are pushed in reverse so the VM pops them in order, then packed.
	pub fn push_array(&mut self, params: &[ContractParameter]) -> Result<&mut Self> {
		if params.is_empty() {
			return Ok(self.emit(OpCode::NewArray0));
		}

		for param in params.iter().rev() {
			self.push_parameter(param)?;
		}
		self.push_integer(params.len() as i64);
		Ok(self.emit(OpCode::Pack))
	}

	/// Appends a syscall by 4-byte interop descriptor.
	pub fn syscall(&mut self, descriptor: [u8; 4]) -> &mut Self {
		self.emit(OpCode::Syscall).emit_bytes(&descriptor)
	}

	/// Appends a contract call.
	///
	/// Emits: arguments array, call flags, method name, contract hash, then the
	/// `System.Contract.Call` syscall — the order the VM expects them popped.
	pub fn contract_call(
		&mut self,
		contract: &Hash160,
		method: &str,
		args: &[ContractParameter],
		flags: CallFlags,
	) -> Result<&mut Self> {
		if method.is_empty() {
			return Err(Error::InvalidData("contract method name must not be empty".into()));
		}

		self.push_array(args)?;
		self.push_integer(flags.as_byte() as i64);
		self.push_string(method)?;
		self.push_hash160(contract)?;
		Ok(self.syscall(interop::CONTRACT_CALL))
	}

	/// Appends a NEP-17 `transfer` call.
	pub fn nep17_transfer(
		&mut self,
		token: &Hash160,
		from: &Hash160,
		to: &Hash160,
		amount: i64,
		data: Option<ContractParameter>,
	) -> Result<&mut Self> {
		if amount < 0 {
			return Err(Error::InvalidData(format!(
				"transfer amount must not be negative: {}",
				amount
			)));
		}

		let args = alloc::vec![
			ContractParameter::Hash160(*from),
			ContractParameter::Hash160(*to),
			ContractParameter::Integer(amount),
			data.unwrap_or(ContractParameter::Null),
		];

		self.contract_call(token, "transfer", &args, CallFlags::All)
	}

	/// Current script length, in bytes.
	pub fn len(&self) -> usize {
		self.writer.len()
	}

	/// Whether nothing has been emitted yet.
	pub fn is_empty(&self) -> bool {
		self.writer.is_empty()
	}

	/// Consumes the builder and returns the script.
	pub fn into_bytes(self) -> Vec<u8> {
		self.writer.into_bytes()
	}

	/// Borrows the script built so far.
	pub fn as_bytes(&self) -> &[u8] {
		self.writer.as_bytes()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use alloc::vec;

	const GAS_TOKEN: &str = "0xd2a4cff31913016155e38e474a2c06d08be276cf";
	const ACCOUNT: &str = "0x69ecca587293047be4c59159bf8bc399985c160d";

	fn gas() -> Hash160 {
		Hash160::from_hex(GAS_TOKEN).unwrap()
	}

	fn account() -> Hash160 {
		Hash160::from_hex(ACCOUNT).unwrap()
	}

	#[test]
	fn test_push_small_integers_use_single_byte() {
		for value in -1..=16i64 {
			let mut builder = ScriptBuilder::new();
			builder.push_integer(value);
			assert_eq!(builder.len(), 1, "push {} should be one byte", value);
		}
	}

	#[test]
	fn test_push_integer_selects_narrowest_form() {
		let cases: &[(i64, &[u8])] = &[
			(0, &[0x10]),
			(16, &[0x20]),
			(-1, &[0x0F]),
			(17, &[0x00, 17]),                          // PushInt8
			(-2, &[0x00, 0xFE]),                        // PushInt8, two's complement
			(1000, &[0x01, 0xE8, 0x03]),                // PushInt16
			(100_000, &[0x02, 0xA0, 0x86, 0x01, 0x00]), // PushInt32
		];

		for (value, expected) in cases {
			let mut builder = ScriptBuilder::new();
			builder.push_integer(*value);
			assert_eq!(builder.as_bytes(), *expected, "push {}", value);
		}
	}

	#[test]
	fn test_push_integer_i64_uses_8_byte_form() {
		let mut builder = ScriptBuilder::new();
		builder.push_integer(i64::MAX);
		assert_eq!(builder.as_bytes()[0], OpCode::PushInt64.as_byte());
		assert_eq!(&builder.as_bytes()[1..], &i64::MAX.to_le_bytes());
	}

	#[test]
	fn test_push_bool() {
		let mut builder = ScriptBuilder::new();
		builder.push_bool(true).push_bool(false);
		assert_eq!(builder.as_bytes(), &[0x08, 0x09]);
	}

	#[test]
	fn test_push_null() {
		let mut builder = ScriptBuilder::new();
		builder.push_null();
		assert_eq!(builder.as_bytes(), &[0x0B]);
	}

	#[test]
	fn test_push_data_uses_pushdata1_for_small_payloads() {
		let mut builder = ScriptBuilder::new();
		builder.push_data(&[0xAA, 0xBB]).unwrap();
		assert_eq!(builder.as_bytes(), &[0x0C, 0x02, 0xAA, 0xBB]);
	}

	#[test]
	fn test_push_data_uses_pushdata2_above_255_bytes() {
		let payload = vec![0u8; 300];
		let mut builder = ScriptBuilder::new();
		builder.push_data(&payload).unwrap();

		assert_eq!(builder.as_bytes()[0], OpCode::PushData2.as_byte());
		assert_eq!(&builder.as_bytes()[1..3], &300u16.to_le_bytes());
		assert_eq!(builder.len(), 3 + 300);
	}

	#[test]
	fn test_push_data_rejects_oversized_payload() {
		let payload = vec![0u8; MAX_SCRIPT_SIZE + 1];
		let mut builder = ScriptBuilder::new();
		assert!(matches!(
			builder.push_data(&payload),
			Err(Error::LengthLimitExceeded { max: MAX_SCRIPT_SIZE, .. })
		));
	}

	#[test]
	fn test_push_string_is_utf8_bytes() {
		let mut builder = ScriptBuilder::new();
		builder.push_string("transfer").unwrap();
		assert_eq!(builder.as_bytes()[0], OpCode::PushData1.as_byte());
		assert_eq!(builder.as_bytes()[1], 8);
		assert_eq!(&builder.as_bytes()[2..], b"transfer");
	}

	#[test]
	fn test_push_hash160_uses_wire_order() {
		let mut builder = ScriptBuilder::new();
		builder.push_hash160(&gas()).unwrap();

		assert_eq!(builder.as_bytes()[0], OpCode::PushData1.as_byte());
		assert_eq!(builder.as_bytes()[1], 20);
		assert_eq!(&builder.as_bytes()[2..], gas().as_bytes());
	}

	#[test]
	fn test_empty_array_uses_newarray0() {
		let mut builder = ScriptBuilder::new();
		builder.push_array(&[]).unwrap();
		assert_eq!(builder.as_bytes(), &[OpCode::NewArray0.as_byte()]);
	}

	#[test]
	fn test_array_pushes_items_in_reverse_then_packs() {
		let mut builder = ScriptBuilder::new();
		builder
			.push_array(&[ContractParameter::Integer(1), ContractParameter::Integer(2)])
			.unwrap();

		// Reverse order: push 2, push 1, push count, PACK
		assert_eq!(builder.as_bytes(), &[0x12, 0x11, 0x12, 0xC0]);
	}

	#[test]
	fn test_nested_array() {
		let mut builder = ScriptBuilder::new();
		builder
			.push_array(&[ContractParameter::Array(vec![ContractParameter::Integer(1)])])
			.unwrap();

		// Inner: push 1, push 1, PACK. Outer: push 1, PACK.
		assert_eq!(builder.as_bytes(), &[0x11, 0x11, 0xC0, 0x11, 0xC0]);
	}

	#[test]
	fn test_syscall_emits_descriptor() {
		let mut builder = ScriptBuilder::new();
		builder.syscall(interop::CONTRACT_CALL);
		assert_eq!(builder.as_bytes(), &[0x41, 0x62, 0x7d, 0x5b, 0x52]);
	}

	#[test]
	fn test_contract_call_structure() {
		let mut builder = ScriptBuilder::new();
		builder.contract_call(&gas(), "symbol", &[], CallFlags::ReadOnly).unwrap();

		let script = builder.as_bytes();

		// NEWARRAY0 (no args)
		assert_eq!(script[0], OpCode::NewArray0.as_byte());
		// call flags 0x05 pushed as a small int (0x10 + 5)
		assert_eq!(script[1], 0x15);
		// method name
		assert_eq!(script[2], OpCode::PushData1.as_byte());
		assert_eq!(script[3], 6);
		assert_eq!(&script[4..10], b"symbol");
		// contract hash in wire order
		assert_eq!(script[10], OpCode::PushData1.as_byte());
		assert_eq!(script[11], 20);
		assert_eq!(&script[12..32], gas().as_bytes());
		// syscall
		assert_eq!(script[32], OpCode::Syscall.as_byte());
		assert_eq!(&script[33..37], &interop::CONTRACT_CALL);
		assert_eq!(script.len(), 37);
	}

	#[test]
	fn test_contract_call_with_arguments() {
		let mut builder = ScriptBuilder::new();
		builder
			.contract_call(
				&gas(),
				"balanceOf",
				&[ContractParameter::Hash160(account())],
				CallFlags::ReadOnly,
			)
			.unwrap();

		let script = builder.as_bytes();
		// Argument pushed first, then packed into a 1-element array.
		assert_eq!(script[0], OpCode::PushData1.as_byte());
		assert_eq!(script[1], 20);
		assert_eq!(&script[2..22], account().as_bytes());
		assert_eq!(script[22], 0x11); // push 1 (array length)
		assert_eq!(script[23], OpCode::Pack.as_byte());
	}

	#[test]
	fn test_contract_call_rejects_empty_method() {
		let mut builder = ScriptBuilder::new();
		assert!(matches!(
			builder.contract_call(&gas(), "", &[], CallFlags::All),
			Err(Error::InvalidData(_))
		));
	}

	#[test]
	fn test_nep17_transfer_builds_four_argument_call() {
		let mut builder = ScriptBuilder::new();
		builder.nep17_transfer(&gas(), &account(), &gas(), 100_000_000, None).unwrap();

		let script = builder.as_bytes();

		// Args are pushed in reverse: data(null), amount, to, from
		assert_eq!(script[0], OpCode::PushNull.as_byte());
		// amount 100_000_000 needs PushInt32
		assert_eq!(script[1], OpCode::PushInt32.as_byte());
		assert_eq!(&script[2..6], &100_000_000i32.to_le_bytes());

		// The call ends with the contract-call syscall.
		assert_eq!(script[script.len() - 5], OpCode::Syscall.as_byte());
		assert_eq!(&script[script.len() - 4..], &interop::CONTRACT_CALL);
	}

	#[test]
	fn test_nep17_transfer_rejects_negative_amount() {
		let mut builder = ScriptBuilder::new();
		assert!(matches!(
			builder.nep17_transfer(&gas(), &account(), &gas(), -1, None),
			Err(Error::InvalidData(_))
		));
	}

	#[test]
	fn test_nep17_transfer_accepts_custom_data() {
		let mut builder = ScriptBuilder::new();
		builder
			.nep17_transfer(
				&gas(),
				&account(),
				&gas(),
				1,
				Some(ContractParameter::String("memo".into())),
			)
			.unwrap();

		// data is the first argument pushed (reverse order)
		let script = builder.as_bytes();
		assert_eq!(script[0], OpCode::PushData1.as_byte());
		assert_eq!(script[1], 4);
		assert_eq!(&script[2..6], b"memo");
	}

	#[test]
	fn test_builder_is_chainable() {
		let mut builder = ScriptBuilder::new();
		builder.push_integer(1).push_bool(true).push_null();
		assert_eq!(builder.as_bytes(), &[0x11, 0x08, 0x0B]);
	}

	#[test]
	fn test_len_and_is_empty() {
		let mut builder = ScriptBuilder::new();
		assert!(builder.is_empty());
		assert_eq!(builder.len(), 0);

		builder.push_integer(1);
		assert!(!builder.is_empty());
		assert_eq!(builder.len(), 1);
	}

	#[test]
	fn test_all_parameter_kinds_round_through_push_parameter() {
		let params = vec![
			ContractParameter::Boolean(true),
			ContractParameter::Integer(42),
			ContractParameter::ByteArray(vec![1, 2, 3]),
			ContractParameter::String("neo".into()),
			ContractParameter::Hash160(gas()),
			ContractParameter::Null,
			ContractParameter::Array(vec![ContractParameter::Integer(1)]),
		];

		for param in &params {
			let mut builder = ScriptBuilder::new();
			builder.push_parameter(param).unwrap();
			assert!(!builder.is_empty(), "{:?} produced no bytes", param);
		}
	}
}
