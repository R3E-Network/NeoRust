use async_trait::async_trait;
use primitive_types::H160;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use neo_builder::{AccountSigner, TransactionBuilder};
use neo_clients::{APITrait, JsonRpcProvider, RpcClient};
use crate::{ContractError, SmartContractTrait};
use neo_protocol::{Account, AccountTrait};
use neo_common::{deserialize_script_hash, serialize_script_hash};
use neo_types::{ContractParameter, ScriptHash};

/// Flamingo Finance contract interface for Neo N3
///
/// Flamingo Finance is a DeFi platform on Neo N3 offering trading, earning, and borrowing services.
/// This contract interface provides methods to interact with the Flamingo Finance smart contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlamingoContract<'a, P: JsonRpcProvider> {
	#[serde(deserialize_with = "deserialize_script_hash")]
	#[serde(serialize_with = "serialize_script_hash")]
	script_hash: ScriptHash,
	#[serde(skip)]
	provider: Option<&'a RpcClient<P>>,
}

impl<'a, P: JsonRpcProvider + 'static> FlamingoContract<'a, P> {
	/// The script hash of the Flamingo Finance contract on Neo N3 MainNet
	pub const CONTRACT_HASH: &'static str = "f970f4cddcd087ab5d8a5697a32b3cfd32c8b465";

	// Method constants
	/// Method name for swapping tokens
	pub const SWAP: &'static str = "swap";
	/// Method name for adding liquidity
	pub const ADD_LIQUIDITY: &'static str = "addLiquidity";
	/// Method name for removing liquidity
	pub const REMOVE_LIQUIDITY: &'static str = "removeLiquidity";
	/// Method name for staking tokens
	pub const STAKE: &'static str = "stake";
	/// Method name for claiming rewards
	pub const CLAIM_REWARDS: &'static str = "claimRewards";

	/// Creates a new FlamingoContract instance with the default contract hash
	///
	/// # Arguments
	///
	/// * `provider` - An optional reference to an RPC client
	///
	/// # Returns
	///
	/// A new FlamingoContract instance
	pub fn new(provider: Option<&'a RpcClient<P>>) -> Self {
		Self { script_hash: ScriptHash::from_str(Self::CONTRACT_HASH).unwrap(), provider }
	}

	/// Creates a new FlamingoContract instance with a custom script hash
	///
	/// # Arguments
	///
	/// * `script_hash` - The script hash of the Flamingo Finance contract
	/// * `provider` - An optional reference to an RPC client
	///
	/// # Returns
	///
	/// A new FlamingoContract instance
	pub fn with_script_hash(script_hash: ScriptHash, provider: Option<&'a RpcClient<P>>) -> Self {
		Self { script_hash, provider }
	}

	/// Swaps tokens on Flamingo Finance
	///
	/// # Arguments
	///
	/// * `path` - The path of tokens to swap through
	/// * `amount_in` - The amount of tokens to swap from
	/// * `min_amount_out` - The minimum amount of tokens to receive
	/// * `account` - The account that will sign the transaction
	/// * `deadline` - The deadline for the transaction
	///
	/// # Returns
	///
	/// A transaction builder that can be used to build and sign the transaction
	pub async fn swap(
		&self,
		path: Vec<H160>,
		amount_in: u64,
		min_amount_out: u64,
		account: &Account<W>,
		deadline: Option<u64>,
	) -> Result<TransactionBuilder<'_>, ContractError>
	where
		W: Clone + Debug + Send + Sync,
	{
		let params = vec![
			ContractParameter::array(path.iter().map(|p| p.into()).collect()),
			ContractParameter::integer(amount_in),
			ContractParameter::integer(min_amount_out),
			ContractParameter::integer(deadline.unwrap_or_default()),
		];

		let mut builder = self.invoke_function(Self::SWAP, params).await?;
		builder.set_signers(vec![AccountSigner::called_by_entry_hash160(account.address_or_scripthash().script_hash()).unwrap().into()]);

		Ok(builder)
	}

	/// Adds liquidity to a Flamingo Finance liquidity pool
	///
	/// # Arguments
	///
	/// * `token_a` - The script hash of the first token
	/// * `token_b` - The script hash of the second token
	/// * `amount_a_desired` - The desired amount of the first token to add
	/// * `amount_b_desired` - The desired amount of the second token to add
	/// * `amount_a_min` - The minimum amount of the first token to add
	/// * `amount_b_min` - The minimum amount of the second token to add
	/// * `account` - The account that will sign the transaction
	/// * `deadline` - The deadline for the transaction
	///
	/// # Returns
	///
	/// A transaction builder that can be used to build and sign the transaction
	pub async fn add_liquidity(
		&self,
		token_a: H160,
		token_b: H160,
		amount_a_desired: u64,
		amount_b_desired: u64,
		amount_a_min: u64,
		amount_b_min: u64,
		account: &Account<W>,
		deadline: Option<u64>,
	) -> Result<TransactionBuilder<'_>, ContractError>
	where
		W: Clone + Debug + Send + Sync,
	{
		let params = vec![
			token_a.into(),
			token_b.into(),
			ContractParameter::integer(amount_a_desired),
			ContractParameter::integer(amount_b_desired),
			ContractParameter::integer(amount_a_min),
			ContractParameter::integer(amount_b_min),
			ContractParameter::integer(deadline.unwrap_or_default()),
		];

		let mut builder = self.invoke_function(Self::ADD_LIQUIDITY, params).await?;
		builder.set_signers(vec![AccountSigner::called_by_entry_hash160(account.address_or_scripthash().script_hash()).unwrap().into()]);

		Ok(builder)
	}

	/// Removes liquidity from a Flamingo Finance liquidity pool
	///
	/// # Arguments
	///
	/// * `token_a` - The script hash of the first token
	/// * `token_b` - The script hash of the second token
	/// * `liquidity` - The amount of liquidity tokens to remove
	/// * `amount_a_min` - The minimum amount of the first token to remove
	/// * `amount_b_min` - The minimum amount of the second token to remove
	/// * `account` - The account that will sign the transaction
	/// * `deadline` - The deadline for the transaction
	///
	/// # Returns
	///
	/// A transaction builder that can be used to build and sign the transaction
	pub async fn remove_liquidity(
		&self,
		token_a: H160,
		token_b: H160,
		liquidity: u64,
		amount_a_min: u64,
		amount_b_min: u64,
		account: &Account<W>,
		deadline: Option<u64>,
	) -> Result<TransactionBuilder<'_>, ContractError>
	where
		W: Clone + Debug + Send + Sync,
	{
		let params = vec![
			token_a.into(),
			token_b.into(),
			ContractParameter::integer(liquidity),
			ContractParameter::integer(amount_a_min),
			ContractParameter::integer(amount_b_min),
			to.into(),
			ContractParameter::integer(deadline),
		];

		let mut builder = self.invoke_function(Self::REMOVE_LIQUIDITY, params).await?;
		builder.set_signers(vec![AccountSigner::called_by_entry_hash160(account.address_or_scripthash().script_hash()).unwrap().into()]);

		Ok(builder)
	}

	/// Stakes tokens on Flamingo Finance
	///
	/// # Arguments
	///
	/// * `pid` - The pool ID of the token to stake
	/// * `amount` - The amount of tokens to stake
	/// * `account` - The account that will sign the transaction
	///
	/// # Returns
	///
	/// A transaction builder that can be used to build and sign the transaction
	pub async fn stake<W>(
		&self,
		pid: i32,
		amount: i64,
		account: &Account<W>,
	) -> Result<TransactionBuilder<'_>, ContractError>
	where
		W: Clone + Debug + Send + Sync,
	{
		let params = vec![pid.into(), ContractParameter::integer(amount)];

		let mut builder = self.invoke_function(Self::STAKE, params).await?;
		builder.set_signers(vec![AccountSigner::called_by_entry_hash160(account.address_or_scripthash().script_hash()).unwrap().into()]);

		Ok(builder)
	}

	/// Claims rewards from Flamingo Finance
	///
	/// # Arguments
	///
	/// * `account` - The account that will sign the transaction
	///
	/// # Returns
	///
	/// A transaction builder that can be used to build and sign the transaction
	pub async fn claim_rewards<W>(
		&self,
		account: &Account<W>,
	) -> Result<TransactionBuilder<'_>, ContractError>
	where
		W: Clone + Debug + Send + Sync,
	{
		let mut builder = self.invoke_function(Self::CLAIM_REWARDS, vec![]).await?;
		builder.set_signers(vec![AccountSigner::called_by_entry_hash160(account.address_or_scripthash().script_hash()).unwrap().into()]);

		Ok(builder)
	}

	pub async fn get_user_info<W>(
		&self,
		token: &H160,
		account: &Account<W>,
	) -> Result<FlamingoUserInfo, ContractError> {
		let params = vec![
			token.into(),
			account.address_or_scripthash().script_hash().into(),
		];
		let result = self
			.invoke_function("getUserInfo", params)
			.await?
			.stack
			.get(0)
			.ok_or(ContractError::InvalidResultError(
				"No result returned from getUserInfo".to_string(),
			))?;

		FlamingoUserInfo::try_from(result.clone())
			.map_err(|e| ContractError::InvalidResultError(format!("Failed to parse user info: {}", e)))
	}

	pub async fn get_pool_info<W>(
		&self,
		token: &H160,
		account: &Account<W>,
	) -> Result<FlamingoPoolInfo, ContractError> {
		let params = vec![
			token.into(),
			account.address_or_scripthash().script_hash().into(),
		];
		let result = self
			.invoke_function("getPoolInfo", params)
			.await?
			.stack
			.get(0)
			.ok_or(ContractError::InvalidResultError(
				"No result returned from getPoolInfo".to_string(),
			))?;

		FlamingoPoolInfo::try_from(result.clone())
			.map_err(|e| ContractError::InvalidResultError(format!("Failed to parse pool info: {}", e)))
	}
}

#[async_trait]
impl<'a, P: JsonRpcProvider> SmartContractTrait<'a> for FlamingoContract<'a, P> {
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
