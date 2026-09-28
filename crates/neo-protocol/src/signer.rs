//! Transaction signers and witness scopes

use crate::{
	constants::MAX_SIGNER_SUBITEMS,
	errors::{Error, Result},
	serialization::{Reader, Serializable, Writer},
};
use alloc::{format, vec::Vec};
use neo_primitives::{Hash160, PublicKey};
use serde::{Deserialize, Serialize};

/// How far a signer's witness extends during contract execution.
///
/// Scopes are bit flags. `None` and `Global` are exclusive: neither may be
/// combined with any other scope, since `None` grants nothing and `Global`
/// already grants everything.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum WitnessScope {
	/// The witness authorizes nothing beyond paying fees.
	None = 0x00,
	/// Valid only for the contract the user directly invoked.
	CalledByEntry = 0x01,
	/// Valid for an explicit list of contracts.
	CustomContracts = 0x10,
	/// Valid for contracts signed by an explicit list of groups.
	CustomGroups = 0x20,
	/// Valid where the attached witness rules allow it.
	WitnessRules = 0x40,
	/// Valid everywhere. Grants the most authority, so use it sparingly.
	Global = 0x80,
}

impl WitnessScope {
	/// The flag byte for this scope.
	pub const fn as_byte(self) -> u8 {
		self as u8
	}

	/// All scopes, in flag order.
	pub const ALL: [WitnessScope; 5] = [
		WitnessScope::CalledByEntry,
		WitnessScope::CustomContracts,
		WitnessScope::CustomGroups,
		WitnessScope::WitnessRules,
		WitnessScope::Global,
	];

	/// Combines scopes into a single flag byte.
	pub fn combine(scopes: &[WitnessScope]) -> u8 {
		scopes.iter().fold(0u8, |acc, scope| acc | scope.as_byte())
	}

	/// Splits a flag byte into individual scopes.
	///
	/// Rejects unknown bits, and rejects `Global` combined with anything else.
	pub fn from_byte(flags: u8) -> Result<Vec<WitnessScope>> {
		if flags == WitnessScope::None.as_byte() {
			return Ok(alloc::vec![WitnessScope::None]);
		}

		let known = WitnessScope::ALL.iter().fold(0u8, |acc, s| acc | s.as_byte());
		if flags & !known != 0 {
			return Err(Error::InvalidData(format!(
				"unknown witness scope bits: {:#04x}",
				flags & !known
			)));
		}

		if flags & WitnessScope::Global.as_byte() != 0 && flags != WitnessScope::Global.as_byte() {
			return Err(Error::InvalidData(
				"Global scope cannot be combined with other scopes".into(),
			));
		}

		Ok(WitnessScope::ALL
			.iter()
			.copied()
			.filter(|scope| flags & scope.as_byte() != 0)
			.collect())
	}
}

/// An account that authorizes a transaction, along with the scope of that
/// authorization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signer {
	/// Script hash of the signing account.
	pub account: Hash160,
	/// Scope flags controlling where the witness applies.
	pub scopes: Vec<WitnessScope>,
	/// Contracts allowed when [`WitnessScope::CustomContracts`] is set.
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub allowed_contracts: Vec<Hash160>,
	/// Groups allowed when [`WitnessScope::CustomGroups`] is set.
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub allowed_groups: Vec<PublicKey>,
}

impl Signer {
	/// Creates a signer with the given scopes.
	pub fn new(account: Hash160, scopes: Vec<WitnessScope>) -> Self {
		Self { account, scopes, allowed_contracts: Vec::new(), allowed_groups: Vec::new() }
	}

	/// Creates a signer scoped to the entry contract only.
	///
	/// This is the recommended default: it lets the invoked contract use the
	/// witness but stops contracts it calls from reusing it.
	pub fn called_by_entry(account: Hash160) -> Self {
		Self::new(account, alloc::vec![WitnessScope::CalledByEntry])
	}

	/// Creates a signer whose witness applies everywhere.
	///
	/// Prefer [`Signer::called_by_entry`] unless a contract genuinely needs it:
	/// a global witness can be used by any contract in the call chain.
	pub fn global(account: Hash160) -> Self {
		Self::new(account, alloc::vec![WitnessScope::Global])
	}

	/// Creates a signer that pays fees without authorizing anything.
	pub fn fee_only(account: Hash160) -> Self {
		Self::new(account, alloc::vec![WitnessScope::None])
	}

	/// Creates a signer restricted to an explicit contract list.
	pub fn custom_contracts(account: Hash160, contracts: Vec<Hash160>) -> Result<Self> {
		if contracts.len() > MAX_SIGNER_SUBITEMS {
			return Err(Error::LengthLimitExceeded {
				len: contracts.len() as u64,
				max: MAX_SIGNER_SUBITEMS,
			});
		}

		Ok(Self {
			account,
			scopes: alloc::vec![WitnessScope::CustomContracts],
			allowed_contracts: contracts,
			allowed_groups: Vec::new(),
		})
	}

	/// Creates a signer restricted to contracts signed by the given groups.
	pub fn custom_groups(account: Hash160, groups: Vec<PublicKey>) -> Result<Self> {
		if groups.len() > MAX_SIGNER_SUBITEMS {
			return Err(Error::LengthLimitExceeded {
				len: groups.len() as u64,
				max: MAX_SIGNER_SUBITEMS,
			});
		}

		Ok(Self {
			account,
			scopes: alloc::vec![WitnessScope::CustomGroups],
			allowed_contracts: Vec::new(),
			allowed_groups: groups,
		})
	}

	/// Whether the given scope is set.
	pub fn has_scope(&self, scope: WitnessScope) -> bool {
		self.scopes.contains(&scope)
	}

	/// The combined scope flag byte.
	pub fn scope_byte(&self) -> u8 {
		WitnessScope::combine(&self.scopes)
	}

	/// Checks that the signer's scopes and sub-items are mutually consistent.
	pub fn validate(&self) -> Result<()> {
		// Reuse the scope-combination rules (Global exclusivity, unknown bits).
		WitnessScope::from_byte(self.scope_byte())?;

		if !self.allowed_contracts.is_empty() && !self.has_scope(WitnessScope::CustomContracts) {
			return Err(Error::InvalidData(
				"allowed_contracts set without CustomContracts scope".into(),
			));
		}

		if !self.allowed_groups.is_empty() && !self.has_scope(WitnessScope::CustomGroups) {
			return Err(Error::InvalidData("allowed_groups set without CustomGroups scope".into()));
		}

		if self.allowed_contracts.len() > MAX_SIGNER_SUBITEMS {
			return Err(Error::LengthLimitExceeded {
				len: self.allowed_contracts.len() as u64,
				max: MAX_SIGNER_SUBITEMS,
			});
		}

		if self.allowed_groups.len() > MAX_SIGNER_SUBITEMS {
			return Err(Error::LengthLimitExceeded {
				len: self.allowed_groups.len() as u64,
				max: MAX_SIGNER_SUBITEMS,
			});
		}

		Ok(())
	}
}

impl Serializable for Signer {
	fn serialize(&self, writer: &mut Writer) {
		writer.write_bytes(self.account.as_bytes());
		writer.write_u8(self.scope_byte());

		if self.has_scope(WitnessScope::CustomContracts) {
			writer.write_var_int(self.allowed_contracts.len() as u64);
			for contract in &self.allowed_contracts {
				writer.write_bytes(contract.as_bytes());
			}
		}

		if self.has_scope(WitnessScope::CustomGroups) {
			writer.write_var_int(self.allowed_groups.len() as u64);
			for group in &self.allowed_groups {
				writer.write_bytes(&group.to_bytes());
			}
		}
	}

	fn deserialize(reader: &mut Reader<'_>) -> Result<Self> {
		let account = Hash160::from_slice(reader.read_bytes(20)?)?;
		let scopes = WitnessScope::from_byte(reader.read_u8()?)?;

		let mut allowed_contracts = Vec::new();
		if scopes.contains(&WitnessScope::CustomContracts) {
			let count = read_subitem_count(reader)?;
			allowed_contracts.reserve(count);
			for _ in 0..count {
				allowed_contracts.push(Hash160::from_slice(reader.read_bytes(20)?)?);
			}
		}

		let mut allowed_groups = Vec::new();
		if scopes.contains(&WitnessScope::CustomGroups) {
			let count = read_subitem_count(reader)?;
			allowed_groups.reserve(count);
			for _ in 0..count {
				allowed_groups.push(PublicKey::from_bytes(reader.read_bytes(33)?)?);
			}
		}

		Ok(Self { account, scopes, allowed_contracts, allowed_groups })
	}
}

/// Reads a sub-item count, enforcing the protocol cap before allocating.
fn read_subitem_count(reader: &mut Reader<'_>) -> Result<usize> {
	let count = reader.read_var_int()?;
	let count = usize::try_from(count)
		.map_err(|_| Error::LengthLimitExceeded { len: count, max: MAX_SIGNER_SUBITEMS })?;

	if count > MAX_SIGNER_SUBITEMS {
		return Err(Error::LengthLimitExceeded { len: count as u64, max: MAX_SIGNER_SUBITEMS });
	}
	Ok(count)
}

#[cfg(test)]
mod tests {
	use super::*;

	const ACCOUNT: &str = "0x69ecca587293047be4c59159bf8bc399985c160d";
	const GAS_TOKEN: &str = "0xd2a4cff31913016155e38e474a2c06d08be276cf";
	const NEO_TOKEN: &str = "0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5";
	const PUBLIC_KEY: &str = "033a4d051b04b7fc0230d2b1aaedfd5a84be279a5361a7358db665ad7857787f1b";

	fn account() -> Hash160 {
		Hash160::from_hex(ACCOUNT).unwrap()
	}

	fn pubkey() -> PublicKey {
		PublicKey::from_hex(PUBLIC_KEY).unwrap()
	}

	#[test]
	fn test_scope_flag_values_match_protocol() {
		assert_eq!(WitnessScope::None.as_byte(), 0x00);
		assert_eq!(WitnessScope::CalledByEntry.as_byte(), 0x01);
		assert_eq!(WitnessScope::CustomContracts.as_byte(), 0x10);
		assert_eq!(WitnessScope::CustomGroups.as_byte(), 0x20);
		assert_eq!(WitnessScope::WitnessRules.as_byte(), 0x40);
		assert_eq!(WitnessScope::Global.as_byte(), 0x80);
	}

	#[test]
	fn test_scope_combine_and_split() {
		let scopes = [WitnessScope::CustomContracts, WitnessScope::CustomGroups];
		let flags = WitnessScope::combine(&scopes);
		assert_eq!(flags, 0x30);
		assert_eq!(WitnessScope::from_byte(flags).unwrap(), scopes.to_vec());
	}

	#[test]
	fn test_scope_none_round_trips() {
		assert_eq!(WitnessScope::from_byte(0x00).unwrap(), vec![WitnessScope::None]);
	}

	#[test]
	fn test_scope_rejects_global_combined_with_others() {
		// Global grants everything, so combining it with a narrower scope is
		// contradictory and the protocol treats it as invalid.
		let flags = WitnessScope::Global.as_byte() | WitnessScope::CalledByEntry.as_byte();
		assert!(matches!(WitnessScope::from_byte(flags), Err(Error::InvalidData(_))));
	}

	#[test]
	fn test_scope_rejects_unknown_bits() {
		assert!(matches!(WitnessScope::from_byte(0x02), Err(Error::InvalidData(_))));
		assert!(matches!(WitnessScope::from_byte(0x08), Err(Error::InvalidData(_))));
	}

	#[test]
	fn test_called_by_entry_roundtrip() {
		let signer = Signer::called_by_entry(account());
		let bytes = signer.to_bytes();

		// 20-byte account + 1 scope byte, no sub-items
		assert_eq!(bytes.len(), 21);
		assert_eq!(bytes[20], 0x01);
		assert_eq!(Signer::from_bytes(&bytes).unwrap(), signer);
	}

	#[test]
	fn test_global_signer_roundtrip() {
		let signer = Signer::global(account());
		let bytes = signer.to_bytes();
		assert_eq!(bytes[20], 0x80);
		assert_eq!(Signer::from_bytes(&bytes).unwrap(), signer);
	}

	#[test]
	fn test_fee_only_signer_roundtrip() {
		let signer = Signer::fee_only(account());
		let bytes = signer.to_bytes();
		assert_eq!(bytes[20], 0x00);
		assert_eq!(Signer::from_bytes(&bytes).unwrap(), signer);
	}

	#[test]
	fn test_account_serialized_in_wire_order() {
		let signer = Signer::called_by_entry(account());
		let bytes = signer.to_bytes();
		assert_eq!(&bytes[..20], account().as_bytes());
	}

	#[test]
	fn test_custom_contracts_roundtrip() {
		let contracts =
			vec![Hash160::from_hex(GAS_TOKEN).unwrap(), Hash160::from_hex(NEO_TOKEN).unwrap()];
		let signer = Signer::custom_contracts(account(), contracts.clone()).unwrap();

		let bytes = signer.to_bytes();
		// 20 account + 1 scope + 1 count + 2*20 contracts
		assert_eq!(bytes.len(), 62);

		let decoded = Signer::from_bytes(&bytes).unwrap();
		assert_eq!(decoded, signer);
		assert_eq!(decoded.allowed_contracts, contracts);
	}

	#[test]
	fn test_custom_groups_roundtrip() {
		let signer = Signer::custom_groups(account(), vec![pubkey()]).unwrap();

		let bytes = signer.to_bytes();
		// 20 account + 1 scope + 1 count + 33 pubkey
		assert_eq!(bytes.len(), 55);

		let decoded = Signer::from_bytes(&bytes).unwrap();
		assert_eq!(decoded, signer);
		assert_eq!(decoded.allowed_groups, vec![pubkey()]);
	}

	#[test]
	fn test_custom_contracts_rejects_too_many() {
		let contracts = vec![Hash160::from_hex(GAS_TOKEN).unwrap(); MAX_SIGNER_SUBITEMS + 1];
		assert!(matches!(
			Signer::custom_contracts(account(), contracts),
			Err(Error::LengthLimitExceeded { max: MAX_SIGNER_SUBITEMS, .. })
		));
	}

	#[test]
	fn test_custom_groups_rejects_too_many() {
		let groups = vec![pubkey(); MAX_SIGNER_SUBITEMS + 1];
		assert!(matches!(
			Signer::custom_groups(account(), groups),
			Err(Error::LengthLimitExceeded { max: MAX_SIGNER_SUBITEMS, .. })
		));
	}

	#[test]
	fn test_deserialize_rejects_too_many_contracts() {
		// Hand-build a signer claiming more sub-items than the protocol allows.
		let mut writer = Writer::new();
		writer.write_bytes(account().as_bytes());
		writer.write_u8(WitnessScope::CustomContracts.as_byte());
		writer.write_var_int((MAX_SIGNER_SUBITEMS + 1) as u64);
		for _ in 0..=MAX_SIGNER_SUBITEMS {
			writer.write_bytes(Hash160::from_hex(GAS_TOKEN).unwrap().as_bytes());
		}

		assert!(matches!(
			Signer::from_bytes(writer.as_bytes()),
			Err(Error::LengthLimitExceeded { max: MAX_SIGNER_SUBITEMS, .. })
		));
	}

	#[test]
	fn test_deserialize_rejects_truncated_input() {
		let signer = Signer::called_by_entry(account());
		let bytes = signer.to_bytes();
		assert!(matches!(Signer::from_bytes(&bytes[..10]), Err(Error::UnexpectedEof { .. })));
	}

	#[test]
	fn test_deserialize_rejects_global_with_extra_scope() {
		let mut writer = Writer::new();
		writer.write_bytes(account().as_bytes());
		writer.write_u8(0x81); // Global | CalledByEntry

		assert!(matches!(Signer::from_bytes(writer.as_bytes()), Err(Error::InvalidData(_))));
	}

	#[test]
	fn test_has_scope() {
		let signer = Signer::called_by_entry(account());
		assert!(signer.has_scope(WitnessScope::CalledByEntry));
		assert!(!signer.has_scope(WitnessScope::Global));
	}

	#[test]
	fn test_validate_accepts_consistent_signers() {
		assert!(Signer::called_by_entry(account()).validate().is_ok());
		assert!(Signer::global(account()).validate().is_ok());
		assert!(Signer::fee_only(account()).validate().is_ok());
		assert!(Signer::custom_contracts(account(), vec![Hash160::from_hex(GAS_TOKEN).unwrap()])
			.unwrap()
			.validate()
			.is_ok());
	}

	#[test]
	fn test_validate_rejects_contracts_without_matching_scope() {
		let mut signer = Signer::called_by_entry(account());
		signer.allowed_contracts.push(Hash160::from_hex(GAS_TOKEN).unwrap());
		assert!(matches!(signer.validate(), Err(Error::InvalidData(_))));
	}

	#[test]
	fn test_validate_rejects_groups_without_matching_scope() {
		let mut signer = Signer::called_by_entry(account());
		signer.allowed_groups.push(pubkey());
		assert!(matches!(signer.validate(), Err(Error::InvalidData(_))));
	}

	#[test]
	fn test_validate_rejects_global_combined_with_others() {
		let signer =
			Signer::new(account(), vec![WitnessScope::Global, WitnessScope::CalledByEntry]);
		assert!(matches!(signer.validate(), Err(Error::InvalidData(_))));
	}

	#[test]
	fn test_size_matches_encoding() {
		let signer =
			Signer::custom_contracts(account(), vec![Hash160::from_hex(GAS_TOKEN).unwrap()])
				.unwrap();
		assert_eq!(signer.size(), signer.to_bytes().len());
	}

	#[test]
	fn test_serde_roundtrip() {
		let signer = Signer::custom_groups(account(), vec![pubkey()]).unwrap();
		let json = serde_json::to_string(&signer).unwrap();
		let back: Signer = serde_json::from_str(&json).unwrap();
		assert_eq!(signer, back);
	}

	#[test]
	fn test_serde_omits_empty_subitem_lists() {
		let json = serde_json::to_string(&Signer::called_by_entry(account())).unwrap();
		assert!(!json.contains("allowedContracts"));
		assert!(!json.contains("allowed_contracts"));
	}
}
