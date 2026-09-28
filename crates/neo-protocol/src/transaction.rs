//! Neo N3 transactions

use crate::{
	attribute::TransactionAttribute,
	constants::{MAX_TRANSACTION_ATTRIBUTES, MAX_TRANSACTION_SIZE},
	errors::{Error, Result},
	serialization::{Reader, Serializable, Writer},
	signer::Signer,
	witness::Witness,
};
use alloc::{format, vec::Vec};
use neo_primitives::{crypto, Hash256, KeyPair};
use serde::{Deserialize, Serialize};

/// Current transaction version.
pub const CURRENT_VERSION: u8 = 0;

/// A Neo N3 transaction.
///
/// # Signing
///
/// The transaction hash and the signed message both cover the *unsigned* form:
/// everything except the witnesses. This is what lets a witness commit to a
/// transaction without committing to itself. Use
/// [`Transaction::unsigned_bytes`] for the signing payload and
/// [`Transaction::hash`] for the identifier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
	/// Format version; currently always 0.
	pub version: u8,
	/// Random value making otherwise identical transactions distinct.
	pub nonce: u32,
	/// GAS paid for execution, in fractions (1 GAS = 100_000_000).
	pub system_fee: i64,
	/// GAS paid for size and verification, in fractions.
	pub network_fee: i64,
	/// Last block height at which this transaction may be included.
	pub valid_until_block: u32,
	/// Accounts authorizing the transaction; the first pays the fees.
	pub signers: Vec<Signer>,
	/// Optional attributes.
	#[serde(default)]
	pub attributes: Vec<TransactionAttribute>,
	/// The script the VM executes.
	pub script: Vec<u8>,
	/// Witnesses proving each signer authorized this transaction.
	#[serde(default)]
	pub witnesses: Vec<Witness>,
}

impl Transaction {
	/// Creates an unsigned transaction with no attributes or witnesses.
	pub fn new(nonce: u32, valid_until_block: u32, script: Vec<u8>, signers: Vec<Signer>) -> Self {
		Self {
			version: CURRENT_VERSION,
			nonce,
			system_fee: 0,
			network_fee: 0,
			valid_until_block,
			signers,
			attributes: Vec::new(),
			script,
			witnesses: Vec::new(),
		}
	}

	/// The account paying the fees, i.e. the first signer.
	pub fn sender(&self) -> Option<&Signer> {
		self.signers.first()
	}

	/// Total fee in GAS fractions.
	pub fn total_fee(&self) -> i64 {
		self.system_fee.saturating_add(self.network_fee)
	}

	/// Encodes the transaction without witnesses.
	///
	/// This is the message that signatures cover and the preimage of
	/// [`Transaction::hash`].
	pub fn unsigned_bytes(&self) -> Vec<u8> {
		let mut writer = Writer::new();
		self.serialize_unsigned(&mut writer);
		writer.into_bytes()
	}

	fn serialize_unsigned(&self, writer: &mut Writer) {
		writer.write_u8(self.version);
		writer.write_u32(self.nonce);
		writer.write_i64(self.system_fee);
		writer.write_i64(self.network_fee);
		writer.write_u32(self.valid_until_block);
		writer.write_serializable_list(&self.signers);
		writer.write_serializable_list(&self.attributes);
		writer.write_var_bytes(&self.script);
	}

	/// The transaction hash: `SHA256(SHA256(unsigned_bytes))`.
	///
	/// Witnesses are excluded, so the hash is stable across signing.
	pub fn hash(&self) -> Hash256 {
		Hash256::from_bytes(crypto::hash256(&self.unsigned_bytes()))
	}

	/// Signs the transaction with `key_pair` and appends the witness.
	///
	/// Witnesses must be appended in signer order; this is the caller's
	/// responsibility when signing a multi-signer transaction.
	pub fn sign(&mut self, key_pair: &KeyPair) {
		let signature = key_pair.sign(&self.unsigned_bytes());
		self.witnesses.push(Witness::from_signature(&signature, key_pair.public_key()));
	}

	/// Verifies that every witness has a valid signature over this
	/// transaction's unsigned bytes and matches its signer's account.
	///
	/// Returns `false` unless the witness count matches the signer count,
	/// since a missing witness leaves a signer unauthorized.
	pub fn verify_witnesses(&self) -> bool {
		if self.witnesses.len() != self.signers.len() {
			return false;
		}

		let message = self.unsigned_bytes();

		self.signers.iter().zip(&self.witnesses).all(|(signer, witness)| {
			match witness.script_hash() {
				Some(hash) if hash == signer.account => {
					verify_single_sig_witness(witness, &message)
				},
				// An empty verification script cannot authorize an account.
				_ => false,
			}
		})
	}

	/// Checks structural limits: size, attribute count, signer presence and
	/// duplicate signers.
	pub fn validate(&self) -> Result<()> {
		if self.signers.is_empty() {
			return Err(Error::InvalidData("transaction must have at least one signer".into()));
		}

		if self.attributes.len() > MAX_TRANSACTION_ATTRIBUTES {
			return Err(Error::LengthLimitExceeded {
				len: self.attributes.len() as u64,
				max: MAX_TRANSACTION_ATTRIBUTES,
			});
		}

		// A repeated signer would make witness-to-signer mapping ambiguous.
		for (i, signer) in self.signers.iter().enumerate() {
			if self.signers[..i].iter().any(|s| s.account == signer.account) {
				return Err(Error::InvalidData(format!("duplicate signer: {}", signer.account)));
			}
			signer.validate()?;
		}

		let size = self.size();
		if size > MAX_TRANSACTION_SIZE {
			return Err(Error::LengthLimitExceeded { len: size as u64, max: MAX_TRANSACTION_SIZE });
		}

		if self.script.is_empty() {
			return Err(Error::InvalidData("transaction script must not be empty".into()));
		}

		Ok(())
	}
}

/// Verifies a witness whose verification script is a single-signature
/// CheckSig script. Returns `false` for any other script shape, since this
/// crate does not evaluate arbitrary VM scripts.
fn verify_single_sig_witness(witness: &Witness, message: &[u8]) -> bool {
	let Some((public_key, signature)) = parse_single_sig_witness(witness) else {
		return false;
	};
	public_key.verify(message, &signature)
}

/// Extracts the public key and signature from a single-signature witness.
fn parse_single_sig_witness(
	witness: &Witness,
) -> Option<(neo_primitives::PublicKey, neo_primitives::Signature)> {
	// Verification script: PUSHDATA1 33 <pubkey> SYSCALL <4-byte interop>
	let script = &witness.verification_script;
	if script.len() != 40 || script[0] != 0x0c || script[1] != 33 || script[35] != 0x41 {
		return None;
	}
	let public_key = neo_primitives::PublicKey::from_bytes(&script[2..35]).ok()?;

	// Invocation script: PUSHDATA1 64 <signature>
	let invocation = &witness.invocation_script;
	if invocation.len() != 66 || invocation[0] != 0x0c || invocation[1] != 64 {
		return None;
	}
	let signature = neo_primitives::Signature::from_bytes(&invocation[2..]).ok()?;

	Some((public_key, signature))
}

impl Serializable for Transaction {
	fn serialize(&self, writer: &mut Writer) {
		self.serialize_unsigned(writer);
		writer.write_serializable_list(&self.witnesses);
	}

	fn deserialize(reader: &mut Reader<'_>) -> Result<Self> {
		let version = reader.read_u8()?;
		let nonce = reader.read_u32()?;
		let system_fee = reader.read_i64()?;
		let network_fee = reader.read_i64()?;
		let valid_until_block = reader.read_u32()?;
		let signers = reader.read_serializable_list::<Signer>(MAX_TRANSACTION_ATTRIBUTES)?;
		let attributes =
			reader.read_serializable_list::<TransactionAttribute>(MAX_TRANSACTION_ATTRIBUTES)?;
		let script = reader.read_var_bytes(MAX_TRANSACTION_SIZE)?.to_vec();
		let witnesses = reader.read_serializable_list::<Witness>(MAX_TRANSACTION_ATTRIBUTES)?;

		if system_fee < 0 {
			return Err(Error::InvalidData(format!("negative system fee: {}", system_fee)));
		}
		if network_fee < 0 {
			return Err(Error::InvalidData(format!("negative network fee: {}", network_fee)));
		}

		Ok(Self {
			version,
			nonce,
			system_fee,
			network_fee,
			valid_until_block,
			signers,
			attributes,
			script,
			witnesses,
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{signer::WitnessScope, witness::Witness};
	use alloc::vec;

	const PRIVATE_KEY_HEX: &str =
		"84180ac9d6eb6fba207ea4ef9d2200102d1ebeb4b9c07e2c6a738a42742e27a5";
	const OTHER_KEY_HEX: &str = "0000000000000000000000000000000000000000000000000000000000000042";

	fn key_pair() -> KeyPair {
		KeyPair::from_bytes(&hex::decode(PRIVATE_KEY_HEX).unwrap()).unwrap()
	}

	fn other_key_pair() -> KeyPair {
		KeyPair::from_bytes(&hex::decode(OTHER_KEY_HEX).unwrap()).unwrap()
	}

	/// A minimal valid transaction signed by nobody yet.
	fn unsigned_tx() -> Transaction {
		let signer = Signer::called_by_entry(key_pair().script_hash());
		Transaction::new(42, 1_000, vec![0x51], vec![signer])
	}

	#[test]
	fn test_new_transaction_defaults() {
		let tx = unsigned_tx();
		assert_eq!(tx.version, CURRENT_VERSION);
		assert_eq!(tx.nonce, 42);
		assert_eq!(tx.valid_until_block, 1_000);
		assert_eq!(tx.system_fee, 0);
		assert_eq!(tx.network_fee, 0);
		assert!(tx.attributes.is_empty());
		assert!(tx.witnesses.is_empty());
	}

	#[test]
	fn test_sender_is_first_signer() {
		let mut tx = unsigned_tx();
		let second = Signer::called_by_entry(other_key_pair().script_hash());
		tx.signers.push(second);

		assert_eq!(tx.sender().unwrap().account, key_pair().script_hash());
	}

	#[test]
	fn test_sender_is_none_without_signers() {
		let mut tx = unsigned_tx();
		tx.signers.clear();
		assert!(tx.sender().is_none());
	}

	#[test]
	fn test_total_fee_sums_components() {
		let mut tx = unsigned_tx();
		tx.system_fee = 1_000_000;
		tx.network_fee = 500_000;
		assert_eq!(tx.total_fee(), 1_500_000);
	}

	#[test]
	fn test_total_fee_saturates_instead_of_overflowing() {
		let mut tx = unsigned_tx();
		tx.system_fee = i64::MAX;
		tx.network_fee = 1;
		assert_eq!(tx.total_fee(), i64::MAX);
	}

	#[test]
	fn test_field_order_matches_protocol() {
		let tx = unsigned_tx();
		let bytes = tx.unsigned_bytes();

		// version | nonce | sysfee | netfee | validuntilblock | ...
		assert_eq!(bytes[0], 0);
		assert_eq!(&bytes[1..5], &42u32.to_le_bytes());
		assert_eq!(&bytes[5..13], &0i64.to_le_bytes());
		assert_eq!(&bytes[13..21], &0i64.to_le_bytes());
		assert_eq!(&bytes[21..25], &1_000u32.to_le_bytes());
	}

	#[test]
	fn test_unsigned_bytes_exclude_witnesses() {
		let mut tx = unsigned_tx();
		let before = tx.unsigned_bytes();

		tx.sign(&key_pair());
		let after = tx.unsigned_bytes();

		assert_eq!(before, after, "signing must not change the signed payload");
		assert!(tx.to_bytes().len() > after.len(), "full encoding includes witnesses");
	}

	#[test]
	fn test_hash_is_stable_across_signing() {
		let mut tx = unsigned_tx();
		let hash_before = tx.hash();

		tx.sign(&key_pair());

		assert_eq!(tx.hash(), hash_before, "witnesses must not affect the hash");
	}

	#[test]
	fn test_hash_is_double_sha256_of_unsigned_bytes() {
		let tx = unsigned_tx();
		let expected = crypto::hash256(&tx.unsigned_bytes());
		assert_eq!(tx.hash().as_bytes(), &expected);
	}

	#[test]
	fn test_hash_changes_with_nonce() {
		let tx1 = unsigned_tx();
		let mut tx2 = unsigned_tx();
		tx2.nonce = 43;
		assert_ne!(tx1.hash(), tx2.hash());
	}

	#[test]
	fn test_sign_appends_verifiable_witness() {
		let mut tx = unsigned_tx();
		tx.sign(&key_pair());

		assert_eq!(tx.witnesses.len(), 1);
		assert!(tx.verify_witnesses());
	}

	#[test]
	fn test_verify_fails_without_witnesses() {
		let tx = unsigned_tx();
		assert!(!tx.verify_witnesses());
	}

	#[test]
	fn test_verify_fails_with_wrong_key() {
		let mut tx = unsigned_tx();
		// Sign with a key that is not the declared signer.
		tx.sign(&other_key_pair());
		assert!(!tx.verify_witnesses());
	}

	#[test]
	fn test_verify_fails_when_payload_changes_after_signing() {
		let mut tx = unsigned_tx();
		tx.sign(&key_pair());
		assert!(tx.verify_witnesses());

		// Raising the fee after signing must invalidate the signature.
		tx.system_fee = 999;
		assert!(!tx.verify_witnesses());
	}

	#[test]
	fn test_verify_fails_on_tampered_signature() {
		let mut tx = unsigned_tx();
		tx.sign(&key_pair());

		let last = tx.witnesses[0].invocation_script.len() - 1;
		tx.witnesses[0].invocation_script[last] ^= 0xFF;
		assert!(!tx.verify_witnesses());
	}

	#[test]
	fn test_verify_fails_on_witness_count_mismatch() {
		let mut tx = unsigned_tx();
		tx.signers.push(Signer::called_by_entry(other_key_pair().script_hash()));
		tx.sign(&key_pair());

		// Two signers, one witness.
		assert!(!tx.verify_witnesses());
	}

	#[test]
	fn test_verify_rejects_empty_witness() {
		let mut tx = unsigned_tx();
		tx.witnesses.push(Witness::empty());
		assert!(!tx.verify_witnesses());
	}

	#[test]
	fn test_multi_signer_verification_in_order() {
		let first = key_pair();
		let second = other_key_pair();

		let mut tx = Transaction::new(
			1,
			100,
			vec![0x51],
			vec![
				Signer::called_by_entry(first.script_hash()),
				Signer::called_by_entry(second.script_hash()),
			],
		);

		tx.sign(&first);
		tx.sign(&second);

		assert!(tx.verify_witnesses());
	}

	#[test]
	fn test_verify_fails_when_witness_order_is_swapped() {
		let first = key_pair();
		let second = other_key_pair();

		let mut tx = Transaction::new(
			1,
			100,
			vec![0x51],
			vec![
				Signer::called_by_entry(first.script_hash()),
				Signer::called_by_entry(second.script_hash()),
			],
		);

		// Sign in reverse order: each witness is individually valid but does
		// not line up with its signer.
		tx.sign(&second);
		tx.sign(&first);

		assert!(!tx.verify_witnesses());
	}

	#[test]
	fn test_serialization_roundtrip_unsigned() {
		let tx = unsigned_tx();
		let bytes = tx.to_bytes();
		assert_eq!(Transaction::from_bytes(&bytes).unwrap(), tx);
	}

	#[test]
	fn test_serialization_roundtrip_signed() {
		let mut tx = unsigned_tx();
		tx.system_fee = 1_000_000;
		tx.network_fee = 200_000;
		tx.attributes.push(TransactionAttribute::HighPriority);
		tx.sign(&key_pair());

		let decoded = Transaction::from_bytes(&tx.to_bytes()).unwrap();
		assert_eq!(decoded, tx);
		assert_eq!(decoded.hash(), tx.hash());
		assert!(decoded.verify_witnesses());
	}

	#[test]
	fn test_deserialize_rejects_negative_system_fee() {
		let mut tx = unsigned_tx();
		tx.system_fee = -1;
		assert!(matches!(Transaction::from_bytes(&tx.to_bytes()), Err(Error::InvalidData(_))));
	}

	#[test]
	fn test_deserialize_rejects_negative_network_fee() {
		let mut tx = unsigned_tx();
		tx.network_fee = -1;
		assert!(matches!(Transaction::from_bytes(&tx.to_bytes()), Err(Error::InvalidData(_))));
	}

	#[test]
	fn test_deserialize_rejects_trailing_bytes() {
		let mut bytes = unsigned_tx().to_bytes();
		bytes.push(0xFF);
		assert!(matches!(
			Transaction::from_bytes(&bytes),
			Err(Error::TrailingBytes { remaining: 1 })
		));
	}

	#[test]
	fn test_deserialize_rejects_truncated_input() {
		let bytes = unsigned_tx().to_bytes();
		assert!(matches!(Transaction::from_bytes(&bytes[..12]), Err(Error::UnexpectedEof { .. })));
	}

	#[test]
	fn test_deserialize_rejects_too_many_attributes() {
		let mut writer = Writer::new();
		writer.write_u8(0);
		writer.write_u32(1);
		writer.write_i64(0);
		writer.write_i64(0);
		writer.write_u32(100);
		writer.write_serializable_list(&[Signer::called_by_entry(key_pair().script_hash())]);
		// Claim more attributes than the protocol allows.
		writer.write_var_int((MAX_TRANSACTION_ATTRIBUTES + 1) as u64);

		assert!(matches!(
			Transaction::from_bytes(writer.as_bytes()),
			Err(Error::LengthLimitExceeded { max: MAX_TRANSACTION_ATTRIBUTES, .. })
		));
	}

	#[test]
	fn test_validate_accepts_well_formed_transaction() {
		assert!(unsigned_tx().validate().is_ok());
	}

	#[test]
	fn test_validate_rejects_missing_signers() {
		let mut tx = unsigned_tx();
		tx.signers.clear();
		assert!(matches!(tx.validate(), Err(Error::InvalidData(_))));
	}

	#[test]
	fn test_validate_rejects_empty_script() {
		let mut tx = unsigned_tx();
		tx.script.clear();
		assert!(matches!(tx.validate(), Err(Error::InvalidData(_))));
	}

	#[test]
	fn test_validate_rejects_duplicate_signers() {
		let mut tx = unsigned_tx();
		tx.signers.push(Signer::global(key_pair().script_hash()));
		assert!(matches!(tx.validate(), Err(Error::InvalidData(_))));
	}

	#[test]
	fn test_validate_rejects_too_many_attributes() {
		let mut tx = unsigned_tx();
		tx.attributes = vec![TransactionAttribute::HighPriority; MAX_TRANSACTION_ATTRIBUTES + 1];
		assert!(matches!(
			tx.validate(),
			Err(Error::LengthLimitExceeded { max: MAX_TRANSACTION_ATTRIBUTES, .. })
		));
	}

	#[test]
	fn test_validate_rejects_oversized_transaction() {
		let mut tx = unsigned_tx();
		tx.script = vec![0x51; MAX_TRANSACTION_SIZE];
		assert!(matches!(
			tx.validate(),
			Err(Error::LengthLimitExceeded { max: MAX_TRANSACTION_SIZE, .. })
		));
	}

	#[test]
	fn test_validate_propagates_signer_errors() {
		let mut tx = unsigned_tx();
		tx.signers[0].scopes = vec![WitnessScope::Global, WitnessScope::CalledByEntry];
		assert!(matches!(tx.validate(), Err(Error::InvalidData(_))));
	}

	#[test]
	fn test_size_matches_encoding() {
		let mut tx = unsigned_tx();
		tx.sign(&key_pair());
		assert_eq!(tx.size(), tx.to_bytes().len());
	}

	#[test]
	fn test_attributes_are_covered_by_the_hash() {
		let tx = unsigned_tx();
		let mut with_attr = unsigned_tx();
		with_attr.attributes.push(TransactionAttribute::HighPriority);

		assert_ne!(tx.hash(), with_attr.hash());
	}

	#[test]
	fn test_signer_scope_is_covered_by_the_hash() {
		let tx = unsigned_tx();
		let mut global = unsigned_tx();
		global.signers[0] = Signer::global(key_pair().script_hash());

		assert_ne!(tx.hash(), global.hash());
	}

	#[test]
	fn test_serde_roundtrip() {
		let mut tx = unsigned_tx();
		tx.sign(&key_pair());

		let json = serde_json::to_string(&tx).unwrap();
		let back: Transaction = serde_json::from_str(&json).unwrap();
		assert_eq!(tx, back);
	}
}
