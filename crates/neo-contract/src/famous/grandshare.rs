use async_trait::async_trait;
use primitive_types::H160;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::fmt::Debug;

use neo_builder::{AccountSigner, TransactionBuilder};
use neo_clients::{APITrait, JsonRpcProvider, RpcClient};
use crate::{ContractError, SmartContractTrait};
use neo_protocol::{Account, AccountTrait};
use neo_common::{deserialize_script_hash, serialize_script_hash};
use neo_types::{ScriptHash, ContractParameter, NNSName};

/// GrandShare contract interface for Neo N3
///
/// GrandShare is a governance and funding platform for Neo ecosystem projects.
/// This contract interface provides methods to interact with the GrandShare smart contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrandShareContract<'a, P: JsonRpcProvider> {
	#[serde(deserialize_with = "deserialize_script_hash")]
	#[serde(serialize_with = "serialize_script_hash")]
	script_hash: ScriptHash,
	#[serde(skip)]
	provider: Option<&'a RpcClient<P>>,
}

impl<'a, P: JsonRpcProvider + 'static> GrandShareContract<'a, P> {
	/// The script hash of the GrandShare contract on Neo N3 MainNet
	pub const CONTRACT_HASH: &'static str = "74f2dc36a68fdc4682034178eb2220729231db76";

	// Method constants
	/// Method name for submitting a proposal
	pub const SUBMIT_PROPOSAL: &'static str = "submitProposal";
	/// Method name for voting on a proposal
	pub const VOTE: &'static str = "vote";
	/// Method name for funding a project
	pub const FUND_PROJECT: &'static str = "fundProject";
	/// Method name for claiming funds
	pub const CLAIM_FUNDS: &'static str = "claimFunds";
	/// Method name for claiming gas
	pub const CLAIM_GAS: &'static str = "claimGas";
	/// Method name for claiming NEO
	pub const CLAIM_NEO: &'static str = "claimNeo";
	/// Method name for claiming all
	pub const CLAIM_ALL: &'static str = "claimAll";
	/// Method name for withdrawing
	pub const WITHDRAW: &'static str = "withdraw";

	/// Creates a new GrandShareContract instance with the default contract hash
	///
	/// # Arguments
	///
	/// * `provider` - An optional reference to an RPC client
	///
	/// # Returns
	///
	/// A new GrandShareContract instance
	pub fn new(provider: Option<&'a RpcClient<P>>) -> Self {
		Self { script_hash: ScriptHash::from_str(Self::CONTRACT_HASH).unwrap(), provider }
	}

	/// Creates a new GrandShareContract instance with a custom script hash
	///
	/// # Arguments
	///
	/// * `script_hash` - The script hash of the GrandShare contract
	/// * `provider` - An optional reference to an RPC client
	///
	/// # Returns
	///
	/// A new GrandShareContract instance
	pub fn with_script_hash(script_hash: ScriptHash, provider: Option<&'a RpcClient<P>>) -> Self {
		Self { script_hash, provider }
	}

	/// Submits a proposal to GrandShare
	///
	/// # Arguments
	///
	/// * `title` - The title of the proposal
	/// * `description` - The description of the proposal
	/// * `requested_amount` - The amount of funds requested
	/// * `account` - The account that will sign the transaction
	///
	/// # Returns
	///
	/// A transaction builder that can be used to build and sign the transaction
	pub async fn submit_proposal<W: Clone + Debug + Send + Sync>(
		&self,
		title: &str,
		description: &str,
		requested_amount: i64,
		account: &Account<W>,
	) -> Result<TransactionBuilder<'_>, ContractError> {
		let params =
			vec![title.into(), description.into(), ContractParameter::integer(requested_amount)];

		let mut builder = self.invoke_function(Self::SUBMIT_PROPOSAL, params).await?;
		builder.set_signers(vec![AccountSigner::called_by_entry_hash160(account.address_or_scripthash().script_hash()).unwrap().into()]);

		Ok(builder)
	}

	/// Votes on a proposal in GrandShare
	///
	/// # Arguments
	///
	/// * `proposal_id` - The ID of the proposal to vote on
	/// * `vote_type` - The type of vote (true for yes, false for no)
	/// * `account` - The account that will sign the transaction
	///
	/// # Returns
	///
	/// A transaction builder that can be used to build and sign the transaction
	pub async fn vote<W: Clone + Debug + Send + Sync>(
		&self,
		proposal_id: i32,
		vote_type: bool,
		account: &Account<W>,
	) -> Result<TransactionBuilder<'_>, ContractError> {
		let params = vec![
			ContractParameter::integer(proposal_id.into()),
			ContractParameter::bool(vote_type),
		];

		let mut builder = self.invoke_function(Self::VOTE, params).await?;
		builder.set_signers(vec![AccountSigner::called_by_entry_hash160(account.address_or_scripthash().script_hash()).unwrap().into()]);

		Ok(builder)
	}

	/// Funds a project in GrandShare
	///
	/// # Arguments
	///
	/// * `project_id` - The ID of the project to fund
	/// * `amount` - The amount of funds to provide
	/// * `account` - The account that will sign the transaction
	///
	/// # Returns
	///
	/// A transaction builder that can be used to build and sign the transaction
	pub async fn fund_project<W: Clone + Debug + Send + Sync>(
		&self,
		project_id: i32,
		amount: i64,
		account: &Account<W>,
	) -> Result<TransactionBuilder<'_>, ContractError> {
		let params = vec![
			ContractParameter::integer(project_id.into()),
			ContractParameter::integer(amount),
		];

		let mut builder = self.invoke_function(Self::FUND_PROJECT, params).await?;
		builder.set_signers(vec![AccountSigner::called_by_entry_hash160(account.address_or_scripthash().script_hash()).unwrap().into()]);

		Ok(builder)
	}

	/// Claims funds from a funded project in GrandShare
	///
	/// # Arguments
	///
	/// * `project_id` - The ID of the project to claim funds from
	/// * `account` - The account that will sign the transaction
	///
	/// # Returns
	///
	/// A transaction builder that can be used to build and sign the transaction
	pub async fn claim_funds<W: Clone + Debug + Send + Sync>(
		&self,
		project_id: i32,
		account: &Account<W>,
	) -> Result<TransactionBuilder<'_>, ContractError> {
		let params = vec![ContractParameter::integer(project_id.into())];

		let mut builder = self.invoke_function(Self::CLAIM_FUNDS, params).await?;
		builder.set_signers(vec![AccountSigner::called_by_entry_hash160(account.address_or_scripthash().script_hash()).unwrap().into()]);

		Ok(builder)
	}

	/// Claims gas from the contract
	///
	/// # Arguments
	///
	/// * `account` - The account that will sign the transaction
	///
	/// # Returns
	///
	/// A transaction builder that can be used to build and sign the transaction
	pub async fn claim_gas(
		&self,
		account: &Account,
	) -> Result<TransactionBuilder<'_>, ContractError> {
		let params = vec![];

		let mut builder = self.invoke_function(Self::CLAIM_GAS, params).await?;
		builder.set_signers(vec![AccountSigner::called_by_entry_hash160(account.address_or_scripthash().script_hash()).unwrap().into()]);

		Ok(builder)
	}

	/// Claims NEO from the contract
	///
	/// # Arguments
	///
	/// * `account` - The account that will sign the transaction
	///
	/// # Returns
	///
	/// A transaction builder that can be used to build and sign the transaction
	pub async fn claim_neo(
		&self,
		account: &Account,
	) -> Result<TransactionBuilder<'_>, ContractError> {
		let params = vec![];

		let mut builder = self.invoke_function(Self::CLAIM_NEO, params).await?;
		builder.set_signers(vec![AccountSigner::called_by_entry_hash160(account.address_or_scripthash().script_hash()).unwrap().into()]);

		Ok(builder)
	}

	/// Claims all assets from the contract
	///
	/// # Arguments
	///
	/// * `account` - The account that will sign the transaction
	///
	/// # Returns
	///
	/// A transaction builder that can be used to build and sign the transaction
	pub async fn claim_all<W: Clone + Debug + Send + Sync>(
		&self,
		account: &Account<W>,
	) -> Result<TransactionBuilder<'_>, ContractError> {
		let params = vec![];

		let mut builder = self.invoke_function(Self::CLAIM_ALL, params).await?;
		builder.set_signers(vec![AccountSigner::called_by_entry_hash160(account.address_or_scripthash().script_hash()).unwrap().into()]);

		Ok(builder)
	}

	/// Withdraws assets from the contract
	///
	/// # Arguments
	///
	/// * `account` - The account that will sign the transaction
	///
	/// # Returns
	///
	/// A transaction builder that can be used to build and sign the transaction
	pub async fn withdraw<W: Clone + Debug + Send + Sync>(
		&self,
		account: &Account<W>,
	) -> Result<TransactionBuilder<'_>, ContractError> {
		let params = vec![];

		let mut builder = self.invoke_function(Self::WITHDRAW, params).await?;
		builder.set_signers(vec![AccountSigner::called_by_entry_hash160(account.address_or_scripthash().script_hash()).unwrap().into()]);

		Ok(builder)
	}

	pub async fn get_proposal<W: Clone + Debug + Send + Sync>(
		&self,
		proposal_id: i32,
		account: &Account<W>,
	) -> Result<Proposal, ContractError> {
		// ... existing code ...
	}

	pub async fn get_project<W: Clone + Debug + Send + Sync>(
		&self,
		project_id: i32,
		account: &Account<W>,
	) -> Result<Project, ContractError> {
		// ... existing code ...
	}

	pub async fn get_vote<W: Clone + Debug + Send + Sync>(
		&self,
		proposal_id: i32,
		account: &Account<W>,
	) -> Result<Vote, ContractError> {
		// ... existing code ...
	}

	pub async fn get_funding<W: Clone + Debug + Send + Sync>(
		&self,
		project_id: i32,
		account: &Account<W>,
	) -> Result<Funding, ContractError> {
		// ... existing code ...
	}
}

#[async_trait]
impl<'a, P: JsonRpcProvider> SmartContractTrait<'a> for GrandShareContract<'a, P> {
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
