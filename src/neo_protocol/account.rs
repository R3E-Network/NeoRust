use std::{
	collections::HashMap,
	fmt::Debug,
	hash::{Hash, Hasher},
	str::FromStr,
	sync::{Arc, Weak},
};

use primitive_types::H160;
use rustc_serialize::hex::ToHex;
use serde_derive::{Deserialize, Serialize};
use signature::{hazmat::PrehashSigner, Error, SignerMut};

use neo::prelude::*;

pub trait AccountTrait: Sized + PartialEq + Send + Sync + Debug + Clone {
	type Error: Sync + Send + Debug + Sized;

	// Methods to access the fields
	fn key_pair(&self) -> &Option<KeyPair>;
	fn address_or_scripthash(&self) -> &AddressOrScriptHash;
	fn label(&self) -> &Option<String>;
	fn verification_script(&self) -> &Option<VerificationScript>;
	fn is_locked(&self) -> bool;
	fn encrypted_private_key(&self) -> &Option<String>;
	fn signing_threshold(&self) -> &Option<u32>;
	fn nr_of_participants(&self) -> &Option<u32>;
	fn set_key_pair(&mut self, key_pair: Option<KeyPair>);
	fn set_address_or_scripthash(&mut self, address_or_scripthash: AddressOrScriptHash);
	fn set_label(&mut self, label: Option<String>);
	fn set_verification_script(&mut self, verification_script: Option<VerificationScript>);
	fn set_locked(&mut self, is_locked: bool);
	fn set_encrypted_private_key(&mut self, encrypted_private_key: Option<String>);

	fn set_signing_threshold(&mut self, signing_threshold: Option<u32>);
	fn set_nr_of_participants(&mut self, nr_of_participants: Option<u32>);

	fn new(
		address: AddressOrScriptHash,
		label: Option<String>,
		verification_script: Option<VerificationScript>,
		signing_threshold: Option<u32>,
		nr_of_participants: Option<u32>,
	) -> Self;

	fn from_key_pair(
		key_pair: KeyPair,
		signing_threshold: Option<u32>,
		nr_of_participants: Option<u32>,
	) -> Result<Self, Self::Error>;

	fn from_key_pair_opt(
		key_pair: Option<KeyPair>,
		address: AddressOrScriptHash,
		label: Option<String>,
		verification_script: Option<VerificationScript>,
		is_locked: bool,
		is_default: bool,
		encrypted_private_key: Option<String>,
		signing_threshold: Option<u32>,
		nr_of_participants: Option<u32>,
	) -> Self;

	fn from_wif(wif: &str) -> Result<Self, Self::Error>;

	fn decrypt_private_key(&mut self, password: &str) -> Result<(), Self::Error>;

	fn encrypt_private_key(&mut self, password: &str) -> Result<(), Self::Error>;

	fn get_script_hash(&self) -> ScriptHash;

	fn get_signing_threshold(&self) -> Result<u32, Self::Error>;

	fn get_nr_of_participants(&self) -> Result<u32, Self::Error>;

	fn from_verification_script(script: &VerificationScript) -> Result<Self, Self::Error>;

	fn from_public_key(public_key: &Secp256r1PublicKey) -> Result<Self, Self::Error>;

	fn set_wallet(&mut self, wallet: Option<Weak<Wallet>>);

	fn get_wallet(&self) -> Option<Arc<Wallet>>;

	fn multi_sig_from_public_keys(
		public_keys: &mut [Secp256r1PublicKey],
		signing_threshold: u32,
	) -> Result<Self, Self::Error>;
	fn multi_sig_from_addr(
		address: String,
		signing_threshold: u8,
		nr_of_participants: u8,
	) -> Result<Self, Self::Error>;

	fn from_address(address: &str) -> Result<Self, Self::Error>;

	fn from_script_hash(script_hash: &H160) -> Result<Self, Self::Error>;

	fn create() -> Result<Self, Self::Error>;

	fn is_multi_sig(&self) -> bool;
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Account {
	#[serde(skip)]
	pub key_pair: Option<KeyPair>,
	#[serde(
		serialize_with = "serialize_address_or_script_hash",
		deserialize_with = "deserialize_address_or_script_hash"
	)]
	pub address_or_scripthash: AddressOrScriptHash,
	pub label: Option<String>,
	pub verification_script: Option<VerificationScript>,
	pub is_default: bool,
	pub is_locked: bool,
	pub encrypted_private_key: Option<String>,
	pub signing_threshold: Option<u32>,
	pub nr_of_participants: Option<u32>,
	#[serde(skip)]
	pub wallet: Option<Weak<Wallet>>,
}

impl Account {
	pub fn get_address(&self) -> String {
		self.address_or_scripthash.address()
	}

	pub fn get_script_hash(&self) -> H160 {
		self.address_or_scripthash.script_hash()
	}

	pub fn get_verification_script(&self) -> Option<VerificationScript> {
		self.verification_script.clone()
	}
	pub fn get_public_key(&self) -> Option<Secp256r1PublicKey> {
		self.key_pair.as_ref().map(|k| k.public_key.clone())
	}
}

impl From<H160> for Account {
	fn from(script_hash: H160) -> Self {
		Self {
			address_or_scripthash: AddressOrScriptHash::ScriptHash(script_hash),
			..Default::default()
		}
	}
}

impl From<&H160> for Account {
	fn from(script_hash: &H160) -> Self {
		Self {
			address_or_scripthash: AddressOrScriptHash::ScriptHash(script_hash.clone()),
			..Default::default()
		}
	}
}

impl PartialEq for Account {
	fn eq(&self, other: &Self) -> bool {
		self.address_or_scripthash == other.address_or_scripthash
			&& self.label == other.label
			&& self.verification_script == other.verification_script
			&& self.is_locked == other.is_locked
			&& self.encrypted_private_key == other.encrypted_private_key
			&& self.signing_threshold == other.signing_threshold
			&& self.nr_of_participants == other.nr_of_participants
	}
}

impl Hash for Account {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.address_or_scripthash.hash(state);
		self.label.hash(state);
		self.verification_script.hash(state);
		self.is_locked.hash(state);
		self.encrypted_private_key.hash(state);
		self.signing_threshold.hash(state);
		self.nr_of_participants.hash(state);
	}
}

impl AccountTrait for Account {
	type Error = ProviderError;

	fn key_pair(&self) -> &Option<KeyPair> {
		&self.key_pair
	}

	fn address_or_scripthash(&self) -> &AddressOrScriptHash {
		&self.address_or_scripthash
	}

	fn label(&self) -> &Option<String> {
		&self.label
	}

	fn verification_script(&self) -> &Option<VerificationScript> {
		&self.verification_script
	}

	fn is_locked(&self) -> bool {
		self.is_locked
	}

	fn encrypted_private_key(&self) -> &Option<String> {
		&self.encrypted_private_key
	}

	fn signing_threshold(&self) -> &Option<u32> {
		&self.signing_threshold
	}

	fn nr_of_participants(&self) -> &Option<u32> {
		&self.nr_of_participants
	}

	fn set_key_pair(&mut self, key_pair: Option<KeyPair>) {
		self.key_pair = key_pair;
	}

	fn set_address_or_scripthash(&mut self, address_or_scripthash: AddressOrScriptHash) {
		self.address_or_scripthash = address_or_scripthash;
	}

	fn set_label(&mut self, label: Option<String>) {
		self.label = label;
	}

	fn set_verification_script(&mut self, verification_script: Option<VerificationScript>) {
		self.verification_script = verification_script;
	}

	fn set_locked(&mut self, is_locked: bool) {
		self.is_locked = is_locked;
	}

	fn set_encrypted_private_key(&mut self, encrypted_private_key: Option<String>) {
		self.encrypted_private_key = encrypted_private_key;
	}

	fn set_signing_threshold(&mut self, signing_threshold: Option<u32>) {
		self.signing_threshold = signing_threshold;
	}

	fn set_nr_of_participants(&mut self, nr_of_participants: Option<u32>) {
		self.nr_of_participants = nr_of_participants;
	}

	fn new(
		address: AddressOrScriptHash,
		label: Option<String>,
		verification_script: Option<VerificationScript>,
		signing_threshold: Option<u32>,
		nr_of_participants: Option<u32>,
	) -> Self {
		Self {
			key_pair: None,
			address_or_scripthash: address,
			label,
			verification_script,
			is_default: false,
			is_locked: false,
			encrypted_private_key: None,
			signing_threshold,
			nr_of_participants,
			wallet: None,
		}
	}

	fn from_key_pair(
		key_pair: KeyPair,
		signing_threshold: Option<u32>,
		nr_of_participants: Option<u32>,
	) -> Result<Self, Self::Error> {
		let address = public_key_to_address(&key_pair.public_key);
		Ok(Self {
			key_pair: Some(key_pair.clone()),
			address_or_scripthash: AddressOrScriptHash::Address(address.clone()),
			label: Some(address),
			verification_script: Some(VerificationScript::from_public_key(
				&key_pair.clone().public_key(),
			)),
			is_default: false,
			is_locked: false,
			encrypted_private_key: None,
			signing_threshold,
			nr_of_participants,
			wallet: None,
		})
	}

	fn from_key_pair_opt(
		key_pair: Option<KeyPair>,
		address: AddressOrScriptHash,
		label: Option<String>,
		verification_script: Option<VerificationScript>,
		is_locked: bool,
		_is_default: bool,
		encrypted_private_key: Option<String>,
		signing_threshold: Option<u32>,
		nr_of_participants: Option<u32>,
	) -> Self {
		Self {
			key_pair,
			address_or_scripthash: address,
			label,
			verification_script,
			is_default: false,
			is_locked,
			encrypted_private_key,
			signing_threshold,
			nr_of_participants,
			wallet: None,
		}
	}

	fn from_wif(wif: &str) -> Result<Self, Self::Error> {
		let key_pair = KeyPair::from_secret_key(&private_key_from_wif(wif).unwrap());
		Self::from_key_pair(key_pair, None, None)
	}

	fn decrypt_private_key(&mut self, password: &str) -> Result<(), Self::Error> {
		if self.key_pair.is_some() {
			return Ok(());
		}

		let encrypted_private_key = self
			.encrypted_private_key
			.as_ref()
			.ok_or(Self::Error::IllegalState("No encrypted private key present".to_string()))
			.unwrap();
		let key_pair = get_private_key_from_nep2(encrypted_private_key, password).unwrap();
		self.key_pair =
			Some(KeyPair::from_private_key(&vec_to_array32(key_pair).unwrap()).unwrap());
		Ok(())
	}

	fn encrypt_private_key(&mut self, password: &str) -> Result<(), Self::Error> {
		let key_pair = self.key_pair.as_ref().ok_or(Self::Error::IllegalState(
			"The account does not hold a decrypted private key.".to_string(),
		))?;

		let encrypted_private_key = get_nep2_from_private_key(
			key_pair.private_key.to_raw_bytes().to_hex().as_str(),
			password,
		)
		.unwrap();
		self.encrypted_private_key = Some(encrypted_private_key);
		self.key_pair = None;
		Ok(())
	}

	fn get_script_hash(&self) -> ScriptHash {
		self.address_or_scripthash.script_hash()
	}

	fn get_signing_threshold(&self) -> Result<u32, Self::Error> {
		self.signing_threshold.ok_or_else(|| {
			Self::Error::IllegalState(format!(
				"Cannot get signing threshold from account {}",
				self.address_or_scripthash().address()
			))
		})
	}

	fn get_nr_of_participants(&self) -> Result<u32, Self::Error> {
		self.nr_of_participants.ok_or_else(|| {
			Self::Error::IllegalState(format!(
				"Cannot get signing threshold from account {}",
				self.address_or_scripthash().address()
			))
		})
	}

	fn from_verification_script(script: &VerificationScript) -> Result<Self, Self::Error> {
		let address = ScriptHash::from_script(&script.script());

		let (signing_threshold, nr_of_participants) = if script.is_multi_sig() {
			(
				Some(script.get_signing_threshold().unwrap()),
				Some(script.get_nr_of_accounts().unwrap()),
			)
		} else {
			(None, None)
		};

		Ok(Self {
			address_or_scripthash: AddressOrScriptHash::ScriptHash(address),
			label: Some(address.to_address()),
			verification_script: Some(script.clone()),
			signing_threshold: signing_threshold.map(|x| x as u32),
			nr_of_participants: nr_of_participants.map(|x| x as u32),
			..Default::default()
		})
	}

	fn from_public_key(public_key: &Secp256r1PublicKey) -> Result<Self, Self::Error> {
		let script = VerificationScript::from_public_key(public_key);
		let address = ScriptHash::from_script(&script.script());

		Ok(Self {
			address_or_scripthash: AddressOrScriptHash::ScriptHash(address),
			label: Some(address.to_address()),
			verification_script: Some(script),
			..Default::default()
		})
	}

	fn set_wallet(&mut self, wallet: Option<Weak<Wallet>>) {
		self.wallet = wallet;
	}

	fn get_wallet(&self) -> Option<Arc<Wallet>> {
		self.wallet.as_ref().and_then(|w| w.upgrade())
	}

	fn multi_sig_from_public_keys(
		public_keys: &mut [Secp256r1PublicKey],
		signing_threshold: u32,
	) -> Result<Self, Self::Error> {
		let script = VerificationScript::from_multi_sig(public_keys, signing_threshold as u8);
		let addr = ScriptHash::from_script(&script.script());

		Ok(Self {
			label: Some(addr.to_address()),
			verification_script: Some(script),
			signing_threshold: Some(signing_threshold),
			nr_of_participants: Some(public_keys.len() as u32),
			address_or_scripthash: AddressOrScriptHash::ScriptHash(addr),
			..Default::default()
		})
	}

	fn multi_sig_from_addr(
		address: String,
		signing_threshold: u8,
		nr_of_participants: u8,
	) -> Result<Self, Self::Error> {
		Ok(Self {
			label: Option::from(address.clone()),
			signing_threshold: Some(signing_threshold as u32),
			nr_of_participants: Some(nr_of_participants as u32),
			address_or_scripthash: AddressOrScriptHash::Address(address),
			..Default::default()
		})
	}

	fn from_address(address: &str) -> Result<Self, Self::Error> {
		let address = Address::from_str(address).unwrap();
		Ok(Self {
			address_or_scripthash: AddressOrScriptHash::Address(address.clone()),
			label: Some(address),
			..Default::default()
		})
	}

	fn from_script_hash(script_hash: &H160) -> Result<Self, Self::Error> {
		let address = script_hash.to_address();
		Self::from_address(&address)
	}

	fn create() -> Result<Self, Self::Error> {
		let key_pair = KeyPair::new_random();
		Self::from_key_pair(key_pair, None, None)
	}

	fn is_multi_sig(&self) -> bool {
		self.signing_threshold.is_some() && self.nr_of_participants.is_some()
	}
}

impl PrehashSigner<Secp256r1Signature> for Account {
	fn sign_prehash(&self, _prehash: &[u8]) -> Result<Secp256r1Signature, Error> {
		if self.key_pair.is_none() {
			return Err(Error::new());
		}

		let key_pair = self.key_pair.as_ref().unwrap();
		let signature = key_pair.private_key.sign_prehash(_prehash).unwrap();
		Ok(signature)
	}
}

impl Account {
	pub fn to_nep6_account(&self) -> Result<NEP6Account, ProviderError> {
		if self.key_pair.is_some() && self.encrypted_private_key.is_none() {
			return Err(ProviderError::IllegalState(
				"Account private key is available but not encrypted.".to_string(),
			));
		}

		if self.verification_script.is_none() {
			return Ok(NEP6Account::new(
				self.address_or_scripthash.address().clone(),
				self.label.clone(),
				self.is_default,
				self.is_locked,
				self.encrypted_private_key.clone(),
				None,
				None,
			));
		}

		let mut parameters = Vec::new();
		let script_data = self.verification_script.as_ref().unwrap();

		if script_data.is_multi_sig() {
			for i in 0..script_data.get_nr_of_accounts().unwrap() {
				parameters.push(NEP6Parameter {
					param_name: format!("signature{}", i),
					param_type: ContractParameterType::Signature,
				});
			}
		} else if script_data.is_single_sig() {
			parameters.push(NEP6Parameter {
				param_name: "signature".to_string(),
				param_type: ContractParameterType::Signature,
			});
		}

		let script_encoded = script_data.script().to_base64();
		let contract = NEP6Contract {
			script: Some(script_encoded),
			is_deployed: false, // Assuming a simple setup; might need actual logic
			nep6_parameters: parameters,
		};

		Ok(NEP6Account::new(
			self.address_or_scripthash.address().clone(),
			self.label.clone(),
			self.is_default,
			self.is_locked,
			self.encrypted_private_key.clone(),
			Some(contract),
			None,
		))
	}

	pub async fn get_nep17_balances<P>(
		&self,
		provider: &NeoClient<P>,
	) -> Result<HashMap<H160, u64>, ProviderError>
	where
		P: JsonRpcClient,
	{
		let response =
			provider.get_nep17_balances(self.address_or_scripthash().script_hash()).await?;
		let mut balances = HashMap::new();
		for balance in response.balances {
			let asset_hash = balance.asset_hash;
			let amount = balance.amount.parse::<u64>().unwrap();
			balances.insert(asset_hash, amount);
		}
		Ok(balances)
	}
}

#[cfg(test)]
mod tests {
	use primitive_types::H160;
	use rustc_serialize::hex::FromHex;
	use serde_json::Value;
	use url::Url;
	use wiremock::{
		matchers::{method, path},
		Mock, MockServer, ResponseTemplate,
	};

	use neo::prelude::{
		Account, AccountTrait, BodyRegexMatcher, HttpProvider, KeyPair, NeoClient, NeoSerializable,
		ProviderError, ScriptHashExtension, Secp256r1PublicKey, TestConstants, ToArray32,
		VerificationScript, Wallet, WalletTrait,
	};

	use super::APITrait;

	#[test]
	fn test_create_generic_account() {
		let account = Account::create().unwrap();
		assert!(account.verification_script.is_some());
		assert!(account.key_pair.is_some());
		assert!(account.label.is_some());
		assert!(account.encrypted_private_key.is_none());
		assert!(!account.is_locked);
		assert!(!account.is_default);
	}

	#[test]
	fn test_init_account_from_existing_key_pair() {
		let key_pair = KeyPair::from_private_key(
			&hex::decode(TestConstants::DEFAULT_ACCOUNT_PRIVATE_KEY)
				.unwrap()
				.to_array32()
				.unwrap(),
		)
		.unwrap();
		let account = Account::from_key_pair(key_pair.clone(), None, None).unwrap();

		assert!(!account.is_multi_sig());
		assert_eq!(
			account.address_or_scripthash().address(),
			TestConstants::DEFAULT_ACCOUNT_ADDRESS
		);
		assert_eq!(*account.key_pair(), Some(key_pair));
		assert_eq!(account.label, Some(TestConstants::DEFAULT_ACCOUNT_ADDRESS.to_string()));
		assert_eq!(
			account.verification_script.unwrap().script(),
			&hex::decode(TestConstants::DEFAULT_ACCOUNT_VERIFICATION_SCRIPT).unwrap()
		);
	}

	#[test]
	fn test_from_public_key() {
		let public_key = Secp256r1PublicKey::from_bytes(
			&hex::decode(TestConstants::DEFAULT_ACCOUNT_PUBLIC_KEY).unwrap(),
		)
		.unwrap();
		let account = Account::from_public_key(&public_key).unwrap();

		assert_eq!(
			account.address_or_scripthash().address(),
			TestConstants::DEFAULT_ACCOUNT_ADDRESS
		);
		assert_eq!(
			account.verification_script.unwrap().script(),
			&hex::decode(TestConstants::DEFAULT_ACCOUNT_VERIFICATION_SCRIPT).unwrap()
		);
	}

	#[test]
	fn test_create_multi_sig_account_from_public_keys() {
		let public_key = Secp256r1PublicKey::from_bytes(
			&hex::decode(TestConstants::DEFAULT_ACCOUNT_PUBLIC_KEY).unwrap(),
		)
		.unwrap();
		let account = Account::multi_sig_from_public_keys(&mut vec![public_key], 1).unwrap();

		assert!(account.is_multi_sig());
		assert_eq!(
			account.address_or_scripthash().address(),
			TestConstants::COMMITTEE_ACCOUNT_ADDRESS.to_string()
		);
		assert_eq!(account.label, Some(TestConstants::COMMITTEE_ACCOUNT_ADDRESS.to_string()));
		assert_eq!(
			account.verification_script.unwrap().script(),
			&hex::decode(TestConstants::COMMITTEE_ACCOUNT_VERIFICATION_SCRIPT).unwrap()
		);
	}

	#[test]
	fn test_create_multi_sig_account_account_with_address() {
		let account = Account::multi_sig_from_addr(
			TestConstants::COMMITTEE_ACCOUNT_ADDRESS.to_string(),
			4,
			7,
		)
		.unwrap();
		assert_eq!(account.get_signing_threshold().unwrap(), 4);
		assert_eq!(account.get_nr_of_participants().unwrap(), 7);
		assert_eq!(
			account.address_or_scripthash().address(),
			TestConstants::COMMITTEE_ACCOUNT_ADDRESS
		);
		assert!(account.is_multi_sig());
		assert_eq!(account.label, Some(TestConstants::COMMITTEE_ACCOUNT_ADDRESS.to_string()));
		assert!(account.verification_script().is_none());
	}

	#[test]
	fn test_create_multi_sig_account_account_from_verification_script() {
		let account = Account::from_verification_script(&VerificationScript::from(
			hex::decode(TestConstants::COMMITTEE_ACCOUNT_VERIFICATION_SCRIPT).unwrap(),
		))
		.unwrap();
		assert!(account.is_multi_sig());
		assert_eq!(
			account.address_or_scripthash().address(),
			TestConstants::COMMITTEE_ACCOUNT_ADDRESS
		);
		assert_eq!(account.label, Some(TestConstants::COMMITTEE_ACCOUNT_ADDRESS.to_string()));
		assert_eq!(
			account.verification_script.unwrap().script(),
			&hex::decode(TestConstants::COMMITTEE_ACCOUNT_VERIFICATION_SCRIPT).unwrap()
		);
	}

	#[test]
	fn test_nil_values_when_not_multi_sig() {
		let account = Account::from_address(TestConstants::DEFAULT_ACCOUNT_ADDRESS).unwrap();
		assert!(account.signing_threshold.is_none());
		assert!(account.nr_of_participants.is_none());
	}

	#[test]
	fn test_encrypt_private_key() {
		let key_pair = KeyPair::from_private_key(
			&TestConstants::DEFAULT_ACCOUNT_PRIVATE_KEY
				.from_hex()
				.unwrap()
				.to_array32()
				.unwrap(),
		)
		.unwrap();
		let mut account = Account::from_key_pair(key_pair, None, None).unwrap();

		assert_eq!(
			account.address_or_scripthash().address(),
			TestConstants::DEFAULT_ACCOUNT_ADDRESS
		);
		account.encrypt_private_key(TestConstants::DEFAULT_ACCOUNT_PASSWORD).unwrap();

		assert_eq!(
			account.encrypted_private_key.unwrap(),
			TestConstants::DEFAULT_ACCOUNT_ENCRYPTED_PRIVATE_KEY
		);
	}

	#[test]
	fn test_fail_encrypt_account_without_private_key() {
		let mut account = Account::from_address(TestConstants::DEFAULT_ACCOUNT_ADDRESS).unwrap();

		let err = account.encrypt_private_key("pwd").unwrap_err();
		assert_eq!(
			err,
			ProviderError::IllegalState(
				"The account does not hold a decrypted private key.".to_string()
			)
		);
	}

	#[test]
	fn test_create_account_from_wif() {
		let account = Account::from_wif(TestConstants::DEFAULT_ACCOUNT_WIF).unwrap();

		let expected_key_pair = KeyPair::from_private_key(
			&hex::decode(TestConstants::DEFAULT_ACCOUNT_PRIVATE_KEY)
				.unwrap()
				.to_array32()
				.unwrap(),
		)
		.unwrap();

		assert_eq!(account.key_pair.clone().unwrap(), expected_key_pair.clone());
		// assert_eq!(
		// 	account.key_pair.clone().unwrap().private_key.to_vec(),
		// 	expected_key_pair.private_key.to_vec()
		// );
		let addr = account.address_or_scripthash();
		assert_eq!(addr.address(), TestConstants::DEFAULT_ACCOUNT_ADDRESS);
		assert_eq!(account.label, Some(TestConstants::DEFAULT_ACCOUNT_ADDRESS.to_string()));
		assert_eq!(account.encrypted_private_key, None);
		assert_eq!(
			account.get_script_hash(),
			H160::from_hex(TestConstants::DEFAULT_ACCOUNT_SCRIPT_HASH).unwrap()
		);
		assert!(!account.is_locked);
		assert!(!account.is_default);
		assert_eq!(
			account.verification_script.unwrap().script(),
			&hex::decode(TestConstants::DEFAULT_ACCOUNT_VERIFICATION_SCRIPT).unwrap()
		);
	}

	#[test]
	fn test_create_account_from_address() {
		let account = Account::from_address(TestConstants::DEFAULT_ACCOUNT_ADDRESS).unwrap();

		assert_eq!(
			account.address_or_scripthash().address(),
			TestConstants::DEFAULT_ACCOUNT_ADDRESS
		);
		assert_eq!(account.label, Some(TestConstants::DEFAULT_ACCOUNT_ADDRESS.to_string()));
		assert_eq!(
			&account.address_or_scripthash.script_hash().to_hex(),
			TestConstants::DEFAULT_ACCOUNT_SCRIPT_HASH
		);
		assert!(!account.is_default);
		assert!(!account.is_locked);
		assert!(account.verification_script.is_none());
	}

	#[tokio::test]
	async fn test_get_nep17_balances() {
		let data =
			include_str!("../../test_resources/responses/getnep17balances_ofDefaultAccount.json");
		let json_response: Value = serde_json::from_str(data).expect("Failed to parse JSON");
		let call = "getnep17balances";
		let param = TestConstants::DEFAULT_ACCOUNT_ADDRESS;
		let pattern = format!(".*\"method\":\"{}\".*.*\"params\":.*.*\"{}\".*", call, param);
		let body_matcher = BodyRegexMatcher::new(&pattern);

		let mock_server = MockServer::start().await;
		Mock::given(method("POST"))
			.and(path("/"))
			.and(body_matcher)
			.respond_with(ResponseTemplate::new(200).set_body_json(json_response))
			.mount(&mock_server)
			.await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = NeoClient::new(http_client);

		let account = Account::from_address(TestConstants::DEFAULT_ACCOUNT_ADDRESS).unwrap();
		//
		let balances = account.get_nep17_balances(&provider).await.unwrap();
		//
		assert_eq!(balances.len(), 2);
		assert!(balances.contains_key(&H160::from_hex(TestConstants::GAS_TOKEN_HASH).unwrap()));
		assert!(balances.contains_key(&H160::from_hex(TestConstants::NEO_TOKEN_HASH).unwrap()));
		assert!(balances.values().any(|&v| v == 300000000));
		assert!(balances.values().any(|&v| v == 5));
	}

	#[test]
	fn test_is_multi_sig() {
		let a = Account::from_address(TestConstants::DEFAULT_ACCOUNT_ADDRESS).unwrap();
		assert!(!a.is_multi_sig());

		let a1 = Account::multi_sig_from_addr(
			TestConstants::COMMITTEE_ACCOUNT_ADDRESS.to_string(),
			1,
			1,
		)
		.unwrap();
		assert!(a1.is_multi_sig());

		let a2 = Account::from_verification_script(&VerificationScript::from(
			hex::decode(TestConstants::COMMITTEE_ACCOUNT_VERIFICATION_SCRIPT).unwrap(),
		))
		.unwrap();
		assert!(a2.is_multi_sig());

		let a3 = Account::from_verification_script(&VerificationScript::from(
			hex::decode(TestConstants::DEFAULT_ACCOUNT_VERIFICATION_SCRIPT).unwrap(),
		))
		.unwrap();
		assert!(!a3.is_multi_sig());
	}

	#[test]
	fn test_unlock() {
		let mut account = Account::from_address(TestConstants::DEFAULT_ACCOUNT_ADDRESS).unwrap();
		account.is_locked = true;
		assert!(account.is_locked);

		account.is_locked = false;
		assert!(!account.is_locked);
	}

	#[test]
	fn test_is_default() {
		let mut account = Account::from_address(TestConstants::DEFAULT_ACCOUNT_ADDRESS).unwrap();
		let mut wallet = Wallet::new();
		let script_hash: H160 = account.get_script_hash();
		wallet.add_account(account);
		{
			let account = wallet.get_account(&script_hash).unwrap();
			assert!(!account.is_default);
		}
		wallet.set_default_account(script_hash);
		{
			let account = wallet.get_account(&script_hash).unwrap();
			assert!(account.is_default);
		}
	}

	#[test]
	fn calling_get_signing_threshold_with_single_sig_should_fail() {
		let mut account = Account::from_address(TestConstants::DEFAULT_ACCOUNT_ADDRESS).unwrap();
		let err = account.get_signing_threshold().unwrap_err();
		assert_eq!(
			err,
			ProviderError::IllegalState(format!(
				"Cannot get signing threshold from account {}",
				TestConstants::DEFAULT_ACCOUNT_ADDRESS
			))
		);
	}

	#[test]
	fn calling_get_nr_of_participants_with_single_sig_should_fail() {
		let mut account = Account::from_address(TestConstants::DEFAULT_ACCOUNT_ADDRESS).unwrap();
		let err = account.get_nr_of_participants().unwrap_err();
		assert_eq!(
			err,
			ProviderError::IllegalState(format!(
				"Cannot get signing threshold from account {}",
				TestConstants::DEFAULT_ACCOUNT_ADDRESS
			))
		);
	}
}
