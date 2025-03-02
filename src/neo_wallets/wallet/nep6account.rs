use std::collections::HashMap;

use getset::{Getters, Setters};
use serde::{Deserialize, Serialize};

#[cfg(feature = "wallet-standard")]
use wasm_bindgen::prelude::*;

use crate::neo_types::{Address, Base64Encode, StringExt};
use crate::neo_wallets::wallet::wallet_error::WalletError;
use crate::neo_wallets::wallet::nep6contract::{NEP6Contract, NEP6Parameter};
use crate::neo_wallets::wallet::wallet::Account;

/// Represents an account in the NEP-6 format.
#[derive(Clone, Debug, Serialize, Deserialize, Getters, Setters)]
pub struct NEP6Account {
	/// The address of the account.
	#[getset(get = "pub")]
	#[serde(rename = "address")]
	pub address: String,

	/// An optional label for the account.
	#[getset(get = "pub")]
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(rename = "label")]
	pub label: Option<String>,

	/// Indicates whether the account is set as default.
	#[getset(get = "pub")]
	#[serde(default)]
	#[serde(rename = "isDefault")]
	pub is_default: bool,

	/// Indicates whether the account is locked.
	#[getset(get = "pub")]
	#[serde(rename = "lock")]
	pub lock: bool,

	/// An optional private key associated with the account.
	#[getset(get = "pub")]
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(rename = "key")]
	pub key: Option<String>,

	/// An optional NEP-6 contract associated with the account.
	#[getset(get = "pub")]
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(rename = "contract")]
	pub contract: Option<NEP6Contract>,

	/// An optional additional data associated with the account.
	#[getset(get = "pub")]
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(rename = "extra")]
	pub extra: Option<HashMap<String, String>>,
}

impl NEP6Account {
	/// Creates a new NEP-6 account with the given parameters.
	///
	/// # Arguments
	///
	/// * `address` - The address of the account.
	/// * `label` - An optional label for the account.
	/// * `is_default` - Indicates whether the account is set as default.
	/// * `lock` - Indicates whether the account is locked.
	/// * `key` - An optional private key associated with the account.
	/// * `contract` - An optional NEP-6 contract associated with the account.
	/// * `extra` - An optional additional data associated with the account.
	///
	/// # Example
	///
	/// ```
	/// use std::collections::HashMap;
	/// use NeoRust::prelude::{Address, NEP6Account, NEP6Contract};
	///
	/// let address = Address::from("example_address");
	/// let label = Some("My Account".to_string());
	/// let is_default = true;
	/// let lock = false;
	/// let key = Some("example_private_key".to_string());
	/// let contract = Some(NEP6Contract::new());
	/// let extra = Some(HashMap::new());
	///
	/// let account = NEP6Account::new(address, label, is_default, lock, key, contract, extra);
	/// ```
	pub fn new(
		address: String,
		label: Option<String>,
		is_default: bool,
		lock: bool,
		key: Option<String>,
		contract: Option<NEP6Contract>,
		extra: Option<HashMap<String, String>>,
	) -> Self {
		Self { address, label, is_default, lock, key, contract, extra }
	}

	/// Converts a standard Account into a NEP6Account.
	///
	/// # Arguments
	///
	/// * `account` - The account to convert.
	///
	/// # Errors
	///
	/// Returns a `WalletError` if there is an issue converting the account.
	///
	/// # Example
	///
	/// ```
	/// use NeoRust::prelude::{Account, NEP6Account};
	///
	/// let account = Account::default();
	/// let nep6_account = NEP6Account::from_account(&account);
	/// ```
	pub fn from_account(account: &Account) -> Result<NEP6Account, WalletError> {
		// Create parameters for the contract
		let mut parameters = Vec::new();
		
		if let Some(contract) = &account.contract {
			let param = NEP6Parameter {
				param_name: "signature".to_string(),
				param_type: "Signature".to_string(),
			};
			parameters.push(param);
		}

		// Create the NEP6Contract if there's a contract in the account
		let contract = if !parameters.is_empty() && account.contract.is_some() {
			Some(NEP6Contract {
				script: account.contract.as_ref().and_then(|c| c.script.clone()),
				nep6_parameters: parameters,
				is_deployed: account.contract.as_ref().map_or(false, |c| c.deployed),
			})
		} else {
			None
		};

		Ok(NEP6Account {
			address: account.address.clone(),
			label: Some(account.label.clone()),
			is_default: account.is_default,
			lock: account.lock,
			key: account.key.clone(),
			contract,
			extra: account.extra.clone(),
		})
	}

	/// Converts a NEP6Account into a standard Account.
	///
	/// # Errors
	///
	/// Returns a `WalletError` if there is an issue converting the account.
	///
	/// # Example
	///
	/// ```
	/// use NeoRust::prelude::NEP6Account;
	/// let nep6_account = NEP6Account::default();
	/// let account = nep6_account.to_account();
	/// ```
	pub fn to_account(&self) -> Result<Account, WalletError> {
		use crate::neo_wallets::wallet::wallet::{Contract, ContractParameter};
		
		// Create contract if the NEP6Account has one
		let contract = if let Some(nep6_contract) = &self.contract {
			let mut parameters = Vec::new();
			
			for nep6_param in &nep6_contract.nep6_parameters {
				let param = ContractParameter {
					name: Some(nep6_param.param_name.clone()),
					param_type: nep6_param.param_type.clone(),
				};
				parameters.push(param);
			}
			
			Some(Contract {
				script: nep6_contract.script.clone().unwrap_or_default(),
				parameters,
				deployed: nep6_contract.is_deployed,
			})
		} else {
			None
		};

		Ok(Account {
			address: self.address.clone(),
			label: self.label.clone().unwrap_or_else(|| self.address.clone()),
			is_default: self.is_default,
			lock: self.lock,
			key: self.key.clone(),
			contract,
			extra: self.extra.clone(),
		})
	}
}

impl PartialEq for NEP6Account {
	/// Checks if two NEP6Account instances are equal based on their addresses.
	///
	/// # Example
	///
	/// ```
	///
	/// use NeoRust::prelude::NEP6Account;
	///
	/// let account1 = NEP6Account::default();
	/// let account2 = NEP6Account::default();
	/// assert_eq!(account1, account2);
	/// ```
	fn eq(&self, other: &Self) -> bool {
		self.address == other.address
	}
}

impl Default for NEP6Account {
    fn default() -> Self {
        Self {
            address: "NeoDefaultAddress".to_string(),
            label: Some("Default Account".to_string()),
            is_default: false,
            lock: false,
            key: None,
            contract: None,
            extra: None,
        }
    }
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::neo_wallets::wallet::wallet::{Account, Contract, ContractParameter};
	
	#[test]
	fn test_new_nep6_account() {
		let address = "NeoAddress123".to_string();
		let label = Some("My Account".to_string());
		let is_default = true;
		let lock = false;
		let key = Some("encrypted_key".to_string());
		let contract = None;
		let extra = None;
		
		let account = NEP6Account::new(
			address.clone(),
			label.clone(),
			is_default,
			lock,
			key.clone(),
			contract.clone(),
			extra.clone(),
		);
		
		assert_eq!(account.address(), &address);
		assert_eq!(account.label(), &label);
		assert_eq!(account.is_default(), is_default);
		assert_eq!(account.lock(), lock);
		assert_eq!(account.key(), &key);
		assert_eq!(account.contract(), &contract);
		assert_eq!(account.extra(), &extra);
	}
	
	#[test]
	fn test_from_account() {
		let standard_account = Account {
			address: "NeoAddress123".to_string(),
			label: "My Account".to_string(),
			is_default: true,
			lock: false,
			key: Some("encrypted_key".to_string()),
			contract: Some(Contract {
				script: "script".to_string(),
				parameters: vec![
					ContractParameter {
						name: Some("signature".to_string()),
						param_type: "Signature".to_string(),
					}
				],
				deployed: false,
			}),
			extra: None,
		};
		
		let nep6_account = NEP6Account::from_account(&standard_account).unwrap();
		
		assert_eq!(nep6_account.address(), &standard_account.address);
		assert_eq!(nep6_account.label(), &Some(standard_account.label.clone()));
		assert_eq!(nep6_account.is_default(), standard_account.is_default);
		assert_eq!(nep6_account.lock(), standard_account.lock);
		assert_eq!(nep6_account.key(), &standard_account.key);
		assert!(nep6_account.contract().is_some());
	}
	
	#[test]
	fn test_to_account() {
		let nep6_account = NEP6Account {
			address: "NeoAddress123".to_string(),
			label: Some("My Account".to_string()),
			is_default: true,
			lock: false,
			key: Some("encrypted_key".to_string()),
			contract: Some(NEP6Contract {
				script: Some("script".to_string()),
				nep6_parameters: vec![
					NEP6Parameter {
						param_name: "signature".to_string(),
						param_type: "Signature".to_string(),
					}
				],
				is_deployed: false,
			}),
			extra: None,
		};
		
		let standard_account = nep6_account.to_account().unwrap();
		
		assert_eq!(standard_account.address, nep6_account.address);
		assert_eq!(standard_account.label, nep6_account.label.clone().unwrap());
		assert_eq!(standard_account.is_default, nep6_account.is_default);
		assert_eq!(standard_account.lock, nep6_account.lock);
		assert_eq!(standard_account.key, nep6_account.key);
		assert!(standard_account.contract.is_some());
	}
}
