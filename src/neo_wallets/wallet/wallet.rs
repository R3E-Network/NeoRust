use std::{
	borrow::Cow,
	collections::HashMap,
	fmt,
	path::Path,
	str::FromStr,
	sync::Arc,
};

use ecdsa::SigningKey;
use p256::NistP256;
use primitive_types::H160;
use serde_derive::{Deserialize, Serialize};
#[cfg(feature = "tokio")]
use tokio::fs;

use crate::{
	neo_crypto::{key_pair::KeyPair, keys::Secp256r1PrivateKey},
	neo_error::TypeError,
	neo_types::{Address, ScryptParamsDef},
	neo_utils::constants,
	neo_wallets::{
		account_trait::AccountTrait,
		wallet_trait::WalletTrait,
	},
};

use super::wallet_error::WalletError;

#[cfg(feature = "wallet-standard")]
use wasm_bindgen::prelude::*;

/// Account type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
	/// The account address
	pub address: String,
	/// The account label
	pub label: String,
	/// Whether this is the default account
	pub is_default: bool,
	/// Whether this account is locked
	pub lock: bool,
	/// The account key
	pub key: Option<String>,
	/// Contract hash
	pub contract: Option<Contract>,
	#[serde(skip_serializing_if = "Option::is_none")]
	/// Extra data associated with this account
	pub extra: Option<HashMap<String, String>>,
}

/// Account contract object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
	/// The script
	pub script: String,
	/// The account parameters
	pub parameters: Vec<ContractParameter>,
	/// Whether this contract is deployed
	pub deployed: bool,
}

/// Contract parameter type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractParameter {
	/// The parameter name
	pub name: Option<String>,
	/// The parameter type
	#[serde(rename = "type")]
	pub param_type: String,
}

/// Core wallet implementation for Neo N3
///
/// This struct provides the core functionality for managing Neo N3 wallets,
/// including account management, key handling, and transaction signing.
/// The implementation supports different feature levels through feature flags.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
	/// The wallet name
	pub name: String,
	/// The wallet version
	pub version: String,
	/// The wallet scrypt parameters
	pub scrypt: ScryptParamsDef,
	/// The wallet accounts
	pub accounts: Vec<Account>,
	/// Additional wallet data
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
		&self.scrypt
	}

	fn accounts(&self) -> Vec<Self::Account> {
		self.accounts.clone()
	}

	fn default_account(&self) -> &Account {
		self.accounts.iter().find(|a| a.is_default).unwrap()
	}

	fn set_name(&mut self, name: String) {
		self.name = name;
	}

	fn set_version(&mut self, version: String) {
		self.version = version;
	}

	fn set_scrypt_params(&mut self, params: ScryptParamsDef) {
		self.scrypt = params;
	}

	fn set_default_account(&mut self, default_account: H160) {
		if let Some(account) = self.accounts.iter_mut().find(|a| a.address == default_account.to_string()) {
			account.is_default = true;
		}
	}

	fn add_account(&mut self, account: Self::Account) {
		self.accounts.push(account);
	}

	fn remove_account(&mut self, hash: &H160) -> Option<Self::Account> {
		self.accounts.iter().find(|a| a.address == hash.to_string()).cloned()
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

		Ok(Self {
			name: Self::DEFAULT_WALLET_NAME.to_string(),
			version: Self::CURRENT_VERSION.to_string(),
			scrypt: ScryptParamsDef::default(),
			accounts: vec![account_with_default],
			extra: None,
		})
	}

	/// Creates a new wallet instance with no accounts.
	pub fn default() -> Self {
		Self {
			name: Self::DEFAULT_WALLET_NAME.to_string(),
			version: Self::CURRENT_VERSION.to_string(),
			scrypt: ScryptParamsDef::default(),
			accounts: Vec::new(),
			extra: None,
		}
	}

	// Core functionality - Available with any wallet feature

	/// Gets an account by its script hash
	pub fn get_account(&self, script_hash: &H160) -> Option<&Account> {
		self.accounts.iter().find(|a| a.address == script_hash.to_string())
	}

	/// Removes an account from the wallet
	pub fn remove_account(&mut self, script_hash: &H160) -> bool {
		self.accounts.iter().position(|a| a.address == script_hash.to_string()).map(|i| self.accounts.remove(i)).is_some()
	}

	/// Gets all accounts in the wallet
	pub fn get_accounts(&self) -> Vec<&Account> {
		self.accounts.iter().collect()
	}

	/// Creates a new account in the wallet
	pub fn create_account(&mut self) -> Result<&Account, WalletError> {
		let account = Account::create()?;
		self.accounts.push(account);
		Ok(self.accounts.last().unwrap())
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
		constants::network_magic::MAINNET
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
			for account in self.accounts.iter_mut() {
				if let Some(private_key) = account.key.as_ref() {
					if let Ok(encrypted_key) = Cryptography::encrypt_private_key(
						private_key,
						password,
						self.scrypt.n,
						self.scrypt.r,
						self.scrypt.p,
					) {
						account.key = Some(encrypted_key);
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
		for account in self.accounts.iter() {
			if let Some(encrypted_key) = account.key.as_ref() {
				return Cryptography::verify_password(
					encrypted_key,
					password,
					self.scrypt.n,
					self.scrypt.r,
					self.scrypt.p,
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
		wallet.add_account(account);
		assert_eq!(wallet.accounts.len(), 1);
		assert!(wallet.accounts.contains(&account));
	}

	#[cfg(feature = "wallet-secure")]
	#[test]
	fn test_encrypt_wallet() {
		let mut wallet = Wallet::new().unwrap();
		let password = "password123";
		wallet.encrypt_accounts(password);

		// Check that all accounts are encrypted
		for account in wallet.accounts.iter() {
			assert!(account.key.is_some());
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
