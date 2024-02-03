use neo_types::contract_parameter_type::ContractParameterType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NEP6Contract {
	pub script: Option<String>,

	#[serde(rename = "deployed")]
	pub is_deployed: bool,

	#[serde(rename = "parameters")]
	pub nep6_parameters: Vec<NEP6Parameter>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NEP6Parameter {
	#[serde(rename = "name")]
	pub param_name: String,

	pub param_type: ContractParameterType,
}

impl PartialEq for NEP6Contract {
	fn eq(&self, other: &Self) -> bool {
		self.script == other.script
			&& self.nep6_parameters == other.nep6_parameters
			&& self.is_deployed == other.is_deployed
	}
}

#[cfg(test)]
mod tests {
	use crate::NEP6Account;
	use neo_config::TestConstants;
	use neo_crypto::keys::Secp256r1PrivateKey;
	use neo_providers::core::account::Account;

	#[test]
	fn test_decrypt_with_standard_scrypt_params() {
		let private_key = Secp256r1PrivateKey::from_bytes(
			&hex::decode(TestConstants::DEFAULT_ACCOUNT_PRIVATE_KEY).unwrap(),
		)
		.unwrap();

		let nep6_account = NEP6Account::new();
		let mut account = Account::from_nep6_account(nep6_account).unwrap();

		account.decrypt_private_key(TestConstants::DEFAULT_ACCOUNT_PASSWORD).unwrap();

		assert_eq!(account.key_pair.unwrap().private_key, private_key);

		// Decrypt again
		account.decrypt_private_key(TestConstants::DEFAULT_ACCOUNT_PASSWORD).unwrap();
		assert_eq!(account.key_pair.unwrap().private_key, private_key);
	}

	#[test]
	fn test_load_account_from_nep6() {
		let path = "./test_data/account.json";
		let file = std::fs::read_to_string(path).unwrap();
		let nep6_account: NEP6Account = serde_json::from_str(&file).unwrap();

		let account = Account::from_nep6_account(nep6_account).unwrap();

		assert!(!account.is_default);
		assert!(!account.is_locked);
		assert_eq!(account.address_or_scripthash(), TestConstants::DEFAULT_ACCOUNT_ADDRESS);
		// ...
	}
}
