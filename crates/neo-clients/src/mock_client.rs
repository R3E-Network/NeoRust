use primitive_types::{H160, H256};
use std::marker::PhantomData;
use neo_crypto::{KeyPair, Secp256r1PrivateKey};
use serde_json::json;
use lazy_static::lazy_static;

// Create a minimal Account struct for the mock client
#[derive(Debug, Clone)]
pub struct Account<T = ()> {
	pub hash: H160,
	pub _marker: PhantomData<T>,
}

impl<T> Account<T> {
	pub fn from_key_pair(
		_key_pair: KeyPair,
		_param1: Option<()>,
		_param2: Option<()>
	) -> Result<Self, &'static str> {
		Ok(Self {
			hash: H160::zero(),
			_marker: PhantomData,
		})
	}
	
	pub fn get_script_hash(&self) -> H160 {
		self.hash
	}
}

// Define the ACCOUNT1 and ACCOUNT2 constants needed by other code
lazy_static! {
	pub static ref ACCOUNT1: Account<()> = Account::from_key_pair(
		KeyPair::from_secret_key(
			&Secp256r1PrivateKey::from_bytes(
				&hex::decode("e6e919577dd7b8e97805151c05ae07ff4f752654d6d8797597aca989c02c4cb3").unwrap()
			).unwrap()
		),
		None,
		None
	).expect("Failed to create ACCOUNT1");
	
	pub static ref ACCOUNT2: Account<()> = Account::from_key_pair(
		KeyPair::from_secret_key(
			&Secp256r1PrivateKey::from_bytes(
				&hex::decode("b4b2b579cac270125259f08a5f414e9235817e7637b9a66cfeb3b77d90c8e7f9").unwrap()
			).unwrap()
		),
		None,
		None
	).expect("Failed to create ACCOUNT2");
}

// Define a minimal MockClient struct
#[derive(Debug)]
pub struct MockClient {
	// Fields omitted for simplicity
}

impl MockClient {
	pub fn new() -> Self {
		Self {}
	}
	
	// Mock methods needed by other parts of the code
	pub fn mock_response_ignore_param(&mut self, _method: &str, _result: serde_json::Value) -> &mut Self {
		self
	}
}
