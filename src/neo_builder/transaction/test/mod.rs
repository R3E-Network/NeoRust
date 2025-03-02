//! Test utilities for the transaction module

use lazy_static::lazy_static;
use primitive_types::H160;
use rustc_serialize::hex::FromHex;

use neo::prelude::{Account, KeyPair, ScriptHash, Secp256r1PrivateKey, Secp256r1PublicKey};

/// Test constants for transaction module tests
#[allow(unused)]
pub struct TestConstants;

lazy_static! {
	// Script hash constants for tests
	pub static ref SCRIPT_HASH: ScriptHash = {
		Account::from_wif("Kzt94tAAiZSgH7Yt4i25DW6jJFprZFPSqTgLr5dWmWgKDKCjXMfZ")
			.unwrap()
			.get_script_hash()
	};
	pub static ref SCRIPT_HASH1: H160 = H160::from_script(&"d802a401".from_hex().unwrap());
	pub static ref SCRIPT_HASH2: H160 = H160::from_script(&"c503b112".from_hex().unwrap());

	// Public key constants for group tests
	pub static ref GROUP_PUB_KEY1: Secp256r1PublicKey = Secp256r1PublicKey::from_encoded(
		"0306d3e7f18e6dd477d34ce3cfeca172a877f3c907cc6c2b66c295d1fcc76ff8f7",
	)
	.unwrap();
	pub static ref GROUP_PUB_KEY2: Secp256r1PublicKey = Secp256r1PublicKey::from_encoded(
		"02958ab88e4cea7ae1848047daeb8883daf5fdf5c1301dbbfe973f0a29fe75de60",
	)
	.unwrap();

	// Account constants for tests
	pub static ref ACCOUNT1: Account = Account::from_key_pair(
		KeyPair::from_secret_key(
			&Secp256r1PrivateKey::from_bytes(
				&hex::decode(
					"e6e919577dd7b8e97805151c05ae07ff4f752654d6d8797597aca989c02c4cb3"
				)
				.expect("Test private key 1 should be valid hex")
			)
			.expect("Test private key 1 should be valid bytes for Secp256r1PrivateKey")
		),
		None,
		None
	)
	.expect("Failed to create ACCOUNT1 from valid key pair");

	pub static ref ACCOUNT2: Account = Account::from_key_pair(
		KeyPair::from_secret_key(
			&Secp256r1PrivateKey::from_bytes(
				&hex::decode(
					"b4b2b579cac270125259f08a5f414e9235817e7637b9a66cfeb3b77d90c8e7f9"
				)
				.expect("Test private key 2 should be valid hex")
			)
			.expect("Test private key 2 should be valid bytes for Secp256r1PrivateKey")
		),
		None,
		None
	)
	.expect("Failed to create ACCOUNT2 from valid key pair");
}

/// NEO blockchain-specific constants
pub struct NeoConstants;
impl NeoConstants {
	// Transaction & Signer constraints
	pub const MAX_TRANSACTION_ATTRIBUTES: u32 = 16;
	pub const MAX_SIGNER_SUBITEMS: u32 = 16;
}

/// Contract parameter types for Neo smart contracts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractParameterType {
	Any,
	Boolean,
	Integer,
	ByteArray,
	String,
	Hash160,
	Hash256,
	PublicKey,
	Signature,
	Array,
	Map,
	InteropInterface,
	Void,
}
