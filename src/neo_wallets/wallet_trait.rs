use neo::prelude::{AccountTrait, ScryptParamsDef};
use primitive_types::H160;

pub trait WalletTrait {
	type Account: AccountTrait;

	fn name(&self) -> &String;
	fn version(&self) -> &String;
	fn scrypt_params(&self) -> &ScryptParamsDef;
	fn accounts(&self) -> Vec<Self::Account>;
	fn default_account(&self) -> &Self::Account;
	fn set_name(&mut self, name: String);
	fn set_version(&mut self, version: String);
	fn set_scrypt_params(&mut self, params: ScryptParamsDef);
	fn set_default_account(&mut self, default_account: H160);
	fn add_account(&mut self, account: Self::Account);
	fn remove_account(&mut self, hash: &H160) -> Option<Self::Account>;
}
