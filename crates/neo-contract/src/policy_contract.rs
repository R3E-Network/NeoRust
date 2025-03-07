use async_trait::async_trait;
use primitive_types::H160;
use serde::{Deserialize, Serialize};

use neo_builder::TransactionBuilder;
use neo_clients::{JsonRpcProvider, RpcClient};
use crate::{traits::SmartContractTrait, ContractError};
use neo_common::{deserialize_script_hash, serialize_script_hash};
use neo_types::{
	ScriptHash, ScriptHashExtension,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyContract<'a, P: JsonRpcProvider> {
	#[serde(deserialize_with = "deserialize_script_hash")]
	#[serde(serialize_with = "serialize_script_hash")]
	script_hash: ScriptHash,
	#[serde(skip)]
	provider: Option<&'a RpcClient<P>>,
}

impl<'a, P: JsonRpcProvider + 'static> PolicyContract<'a, P> {
	pub const NAME: &'static str = "PolicyContract";
	pub const SET_FEE_PER_BYTE: &'static str = "setFeePerByte";
	pub const SET_EXEC_FEE_FACTOR: &'static str = "setExecFeeFactor";
	pub const SET_STORAGE_PRICE: &'static str = "setStoragePrice";
	pub const BLOCK_ACCOUNT: &'static str = "blockAccount";
	pub const UNBLOCK_ACCOUNT: &'static str = "unblockAccount";
	pub const SET_GAS_PER_BLOCK: &'static str = "setGasPerBlock";
	pub const SET_CANDIDATE_FEE: &'static str = "setRegisterPrice";
	// pub const SCRIPT_HASH: H160 = Self::calc_native_contract_hash(Self::NAME).unwrap();

	pub fn new(provider: Option<&'a RpcClient<P>>) -> Self {
		Self { script_hash: Self::calc_native_contract_hash(Self::NAME).unwrap(), provider }
	}

	pub async fn get_fee_per_byte(&self) -> Result<i32, ContractError> {
		self.call_function_returning_int("getFeePerByte", vec![]).await
	}

	pub async fn get_exec_fee_factor(&self) -> Result<i32, ContractError> {
		self.call_function_returning_int("getExecFeeFactor", vec![]).await
	}

	pub async fn get_storage_price(&self) -> Result<i32, ContractError> {
		self.call_function_returning_int("getStoragePrice", vec![]).await
	}

	pub async fn is_blocked(&self, script_hash: &H160) -> Result<bool, ContractError> {
		self.call_function_returning_bool("isBlocked", vec![script_hash.into()]).await
	}

	// State modifying methods

	pub async fn set_fee_per_byte(&self, fee: i32) -> Result<TransactionBuilder<'_>, ContractError> {
		self.invoke_function(Self::SET_FEE_PER_BYTE, vec![fee.into()]).await
	}

	pub async fn set_exec_fee_factor(
		&self,
		fee_factor: i32,
	) -> Result<TransactionBuilder<'_>, ContractError> {
		self.invoke_function(Self::SET_EXEC_FEE_FACTOR, vec![fee_factor.into()]).await
	}

	pub async fn set_storage_price(
		&self,
		price: i32,
	) -> Result<TransactionBuilder<'_>, ContractError> {
		self.invoke_function(Self::SET_STORAGE_PRICE, vec![price.into()]).await
	}

	pub async fn block_account(
		&self,
		account: &H160,
	) -> Result<TransactionBuilder<'_>, ContractError> {
		self.invoke_function(Self::BLOCK_ACCOUNT, vec![account.into()]).await
	}

	pub async fn block_account_address(
		&self,
		address: &str,
	) -> Result<TransactionBuilder<'_>, ContractError> {
		let account = ScriptHash::from_address(address).unwrap();
		self.block_account(&account).await
	}

	pub async fn unblock_account(
		&self,
		account: &H160,
	) -> Result<TransactionBuilder<'_>, ContractError> {
		self.invoke_function(Self::UNBLOCK_ACCOUNT, vec![account.into()]).await
	}

	pub async fn unblock_account_address(
		&self,
		address: &str,
	) -> Result<TransactionBuilder<'_>, ContractError> {
		let account = ScriptHash::from_address(address).unwrap();
		self.unblock_account(&account).await
	}

	pub async fn set_gas_per_block(
		&self,
		gas: i32,
	) -> Result<TransactionBuilder<'_>, ContractError> {
		self.invoke_function(Self::SET_GAS_PER_BLOCK, vec![gas.into()]).await
	}

	pub async fn set_candidate_registration_fee(
		&self,
		fee: i32,
	) -> Result<TransactionBuilder<'_>, ContractError> {
		self.invoke_function(Self::SET_CANDIDATE_FEE, vec![fee.into()]).await
	}
}

#[async_trait]
impl<'a, P: JsonRpcProvider> SmartContractTrait<'a> for PolicyContract<'a, P> {
	type P = P;

	fn script_hash(&self) -> H160 {
		self.script_hash
	}

	fn set_script_hash(&mut self, script_hash: H160) {
		self.script_hash = script_hash;
	}

	fn provider(&self) -> Option<&RpcClient<P>> {
		self.provider
	}
}
