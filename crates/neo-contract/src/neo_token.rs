use async_trait::async_trait;
use primitive_types::H160;
use serde::{Deserialize, Serialize};
use futures::StreamExt;
use std::str::FromStr;

use neo_builder::{AccountSigner, TransactionBuilder};
use neo_clients::{APITrait, JsonRpcProvider, RpcClient};
use crate::{
	traits::{FungibleTokenTrait, SmartContractTrait, TokenTrait},
	ContractError,
};
use neo_crypto::Secp256r1PublicKey;
use neo_protocol::Account;
use neo_common::{deserialize_script_hash, serialize_script_hash};
use neo_types::{
	ContractParameter, ContractParameterType, NNSName, ScriptHash, StackItem, Bytes, ContractParameterBuilder
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeoToken<'a, P: JsonRpcProvider> {
	#[serde(deserialize_with = "deserialize_script_hash")]
	#[serde(serialize_with = "serialize_script_hash")]
	script_hash: ScriptHash,
	#[serde(skip_serializing_if = "Option::is_none")]
	total_supply: Option<u64>,
	#[serde(skip_serializing_if = "Option::is_none")]
	decimals: Option<u8>,
	symbol: Option<String>,
	#[serde(skip)]
	provider: Option<&'a RpcClient<P>>,
}

impl<'a, P: JsonRpcProvider + 'static> NeoToken<'a, P> {
	pub const NAME: &'static str = "NeoToken";
	// pub const SCRIPT_HASH: H160 = Self::calc_native_contract_hash(Self::NAME).unwrap();
	pub const DECIMALS: u8 = 0;
	pub const SYMBOL: &'static str = "NEO";
	pub const TOTAL_SUPPLY: u64 = 100_000_000;
	pub const UNCLAIMED_GAS: &'static str = "unclaimedGas";

	pub(crate) fn new(provider: Option<&'a RpcClient<P>>) -> Self {
		Self {
			script_hash: Self::calc_native_contract_hash(Self::NAME).unwrap(),
			total_supply: Some(Self::TOTAL_SUPPLY),
			decimals: Some(Self::DECIMALS),
			symbol: Some(Self::SYMBOL.to_string()),
			provider,
		}
	}

	// Unclaimed Gas

	async fn unclaimed_gas<W: Clone + Debug + Send + Sync>(
		&self,
		account: &Account<W>,
		block_height: i32,
	) -> Result<i64, ContractError> {
		let params = vec![
			account.get_script_hash().into(),
			ContractParameterBuilder::number(block_height),
		];

		let result = self.call_function_returning_int(Self::UNCLAIMED_GAS, params).await?;
		Ok(result as i64)
	}

	async fn unclaimed_gas_contract(
		&self,
		script_hash: &H160,
		block_height: i32,
	) -> Result<i64, ContractError> {
		Ok(self
			.call_function_returning_int(
				"unclaimedGas",
				vec![script_hash.into(), block_height.into()],
			)
			.await
			.unwrap() as i64)
	}

	// Candidate Registration

	async fn register_candidate(
		&self,
		candidate_key: &Secp256r1PublicKey,
	) -> Result<TransactionBuilder<'_>, ContractError> {
		let key_bytes = candidate_key.get_encoded(true);
		let type_key = neo_types::serde_value::Secp256r1PublicKey::from_bytes(&key_bytes).unwrap();
		let key_param = ContractParameter::from(&type_key);
		self.invoke_function("registerCandidate", vec![key_param]).await
	}

	async fn unregister_candidate(
		&self,
		candidate_key: &Secp256r1PublicKey,
	) -> Result<TransactionBuilder<'_>, ContractError> {
		let key_bytes = candidate_key.get_encoded(true);
		let type_key = neo_types::serde_value::Secp256r1PublicKey::from_bytes(&key_bytes).unwrap();
		let key_param = ContractParameter::from(&type_key);
		self.invoke_function("unregisterCandidate", vec![key_param]).await
	}

	// Committee and Candidates Information

	async fn get_committee(&self) -> Result<Vec<Secp256r1PublicKey>, ContractError> {
		self.call_function_returning_list_of_public_keys("getCommittee")
			.await
			.map_err(|e| ContractError::UnexpectedReturnType(e.to_string()))
	}

	async fn get_candidates(&self) -> Result<Vec<Candidate>, ContractError> {
		let candidates = self.call_invoke_function("getCandidates", vec![], vec![]).await.unwrap();
		let item = candidates.stack.first().unwrap();
		if let StackItem::Array { value: array } = item {
			Ok(array
				.to_vec()
				.chunks(2)
				.filter_map(|v| {
					if v.len() == 2 {
						Some(Candidate::from(v.to_vec()).unwrap())
					} else {
						None
					}
				})
				.collect::<Vec<Candidate>>())
		} else {
			Err(ContractError::UnexpectedReturnType("Candidates".to_string()))
		}
	}

	async fn is_candidate(&self, public_key: &Secp256r1PublicKey) -> Result<bool, ContractError> {
		let candidates = self.get_candidates().await?;
		
		// Convert the target public key to a string for comparison
		let target_key_str = format!("{:?}", public_key);
		
		// Check if any candidate's public key matches the target key
		Ok(candidates
			.into_iter()
			.any(|c| format!("{:?}", c.public_key) == target_key_str))
	}

	// Voting

	async fn vote(
		&self,
		voter: &H160,
		candidate: Option<&Secp256r1PublicKey>,
	) -> Result<TransactionBuilder<'_>, ContractError> {
		let params = match candidate {
			Some(key) => {
				let key_bytes = key.get_encoded(true);
				let type_key = neo_types::serde_value::Secp256r1PublicKey::from_bytes(&key_bytes).unwrap();
				vec![voter.into(), ContractParameter::from(&type_key)]
			},
			None => vec![voter.into(), ContractParameter::bool(false)],
		};

		self.invoke_function("vote", params).await
	}

	async fn cancel_vote(&self, voter: &H160) -> Result<TransactionBuilder<'_>, ContractError> {
		self.vote(voter, None).await
	}

	async fn build_vote_script(
		&self,
		voter: &H160,
		candidate: Option<&Secp256r1PublicKey>,
	) -> Result<Bytes, ContractError> {
		let params = match candidate {
			Some(key) => {
				let key_bytes = key.get_encoded(true);
				let type_key = neo_types::serde_value::Secp256r1PublicKey::from_bytes(&key_bytes).unwrap();
				vec![voter.into(), ContractParameter::from(&type_key)]
			},
			None => vec![voter.into(), ContractParameter::bool(false)],
		};

		self.build_invoke_function_script("vote", params).await
	}

	// Network Settings

	async fn get_gas_per_block(&self) -> Result<i32, ContractError> {
		self.call_function_returning_int("getGasPerBlock", vec![]).await
	}

	async fn set_gas_per_block(
		&self,
		gas_per_block: i32,
	) -> Result<TransactionBuilder<'_>, ContractError> {
		self.invoke_function("setGasPerBlock", vec![gas_per_block.into()]).await
	}

	async fn get_register_price(&self) -> Result<i32, ContractError> {
		self.call_function_returning_int("getRegisterPrice", vec![]).await
	}

	async fn set_register_price(
		&self,
		register_price: i32,
	) -> Result<TransactionBuilder<'_>, ContractError> {
		self.invoke_function("setRegisterPrice", vec![register_price.into()]).await
	}

	async fn get_account_state(&self, account: &H160) -> Result<AccountState, ContractError> {
		let result = self
			.call_invoke_function("getAccountState", vec![account.into()], vec![])
			.await
			.unwrap()
			.stack
			.first()
			.unwrap()
			.clone();

		match result {
			StackItem::Any => Ok(AccountState::with_no_balance()),
			StackItem::Array { value: items } if items.len() >= 3 => {
				let balance = items[0].as_int().unwrap();
				let update_height = items[1].as_int();
				let public_key = items[2].clone();

				if let StackItem::Any = public_key {
					return Ok(AccountState {
						balance,
						balance_height: update_height,
						public_key: None,
					});
				} else {
					let pubkey =
						Secp256r1PublicKey::from_bytes(public_key.as_bytes().unwrap().as_slice())
							.unwrap();
					Ok(AccountState {
						balance,
						balance_height: update_height,
						public_key: Some(pubkey),
					})
				}
			},
			_ => Err(ContractError::InvalidNeoName("Account state malformed".to_string())),
		}
	}

	async fn call_function_returning_list_of_public_keys(
		&self,
		function: &str,
	) -> Result<Vec<Secp256r1PublicKey>, ContractError> {
		let result = self.call_invoke_function(function, vec![], vec![]).await.unwrap();
		let stack_item = result.stack.first().unwrap();

		if let StackItem::Array { value: array } = stack_item {
			let keys = array
				.iter()
				.map(|item| {
					if let StackItem::ByteString { value: bytes } = item {
						Secp256r1PublicKey::from_bytes(bytes.as_bytes()).unwrap()
					} else {
						panic!("Unexpected stack item type")
					}
				})
				.collect::<Vec<Secp256r1PublicKey>>();

			Ok(keys)
		} else {
			Err(ContractError::UnexpectedReturnType("UnexpectedReturnType".to_string()))
		}
	}
}

#[async_trait]
impl<'a, P: JsonRpcProvider> TokenTrait<'a, P> for NeoToken<'a, P> {
	fn total_supply(&self) -> Option<u64> {
		self.total_supply
	}

	fn set_total_supply(&mut self, total_supply: u64) {
		self.total_supply = Some(total_supply)
	}

	fn decimals(&self) -> Option<u8> {
		self.decimals
	}

	fn set_decimals(&mut self, decimals: u8) {
		self.decimals = Some(decimals)
	}

	fn symbol(&self) -> Option<String> {
		self.symbol.clone()
	}

	fn set_symbol(&mut self, symbol: String) {
		self.symbol = Some(symbol)
	}

	async fn resolve_nns_text_record(&self, _name: &NNSName) -> Result<H160, ContractError> {
		todo!()
	}
}

#[async_trait]
impl<'a, P: JsonRpcProvider> SmartContractTrait<'a> for NeoToken<'a, P> {
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

#[async_trait]
impl<'a, P: JsonRpcProvider> FungibleTokenTrait<'a, P> for NeoToken<'a, P> {}

pub struct Candidate {
	pub public_key: neo_types::serde_value::Secp256r1PublicKey,
	pub votes: i64,
}

impl Candidate {
	fn from(items: Vec<StackItem>) -> Result<Self, ContractError> {
		let key = items[0].as_public_key().unwrap();
		let votes = items[1].as_int().unwrap() as i64;
		Ok(Self { public_key: key, votes })
	}
}

pub struct AccountState {
	pub balance: i64,
	pub balance_height: Option<i64>,
	pub public_key: Option<Secp256r1PublicKey>,
}

impl AccountState {
	pub fn with_no_balance() -> Self {
		Self { balance: 0, balance_height: None, public_key: None }
	}
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CandidateJson {
	#[serde(rename = "publicKey")]
	pub public_key: String,
	pub votes: String,
}

impl Candidate {
	pub fn from_json(json: CandidateJson) -> Result<Self, ContractError> {
		let votes = json.votes.parse::<i64>().map_err(|e| ContractError::ParseError(e.to_string()))?;
		let key = neo_types::serde_value::Secp256r1PublicKey::from_str(&json.public_key)
			.map_err(|e| ContractError::ParseError(e.to_string()))?;
		
		Ok(Self { public_key: key, votes })
	}
}
