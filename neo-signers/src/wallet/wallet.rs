use crate::{
	wallet::{nep6wallet::NEP6Wallet, wallet_error::WalletError},
	NEP6Account, NEP6Contract, NEP6Parameter, Signer,
};
use async_trait::async_trait;
use neo_crypto::keys::Secp256r1Signature;
use neo_providers::{
	core::{
		account::{Account, AccountTrait},
		transaction::{
			transaction::Transaction, verification_script::VerificationScript, witness,
			witness::Witness,
		},
		wallet::WalletTrait,
	},
	Middleware,
};
use neo_types::{
	address::{Address, AddressExtension},
	address_or_scripthash::AddressOrScriptHash,
	contract_parameter_type::ContractParameterType,
	ScryptParamsDef, *,
};
use primitive_types::H160;
use serde_derive::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::Write, path::PathBuf, str::FromStr};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
	pub name: String,
	pub version: String,
	pub scrypt_params: ScryptParamsDef,
	#[serde(deserialize_with = "deserialize_hash_map_h160_account")]
	#[serde(serialize_with = "serialize_hash_map_h160_account")]
	pub accounts: HashMap<H160, Account>,
	#[serde(deserialize_with = "deserialize_script_hash")]
	#[serde(serialize_with = "serialize_script_hash")]
	pub(crate) default_account: H160,
}

impl WalletTrait for Wallet {
	type Account = Account;

	fn name(&self) -> &String {
		&self.name
	}

	fn version(&self) -> &String {
		&self.version
	}

	fn scrypt_params(&self) -> &ScryptParamsDef {
		&self.scrypt_params
	}

	fn accounts(&self) -> &HashMap<H160, Self::Account> {
		&self.accounts
	}

	fn default_account(&self) -> &Account {
		&self.accounts[&self.default_account]
	}

	fn set_name(&mut self, name: String) {
		self.name = name;
	}

	fn set_version(&mut self, version: String) {
		self.version = version;
	}

	fn set_scrypt_params(&mut self, params: ScryptParamsDef) {
		self.scrypt_params = params;
	}

	fn set_default_account(&mut self, default_account: H160) {
		self.default_account = default_account;
	}

	fn add_account(&mut self, account: Self::Account) {
		self.accounts.insert(account.get_script_hash().clone(), account);
	}

	fn remove_account(&mut self, hash: &H160) -> Option<Self::Account> {
		self.accounts.remove(hash)
	}
}

impl Wallet {
	pub const DEFAULT_WALLET_NAME: &'static str = "NeoRustWallet";
	pub const CURRENT_VERSION: &'static str = "3.0";

	pub fn new() -> Self {
		Self {
			name: "MyWallet".to_string(),
			version: "1.0".to_string(),
			scrypt_params: ScryptParamsDef::default(),
			accounts: HashMap::new(),
			default_account: H160::default(),
		}
	}

	// pub fn set_name(&mut self, name: &str) {
	// 	self.name = name.to_string();
	// }

	// pub fn add_account(&mut self, account: Account) {
	// 	self.accounts.insert(account.get_script_hash().clone(), account);
	// }

	// pub fn set_default_account(&mut self, script_hash: H160) {
	// 	self.default_account = script_hash;
	// }

	// Serialization methods

	pub fn to_nep6(&self) -> Result<NEP6Wallet, WalletError> {
		let accounts =
			self.accounts.values().filter_map(|a| Wallet::from_account(a).ok()).collect();

		Ok(NEP6Wallet {
			name: self.name.clone(),
			version: self.version.clone(),
			scrypt: self.scrypt_params.clone(),
			accounts,
			extra: None,
		})
	}

	pub fn from_nep6(nep6: NEP6Wallet) -> Result<Self, WalletError> {
		let accounts = nep6
			.accounts()
			.into_iter()
			.filter_map(|v| Wallet::to_account(v).ok())
			.collect::<Vec<_>>();

		let default_account = nep6
			.accounts()
			.iter()
			.find(|a| a.is_default)
			.map(|a| a.address())
			.ok_or(WalletError::NoDefaultAccount)
			.unwrap();

		Ok(Self {
			name: nep6.name().clone(),
			version: nep6.version().clone(),
			scrypt_params: nep6.scrypt().clone(),
			accounts: accounts.into_iter().map(|a| (a.get_script_hash().clone(), a)).collect(),
			default_account: default_account.to_script_hash().unwrap(),
		})
	}

	fn to_account(nep6_account: &NEP6Account) -> Result<Account, WalletError> {
		let (verification_script, signing_threshold, nr_of_participants) =
			match nep6_account.contract {
				Some(ref contract) if contract.script.is_some() => {
					let script = contract.script.clone().unwrap();
					let verification_script = VerificationScript::from(script.as_bytes().to_vec());
					let signing_threshold = if verification_script.is_multi_sig() {
						Some(verification_script.get_signing_threshold().unwrap())
					} else {
						None
					};
					let nr_of_participants = if verification_script.is_multi_sig() {
						Some(verification_script.get_nr_of_accounts().unwrap())
					} else {
						None
					};
					(Some(verification_script), signing_threshold, nr_of_participants)
				},
				_ => (None, None, None),
			};

		Ok(Account {
			address_or_scripthash: AddressOrScriptHash::Address(nep6_account.address.clone()),
			label: nep6_account.label.clone(),
			verification_script,
			is_locked: nep6_account.lock,
			encrypted_private_key: nep6_account.key.clone(),
			signing_threshold: signing_threshold.map(|x| x as u32),
			nr_of_participants: nr_of_participants.map(|x| x as u32),
			..Default::default()
		})
	}

	// pub async fn get_nep17_balances(&self) -> Result<HashMap<H160, u32>, WalletError> {
	// 	let balances = HTTP_PROVIDER
	// 		.get_nep17_balances(self.get_script_hash().clone())
	// 		.await
	// 		.unwrap();
	// 	let mut nep17_balances = HashMap::new();
	// 	for balance in balances.balances {
	// 		nep17_balances.insert(balance.asset_hash, u32::from_str(&balance.amount).unwrap());
	// 	}
	// 	Ok(nep17_balances)
	// }

	fn from_account(account: &Account) -> Result<NEP6Account, WalletError> {
		if account.key_pair.is_some() && account.encrypted_private_key.is_none() {
			return Err(WalletError::AccountState(
				"Account private key is decrypted but not encrypted".to_string(),
			))
		}

		let contract = match &account.verification_script {
			Some(script) => {
				let parameters = if script.is_multi_sig() {
					let threshold = script.get_signing_threshold().unwrap();
					let nr_accounts = script.get_nr_of_accounts().unwrap();
					(0..nr_accounts)
						.map(|i| NEP6Parameter {
							param_name: format!("signature{}", i),
							param_type: ContractParameterType::Signature,
						})
						.collect()
				} else if script.is_single_sig() {
					vec![NEP6Parameter {
						param_name: "signature".to_string(),
						param_type: ContractParameterType::Signature,
					}]
				} else {
					vec![]
				};

				Some(NEP6Contract {
					script: Some(script.script().to_base64()),
					nep6_parameters: parameters,
					is_deployed: false,
				})
			},
			None => None,
		};

		Ok(NEP6Account {
			address: account.address_or_scripthash.address(),
			label: account.label.clone(),
			is_default: false, // TODO
			lock: account.is_locked,
			key: account.encrypted_private_key.clone(),
			contract,
			extra: None,
		})
	}

	pub fn save_to_file(&self, path: PathBuf) -> Result<(), WalletError> {
		// Convert wallet to NEP6
		let nep6 = self.to_nep6().unwrap();

		// Encode as JSON
		let json = serde_json::to_string(&nep6).unwrap();

		// Write to file at path
		let mut file = File::create(path).unwrap();
		file.write_all(json.as_bytes()).unwrap();

		Ok(())
	}

	pub fn get_account(&self, script_hash: &H160) -> Option<&Account> {
		self.accounts.get(script_hash)
	}

	pub fn remove_account(&mut self, script_hash: &H160) -> bool {
		self.accounts.remove(script_hash).is_some()
	}

	pub fn encrypt_accounts(&mut self, password: &str) {
		for account in self.accounts.values_mut() {
			account.encrypt_private_key(password);
		}
	}
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Signer for Wallet {
	type Error = WalletError;
	async fn sign_message<S: Send + Sync + AsRef<[u8]>>(
		&self,
		message: S,
	) -> Result<Secp256r1Signature, Self::Error> {
		let message = message.as_ref();
		let binding = hash_message(message);
		let message_hash = binding.as_bytes();
		self.default_account()
			.clone()
			.key_pair()
			.clone()
			.unwrap()
			.private_key()
			.sign_tx(message_hash)
			.map_err(|e| WalletError::NoKeyPair)
	}

	async fn get_witness(&self, tx: &Transaction) -> Result<Witness, Self::Error> {
		let mut tx_with_chain = tx.clone();
		if tx_with_chain.network_magic().is_none() {
			// in the case we don't have a network_magic, let's use the signer network magic instead
			tx_with_chain.set_network_magic(self.network_magic());
		}

		Witness::create(tx.get_hash_data()?, &self.default_account().key_pair.clone().unwrap())
			.map_err(|e| WalletError::NoKeyPair)
	}

	fn address(&self) -> Address {
		self.address()
	}
	fn network_magic(&self) -> u32 {
		todo!()
	}

	/// Sets the wallet's network_magic, used in conjunction with EIP-155 signing
	fn with_network_magic<T: Into<u32>>(mut self, network_magic: T) -> Self {
		todo!()
	}
}

#[cfg(test)]
mod tests {
	use crate::Wallet;
	use neo_config::TestConstants;
	use neo_providers::core::account::{Account, AccountTrait};

	#[test]
	fn test_is_default() {
		let account = Account::from_address(TestConstants::DEFAULT_ACCOUNT_ADDRESS).unwrap();
		let mut wallet = Wallet::create().unwrap();
		wallet.add_account(account).unwrap();

		// assert!(!account.is_default);

		wallet.set_default_account(account.address_or_scripthash.script_hash());
		// assert!(account.is_default);
	}

	// #[test]
	// fn test_wallet_link() {
	// 	let account = Account::from_address(TestConstants::DEFAULT_ACCOUNT_ADDRESS).unwrap();
	// 	let wallet = Wallet::create().unwrap();
	//
	// 	assert!(account.wallet.is_none());
	//
	// 	wallet.add_account(account).unwrap();
	// 	assert_eq!(account.wallet.as_ref().unwrap().as_ptr(), wallet.as_ptr());
	// }

	#[test]
	fn test_create_default_wallet() {
		let wallet = Wallet::create().unwrap();

		assert_eq!(wallet.name, "NeoRustWallet");
		assert_eq!(wallet.version, Wallet::CURRENT_VERSION);
		assert!(!wallet.accounts.is_empty());
	}

	#[test]
	fn test_create_wallet_with_accounts() {
		let account1 = Account::create().unwrap();
		let account2 = Account::create().unwrap();

		let wallet = Wallet::with_accounts(vec![account1, account2]).unwrap();

		assert_eq!(wallet.default_account.as_ref().unwrap(), &account1);
		assert_eq!(wallet.accounts.len(), 2);
		assert!(wallet.accounts.contains(&account1));
		assert!(wallet.accounts.contains(&account2));
	}

	#[test]
	fn test_is_default_account() {
		let account = Account::create().unwrap();
		let mut wallet = Wallet::with_accounts(vec![account]).unwrap();

		assert!(wallet.is_default(&account));
	}

	#[test]
	fn test_add_account() {
		let account = Account::create().unwrap();
		let mut wallet = Wallet::create().unwrap();

		wallet.add_account(account).unwrap();

		assert_eq!(wallet.accounts.len(), 2);
		assert_eq!(wallet.get_account(&account.script_hash().unwrap()), Some(&account));
	}

	#[test]
	fn test_encrypt_wallet() {
		let mut wallet = Wallet::create().unwrap();
		wallet.add_account(Account::create().unwrap());

		assert!(wallet.accounts[0].key_pair().is_some());
		assert!(wallet.accounts[1].key_pair().is_some());

		wallet.encrypt_all("pw").unwrap();

		assert!(wallet.accounts[0].key_pair().is_none());
		assert!(wallet.accounts[1].key_pair().is_none());
	}
}
