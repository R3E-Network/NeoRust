//! Fluent builder for Neo N3 transactions
//!
//! [`Transaction`] fields are public, so a builder is not required — it exists
//! to make the common path (set fees, add signers, sign) read in order and to
//! run [`Transaction::validate`] before handing back a transaction.

use crate::{
	attribute::TransactionAttribute,
	errors::Result,
	signer::Signer,
	transaction::{Transaction, CURRENT_VERSION},
	witness::Witness,
};
use alloc::vec::Vec;
use neo_primitives::KeyPair;

/// Builds a [`Transaction`] step by step.
///
/// # Example
///
/// ```
/// use neo_protocol::{ScriptBuilder, Signer, TransactionBuilder, CallFlags};
/// use neo_primitives::{Hash160, KeyPair};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let key = KeyPair::random()?;
/// let gas = Hash160::from_hex("0xd2a4cff31913016155e38e474a2c06d08be276cf")?;
///
/// let mut script = ScriptBuilder::new();
/// script.contract_call(&gas, "symbol", &[], CallFlags::ReadOnly)?;
///
/// let tx = TransactionBuilder::new(script.into_bytes())
///     .nonce(42)
///     .valid_until_block(1_000)
///     .system_fee(1_000_000)
///     .signer(Signer::called_by_entry(key.script_hash()))
///     .sign_with(&key)
///     .build()?;
///
/// assert!(tx.verify_witnesses());
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct TransactionBuilder {
	transaction: Transaction,
	signing_keys: Vec<KeyPair>,
}

impl TransactionBuilder {
	/// Starts a builder for the given script.
	pub fn new(script: Vec<u8>) -> Self {
		Self {
			transaction: Transaction {
				version: CURRENT_VERSION,
				nonce: 0,
				system_fee: 0,
				network_fee: 0,
				valid_until_block: 0,
				signers: Vec::new(),
				attributes: Vec::new(),
				script,
				witnesses: Vec::new(),
			},
			signing_keys: Vec::new(),
		}
	}

	/// Sets the format version. Defaults to [`CURRENT_VERSION`].
	pub fn version(mut self, version: u8) -> Self {
		self.transaction.version = version;
		self
	}

	/// Sets the nonce.
	pub fn nonce(mut self, nonce: u32) -> Self {
		self.transaction.nonce = nonce;
		self
	}

	/// Sets the system fee, in GAS fractions (1 GAS = 100_000_000).
	pub fn system_fee(mut self, fee: i64) -> Self {
		self.transaction.system_fee = fee;
		self
	}

	/// Sets the network fee, in GAS fractions.
	pub fn network_fee(mut self, fee: i64) -> Self {
		self.transaction.network_fee = fee;
		self
	}

	/// Sets the last block height at which the transaction may be included.
	pub fn valid_until_block(mut self, block: u32) -> Self {
		self.transaction.valid_until_block = block;
		self
	}

	/// Sets the expiry as `current_height + offset`, saturating at [`u32::MAX`].
	pub fn valid_until_block_offset(mut self, current_height: u32, offset: u32) -> Self {
		self.transaction.valid_until_block = current_height.saturating_add(offset);
		self
	}

	/// Appends an attribute.
	pub fn attribute(mut self, attribute: TransactionAttribute) -> Self {
		self.transaction.attributes.push(attribute);
		self
	}

	/// Appends a signer. The first signer pays the fees.
	pub fn signer(mut self, signer: Signer) -> Self {
		self.transaction.signers.push(signer);
		self
	}

	/// Appends several signers, preserving order.
	pub fn signers(mut self, signers: impl IntoIterator<Item = Signer>) -> Self {
		self.transaction.signers.extend(signers);
		self
	}

	/// Appends a pre-built witness.
	///
	/// Witnesses must line up with signers by position; prefer
	/// [`TransactionBuilder::sign_with`] unless you are supplying a witness
	/// this crate cannot build (a multi-sig or contract witness, say).
	pub fn witness(mut self, witness: Witness) -> Self {
		self.transaction.witnesses.push(witness);
		self
	}

	/// Registers a key to sign with at [`build`](Self::build) time.
	///
	/// Signing is deferred so that fees and attributes set after this call are
	/// still covered by the signature. Keys sign in the order registered, so
	/// register them in signer order.
	pub fn sign_with(mut self, key: &KeyPair) -> Self {
		self.signing_keys.push(key.clone());
		self
	}

	/// The transaction as built so far, before deferred signing.
	pub fn peek(&self) -> &Transaction {
		&self.transaction
	}

	/// Builds the transaction without validating it.
	///
	/// Deferred keys still sign. Use this only when the caller validates
	/// separately, or deliberately wants an incomplete transaction.
	pub fn build_unchecked(mut self) -> Transaction {
		self.apply_signatures();
		self.transaction
	}

	/// Signs with any registered keys, then validates and returns the
	/// transaction.
	///
	/// Signatures are applied before validation so that size checks account for
	/// the witnesses.
	pub fn build(mut self) -> Result<Transaction> {
		self.apply_signatures();
		self.transaction.validate()?;
		Ok(self.transaction)
	}

	fn apply_signatures(&mut self) {
		// Take the keys first: `sign` borrows the transaction mutably.
		for key in core::mem::take(&mut self.signing_keys) {
			self.transaction.sign(&key);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{constants::MAX_TRANSACTION_SIZE, errors::Error, signer::WitnessScope};
	use alloc::vec;

	const PRIVATE_KEY_HEX: &str =
		"84180ac9d6eb6fba207ea4ef9d2200102d1ebeb4b9c07e2c6a738a42742e27a5";
	const OTHER_KEY_HEX: &str = "0000000000000000000000000000000000000000000000000000000000000042";

	fn key() -> KeyPair {
		KeyPair::from_bytes(&hex::decode(PRIVATE_KEY_HEX).unwrap()).unwrap()
	}

	fn other_key() -> KeyPair {
		KeyPair::from_bytes(&hex::decode(OTHER_KEY_HEX).unwrap()).unwrap()
	}

	/// A builder that already satisfies `validate()`.
	fn valid_builder() -> TransactionBuilder {
		TransactionBuilder::new(vec![0x51])
			.valid_until_block(1_000)
			.signer(Signer::called_by_entry(key().script_hash()))
	}

	#[test]
	fn test_builder_sets_every_field() {
		let tx = valid_builder()
			.version(0)
			.nonce(123_456)
			.system_fee(1_000_000)
			.network_fee(500_000)
			.attribute(TransactionAttribute::HighPriority)
			.build()
			.unwrap();

		assert_eq!(tx.version, CURRENT_VERSION);
		assert_eq!(tx.nonce, 123_456);
		assert_eq!(tx.system_fee, 1_000_000);
		assert_eq!(tx.network_fee, 500_000);
		assert_eq!(tx.valid_until_block, 1_000);
		assert_eq!(tx.signers.len(), 1);
		assert_eq!(tx.attributes, vec![TransactionAttribute::HighPriority]);
	}

	#[test]
	fn test_default_version_is_current() {
		assert_eq!(valid_builder().peek().version, CURRENT_VERSION);
	}

	#[test]
	fn test_valid_until_block_offset() {
		let tx = TransactionBuilder::new(vec![0x51])
			.valid_until_block_offset(100, 20)
			.signer(Signer::called_by_entry(key().script_hash()))
			.build()
			.unwrap();

		assert_eq!(tx.valid_until_block, 120);
	}

	#[test]
	fn test_valid_until_block_offset_saturates() {
		let tx = TransactionBuilder::new(vec![0x51])
			.valid_until_block_offset(u32::MAX, 10)
			.signer(Signer::called_by_entry(key().script_hash()))
			.build()
			.unwrap();

		assert_eq!(tx.valid_until_block, u32::MAX);
	}

	#[test]
	fn test_signers_preserves_order() {
		let first = Signer::called_by_entry(key().script_hash());
		let second = Signer::global(other_key().script_hash());

		let tx = TransactionBuilder::new(vec![0x51])
			.valid_until_block(1_000)
			.signers([first.clone(), second.clone()])
			.build()
			.unwrap();

		assert_eq!(tx.signers, vec![first, second]);
		assert_eq!(tx.sender().unwrap().account, key().script_hash());
	}

	#[test]
	fn test_sign_with_produces_verifiable_witness() {
		let tx = valid_builder().sign_with(&key()).build().unwrap();

		assert_eq!(tx.witnesses.len(), 1);
		assert!(tx.verify_witnesses());
	}

	#[test]
	fn test_signing_is_deferred_until_build() {
		// The fee is set *after* sign_with, so a signature taken eagerly would
		// not cover it and verification would fail.
		let tx = valid_builder().sign_with(&key()).system_fee(9_999_999).build().unwrap();

		assert_eq!(tx.system_fee, 9_999_999);
		assert!(tx.verify_witnesses());
	}

	#[test]
	fn test_attributes_added_after_sign_with_are_still_covered() {
		let tx = valid_builder()
			.sign_with(&key())
			.attribute(TransactionAttribute::HighPriority)
			.build()
			.unwrap();

		assert!(tx.verify_witnesses());
	}

	#[test]
	fn test_peek_shows_unsigned_transaction() {
		let builder = valid_builder().sign_with(&key());
		assert!(builder.peek().witnesses.is_empty());

		let tx = builder.build().unwrap();
		assert_eq!(tx.witnesses.len(), 1);
	}

	#[test]
	fn test_multi_signer_signing_order() {
		let first = key();
		let second = other_key();

		let tx = TransactionBuilder::new(vec![0x51])
			.valid_until_block(1_000)
			.signers([
				Signer::called_by_entry(first.script_hash()),
				Signer::called_by_entry(second.script_hash()),
			])
			.sign_with(&first)
			.sign_with(&second)
			.build()
			.unwrap();

		assert_eq!(tx.witnesses.len(), 2);
		assert!(tx.verify_witnesses());
	}

	#[test]
	fn test_manual_witness_is_preserved() {
		let witness = Witness::new(vec![0x01], vec![0x02]);
		let tx = valid_builder().witness(witness.clone()).build_unchecked();

		assert_eq!(tx.witnesses, vec![witness]);
	}

	#[test]
	fn test_build_rejects_missing_signers() {
		let result = TransactionBuilder::new(vec![0x51]).valid_until_block(1_000).build();
		assert!(matches!(result, Err(Error::InvalidData(_))));
	}

	#[test]
	fn test_build_rejects_empty_script() {
		let result = TransactionBuilder::new(Vec::new())
			.valid_until_block(1_000)
			.signer(Signer::called_by_entry(key().script_hash()))
			.build();

		assert!(matches!(result, Err(Error::InvalidData(_))));
	}

	#[test]
	fn test_build_rejects_duplicate_signers() {
		let account = key().script_hash();
		let result = TransactionBuilder::new(vec![0x51])
			.valid_until_block(1_000)
			.signer(Signer::called_by_entry(account))
			.signer(Signer::global(account))
			.build();

		assert!(matches!(result, Err(Error::InvalidData(_))));
	}

	#[test]
	fn test_build_rejects_inconsistent_signer_scopes() {
		let mut signer = Signer::called_by_entry(key().script_hash());
		signer.scopes = vec![WitnessScope::Global, WitnessScope::CalledByEntry];

		let result = TransactionBuilder::new(vec![0x51])
			.valid_until_block(1_000)
			.signer(signer)
			.build();

		assert!(matches!(result, Err(Error::InvalidData(_))));
	}

	#[test]
	fn test_build_rejects_oversized_transaction() {
		let result = TransactionBuilder::new(vec![0x51; MAX_TRANSACTION_SIZE])
			.valid_until_block(1_000)
			.signer(Signer::called_by_entry(key().script_hash()))
			.build();

		assert!(matches!(
			result,
			Err(Error::LengthLimitExceeded { max: MAX_TRANSACTION_SIZE, .. })
		));
	}

	#[test]
	fn test_build_unchecked_skips_validation() {
		// No signers: `build()` would reject this.
		let tx = TransactionBuilder::new(vec![0x51]).build_unchecked();
		assert!(tx.signers.is_empty());
		assert!(tx.validate().is_err());
	}

	#[test]
	fn test_build_unchecked_still_signs() {
		let tx = valid_builder().sign_with(&key()).build_unchecked();
		assert_eq!(tx.witnesses.len(), 1);
		assert!(tx.verify_witnesses());
	}

	#[test]
	fn test_built_transaction_round_trips() {
		use crate::Serializable;

		let tx = valid_builder().nonce(7).system_fee(1).sign_with(&key()).build().unwrap();
		let decoded = Transaction::from_bytes(&tx.to_bytes()).unwrap();

		assert_eq!(decoded, tx);
		assert_eq!(decoded.hash(), tx.hash());
		assert!(decoded.verify_witnesses());
	}
}
