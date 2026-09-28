//! Witness data: the scripts that authorize a transaction

use crate::{
	constants::{MAX_INVOCATION_SCRIPT_SIZE, MAX_VERIFICATION_SCRIPT_SIZE},
	errors::Result,
	serialization::{Reader, Serializable, Writer},
};
use alloc::vec::Vec;
use neo_primitives::{Hash160, PublicKey, Signature};
use serde::{Deserialize, Serialize};

/// A witness: the invocation script that pushes arguments and the verification
/// script that checks them.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Witness {
	/// Script that pushes signatures or other arguments onto the stack.
	pub invocation_script: Vec<u8>,
	/// Script that verifies those arguments.
	pub verification_script: Vec<u8>,
}

impl Witness {
	/// Creates a witness from raw scripts.
	pub fn new(invocation_script: Vec<u8>, verification_script: Vec<u8>) -> Self {
		Self { invocation_script, verification_script }
	}

	/// Builds a single-signature witness for `signature` and `public_key`.
	///
	/// The invocation script is `PUSHDATA1 64 <signature>`; the verification
	/// script is the standard CheckSig script for the key.
	pub fn from_signature(signature: &Signature, public_key: &PublicKey) -> Self {
		let sig_bytes = signature.to_bytes();

		let mut invocation = Vec::with_capacity(2 + sig_bytes.len());
		invocation.push(0x0c); // PUSHDATA1
		invocation.push(sig_bytes.len() as u8);
		invocation.extend_from_slice(&sig_bytes);

		Self {
			invocation_script: invocation,
			verification_script: public_key.to_verification_script().as_bytes().to_vec(),
		}
	}

	/// An empty witness, used for contract signers that need no scripts.
	pub fn empty() -> Self {
		Self::default()
	}

	/// The script hash of the verification script, i.e. the account this
	/// witness authorizes.
	///
	/// Returns `None` for a witness with no verification script, which cannot
	/// be attributed to an account on its own.
	pub fn script_hash(&self) -> Option<Hash160> {
		if self.verification_script.is_empty() {
			None
		} else {
			Some(Hash160::from_script(&self.verification_script))
		}
	}
}

impl Serializable for Witness {
	fn serialize(&self, writer: &mut Writer) {
		writer.write_var_bytes(&self.invocation_script);
		writer.write_var_bytes(&self.verification_script);
	}

	fn deserialize(reader: &mut Reader<'_>) -> Result<Self> {
		let invocation_script = reader.read_var_bytes(MAX_INVOCATION_SCRIPT_SIZE)?.to_vec();
		let verification_script = reader.read_var_bytes(MAX_VERIFICATION_SCRIPT_SIZE)?.to_vec();
		Ok(Self { invocation_script, verification_script })
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::errors::Error;
	use neo_primitives::PrivateKey;

	const PRIVATE_KEY_HEX: &str =
		"84180ac9d6eb6fba207ea4ef9d2200102d1ebeb4b9c07e2c6a738a42742e27a5";
	const SCRIPT_HASH: &str = "0x69ecca587293047be4c59159bf8bc399985c160d";

	fn test_key() -> PrivateKey {
		PrivateKey::from_bytes(&hex::decode(PRIVATE_KEY_HEX).unwrap()).unwrap()
	}

	#[test]
	fn test_witness_roundtrip() {
		let witness = Witness::new(vec![0x01, 0x02], vec![0x03, 0x04, 0x05]);
		let bytes = witness.to_bytes();
		assert_eq!(Witness::from_bytes(&bytes).unwrap(), witness);
	}

	#[test]
	fn test_witness_encoding_is_length_prefixed() {
		let witness = Witness::new(vec![0xAA], vec![0xBB, 0xCC]);
		// 0x01 AA | 0x02 BB CC
		assert_eq!(witness.to_bytes(), vec![0x01, 0xAA, 0x02, 0xBB, 0xCC]);
	}

	#[test]
	fn test_empty_witness_roundtrip() {
		let witness = Witness::empty();
		assert_eq!(witness.to_bytes(), vec![0x00, 0x00]);
		assert_eq!(Witness::from_bytes(&[0x00, 0x00]).unwrap(), witness);
	}

	#[test]
	fn test_witness_from_signature_builds_neo_scripts() {
		let key = test_key();
		let signature = key.sign(b"payload");
		let witness = Witness::from_signature(&signature, &key.public_key());

		// PUSHDATA1 + length + 64-byte signature
		assert_eq!(witness.invocation_script.len(), 66);
		assert_eq!(witness.invocation_script[0], 0x0c);
		assert_eq!(witness.invocation_script[1], 64);
		assert_eq!(&witness.invocation_script[2..], &signature.to_bytes());

		// Verification script matches the key's CheckSig script
		assert_eq!(
			witness.verification_script,
			key.public_key().to_verification_script().as_bytes()
		);
	}

	#[test]
	fn test_witness_script_hash_matches_account() {
		let key = test_key();
		let witness = Witness::from_signature(&key.sign(b"x"), &key.public_key());
		assert_eq!(witness.script_hash().unwrap().to_hex(), SCRIPT_HASH);
	}

	#[test]
	fn test_empty_witness_has_no_script_hash() {
		assert!(Witness::empty().script_hash().is_none());
	}

	#[test]
	fn test_witness_rejects_oversized_invocation_script() {
		let mut writer = Writer::new();
		writer.write_var_bytes(&vec![0u8; MAX_INVOCATION_SCRIPT_SIZE + 1]);
		writer.write_var_bytes(&[]);

		assert!(matches!(
			Witness::from_bytes(writer.as_bytes()),
			Err(Error::LengthLimitExceeded { .. })
		));
	}

	#[test]
	fn test_witness_rejects_trailing_bytes() {
		let mut bytes = Witness::empty().to_bytes();
		bytes.push(0xFF);
		assert!(matches!(Witness::from_bytes(&bytes), Err(Error::TrailingBytes { remaining: 1 })));
	}

	#[test]
	fn test_witness_size_matches_encoding() {
		let witness = Witness::new(vec![0u8; 66], vec![0u8; 40]);
		assert_eq!(witness.size(), witness.to_bytes().len());
		// 1 + 66 + 1 + 40
		assert_eq!(witness.size(), 108);
	}

	#[test]
	fn test_serde_roundtrip() {
		let witness = Witness::new(vec![1, 2, 3], vec![4, 5]);
		let json = serde_json::to_string(&witness).unwrap();
		let back: Witness = serde_json::from_str(&json).unwrap();
		assert_eq!(witness, back);
	}
}
