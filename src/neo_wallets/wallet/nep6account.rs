use getset::{Getters, Setters};
use neo::prelude::{
	Account, Address, AddressOrScriptHash, Base64Encode, ContractParameterType, NEP6Contract,
	NEP6Parameter, NeoSerializable, StringExt, VerificationScript, WalletError,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize, Getters, Setters)]
pub struct NEP6Account {
	#[getset(get = "pub", set = "pub")]
	#[serde(rename = "address")]
	pub address: Address,

	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(rename = "label")]
	pub label: Option<String>,

	#[serde(default)]
	#[serde(rename = "isDefault")]
	pub is_default: bool,

	#[serde(rename = "lock")]
	pub lock: bool,

	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(rename = "key")]
	pub key: Option<String>,

	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(rename = "contract")]
	pub contract: Option<NEP6Contract>,

	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(rename = "extra")]
	pub extra: Option<HashMap<String, String>>,
}

impl NEP6Account {
	pub fn new(
		address: Address,
		label: Option<String>,
		is_default: bool,
		lock: bool,
		key: Option<String>,
		contract: Option<NEP6Contract>,
		extra: Option<HashMap<String, String>>,
	) -> Self {
		Self { address, label, is_default, lock, key, contract, extra }
	}

	pub fn from_account(account: &Account) -> Result<NEP6Account, WalletError> {
		if account.key_pair.is_some() && account.encrypted_private_key.is_none() {
			return Err(WalletError::AccountState(
				"Account private key is available but not encrypted.".to_string(),
			))
		}

		let mut parameters = Vec::new();
		if let Some(verification_script) = &account.verification_script {
			if verification_script.is_multi_sig() {
				for i in 0..verification_script.get_nr_of_accounts()? {
					parameters.push(NEP6Parameter {
						param_name: format!("signature{}", i),
						param_type: ContractParameterType::Signature,
					});
				}
			} else if verification_script.is_single_sig() {
				parameters.push(NEP6Parameter {
					param_name: "signature".to_string(),
					param_type: ContractParameterType::Signature,
				});
			}
		}

		let contract = if !parameters.is_empty() {
			Some(NEP6Contract {
				script: Some(account.clone().verification_script.unwrap().to_array().to_base64()),
				is_deployed: false,
				nep6_parameters: parameters,
			})
		} else {
			None
		};

		Ok(NEP6Account {
			address: account.address_or_scripthash.address().clone(),
			label: account.label.clone(),
			is_default: account.is_default,
			lock: account.is_locked,
			key: account.encrypted_private_key.clone(),
			contract,
			extra: None,
		})
	}

	// fn from_account(account: &Account) -> Result<NEP6Account, WalletError> {
	// 	if account.key_pair.is_some() && account.encrypted_private_key.is_none() {
	// 		return Err(WalletError::AccountState(
	// 			"Account private key is decrypted but not encrypted".to_string(),
	// 		))
	// 	}
	//
	// 	let contract = match &account.verification_script {
	// 		Some(script) => {
	// 			let parameters = if script.is_multi_sig() {
	// 				let threshold = script.get_signing_threshold().unwrap();
	// 				let nr_accounts = script.get_nr_of_accounts().unwrap();
	// 				(0..nr_accounts)
	// 					.map(|i| NEP6Parameter {
	// 						param_name: format!("signature{}", i),
	// 						param_type: ContractParameterType::Signature,
	// 					})
	// 					.collect()
	// 			} else if script.is_single_sig() {
	// 				vec![NEP6Parameter {
	// 					param_name: "signature".to_string(),
	// 					param_type: ContractParameterType::Signature,
	// 				}]
	// 			} else {
	// 				vec![]
	// 			};
	//
	// 			Some(NEP6Contract {
	// 				script: Some(script.script().to_base64()),
	// 				nep6_parameters: parameters,
	// 				is_deployed: false,
	// 			})
	// 		},
	// 		None => None,
	// 	};
	//
	// 	Ok(NEP6Account {
	// 		address: account.address_or_scripthash.address(),
	// 		label: account.label.clone(),
	// 		is_default: false, // TODO
	// 		lock: account.is_locked,
	// 		key: account.encrypted_private_key.clone(),
	// 		contract,
	// 		extra: None,
	// 	})
	// }

	pub fn to_account(&self) -> Result<Account, WalletError> {
		let mut verification_script: Option<VerificationScript> = None;
		let mut signing_threshold: Option<u8> = None;
		let mut nr_of_participants: Option<u8> = None;

		if let Some(contract) = &self.contract {
			if contract.script.is_some() {
				verification_script = Some(VerificationScript::from(
					contract.script.clone().unwrap().base64_decoded().unwrap(),
				));

				if verification_script.as_ref().unwrap().is_multi_sig() {
					signing_threshold =
						Some(verification_script.as_ref().unwrap().get_signing_threshold()? as u8);
					nr_of_participants =
						Some(verification_script.as_ref().unwrap().get_nr_of_accounts()? as u8);
				}
			}
		}

		Ok(Account {
			address_or_scripthash: AddressOrScriptHash::Address(self.clone().address),
			label: self.clone().label,
			verification_script,
			is_locked: self.clone().lock,
			encrypted_private_key: self.clone().key,
			signing_threshold: signing_threshold.map(|s| s as u32),
			nr_of_participants: nr_of_participants.map(|s| s as u32),
			..Default::default()
		})
	}

	// fn to_account(nep6_account: &NEP6Account) -> Result<Account, WalletError> {
	// 	let (verification_script, signing_threshold, nr_of_participants) =
	// 		match nep6_account.contract {
	// 			Some(ref contract) if contract.script.is_some() => {
	// 				let script = contract.script.clone().unwrap();
	// 				let verification_script = VerificationScript::from(script.as_bytes().to_vec());
	// 				let signing_threshold = if verification_script.is_multi_sig() {
	// 					Some(verification_script.get_signing_threshold().unwrap())
	// 				} else {
	// 					None
	// 				};
	// 				let nr_of_participants = if verification_script.is_multi_sig() {
	// 					Some(verification_script.get_nr_of_accounts().unwrap())
	// 				} else {
	// 					None
	// 				};
	// 				(Some(verification_script), signing_threshold, nr_of_participants)
	// 			},
	// 			_ => (None, None, None),
	// 		};
	//
	// 	Ok(Account {
	// 		address_or_scripthash: AddressOrScriptHash::Address(nep6_account.address.clone()),
	// 		label: nep6_account.label.clone(),
	// 		verification_script,
	// 		is_locked: nep6_account.lock,
	// 		encrypted_private_key: nep6_account.key.clone(),
	// 		signing_threshold: signing_threshold.map(|x| x as u32),
	// 		nr_of_participants: nr_of_participants.map(|x| x as u32),
	// 		..Default::default()
	// 	})
	// }
}

impl PartialEq for NEP6Account {
	fn eq(&self, other: &Self) -> bool {
		self.address == other.address
	}
}

#[cfg(test)]
mod tests {
	use neo::prelude::{
		AccountTrait, NEP6Account, PrivateKeyExtension, Secp256r1PrivateKey, TestConstants,
	};

	#[test]
	fn test_decrypt_with_standard_scrypt_params() {
		let private_key = Secp256r1PrivateKey::from_bytes(
			&hex::decode(TestConstants::DEFAULT_ACCOUNT_PRIVATE_KEY).unwrap(),
		)
		.unwrap();

		let nep6_account = NEP6Account::new(
			"".to_string(),
			None,
			true,
			false,
			Some(TestConstants::DEFAULT_ACCOUNT_ENCRYPTED_PRIVATE_KEY.to_string()),
			None,
			None,
		);

		let mut account = nep6_account.to_account().unwrap();

		account.decrypt_private_key(TestConstants::DEFAULT_ACCOUNT_PASSWORD).unwrap();

		assert_eq!(account.key_pair.clone().unwrap().private_key.to_vec(), private_key.to_vec());

		// Decrypt again
		account.decrypt_private_key(TestConstants::DEFAULT_ACCOUNT_PASSWORD).unwrap();
		assert_eq!(account.key_pair.clone().unwrap().private_key, private_key);
	}

	#[test]
	fn test_load_account_from_nep6() {
		let data = include_str!("../../../test_resources/wallet/account.json");
		let nep6_account: NEP6Account = serde_json::from_str(data).unwrap();

		let account = nep6_account.to_account().unwrap();

		assert!(!account.is_default);
		assert!(!account.is_locked);
		assert_eq!(
			account.address_or_scripthash().address(),
			TestConstants::DEFAULT_ACCOUNT_ADDRESS
		);
		// ...
	}
}
