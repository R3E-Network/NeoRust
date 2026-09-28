//! NeoVM opcodes
//!
//! Only the opcodes this crate emits or inspects are listed; the VM's full
//! instruction set is much larger.

/// A NeoVM instruction byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum OpCode {
	// --- Constants ---
	/// Push a 1-byte signed integer.
	PushInt8 = 0x00,
	/// Push a 2-byte signed integer.
	PushInt16 = 0x01,
	/// Push a 4-byte signed integer.
	PushInt32 = 0x02,
	/// Push an 8-byte signed integer.
	PushInt64 = 0x03,
	/// Push a 16-byte signed integer.
	PushInt128 = 0x04,
	/// Push a 32-byte signed integer.
	PushInt256 = 0x05,
	/// Push boolean `true`.
	PushTrue = 0x08,
	/// Push boolean `false`.
	PushFalse = 0x09,
	/// Push `null`.
	PushNull = 0x0B,
	/// Push bytes with a 1-byte length prefix.
	PushData1 = 0x0C,
	/// Push bytes with a 2-byte length prefix.
	PushData2 = 0x0D,
	/// Push bytes with a 4-byte length prefix.
	PushData4 = 0x0E,
	/// Push the integer -1.
	PushM1 = 0x0F,
	/// Push the integer 0.
	Push0 = 0x10,
	/// Push the integer 1.
	Push1 = 0x11,
	/// Push the integer 16 (the largest single-byte push).
	Push16 = 0x20,

	// --- Stack ---
	/// Reverse the top three stack items.
	Reverse3 = 0x53,

	// --- Control ---
	/// Invoke an interop service by 4-byte descriptor.
	Syscall = 0x41,

	// --- Compound types ---
	/// Pack the top `n` items into an array.
	Pack = 0xC0,
	/// Push an empty array.
	NewArray0 = 0xC2,
}

impl OpCode {
	/// The instruction byte.
	pub const fn as_byte(self) -> u8 {
		self as u8
	}

	/// The single-byte opcode that pushes `value`, for `-1..=16`.
	///
	/// Returns `None` outside that range; larger values need a `PushInt*`
	/// instruction with an operand.
	///
	/// `Push0` through `Push16` occupy the contiguous range `0x10..=0x20`, so
	/// the byte is computed rather than matched arm by arm.
	pub const fn push_small_int(value: i64) -> Option<u8> {
		match value {
			-1 => Some(Self::PushM1.as_byte()),
			0..=16 => Some(Self::Push0.as_byte() + value as u8),
			_ => None,
		}
	}
}

/// First 4 bytes of `SHA256(name)`: how the VM identifies an interop service.
pub mod interop {
	/// `System.Contract.Call`
	pub const CONTRACT_CALL: [u8; 4] = [0x62, 0x7d, 0x5b, 0x52];
	/// `System.Crypto.CheckSig`
	pub const CRYPTO_CHECK_SIG: [u8; 4] = [0x56, 0xe7, 0xb3, 0x27];
}

/// How a contract call may modify state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum CallFlags {
	/// No permissions.
	None = 0x00,
	/// May read state.
	ReadStates = 0x01,
	/// May write state.
	WriteStates = 0x02,
	/// May call other contracts.
	AllowCall = 0x04,
	/// May emit notifications.
	AllowNotify = 0x08,
	/// Read state and call other contracts.
	ReadOnly = 0x05,
	/// All permissions.
	All = 0x0F,
}

impl CallFlags {
	/// The flag byte.
	pub const fn as_byte(self) -> u8 {
		self as u8
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_opcode_byte_values() {
		assert_eq!(OpCode::PushData1.as_byte(), 0x0C);
		assert_eq!(OpCode::Push0.as_byte(), 0x10);
		assert_eq!(OpCode::Push1.as_byte(), 0x11);
		assert_eq!(OpCode::Push16.as_byte(), 0x20);
		assert_eq!(OpCode::PushM1.as_byte(), 0x0F);
		assert_eq!(OpCode::Syscall.as_byte(), 0x41);
		assert_eq!(OpCode::Pack.as_byte(), 0xC0);
		assert_eq!(OpCode::NewArray0.as_byte(), 0xC2);
	}

	#[test]
	fn test_push_small_int_covers_single_byte_range() {
		assert_eq!(OpCode::push_small_int(-1), Some(0x0F));
		assert_eq!(OpCode::push_small_int(0), Some(0x10));
		assert_eq!(OpCode::push_small_int(1), Some(0x11));
		assert_eq!(OpCode::push_small_int(16), Some(0x20));
	}

	#[test]
	fn test_push_small_int_is_contiguous() {
		// Every value in 0..=16 maps to a distinct byte in 0x10..=0x20.
		for value in 0..=16i64 {
			let byte = OpCode::push_small_int(value).unwrap();
			assert_eq!(byte, 0x10 + value as u8, "push {}", value);
		}
	}

	#[test]
	fn test_push_small_int_rejects_out_of_range() {
		assert_eq!(OpCode::push_small_int(17), None);
		assert_eq!(OpCode::push_small_int(-2), None);
		assert_eq!(OpCode::push_small_int(i64::MAX), None);
	}

	#[test]
	fn test_interop_hashes_match_sha256_prefixes() {
		// sha256("System.Contract.Call")[..4]
		assert_eq!(interop::CONTRACT_CALL, [0x62, 0x7d, 0x5b, 0x52]);
		// sha256("System.Crypto.CheckSig")[..4]
		assert_eq!(interop::CRYPTO_CHECK_SIG, [0x56, 0xe7, 0xb3, 0x27]);
	}

	#[test]
	fn test_call_flags_byte_values() {
		assert_eq!(CallFlags::None.as_byte(), 0x00);
		assert_eq!(CallFlags::ReadStates.as_byte(), 0x01);
		assert_eq!(CallFlags::ReadOnly.as_byte(), 0x05);
		assert_eq!(CallFlags::All.as_byte(), 0x0F);
	}
}
