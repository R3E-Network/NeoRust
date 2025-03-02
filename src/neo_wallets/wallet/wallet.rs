use std::{collections::HashMap, path::PathBuf};

#[cfg(feature = "wallet-standard")]
use std::{fs::File, io::Write};

use primitive_types::H160;
use serde_derive::{Deserialize, Serialize};

use neo::prelude::*;

/// Core wallet implementation for Neo N3
///
/// This struct provides the core functionality for managing Neo N3 wallets,
/// including account management, key handling, and transaction signing.
/// The implementation supports different feature levels through feature flags.
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
	/// Additional wallet metadata stored as key-value pairs
	#[serde(skip_serializing_if = "Option::is_none")]
	pub extra: Option<HashMap<String, String>>,
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
			.map(|(_k, v)| v.clone())
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
		// let weak_self = Arc::new(&self);
		// account.set_wallet(Some(Arc::downgrade(weak_self)));
		self.accounts.insert(account.get_script_hash().clone(), account);
	}

	fn remove_account(&mut self, hash: &H160) -> Option<Self::Account> {
		self.accounts.remove(hash)
	}
}

impl Wallet {
	/// The default wallet name.
	pub const DEFAULT_WALLET_NAME: &'static str = "NeoWallet";
	/// The current wallet version.
	pub const CURRENT_VERSION: &'static str = "1.0";

	/// Creates a new wallet instance with a default account.
	pub fn new() -> Result<Self, WalletError> {
		let account = Account::create()?;
		let mut account_with_default = account;
		account_with_default.is_default = true;

		let mut accounts = HashMap::new();
		let script_hash = account_with_default.get_script_hash().clone();
		accounts.insert(script_hash.clone(), account_with_default);

		Ok(Self {
			name: Self::DEFAULT_WALLET_NAME.to_string(),
			version: Self::CURRENT_VERSION.to_string(),
			scrypt_params: ScryptParamsDef::default(),
			accounts,
			default_account: script_hash,
			extra: None,
		})
	}

	/// Creates a new wallet instance with no accounts.
	pub fn default() -> Self {
		Self {
			name: Self::DEFAULT_WALLET_NAME.to_string(),
			version: Self::CURRENT_VERSION.to_string(),
			scrypt_params: ScryptParamsDef::default(),
			accounts: HashMap::new(),
			default_account: H160::zero(),
			extra: None,
		}
	}

	// Core functionality - Available with any wallet feature

	/// Gets an account by its script hash
	pub fn get_account(&self, script_hash: &H160) -> Option<&Account> {
		self.accounts.get(script_hash)
	}

	/// Removes an account from the wallet
	pub fn remove_account(&mut self, script_hash: &H160) -> bool {
		self.accounts.remove(script_hash).is_some()
	}

	/// Gets all accounts in the wallet
	pub fn get_accounts(&self) -> Vec<&Account> {
		self.accounts.values().collect()
	}

	/// Creates a new account in the wallet
	pub fn create_account(&mut self) -> Result<&Account, WalletError> {
		let account = Account::create()?;
		let script_hash = account.get_script_hash().clone();
		self.accounts.insert(script_hash.clone(), account);
		Ok(self.accounts.get(&script_hash).unwrap())
	}

	/// Sets the wallet's network magic number
	pub fn with_network(mut self, network: u32) -> Self {
		self.extra
			.get_or_insert_with(HashMap::new)
			.insert("Network".to_string(), network.to_string());
		self
	}

	/// Gets the wallet's network magic number
	pub fn network(&self) -> u32 {
		if let Some(extra) = &self.extra {
			if let Some(network) = extra.get("Network") {
				if let Ok(network) = network.parse::<u32>() {
					return network;
				}
			}
		}
		NeoConstants::MAGIC_NUMBER_MAINNET
	}

	// NEP-6 Wallet Standard - Only available with wallet-standard feature

	/// Converts a wallet to NEP-6 format
	#[cfg(feature = "wallet-standard")]
	#[cfg_attr(docsrs, doc(cfg(feature = "wallet-standard")))]
	pub fn to_nep6(&self) -> Result<Nep6Wallet, WalletError> {
		// NEP-6 conversion implementation
		unimplemented!("NEP-6 conversion not implemented in this example")
	}

	/// Creates a wallet from NEP-6 format
	#[cfg(feature = "wallet-standard")]
	#[cfg_attr(docsrs, doc(cfg(feature = "wallet-standard")))]
	pub fn from_nep6(nep6: Nep6Wallet) -> Result<Self, WalletError> {
		// NEP-6 conversion implementation
		unimplemented!("NEP-6 conversion not implemented in this example")
	}

	/// Saves the wallet to a file
	#[cfg(feature = "wallet-standard")]
	#[cfg_attr(docsrs, doc(cfg(feature = "wallet-standard")))]
	pub fn save_to_file(&self, path: PathBuf) -> Result<(), WalletError> {
		let json = serde_json::to_string_pretty(self)
			.map_err(|e| WalletError::SerializationError(e.to_string()))?;

		let mut file = File::create(path).map_err(|e| WalletError::IoError(e.to_string()))?;
		file.write_all(json.as_bytes()).map_err(|e| WalletError::IoError(e.to_string()))
	}

	/// Creates a new wallet and saves it to a file
	#[cfg(feature = "wallet-standard")]
	#[cfg_attr(docsrs, doc(cfg(feature = "wallet-standard")))]
	pub fn create(path: &PathBuf, password: &str) -> Result<Self, WalletError> {
		let mut wallet = Self::new()?;

		// Encrypt the wallet accounts if a password is provided
		if !password.is_empty() {
			wallet.encrypt_accounts(password);
		}

		wallet.save_to_file(path.clone())?;
		Ok(wallet)
	}

	/// Opens a wallet from a file
	#[cfg(feature = "wallet-standard")]
	#[cfg_attr(docsrs, doc(cfg(feature = "wallet-standard")))]
	pub fn open(path: &PathBuf, password: &str) -> Result<Self, WalletError> {
		// File loading implementation
		unimplemented!("File loading not implemented in this example")
	}

	// Secure Wallet Features - Only available with wallet-secure feature

	/// Encrypts all accounts in the wallet
	#[cfg(feature = "wallet-secure")]
	#[cfg_attr(docsrs, doc(cfg(feature = "wallet-secure")))]
	pub fn encrypt_accounts(&mut self, password: &str) {
		if !password.is_empty() {
			for account in self.accounts.values_mut() {
				if let Some(private_key) = account.get_private_key() {
					if let Ok(encrypted_key) = Cryptography::encrypt_private_key(
						&private_key.to_string(),
						password,
						self.scrypt_params.n,
						self.scrypt_params.r,
						self.scrypt_params.p,
					) {
						account.update_encrypted_key(encrypted_key);
					}
				}
			}
		}
	}

	/// Verifies the wallet password
	#[cfg(feature = "wallet-secure")]
	#[cfg_attr(docsrs, doc(cfg(feature = "wallet-secure")))]
	pub fn verify_password(&self, password: &str) -> bool {
		// Find an account with an encrypted key
		for account in self.accounts.values() {
			if let Some(encrypted_key) = account.get_encrypted_key() {
				return Cryptography::verify_password(
					encrypted_key,
					password,
					self.scrypt_params.n,
					self.scrypt_params.r,
					self.scrypt_params.p,
				);
			}
		}
		false
	}

	/// Changes the wallet password
	#[cfg(feature = "wallet-secure")]
	#[cfg_attr(docsrs, doc(cfg(feature = "wallet-secure")))]
	pub fn change_password(
		&mut self,
		current_password: &str,
		new_password: &str,
	) -> Result<(), WalletError> {
		if !self.verify_password(current_password) {
			return Err(WalletError::InvalidPassword);
		}
		self.encrypt_accounts(new_password);
		Ok(())
	}

	// Transaction Signing - Only available with transaction feature

	/// Signs a transaction with the wallet's default account
	#[cfg(feature = "transaction")]
	#[cfg_attr(docsrs, doc(cfg(feature = "transaction")))]
	pub async fn sign_transaction<'a, P>(
		&self,
		tx_builder: &'a mut TransactionBuilder<'a, P>,
		account_address: &str,
		password: &str,
	) -> Result<Transaction<'a, P>, WalletError>
	where
		P: JsonRpcProvider + 'static,
	{
		// Transaction signing implementation
		unimplemented!("Transaction signing not implemented in this example")
	}

	/// Gets a witness for a transaction
	#[cfg(feature = "transaction")]
	#[cfg_attr(docsrs, doc(cfg(feature = "transaction")))]
	pub async fn get_witness<'a, P: JsonRpcProvider + 'static>(
		&self,
		tx: &Transaction<'a, P>,
	) -> Result<Witness, WalletError> {
		// Witness creation implementation
		unimplemented!("Witness creation not implemented in this example")
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_create_default_wallet() {
		let wallet = Wallet::new().unwrap();
		assert_eq!(wallet.name, Wallet::DEFAULT_WALLET_NAME);
		assert_eq!(wallet.version, Wallet::CURRENT_VERSION);
		assert_eq!(wallet.accounts.len(), 1);
	}

	#[test]
	fn test_add_account() {
		let mut wallet = Wallet::default();
		let account = Account::create().unwrap();
		let script_hash = account.get_script_hash().clone();
		wallet.add_account(account);
		assert_eq!(wallet.accounts.len(), 1);
		assert!(wallet.accounts.contains_key(&script_hash));
	}

	#[cfg(feature = "wallet-secure")]
	#[test]
	fn test_encrypt_wallet() {
		let mut wallet = Wallet::new().unwrap();
		let password = "password123";
		wallet.encrypt_accounts(password);

		// Check that all accounts are encrypted
		for account in wallet.accounts.values() {
			assert!(account.get_encrypted_key().is_some());
		}
	}

	#[cfg(feature = "wallet-secure")]
	#[test]
	fn test_verify_password() {
		let mut wallet = Wallet::new().unwrap();
		let password = "password123";
		wallet.encrypt_accounts(password);

		assert!(wallet.verify_password(password));
		assert!(!wallet.verify_password("wrong_password"));
	}
}
