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

	fn accounts(&self) -> Vec<Self::Account> {
		self.accounts
			.clone()
			.into_iter()
			.map(|(k, v)| v.clone())
			.collect::<Vec<Self::Account>>()
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
		self.default_account = default_account.clone();
		if let Some(account) = self.accounts.get_mut(&self.default_account) {
			account.is_default = true;
		}
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
	pub const CURRENT_VERSION: &'static str = "1.0";

	pub fn new() -> Self {
		let mut account = Account::create().unwrap();
		account.is_default = true;
		let mut accounts = HashMap::new();
		accounts.insert(account.address_or_scripthash.script_hash(), account.clone());
		Self {
			name: "NeoRustWallet".to_string(),
			version: "1.0".to_string(),
			scrypt_params: ScryptParamsDef::default(),
			accounts,
			default_account: account.clone().address_or_scripthash.script_hash(),
		}
	}

	pub fn default() -> Self {
		Self {
			name: "NeoRustWallet".to_string(),
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

	pub fn to_nep6(&self) -> Result<NEP6Wallet, WalletError> {
		// let accounts =
		// 	self.accounts.values().filter_map(|a| Wallet::from_account(a).ok()).collect();

		Ok(NEP6Wallet {
			name: self.name.clone(),
			version: self.version.clone(),
			scrypt: self.scrypt_params.clone(),
			accounts: self
				.accounts
				.clone()
				.into_iter()
				.map(|(_, account)| NEP6Account::from_account(&account).unwrap())
				.collect::<Vec<NEP6Account>>(),
			extra: None,
		})
	}

	pub fn from_nep6(nep6: NEP6Wallet) -> Result<Self, WalletError> {
		let accounts = nep6
			.accounts()
			.into_iter()
			.filter_map(|v| v.to_account().ok())
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

	pub fn from_account(account: &Account) -> Result<Wallet, WalletError> {
		let mut wallet: Wallet = Wallet::new();
		wallet.add_account(account.clone());
		wallet.set_default_account(account.get_script_hash());
		Ok(wallet)
	}

	/// Adds the given accounts to this wallet, if it doesn't contain an account with the same script hash (address).
	///
	/// # Parameters
	///
	/// * `accounts` - The accounts to add
	///
	/// # Returns
	///
	/// Returns the mutable wallet reference if the accounts were successfully added, or a `WalletError` if an account is already contained in another wallet.
	///
	/// # Errors
	///
	/// Returns a `WalletError::IllegalArgument` error if an account is already contained in another wallet.
	///
	/// # Example
	///
	/// ```
	/// use neo_providers::core::account::Account;
	/// use neo_signers::Wallet;
	///
	/// let account1 = Account::default();
	/// let account2 = Account::default();
	///
	/// let mut wallet = Wallet::from_accounts(vec![account1, account2]).unwrap();
	/// ```
	pub fn from_accounts(accounts: Vec<Account>) -> Result<Wallet, WalletError> {
		// for account in &accounts {
		// 	if account.wallet().is_some() {
		// 		return Err(WalletError::AccountState(format!(
		// 			"The account {} is already contained in a wallet. Please remove this account from its containing wallet before adding it to another wallet.",
		// 			account.address_or_scripthash.address()
		// 		)));
		// 	}
		// }

		let mut wallet: Wallet = Wallet::default();
		for account in &accounts {
			wallet.add_account(account.clone());
			// account.wallet = Some(self);
		}
		wallet.set_default_account(accounts.first().unwrap().get_script_hash());
		Ok(wallet)
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
			account.encrypt_private_key(password).expect("Failed to encrypt private key");
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
	use neo_providers::core::{
		account::{Account, AccountTrait},
		wallet::WalletTrait,
	};

	#[test]
	fn test_is_default() {
		let account = Account::from_address(TestConstants::DEFAULT_ACCOUNT_ADDRESS).unwrap();
		let mut wallet: Wallet = Wallet::new();
		wallet.add_account(account.clone());

		assert!(!account.is_default);

		let hash = account.address_or_scripthash.script_hash();
		wallet.set_default_account(hash.clone());
		assert!(wallet.get_account(&hash).unwrap().is_default);
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
		let wallet: Wallet = Wallet::default();

		assert_eq!(&wallet.name, "NeoRustWallet");
		assert_eq!(&wallet.version, Wallet::CURRENT_VERSION);
		assert_eq!(wallet.accounts.len(), 0usize);
	}

	#[test]
	fn test_create_wallet_with_accounts() {
		let account1 = Account::create().unwrap();
		let account2 = Account::create().unwrap();

		let wallet = Wallet::from_accounts(vec![account1.clone(), account2.clone()]).unwrap();

		assert_eq!(wallet.default_account(), &account1);
		assert_eq!(wallet.accounts.len(), 2);
		assert!(wallet
			.accounts
			.clone()
			.into_iter()
			.any(|(s, _)| s == account1.address_or_scripthash.script_hash()));
		assert!(wallet
			.accounts
			.clone()
			.into_iter()
			.any(|(s, _)| s == account2.address_or_scripthash.script_hash()));
	}

	#[test]
	fn test_is_default_account() {
		let account = Account::create().unwrap();
		let mut wallet = Wallet::from_accounts(vec![account.clone()]).unwrap();

		assert_eq!(wallet.default_account, account.get_script_hash());
	}

	#[test]
	fn test_add_account() {
		let account = Account::create().unwrap();
		let mut wallet: Wallet = Wallet::new();

		wallet.add_account(account.clone());

		assert_eq!(wallet.accounts.len(), 2);
		assert_eq!(
			wallet.get_account(&account.address_or_scripthash.script_hash()),
			Some(&account)
		);
	}

	#[test]
	fn test_encrypt_wallet() {
		let mut wallet: Wallet = Wallet::new();
		wallet.add_account(Account::create().unwrap());

		assert!(wallet.accounts()[0].key_pair().is_some());
		assert!(wallet.accounts()[1].key_pair().is_some());

		wallet.encrypt_accounts("pw");

		assert!(wallet.accounts()[0].key_pair().is_none());
		assert!(wallet.accounts()[1].key_pair().is_none());
	}
}
