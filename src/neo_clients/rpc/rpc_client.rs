use async_trait::async_trait;
use futures_util::lock::Mutex;
use primitive_types::{H160, H256};
use rustc_serialize::{base64, base64::ToBase64, hex::FromHex};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, value::Value};
use std::{
	collections::HashMap,
	convert::TryFrom,
	fmt::{Debug, Display},
	future::Future,
	net::Ipv4Addr,
	pin::Pin,
	str::FromStr,
	sync::Arc,
	time::Duration,
};
use tracing::trace;
use tracing_futures::Instrument;
use url::{Host, ParseError, Url};

use neo::prelude::*;

use crate::{
	neo_clients::rpc::rpc_client::sealed::Sealed, neo_types::ScriptHashExtension,
	prelude::Base64Encode,
};

/// Node Clients
#[derive(Copy, Clone)]
pub enum NeoClient {
	/// RNEO
	NEO,
}

impl FromStr for NeoClient {
	type Err = ProviderError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.split('/').next().unwrap().to_lowercase().as_str() {
			"NEO" => Ok(NeoClient::NEO),
			_ => Err(ProviderError::UnsupportedNodeClient),
		}
	}
}

/// An abstract provider for interacting with the [Neo JSON RPC
/// API](https://github.com/neo/wiki/JSON-RPC). Must be instantiated
/// with a data transport which implements the [`JsonRpcClient`](trait@crate::JsonRpcClient) trait
/// (e.g. [HTTP](crate::Http), Websockets etc.)
///
/// # Example
///
/// ```no_run
///  use neo_rs::prelude::{Http, Middleware, NeoConstants, Provider};
///  async fn foo() -> Result<(), Box<dyn std::error::Error>> {
/// use std::convert::TryFrom;
///
/// let provider = Provider::<Http>::try_from(
///     NeoConstants::SEED_1
/// ).expect("could not instantiate HTTP Provider");
///
/// let block = provider.get_block_by_index(100u32, false).await?;
/// println!("Got block: {}", serde_json::to_string(&block)?);
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct RpcClient<P> {
	provider: P,
	nns: Option<Address>,
	interval: Option<Duration>,
	from: Option<Address>,
	_node_client: Arc<Mutex<Option<NeoVersion>>>,
}

impl<P> AsRef<P> for RpcClient<P> {
	fn as_ref(&self) -> &P {
		&self.provider
	}
}

// JSON RPC bindings
impl<P: JsonRpcProvider> RpcClient<P> {
	/// Instantiate a new provider with a backend.
	pub fn new(provider: P) -> Self {
		Self {
			provider,
			nns: None,
			interval: None,
			from: None,
			_node_client: Arc::new(Mutex::new(None)),
		}
	}

	/// Returns the type of node we're connected to, while also caching the value for use
	/// in other node-specific API calls, such as the get_block_receipts call.
	pub async fn node_client(&self) -> Result<NeoVersion, ProviderError> {
		let mut node_client = self._node_client.lock().await;

		if let Some(ref node_client) = *node_client {
			Ok(node_client.clone())
		} else {
			let client_version = self.get_version().await?;
			*node_client = Some(client_version.clone());
			Ok(client_version)
		}
	}

	#[must_use]
	/// Set the default sender on the provider
	pub fn with_sender(mut self, address: impl Into<Address>) -> Self {
		self.from = Some(address.into());
		self
	}

	/// Make an RPC request via the internal connection, and return the result.
	pub async fn request<T, R>(&self, method: &str, params: T) -> Result<R, ProviderError>
	where
		T: Debug + Serialize + Send + Sync,
		R: Serialize + DeserializeOwned + Debug + Send,
	{
		let span =
			tracing::trace_span!("rpc", method = method, params = ?serde_json::to_string(&params)?);
		// https://docs.rs/tracing/0.1.22/tracing/span/struct.Span.html#in-asynchronous-code
		let res = async move {
			trace!("tx");
			let fetched = self.provider.fetch(method, params).await;
			let res: R = fetched.map_err(Into::into)?;
			debug!("{:?}", res);
			trace!(rx = ?serde_json::to_string(&res)?);
			Ok::<_, ProviderError>(res)
		}
		.instrument(span)
		.await?;
		Ok(res)
	}
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<P: JsonRpcProvider> APITrait for RpcClient<P> {
	type Error = ProviderError;
	type Provider = P;

	fn rpc_client(&self) -> &RpcClient<Self::Provider> {
		self
	}

	async fn network(&self) -> u32 {
		self.get_version().await.unwrap().protocol.unwrap().network
	}

	//////////////////////// Neo methods////////////////////////////

	// Blockchain methods
	/// Gets the hash of the latest block in the blockchain.
	/// - Returns: The request object
	async fn get_best_block_hash(&self) -> Result<H256, ProviderError> {
		self.request("getbestblockhash", Vec::<H256>::new()).await
	}

	/// Gets the block hash of the corresponding block based on the specified block index.
	/// - Parameter blockIndex: The block index
	/// - Returns: The request object
	async fn get_block_hash(&self, block_index: u32) -> Result<H256, ProviderError> {
		self.request("getblockhash", [block_index.to_value()].to_vec()).await
	}

	/// Gets the corresponding block information according to the specified block hash.
	/// - Parameters:
	///   - blockHash: The block hash
	///   - returnFullTransactionObjects: Whether to get block information with all transaction objects or just the block header
	/// - Returns: The request object
	async fn get_block(&self, block_hash: H256, full_tx: bool) -> Result<NeoBlock, ProviderError> {
		Ok(if full_tx {
			self.request("getblock", [block_hash.to_value(), 1.to_value()].to_vec()).await?
		} else {
			self.get_block_header(block_hash).await?
		})
	}

	/// Gets the corresponding block information for the specified block hash.
	/// - Parameter blockHash: The block hash
	/// - Returns: The request object
	async fn get_raw_block(&self, block_hash: H256) -> Result<String, ProviderError> {
		self.request("getblock", [block_hash.to_value(), 0.to_value()]).await
	}

	// Node methods
	/// Gets the block header count of the blockchain.
	/// - Returns: The request object
	async fn get_block_header_count(&self) -> Result<u32, ProviderError> {
		self.request("getblockheadercount", Vec::<u32>::new()).await
	}

	/// Gets the block count of the blockchain.
	/// - Returns: The request object
	async fn get_block_count(&self) -> Result<u32, ProviderError> {
		self.request("getblockcount", Vec::<u32>::new()).await
	}

	/// Gets the corresponding block header information according to the specified block hash.
	/// - Parameter blockHash: The block hash
	/// - Returns: The request object
	async fn get_block_header(&self, block_hash: H256) -> Result<NeoBlock, ProviderError> {
		self.request("getblockheader", vec![block_hash.to_value(), 1.to_value()]).await
	}

	/// Gets the corresponding block header information according to the specified index.
	/// - Parameter blockIndex: The block index
	/// - Returns: The request object
	async fn get_block_header_by_index(&self, index: u32) -> Result<NeoBlock, ProviderError> {
		self.request("getblockheader", vec![index.to_value(), 1.to_value()]).await
	}

	/// Gets the corresponding block header information according to the specified block hash.
	/// - Parameter blockHash: The block hash
	/// - Returns: The request object
	async fn get_raw_block_header(&self, block_hash: H256) -> Result<String, ProviderError> {
		self.request("getblockheader", vec![block_hash.to_value(), 0.to_value()]).await
	}

	/// Gets the corresponding block header information according to the specified index.
	/// - Parameter blockIndex: The block index
	/// - Returns: The request object
	async fn get_raw_block_header_by_index(&self, index: u32) -> Result<String, ProviderError> {
		self.request("getblockheader", vec![index.to_value(), 0.to_value()]).await
	}

	/// Gets the native contracts list, which includes the basic information of native contracts and the contract descriptive file `manifest.json`.
	/// - Returns: The request object
	async fn get_native_contracts(&self) -> Result<Vec<NativeContractState>, ProviderError> {
		self.request("getnativecontracts", Vec::<NativeContractState>::new()).await
	}

	/// Gets the contract information.
	/// - Parameter contractHash: The contract script hash
	/// - Returns: The request object
	async fn get_contract_state(&self, hash: H160) -> Result<ContractState, ProviderError> {
		self.request("getcontractstate", vec![hash.to_hex()]).await
	}

	/// Gets the contract information.
	/// - Parameter contractHash: The contract script hash
	/// - Returns: The request object
	async fn get_contract_state_by_id(&self, id: i64) -> Result<ContractState, ProviderError> {
		self.request("getcontractstate", vec![id.to_value()]).await
	}

	/// Gets the native contract information by its name.
	///
	/// This RPC only works for native contracts.
	/// - Parameter contractName: The name of the native contract
	/// - Returns: The request object
	async fn get_native_contract_state(&self, name: &str) -> Result<ContractState, ProviderError> {
		self.request("getcontractstate", vec![name.to_value()]).await
	}

	/// Gets a list of unconfirmed or confirmed transactions in memory.
	/// - Returns: The request object
	async fn get_mem_pool(&self) -> Result<MemPoolDetails, ProviderError> {
		self.request("getrawmempool", vec![1.to_value()]).await
	}

	/// Gets a list of confirmed transactions in memory.
	/// - Returns: The request object
	async fn get_raw_mem_pool(&self) -> Result<Vec<H256>, ProviderError> {
		self.request("getrawmempool", Vec::<H256>::new()).await
	}

	/// Gets the corresponding transaction information based on the specified transaction hash.
	/// - Parameter txHash: The transaction hash
	/// - Returns: The request object
	async fn get_transaction(&self, hash: H256) -> Result<TransactionResult, ProviderError> {
		self.request("getrawtransaction", vec![hash.to_value(), 1.to_value()]).await
	}

	/// Gets the corresponding transaction information based on the specified transaction hash.
	/// - Parameter txHash: The transaction hash
	/// - Returns: The request object
	async fn get_raw_transaction(&self, tx_hash: H256) -> Result<String, ProviderError> {
		self.request("getrawtransaction", vec![tx_hash.to_value(), 0.to_value()]).await
	}

	/// Gets the stored value according to the contract hash and the key.
	/// - Parameters:
	///   - contractHash: The contract hash
	///   - keyHexString: The key to look up in storage as a hexadecimal string
	/// - Returns: The request object
	async fn get_storage(&self, contract_hash: H160, key: &str) -> Result<String, ProviderError> {
		let params: [String; 2] =
			[contract_hash.to_hex(), Base64Encode::to_base64(&key.to_string())];
		self.request("getstorage", params.to_vec()).await
	}

	/// Finds the storage entries of a contract based on the prefix and  start index.
	/// - Parameters:
	///   - contractHash: The contract hash
	///   - prefix_hex_string: The prefix to filter the storage entries
	///   - start_index: the start index
	/// - Returns: The request object
	async fn find_storage(
		&self,
		contract_hash: H160,
		prefix_hex_string: &str,
		start_index: u64,
	) -> Result<String, ProviderError> {
		//let params = [contract_hash.to_hex(), Base64Encode::to_base64(&prefix_hex_string.to_string()), start_index.to_value()];
		let params = json!([
			contract_hash.to_hex(),
			Base64Encode::to_base64(&prefix_hex_string.to_string()),
			start_index
		]);
		self.request("findstorage", params).await
	}

	/// Finds the storage entries of a contract based on the prefix and  start index.
	/// - Parameters:
	///   - contract_id: The contract id
	///   - prefix_hex_string: The prefix to filter the storage entries
	///   - start_index: the start index
	/// - Returns: The request object
	async fn find_storage_with_id(
		&self,
		contract_id: i64,
		prefix_hex_string: &str,
		start_index: u64,
	) -> Result<String, ProviderError> {
		//let params = [contract_hash.to_hex(), Base64Encode::to_base64(&prefix_hex_string.to_string()), start_index.to_value()];
		let params = json!([
			contract_id,
			Base64Encode::to_base64(&prefix_hex_string.to_string()),
			start_index
		]);
		self.request("findstorage", params).await
	}

	/// Gets the transaction height with the specified transaction hash.
	/// - Parameter txHash: The transaction hash
	/// - Returns: The request object
	async fn get_transaction_height(&self, tx_hash: H256) -> Result<u32, ProviderError> {
		let params = [tx_hash.to_value()];
		self.request("gettransactionheight", params.to_vec()).await
	}

	/// Gets the validators of the next block.
	/// - Returns: The request object
	async fn get_next_block_validators(&self) -> Result<Vec<Validator>, ProviderError> {
		self.request("getnextblockvalidators", Vec::<Validator>::new()).await
	}

	/// Gets the public key list of current Neo committee members.
	/// - Returns: The request object
	async fn get_committee(&self) -> Result<Vec<String>, ProviderError> {
		self.request("getcommittee", Vec::<String>::new()).await
	}

	/// Gets the current number of connections for the node.
	/// - Returns: The request object
	async fn get_connection_count(&self) -> Result<u32, ProviderError> {
		self.request("getconnectioncount", Vec::<u32>::new()).await
	}

	/// Gets a list of nodes that the node is currently connected or disconnected from.
	/// - Returns: The request object
	async fn get_peers(&self) -> Result<Peers, ProviderError> {
		self.request("getpeers", Vec::<Peers>::new()).await
	}

	/// Gets the version information of the node.
	/// - Returns: The request object
	async fn get_version(&self) -> Result<NeoVersion, ProviderError> {
		self.request("getversion", Vec::<NeoVersion>::new()).await
	}

	/// Broadcasts a transaction over the NEO network.
	/// - Parameter rawTransactionHex: The raw transaction in hexadecimal
	/// - Returns: The request object
	async fn send_raw_transaction(&self, hex: String) -> Result<RawTransaction, ProviderError> {
		self.request("sendrawtransaction", vec![Base64Encode::to_base64(&hex)]).await
	}

	/// Broadcasts a new block over the NEO network.
	/// - Parameter serializedBlockAsHex: The block in hexadecimal
	/// - Returns: The request object
	async fn submit_block(&self, hex: String) -> Result<SubmitBlock, ProviderError> {
		self.request("submitblock", vec![hex.to_value()]).await
	}

	// MARK: SmartContract Methods

	/// Invokes the function with `functionName` of the smart contract with the specified contract hash.
	/// - Parameters:
	///   - contractHash: The contract hash to invoke
	///   - functionName: The function to invoke
	///   - contractParams: The parameters of the function
	///   - signers: The signers
	/// - Returns: The request object
	async fn invoke_function(
		&self,
		contract_hash: &H160,
		method: String,
		params: Vec<ContractParameter>,
		signers: Option<Vec<Signer>>,
	) -> Result<InvocationResult, ProviderError> {
		match signers {
			Some(signers) => {
				let signers: Vec<TransactionSigner> = signers.iter().map(|f| f.into()).collect();
				self.request(
					"invokefunction",
					json!([contract_hash.to_hex(), method, params, signers,]),
				)
				.await
			},
			None => {
				let signers: Vec<TransactionSigner> = vec![];
				self.request(
					"invokefunction",
					json!([
						//ScriptHashExtension::to_hex_big_endian(contract_hash),
						contract_hash.to_hex(),
						method,
						params,
						signers
					]), // 	ScriptHashExtension::to_hex_big_endian(contract_hash),
					    // 	method,
					    // 	params,
					    // 	signers
					    // ]),
				)
				.await
			},
		}
	}

	/// Invokes a script.
	/// - Parameters:
	///   - scriptHex: The script to invoke
	///   - signers: The signers
	/// - Returns: The request object
	async fn invoke_script(
		&self,
		hex: String,
		signers: Vec<Signer>,
	) -> Result<InvocationResult, ProviderError> {
		let signers: Vec<TransactionSigner> =
			signers.into_iter().map(|signer| signer.into()).collect::<Vec<_>>();
		let script_base64 = serde_json::to_value(hex.from_hex().unwrap().to_base64())?;
		let signers_json = serde_json::to_value(&signers)?;
		self.request("invokescript", [script_base64, signers_json]).await
	}

	/// Gets the unclaimed GAS of the account with the specified script hash.
	/// - Parameter scriptHash: The account's script hash
	/// - Returns: The request object
	async fn get_unclaimed_gas(&self, hash: H160) -> Result<UnclaimedGas, ProviderError> {
		self.request("getunclaimedgas", [hash.to_address()]).await
	}

	/// Gets a list of plugins loaded by the node.
	/// - Returns: The request object
	async fn list_plugins(&self) -> Result<Vec<Plugin>, ProviderError> {
		self.request("listplugins", Vec::<u32>::new()).await
	}

	/// Verifies whether the address is a valid NEO address.
	/// - Parameter address: The address to verify
	/// - Returns: The request object
	async fn validate_address(&self, address: &str) -> Result<ValidateAddress, ProviderError> {
		self.request("validateaddress", vec![address.to_value()]).await
	}

	/// Closes the current wallet.
	/// - Returns: The request object
	async fn close_wallet(&self) -> Result<bool, ProviderError> {
		self.request("closewallet", Vec::<u32>::new()).await
	}

	/// Exports the private key of the specified script hash.
	/// - Parameter scriptHash: The account's script hash
	/// - Returns: The request object
	async fn dump_priv_key(&self, script_hash: H160) -> Result<String, ProviderError> {
		let params = [script_hash.to_address()].to_vec();
		self.request("dumpprivkey", params).await
	}

	/// Gets the wallet balance of the corresponding token.
	/// - Parameter tokenHash: The token hash
	/// - Returns: The request object
	async fn get_wallet_balance(&self, token_hash: H160) -> Result<Balance, ProviderError> {
		self.request("getwalletbalance", vec![token_hash.to_value()]).await
	}

	/// Creates a new address.
	/// - Returns: The request object
	async fn get_new_address(&self) -> Result<String, ProviderError> {
		self.request("getnewaddress", Vec::<u32>::new()).await
	}

	/// Gets the amount of unclaimed GAS in the wallet.
	/// - Returns: The request object
	async fn get_wallet_unclaimed_gas(&self) -> Result<String, ProviderError> {
		self.request("getwalletunclaimedgas", Vec::<String>::new()).await
	}

	/// Imports a private key to the wallet.
	/// - Parameter privateKeyInWIF: The private key in WIF-format
	/// - Returns: The request object
	async fn import_priv_key(&self, priv_key: String) -> Result<NeoAddress, ProviderError> {
		let params = [priv_key.to_value()].to_vec();
		self.request("importprivkey", params).await
	}

	/// Calculates the network fee for the specified transaction.
	/// - Parameter txBase64: The transaction in hexadecimal
	/// - Returns: The request object
	async fn calculate_network_fee(&self, txBase64: String) -> Result<i64, ProviderError> {
		self.request("calculatenetworkfee", vec![Base64Encode::to_base64(&txBase64)])
			.await
	}

	/// Lists all the addresses in the current wallet.
	/// - Returns: The request object
	async fn list_address(&self) -> Result<Vec<NeoAddress>, ProviderError> {
		self.request("listaddress", Vec::<NeoAddress>::new()).await
	}

	/// Opens the specified wallet.
	/// - Parameters:
	///   - walletPath: The wallet file path
	///   - password: The password for the wallet
	/// - Returns: The request object
	async fn open_wallet(&self, path: String, password: String) -> Result<bool, ProviderError> {
		self.request("openwallet", vec![path.to_value(), password.to_value()]).await
	}

	/// Transfers an amount of a token from an account to another account.
	/// - Parameters:
	///   - tokenHash: The token hash of the NEP-17 contract
	///   - from: The transferring account's script hash
	///   - to: The recipient
	///   - amount: The transfer amount in token fractions
	/// - Returns: The request object
	async fn send_from(
		&self,
		token_hash: H160,
		from: H160,
		to: H160,
		amount: u32,
	) -> Result<Transaction, ProviderError> {
		// let params =
		// 	[token_hash.to_value(), from.to_value(), to.to_value(), amount.to_value()].to_vec();
		let params = json!([token_hash.to_hex(), from.to_address(), to.to_address(), amount,]);
		self.request("sendfrom", params).await
	}

	/// Initiates multiple transfers to multiple accounts from one specific account in a transaction.
	/// - Parameters:
	///   - from: The transferring account's script hash
	///   - txSendTokens: a list of ``TransactionSendToken`` objects, that each contains the token hash, the recipient and the transfer amount.
	/// - Returns: The request object
	async fn send_many(
		&self,
		from: Option<H160>,
		send_tokens: Vec<TransactionSendToken>,
	) -> Result<Transaction, ProviderError> {
		let params = [from.unwrap().to_value(), send_tokens.to_value()].to_vec();
		self.request("sendmany", params).await
	}

	/// Transfers an amount of a token to another account.
	/// - Parameters:
	///   - tokenHash: The token hash of the NEP-17 contract
	///   - to: The recipient
	///   - amount: The transfer amount in token fractions
	/// - Returns: The request object
	async fn send_to_address(
		&self,
		token_hash: H160,
		to: Address,
		amount: u32,
	) -> Result<Transaction, ProviderError> {
		let params = [token_hash.to_value(), to.to_value(), amount.to_value()].to_vec();
		self.request("sendtoaddress", params).await
	}

	/// Gets the application logs of the specified transaction hash.
	/// - Parameter txHash: The transaction hash
	/// - Returns: The request object
	async fn get_application_log(&self, tx_hash: H256) -> Result<ApplicationLog, ProviderError> {
		self.request("getapplicationlog", vec![tx_hash.to_value()]).await
	}

	/// Gets the balance of all NEP-17 token assets in the specified script hash.
	/// - Parameter scriptHash: The account's script hash
	/// - Returns: The request object
	async fn get_nep17_balances(&self, script_hash: H160) -> Result<Nep17Balances, ProviderError> {
		self.request("getnep17balances", [script_hash.to_address().to_value()].to_vec())
			.await
	}

	/// Gets all the NEP-17 transaction information occurred in the specified script hash.
	/// - Parameter scriptHash: The account's script hash
	/// - Returns: The request object
	async fn get_nep17_transfers(
		&self,
		script_hash: H160,
	) -> Result<Nep17Transfers, ProviderError> {
		let params = [script_hash.to_value()].to_vec();
		self.request("getnep17transfers", params).await
	}

	/// Gets all the NEP17 transaction information occurred in the specified script hash since the specified time.
	/// - Parameters:
	///   - scriptHash: The account's script hash
	///   - from: The timestamp transactions occurred since
	/// - Returns: The request object
	async fn get_nep17_transfers_from(
		&self,
		script_hash: H160,
		from: u64,
	) -> Result<Nep17Transfers, ProviderError> {
		// let params = [script_hash.to_value(), from.to_value()].to_vec();
		self.request("getnep17transfers", [script_hash.to_value(), from.to_value()])
			.await
	}

	/// Gets all the NEP17 transaction information occurred in the specified script hash in the specified time range.
	/// - Parameters:
	///   - scriptHash: The account's script hash
	///   - from: The start timestamp
	///   - to: The end timestamp
	/// - Returns: The request object
	async fn get_nep17_transfers_range(
		&self,
		script_hash: H160,
		from: u64,
		to: u64,
	) -> Result<Nep17Transfers, ProviderError> {
		let params = [script_hash.to_value(), from.to_value(), to.to_value()].to_vec();
		self.request("getnep17transfers", params).await
	}

	/// Gets all NEP-11 balances of the specified account.
	/// - Parameter scriptHash: The account's script hash
	/// - Returns: The request object
	async fn get_nep11_balances(&self, script_hash: H160) -> Result<Nep11Balances, ProviderError> {
		let params = [script_hash.to_value()].to_vec();
		self.request("getnep11balances", params).await
	}

	/// Gets all NEP-11 transaction of the given account.
	/// - Parameter scriptHash: The account's script hash
	/// - Returns: The request object
	async fn get_nep11_transfers(
		&self,
		script_hash: H160,
	) -> Result<Nep11Transfers, ProviderError> {
		let params = [script_hash.to_value()].to_vec();
		self.request("getnep11transfers", params).await
	}

	/// Gets all NEP-11 transaction of the given account since the given time.
	/// - Parameters:
	///   - scriptHash: The account's script hash
	///   - from: The date from when to report transactions
	/// - Returns: The request object
	async fn get_nep11_transfers_from(
		&self,
		script_hash: H160,
		from: u64,
	) -> Result<Nep11Transfers, ProviderError> {
		let params = [script_hash.to_value(), from.to_value()].to_vec();
		self.request("getnep11transfers", params).await
	}

	/// Gets all NEP-11 transactions of the given account in the time span between `from` and `to`.
	/// - Parameters:
	///   - scriptHash: The account's script hash
	///   - from: The start timestamp
	///   - to: The end timestamp
	/// - Returns: The request object
	async fn get_nep11_transfers_range(
		&self,
		script_hash: H160,
		from: u64,
		to: u64,
	) -> Result<Nep11Transfers, ProviderError> {
		let params = [script_hash.to_value(), from.to_value(), to.to_value()].to_vec();
		self.request("getnep11transfers", params).await
	}

	/// Gets the properties of the token with `tokenId` from the NEP-11 contract with `scriptHash`.
	///
	/// The properties are a mapping from the property name string to the value string.
	/// The value is plain text if the key is one of the properties defined in the NEP-11 standard.
	/// Otherwise, the value is a Base64-encoded byte array.
	///
	/// To receive custom property values that consist of nested types (e.g., Maps or Arrays) use ``invokeFunction(_:_:_:)``  to directly invoke the method `properties` of the NEP-11 smart contract.
	/// - Parameters:
	///   - scriptHash: The account's script hash
	///   - tokenId: The ID of the token as a hexadecimal string
	/// - Returns: The request object
	async fn get_nep11_properties(
		&self,
		script_hash: H160,
		token_id: &str,
	) -> Result<HashMap<String, String>, ProviderError> {
		let params = [script_hash.to_value(), token_id.to_value()].to_vec();
		self.request("getnep11properties", params).await
	}

	/// Gets the state root by the block height.
	/// - Parameter blockIndex: The block index
	/// - Returns: The request object
	async fn get_state_root(&self, block_index: u32) -> Result<StateRoot, ProviderError> {
		let params = [block_index.to_value()].to_vec();
		self.request("getstateroot", params).await
	}

	/// Gets the proof based on the root hash, the contract hash and the storage key.
	/// - Parameters:
	///   - rootHash: The root hash
	///   - contractHash: The contract hash
	///   - storageKeyHex: The storage key
	/// - Returns: The request object
	async fn get_proof(
		&self,
		root_hash: H256,
		contract_hash: H160,
		key: &str,
	) -> Result<String, ProviderError> {
		self.request(
			"getproof",
			vec![root_hash.to_value(), contract_hash.to_value(), key.to_value()],
		)
		.await
	}

	/// Verifies the proof data and gets the value of the storage corresponding to the key.
	/// - Parameters:
	///   - rootHash: The root hash
	///   - proofDataHex: The proof data of the state root
	/// - Returns: The request object
	async fn verify_proof(&self, root_hash: H256, proof: &str) -> Result<bool, ProviderError> {
		let params = [root_hash.to_value(), proof.to_value()].to_vec();
		self.request("verifyproof", params).await
	}

	/// Gets the state root height.
	/// - Returns: The request object
	async fn get_state_height(&self) -> Result<StateHeight, ProviderError> {
		self.request("getstateheight", ()).await
	}

	/// Gets the state.
	/// - Parameters:
	///   - rootHash: The root hash
	///   - contractHash: The contract hash
	///   - keyHex: The storage key
	/// - Returns: The request object
	async fn get_state(
		&self,
		root_hash: H256,
		contract_hash: H160,
		key: &str,
	) -> Result<String, ProviderError> {
		self.request(
			"getstate",
			vec![root_hash.to_value(), contract_hash.to_value(), key.to_value()], //key.to_base64()],
		)
		.await
	}

	/// Gets a list of states that match the provided key prefix.
	///
	/// Includes proofs of the first and last entry.
	/// - Parameters:
	///   - rootHash: The root hash
	///   - contractHash: The contact hash
	///   - keyPrefixHex: The key prefix
	///   - startKeyHex: The start key
	///   - countFindResultItems: The number of results. An upper limit is defined in the Neo core
	/// - Returns: The request object
	async fn find_states(
		&self,
		root_hash: H256,
		contract_hash: H160,
		key_prefix: &str,
		start_key: Option<&str>,
		count: Option<u32>,
	) -> Result<States, ProviderError> {
		let mut params =
			vec![root_hash.to_value(), contract_hash.to_value(), key_prefix.to_value()];
		if let Some(start_key) = start_key {
			params.push(start_key.to_value())
		}
		if let Some(count) = count {
			params.push(count.to_value())
		}
		self.request("findstates", params).await
	}

	async fn get_block_by_index(
		&self,
		index: u32,
		full_tx: bool,
	) -> Result<NeoBlock, ProviderError> {
		// let full_tx = if full_tx { 1 } else { 0 };
		// self.request("getblock", vec![index.to_value(), 1.to_value()]).await
		return Ok(if full_tx {
			self.request("getblock", vec![index.to_value(), 1.to_value()]).await?
		} else {
			self.get_block_header_by_index(index).await?
		});
	}

	async fn get_raw_block_by_index(&self, index: u32) -> Result<String, ProviderError> {
		self.request("getblock", vec![index.to_value(), 0.to_value()]).await
	}

	/// Invokes the function with `functionName` of the smart contract with the specified contract hash.
	///
	/// Includes diagnostics from the invocation.
	/// - Parameters:
	///   - contractHash: The contract hash to invoke
	///   - functionName: The function to invoke
	///   - contractParams: The parameters of the function
	///   - signers: The signers
	/// - Returns: The request object
	async fn invoke_function_diagnostics(
		&self,
		contract_hash: H160,
		function_name: String,
		params: Vec<ContractParameter>,
		signers: Vec<Signer>,
	) -> Result<InvocationResult, ProviderError> {
		let signers: Vec<TransactionSigner> = signers.iter().map(|f| f.into()).collect();
		let params = json!([contract_hash.to_hex(), function_name, params, signers, true]);
		self.request("invokefunction", params).await
	}

	/// Invokes a script.
	///
	/// Includes diagnostics from the invocation.
	/// - Parameters:
	///   - scriptHex: The script to invoke
	///   - signers: The signers
	/// - Returns: The request object
	async fn invoke_script_diagnostics(
		&self,
		hex: String,
		signers: Vec<Signer>,
	) -> Result<InvocationResult, ProviderError> {
		let signers: Vec<TransactionSigner> =
			signers.into_iter().map(|signer| signer.into()).collect::<Vec<_>>();
		let script_base64 = serde_json::to_value(hex.from_hex().unwrap().to_base64())?;
		let signers_json = serde_json::to_value(&signers)?;
		let params = vec![script_base64, signers_json, true.to_value()];
		self.request("invokescript", params).await
	}

	/// Returns the results from an iterator.
	///
	/// The results are limited to `count` items. If `count` is greater than `MaxIteratorResultItems` in the Neo Node's configuration file, this request fails.
	/// - Parameters:
	///   - sessionId: The session id
	///   - iteratorId: The iterator id
	///   - count: The maximal number of stack items returned
	/// - Returns: The request object
	async fn traverse_iterator(
		&self,
		session_id: String,
		iterator_id: String,
		count: u32,
	) -> Result<Vec<StackItem>, ProviderError> {
		let params = vec![session_id.to_value(), iterator_id.to_value(), count.to_value()];
		self.request("traverseiterator", params).await
	}

	async fn terminate_session(&self, session_id: &str) -> Result<bool, ProviderError> {
		self.request("terminatesession", vec![session_id.to_value()]).await
	}

	async fn invoke_contract_verify(
		&self,
		hash: H160,
		params: Vec<ContractParameter>,
		signers: Vec<Signer>,
	) -> Result<InvocationResult, ProviderError> {
		let signers: Vec<TransactionSigner> =
			signers.into_iter().map(|signer| signer.into()).collect::<Vec<_>>();
		let params = json!([hash.to_hex(), params, signers]);
		self.request("invokecontractverify", params).await
	}

	fn get_raw_mempool<'life0, 'async_trait>(
		&'life0 self,
	) -> Pin<Box<dyn Future<Output = Result<MemPoolDetails, Self::Error>> + Send + 'async_trait>>
	where
		'life0: 'async_trait,
		Self: 'async_trait,
	{
		todo!()
	}

	fn import_private_key<'life0, 'async_trait>(
		&'life0 self,
		wif: String,
	) -> Pin<Box<dyn Future<Output = Result<NeoAddress, Self::Error>> + Send + 'async_trait>>
	where
		'life0: 'async_trait,
		Self: 'async_trait,
	{
		todo!()
	}

	fn get_block_header_hash<'life0, 'async_trait>(
		&'life0 self,
		hash: H256,
	) -> Pin<Box<dyn Future<Output = Result<NeoBlock, Self::Error>> + Send + 'async_trait>>
	where
		'life0: 'async_trait,
		Self: 'async_trait,
	{
		todo!()
	}

	async fn send_to_address_send_token(
		&self,
		send_token: &TransactionSendToken,
	) -> Result<Transaction, ProviderError> {
		let params = [send_token.to_value()].to_vec();
		self.request("sendtoaddress", params).await
	}

	async fn send_from_send_token(
		&self,
		send_token: &TransactionSendToken,
		from: H160,
	) -> Result<Transaction, ProviderError> {
		let params = json!([
			send_token.token.to_hex(),
			from.to_address(),
			send_token.address,
			send_token.value,
		]);
		// let params = [from.to_value(), vec![send_token.to_value()].into()].to_vec();
		self.request("sendfrom", params).await
	}
}

impl<P: JsonRpcProvider> RpcClient<P> {
	/// Sets the default polling interval for event filters and pending transactions
	/// (default: 7 seconds)
	pub fn set_interval<T: Into<Duration>>(&mut self, interval: T) -> &mut Self {
		self.interval = Some(interval.into());
		self
	}

	/// Sets the default polling interval for event filters and pending transactions
	/// (default: 7 seconds)
	#[must_use]
	pub fn interval<T: Into<Duration>>(mut self, interval: T) -> Self {
		self.set_interval(interval);
		self
	}
}

#[cfg(all(feature = "ipc", any(unix, windows)))]
impl RpcClient<crate::Ipc> {
	#[cfg_attr(unix, doc = "Connects to the Unix socket at the provided path.")]
	#[cfg_attr(windows, doc = "Connects to the named pipe at the provided path.\n")]
	#[cfg_attr(
		windows,
		doc = r"Note: the path must be the fully qualified, like: `\\.\pipe\<name>`."
	)]
	pub async fn connect_ipc(path: impl AsRef<std::path::Path>) -> Result<Self, ProviderError> {
		let ipc = crate::Ipc::connect(path).await?;
		Ok(Self::new(ipc))
	}
}

impl RpcClient<Http> {
	/// The Url to which requests are made
	pub fn url(&self) -> &Url {
		self.provider.url()
	}

	/// Mutable access to the Url to which requests are made
	pub fn url_mut(&mut self) -> &mut Url {
		self.provider.url_mut()
	}
}

impl<Read, Write> RpcClient<RwClient<Read, Write>>
where
	Read: JsonRpcProvider + 'static,
	<Read as JsonRpcProvider>::Error: Sync + Send + 'static + Display,
	Write: JsonRpcProvider + 'static,
	<Write as JsonRpcProvider>::Error: Sync + Send + 'static + Display,
{
	/// Creates a new [RpcClient] with a [RwClient]
	pub fn rw(r: Read, w: Write) -> Self {
		Self::new(RwClient::new(r, w))
	}
}

/// infallible conversion of Bytes to Address/String
///
/// # Panics
///
/// If the provided bytes were not an interpretation of an address
// fn decode_bytes<T: Detokenize>(param: ParamType, bytes: Bytes) -> T {
// 	let tokens = abi::decode(&[param], bytes.as_ref())
// 		.expect("could not abi-decode bytes to address tokens");
// 	T::from_tokens(tokens).expect("could not parse tokens as address")
// }

impl TryFrom<&str> for RpcClient<Http> {
	type Error = ParseError;

	fn try_from(src: &str) -> Result<Self, Self::Error> {
		Ok(RpcClient::new(Http::new(Url::parse(src)?)))
	}
}

impl TryFrom<String> for RpcClient<Http> {
	type Error = ParseError;

	fn try_from(src: String) -> Result<Self, Self::Error> {
		RpcClient::try_from(src.as_str())
	}
}

impl<'a> TryFrom<&'a String> for RpcClient<Http> {
	type Error = ParseError;

	fn try_from(src: &'a String) -> Result<Self, Self::Error> {
		RpcClient::try_from(src.as_str())
	}
}

#[cfg(not(target_arch = "wasm32"))]
impl RpcClient<RetryClient<Http>> {
	/// Create a new [`RetryClient`] by connecting to the provided URL. Errors
	/// if `src` is not a valid URL
	pub fn new_client(src: &str, max_retry: u32, initial_backoff: u64) -> Result<Self, ParseError> {
		Ok(RpcClient::new(RetryClient::new(
			Http::new(Url::parse(src)?),
			Box::new(HttpRateLimitRetryPolicy),
			max_retry,
			initial_backoff,
		)))
	}
}

mod sealed {
	use neo::prelude::{Http, RpcClient};

	/// private trait to ensure extension trait is not implement outside of this crate
	pub trait Sealed {}
	impl Sealed for RpcClient<Http> {}
}

/// Extension trait for `Provider`
///
/// **Note**: this is currently sealed until <https://github.com/gakonst/neo-rs/pull/1267> is finalized
///
/// # Example
///
/// Automatically configure poll interval via `neo_getChainId`
///
/// Note that this will send an RPC to retrieve the network magic.
///
/// ```no_run
///  # use neo_rs::prelude::{Http, Provider, ProviderExt};
///  async fn t() {
/// let http_provider = Provider::<Http>::connect("https://seed1.neo.org:10333").await;
/// # }
/// ```
///
/// This is essentially short for
///
/// ```no_run
/// use std::convert::TryFrom;
/// use NeoRust::prelude::{Http, NeoNetwork, Provider, ProviderExt};
/// let http_provider = Provider::<Http>::try_from("https://seed1.neo.org:10333").unwrap().set_network(NeoNetwork::MainNet.to_magic());
/// ```
#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait ProviderExt: Sealed {
	/// The error type that can occur when creating a provider
	type Error: Debug;

	/// Creates a new instance connected to the given `url`, exit on error
	async fn connect(url: &str) -> Self
	where
		Self: Sized,
	{
		Self::try_connect(url).await.unwrap()
	}

	/// Try to create a new `Provider`
	async fn try_connect(url: &str) -> Result<Self, Self::Error>
	where
		Self: Sized;

	/// Customize `Provider` settings for chain.
	///
	/// E.g. [`Chain::average_blocktime_hint()`] returns the average block time which can be used to
	/// tune the polling interval.
	///
	/// Returns the customized `Provider`
	fn for_network(mut self, network: u32) -> Self
	where
		Self: Sized,
	{
		self.set_network(network);
		self
	}

	/// Customized `Provider` settings for chain
	fn set_network(&mut self, network: u32) -> &mut Self;
}

#[cfg_attr(target_arch = "wasm32", async_trait(? Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl ProviderExt for RpcClient<Http> {
	type Error = ParseError;

	async fn try_connect(url: &str) -> Result<Self, Self::Error>
	where
		Self: Sized,
	{
		let mut provider = RpcClient::try_from(url)?;
		let Some(network) = provider.get_version().await.ok() else { panic!("") };
		provider.set_network(network.protocol.unwrap().network);

		Ok(provider)
	}

	fn set_network(&mut self, network: u32) -> &mut Self {
		self.set_interval(Duration::from_millis(network as u64 / 2));
		self
	}
}

/// Returns true if the endpoint is local
///
/// # Example
///
/// ```
/// use NeoRust::prelude::is_local_endpoint;
/// assert!(is_local_endpoint("http://localhost:8545"));
/// assert!(is_local_endpoint("http://test.localdev.me"));
/// assert!(is_local_endpoint("http://169.254.0.0:8545"));
/// assert!(is_local_endpoint("http://127.0.0.1:8545"));
/// assert!(!is_local_endpoint("http://206.71.50.230:8545"));
/// assert!(!is_local_endpoint("http://[2001:0db8:85a3:0000:0000:8a2e:0370:7334]"));
/// assert!(is_local_endpoint("http://[::1]"));
/// assert!(!is_local_endpoint("havenofearlucishere"));
/// ```
#[inline]
pub fn is_local_endpoint(endpoint: &str) -> bool {
	if let Ok(url) = Url::parse(endpoint) {
		if let Some(host) = url.host() {
			return match host {
				Host::Domain(domain) =>
					domain.contains("localhost") || domain.contains("localdev.me"),
				Host::Ipv4(ipv4) =>
					ipv4 == Ipv4Addr::LOCALHOST
						|| ipv4.is_link_local() || ipv4.is_loopback()
						|| ipv4.is_private(),
				Host::Ipv6(ipv6) => ipv6.is_loopback(),
			};
		}
	}
	false
}

#[cfg(test)]
mod tests {
	use std::{str::FromStr, sync::Mutex};

	use blake2::digest::Mac;
	use lazy_static::lazy_static;
	use log::debug;
	use primitive_types::{H160, H256};
	use rustc_serialize::{
		base64::FromBase64,
		hex::{FromHex, ToHex},
	};
	use serde_json::{json, Value};
	use tokio::{self, sync::OnceCell};
	use tracing::field::debug;
	use url::Url;
	use wiremock::{
		matchers::{body_json, method as http_method, method, path},
		Mock, MockServer, ResponseTemplate,
	};

	use neo::prelude::{
		AccountSigner, HttpProvider, ScriptHashExtension, Secp256r1PublicKey, Signer::Account,
		SignerTrait, TestConstants, TransactionSendToken, WitnessAction, WitnessCondition,
		WitnessRule,
	};

	use crate::{
		neo_types::{Base64Encode, ToBase64},
		prelude::{MockClient, NativeContractState},
		providers::RpcClient,
	};

	use crate::neo_clients::api_trait::APITrait;

	async fn setup_mock_server() -> MockServer {
		MockServer::start().await
	}

	async fn mock_rpc_response(
		mock_server: &MockServer,
		rpc_method: &str,
		params: serde_json::Value,
		result: serde_json::Value,
	) -> RpcClient<HttpProvider> {
		Mock::given(http_method("POST"))
			.and(path("/"))
			.and(body_json(json!({
				"jsonrpc": "2.0",
				"method": rpc_method,
				"params": params,
				"id": 1
			})))
			.respond_with(ResponseTemplate::new(200).set_body_json(json!({
				"jsonrpc": "2.0",
				"id": 1,
				"result": result
			})))
			.mount(mock_server)
			.await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		RpcClient::new(http_client)
	}

	#[tokio::test]
	async fn test_get_best_block_hash() {
		let _ = env_logger::builder().is_test(true).try_init();

		let mock_provider = MockClient::new().await;
		mock_provider
			.mock_response(
				"getbestblockhash",
				json!([]),
				json!("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"),
			)
			.await;
		let provider = mock_provider.into_client();
		let result = provider.get_best_block_hash().await;
		assert!(result.is_ok(), "Result is not okay: {:?}", result);

		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getbestblockhash",
            "params": [],
            "id": 1
        }"#;
		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		verify_request(mock_provider.server(), expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_block_hash() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response(
			&mock_server,
			"getblockhash",
			json!([16293]),
			json!("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"),
		)
		.await;

		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getblockhash",
            "params": [16293],
            "id": 1
        }"#;

		let result = provider.get_block_hash(16293).await;
		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_block_index() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response(
            &mock_server,
            "getblock",
            json!([12345, 1]),
            json!( {
        "hash": "0x1de7e5eaab0f74ac38f5191c038e009d3c93ef5c392d1d66fa95ab164ba308b8",
        "size": 1217,
        "version": 0,
        "previousblockhash": "0x045cabde4ecbd50f5e4e1b141eaf0842c1f5f56517324c8dcab8ccac924e3a39",
        "merkleroot": "0x6afa63201b88b55ad2213e5a69a1ad5f0db650bc178fc2bedd2fb301c1278bf7",
        "time": 1539968858,
        "index": 1914006,
        "nextconsensus": "AWZo4qAxhT8fwKL93QATSjCYCgHmCY1XLB",
        "witnesses": [
            {
                "invocation": "DEBJVWapboNkCDlH9uu+tStOgGnwODlolRifxTvQiBkhM0vplSPo4vMj9Jt3jvzztMlwmO75Ss5cptL8wUMxASjZ",
                "verification": "EQwhA/HsPB4oPogN5unEifDyfBkAfFM4WqpMDJF8MgB57a3yEQtBMHOzuw=="
            }
        ],
        "tx": [],
        "confirmations": 7878,
        "nextblockhash": "0x4a97ca89199627f877b6bffe865b8327be84b368d62572ef20953829c3501643"
    }),
        ).await;

		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getblock",
            "params": [12345,1],
            "id": 1
        }"#;

		let result = provider.get_block_by_index(12345, true).await;

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_block_index_only_header() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response(
            &mock_server,
            "getblockheader",
            json!([12345,1]),
            json!({
        "hash": "0x1de7e5eaab0f74ac38f5191c038e009d3c93ef5c392d1d66fa95ab164ba308b8",
        "size": 1217,
        "version": 0,
        "previousblockhash": "0x045cabde4ecbd50f5e4e1b141eaf0842c1f5f56517324c8dcab8ccac924e3a39",
        "merkleroot": "0x6afa63201b88b55ad2213e5a69a1ad5f0db650bc178fc2bedd2fb301c1278bf7",
        "time": 1539968858,
        "index": 1914006,
        "nextconsensus": "AWZo4qAxhT8fwKL93QATSjCYCgHmCY1XLB",
        "witnesses": [
            {
                "invocation": "DEBJVWapboNkCDlH9uu+tStOgGnwODlolRifxTvQiBkhM0vplSPo4vMj9Jt3jvzztMlwmO75Ss5cptL8wUMxASjZ",
                "verification": "EQwhA/HsPB4oPogN5unEifDyfBkAfFM4WqpMDJF8MgB57a3yEQtBMHOzuw=="
            }
        ],
        "tx": [
            {
                "hash": "0x46eca609a9a8c8340ee56b174b04bc9c9f37c89771c3a8998dc043f5a74ad510",
                "size": 267,
                "version": 0,
                "nonce": 565086327,
                "sender": "AHE5cLhX5NjGB5R2PcdUvGudUoGUBDeHX4",
                "sysfee": "0",
                "netfee": "0",
                "validuntilblock": 2107425,
                "signers": [
                    {
                        "account": "0xf68f181731a47036a99f04dad90043a744edec0f",
                        "scopes": "CalledByEntry"
                    }
                ],
                "attributes": [],
                "script": "AGQMFObBATZUrxE9ipaL3KUsmUioK5U9DBQP7O1Ep0MA2doEn6k2cKQxFxiP9hPADAh0cmFuc2ZlcgwUiXcg2M129PAKv6N8Dt2InCCP3ptBYn1bUjg",
                "witnesses": [
                    {
                        "invocation": "DEBR7EQOb1NUjat1wrINzBNKOQtXoUmRVZU8h5c8K5CLMCUVcGkFVqAAGUJDh3mVcz6sTgXvmMuujWYrBveeM4q+",
                        "verification": "EQwhA/HsPB4oPogN5unEifDyfBkAfFM4WqpMDJF8MgB57a3yEQtBMHOzuw=="
                    }
                ]
            },
            {
                "hash": "0x46eca609a9a8c8340ee56b174b04bc9c9f37c89771c3a8998dc043f5a74ad510",
                "size": 267,
                "version": 0,
                "nonce": 565086327,
                "sender": "AHE5cLhX5NjGB5R2PcdUvGudUoGUBDeHX4",
                "sysfee": "0",
                "netfee": "0",
                "validuntilblock": 2107425,
                "signers": [
                    {
                        "account": "0xf68f181731a47036a99f04dad90043a744edec0f",
                        "scopes": "CalledByEntry"
                    }
                ],
                "attributes": [],
                "script": "AGQMFObBATZUrxE9ipaL3KUsmUioK5U9DBQP7O1Ep0MA2doEn6k2cKQxFxiP9hPADAh0cmFuc2ZlcgwUiXcg2M129PAKv6N8Dt2InCCP3ptBYn1bUjg",
                "witnesses": [
                    {
                        "invocation": "DEBR7EQOb1NUjat1wrINzBNKOQtXoUmRVZU8h5c8K5CLMCUVcGkFVqAAGUJDh3mVcz6sTgXvmMuujWYrBveeM4q+",
                        "verification": "EQwhA/HsPB4oPogN5unEifDyfBkAfFM4WqpMDJF8MgB57a3yEQtBMHOzuw=="
                    }
                ]
            }
        ],
        "confirmations": 7878,
        "nextblockhash": "0x4a97ca89199627f877b6bffe865b8327be84b368d62572ef20953829c3501643"
    }),
        ).await;

		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getblockheader",
            "params": [12345,1],
            "id": 1
        }"#;

		let result = provider.get_block_by_index(12345, false).await;

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_block_by_hash() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response(
            &mock_server,
            "getblock",
            json!(["2240b34669038f82ac492150d391dfc3d7fe5e3c1d34e5b547d50e99c09b468d", 1]),
            json!({
        "hash": "0x2240b34669038f82ac492150d391dfc3d7fe5e3c1d34e5b547d50e99c09b468d",
        "size": 1217,
        "version": 0,
        "previousblockhash": "0x045cabde4ecbd50f5e4e1b141eaf0842c1f5f56517324c8dcab8ccac924e3a39",
        "merkleroot": "0x6afa63201b88b55ad2213e5a69a1ad5f0db650bc178fc2bedd2fb301c1278bf7",
        "time": 1539968858,
        "index": 1914006,
        "nextconsensus": "AWZo4qAxhT8fwKL93QATSjCYCgHmCY1XLB",
        "witnesses": [
            {
                "invocation": "DEBJVWapboNkCDlH9uu+tStOgGnwODlolRifxTvQiBkhM0vplSPo4vMj9Jt3jvzztMlwmO75Ss5cptL8wUMxASjZ",
                "verification": "EQwhA/HsPB4oPogN5unEifDyfBkAfFM4WqpMDJF8MgB57a3yEQtBMHOzuw=="
            }
        ],
        "tx": [
            {
                "hash": "0x46eca609a9a8c8340ee56b174b04bc9c9f37c89771c3a8998dc043f5a74ad510",
                "size": 267,
                "version": 0,
                "nonce": 565086327,
                "sender": "AHE5cLhX5NjGB5R2PcdUvGudUoGUBDeHX4",
                "sysfee": "0",
                "netfee": "0",
                "validuntilblock": 2107425,
                "signers": [
                    {
                        "account": "0xf68f181731a47036a99f04dad90043a744edec0f",
                        "scopes": "CalledByEntry"
                    }
                ],
                "attributes": [],
                "script": "AGQMFObBATZUrxE9ipaL3KUsmUioK5U9DBQP7O1Ep0MA2doEn6k2cKQxFxiP9hPADAh0cmFuc2ZlcgwUiXcg2M129PAKv6N8Dt2InCCP3ptBYn1bUjg",
                "witnesses": [
                    {
                        "invocation": "DEBR7EQOb1NUjat1wrINzBNKOQtXoUmRVZU8h5c8K5CLMCUVcGkFVqAAGUJDh3mVcz6sTgXvmMuujWYrBveeM4q+",
                        "verification": "EQwhA/HsPB4oPogN5unEifDyfBkAfFM4WqpMDJF8MgB57a3yEQtBMHOzuw=="
                    }
                ]
            },
            {
                "hash": "0x46eca609a9a8c8340ee56b174b04bc9c9f37c89771c3a8998dc043f5a74ad510",
                "size": 267,
                "version": 0,
                "nonce": 565086327,
                "sender": "AHE5cLhX5NjGB5R2PcdUvGudUoGUBDeHX4",
                "sysfee": "0",
                "netfee": "0",
                "validuntilblock": 2107425,
                "signers": [
                    {
                        "account": "0xf68f181731a47036a99f04dad90043a744edec0f",
                        "scopes": "CalledByEntry"
                    }
                ],
                "attributes": [],
                "script": "AGQMFObBATZUrxE9ipaL3KUsmUioK5U9DBQP7O1Ep0MA2doEn6k2cKQxFxiP9hPADAh0cmFuc2ZlcgwUiXcg2M129PAKv6N8Dt2InCCP3ptBYn1bUjg",
                "witnesses": [
                    {
                        "invocation": "DEBR7EQOb1NUjat1wrINzBNKOQtXoUmRVZU8h5c8K5CLMCUVcGkFVqAAGUJDh3mVcz6sTgXvmMuujWYrBveeM4q+",
                        "verification": "EQwhA/HsPB4oPogN5unEifDyfBkAfFM4WqpMDJF8MgB57a3yEQtBMHOzuw=="
                    }
                ]
            }
        ],
        "confirmations": 7878,
        "nextblockhash": "0x4a97ca89199627f877b6bffe865b8327be84b368d62572ef20953829c3501643"
    })).await;

		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getblock",
            "params": ["2240b34669038f82ac492150d391dfc3d7fe5e3c1d34e5b547d50e99c09b468d",1],
            "id": 1
        }"#;

		let result = provider
			.get_block(
				H256::from_str(
					"0x2240b34669038f82ac492150d391dfc3d7fe5e3c1d34e5b547d50e99c09b468d",
				)
				.unwrap(),
				true,
			)
			.await;

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_block_not_full_tx_objects() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response(
			&mock_server,
			"getblockhash",
			json!([16293]),
			json!("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"),
		)
		.await;

		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getblockheader",
            "params": ["2240b34669038f82ac492150d391dfc3d7fe5e3c1d34e5b547d50e99c09b468d",1],
            "id": 1
        }"#;

		let result = provider
			.get_block(
				H256::from_str(
					"0x2240b34669038f82ac492150d391dfc3d7fe5e3c1d34e5b547d50e99c09b468d",
				)
				.unwrap(),
				false,
			)
			.await;
		// assert!(result.is_ok(), "Result is not okay: {:?}", result);
		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_raw_block_index() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response(
            &mock_server,
            "getblock",
            json!([12345, 0]),
            json!("AAAAAM5doa+yo+aKrc8RO/Pfo96BYyedF2ed+jODYAzESzgvm458FX3T6b9rYw2KSBTWfttaeiA9McxN1LiWuQwI1O/6eEAaeAEAAKhnAAAG+CRl3iIpI2tQ6MaMSCq5GLum1uwB/UoBDEA2ZR0uduwN/5tFVCKoHJtAnSJINfqlRDcNXnYl0H0Jcb3YBy1M0G4Z1LB3PQMIb6J4kOtFm7TBL0B6vfPuDpigDEBLHKna+SPlL9vn755blCr3vxvc2HLP5dUch0isPARVDbg24QwVuvx3mbQ6awn0cQ/h+Jym/9xFo0MR0ddKXKzCDEAIcXyoqzFq4+3N9JtyK46LLeyx9ikidPLiXg9HWQk9Ps5wx9+XIe8zziS9dRAOqT4od7tW1SA6cRU3U8ZCexJ1DECxN+nyE15RaIvwpJ0JK3/RJvAM++YKou/ljVef5atDx9pce5nkuibfZzvzrjcoJF53qnP8ZSXBKThjrN2kGCxlDEBSVfUQAFNm4j4KihACjt5Kx1A9hN8RIDQkLw7udpcFm7Nf4PtDtucw7pS7WGEkmu3c+yt2BV3KeCzNBAreypb7/BUMIQMCJCsdztY+G/frFIdvfvAmt5Vn+cW+g94ZQ90YXsKOaAwhAg34hYtm/017Cmpo0R3e3MfZDSpk/6LNCHxMXav0FQtADCECF5VDAAGEeB5UR7Pw+6zmZOqSt+MSJ8jnG8Tnza/M244MIQJehJSQO5PcNp8Ior1+Ih9XTHXZZ1WR8EkHy6na7rg9EAwhA4QV0L6NwSth0+O3a5j0ZN+rf93udCccNeLeYku1ECOmDCEDybHInG4tSr1imi24t9A6ztUYpWeTvJD0mF737T8bSBoMIQPoq1GG4d6rzRDsDlCd7U//reb931NKw+BQYmi64/1EphdBe85spQEAKihGP0BY1TwAAAAAAKBVAAAAAAAnfgAAAW78pd5nDtnbpGznfCdo9+vKgwVMAAD9dBENSgh7Im5hbWUiOiJUb2tlbiBOYW1lIiwiZ3JvdXBzIjpbXSwic3VwcG9ydGVkc3RhbmRhcmRzIjpbIk5FUDE3IiwiTkVQMTAiXSwiYWJpIjp7Im1ldGhvZHMiOlt7Im5hbWUiOiJfZGVwbG95IiwicGFyYW1ldGVycyI6W3sibmFtZSI6ImRhdGEiLCJ0eXBlIjoiQW55In0seyJuYW1lIjoidXBkYXRlIiwidHlwZSI6IkJvb2xlYW4ifV0sInJldHVybnR5cGUiOiJWb2lkIiwib2Zmc2V0IjowLCJzYWZlIjpmYWxzZX0seyJuYW1lIjoiX2luaXRpYWxpemUiLCJwYXJhbWV0ZXJzIjpbXSwicmV0dXJudHlwZSI6IlZvaWQiLCJvZmZzZXQiOjI3Miwic2FmZSI6ZmFsc2V9LHsibmFtZSI6ImJhbGFuY2VPZiIsInBhcmFtZXRlcnMiOlt7Im5hbWUiOiJhY2NvdW50IiwidHlwZSI6Ikhhc2gxNjAifV0sInJldHVybnR5cGUiOiJJbnRlZ2VyIiwib2Zmc2V0IjozNjgsInNhZmUiOmZhbHNlfSx7Im5hbWUiOiJkZWNpbWFscyIsInBhcmFtZXRlcnMiOltdLCJyZXR1cm50eXBlIjoiSW50ZWdlciIsIm9mZnNldCI6NTE1LCJzYWZlIjpmYWxzZX0seyJuYW1lIjoiZGVzdHJveSIsInBhcmFtZXRlcnMiOltdLCJyZXR1cm50eXBlIjoiVm9pZCIsIm9mZnNldCI6NTM3LCJzYWZlIjpmYWxzZX0seyJuYW1lIjoiZGlzYWJsZVBheW1lbnQiLCJwYXJhbWV0ZXJzIjpbXSwicmV0dXJudHlwZSI6IlZvaWQiLCJvZmZzZXQiOjYwNywic2FmZSI6ZmFsc2V9LHsibmFtZSI6ImVuYWJsZVBheW1lbnQiLCJwYXJhbWV0ZXJzIjpbXSwicmV0dXJudHlwZSI6IlZvaWQiLCJvZmZzZXQiOjY3OSwic2FmZSI6ZmFsc2V9LHsibmFtZSI6Im9uTkVQMTdQYXltZW50IiwicGFyYW1ldGVycyI6W3sibmFtZSI6ImZyb20iLCJ0eXBlIjoiSGFzaDE2MCJ9LHsibmFtZSI6ImFtb3VudCIsInR5cGUiOiJJbnRlZ2VyIn0seyJuYW1lIjoiZGF0YSIsInR5cGUiOiJBbnkifV0sInJldHVybnR5cGUiOiJWb2lkIiwib2Zmc2V0IjoxMjMwLCJzYWZlIjpmYWxzZX0seyJuYW1lIjoic3ltYm9sIiwicGFyYW1ldGVycyI6W10sInJldHVybnR5cGUiOiJTdHJpbmciLCJvZmZzZXQiOjE2MDIsInNhZmUiOmZhbHNlfSx7Im5hbWUiOiJ0ZXN0Y29udHJhY3QiLCJwYXJhbWV0ZXJzIjpbXSwicmV0dXJudHlwZSI6IlN0cmluZyIsIm9mZnNldCI6MTYzMywic2FmZSI6ZmFsc2V9LHsibmFtZSI6InRlc3RkeW5hbWljY2FsbCIsInBhcmFtZXRlcnMiOlt7Im5hbWUiOiJoYXNoIiwidHlwZSI6Ikhhc2gxNjAifSx7Im5hbWUiOiJtZXRob2QiLCJ0eXBlIjoiU3RyaW5nIn1dLCJyZXR1cm50eXBlIjoiVm9pZCIsIm9mZnNldCI6MTY0Miwic2FmZSI6ZmFsc2V9LHsibmFtZSI6InRvdGFsU3VwcGx5IiwicGFyYW1ldGVycyI6W10sInJldHVybnR5cGUiOiJJbnRlZ2VyIiwib2Zmc2V0IjoxNjU4LCJzYWZlIjpmYWxzZX0seyJuYW1lIjoidHJhbnNmZXIiLCJwYXJhbWV0ZXJzIjpbeyJuYW1lIjoiZnJvbSIsInR5cGUiOiJIYXNoMTYwIn0seyJuYW1lIjoidG8iLCJ0eXBlIjoiSGFzaDE2MCJ9LHsibmFtZSI6ImFtb3VudCIsInR5cGUiOiJJbnRlZ2VyIn0seyJuYW1lIjoiZGF0YSIsInR5cGUiOiJBbnkifV0sInJldHVybnR5cGUiOiJCb29sZWFuIiwib2Zmc2V0IjoxNjY0LCJzYWZlIjpmYWxzZX0seyJuYW1lIjoidXBkYXRlIiwicGFyYW1ldGVycyI6W3sibmFtZSI6Im5lZkZpbGUiLCJ0eXBlIjoiU3RyaW5nIn0seyJuYW1lIjoibWFuaWZlc3QiLCJ0eXBlIjoiU3RyaW5nIn1dLCJyZXR1cm50eXBlIjoiVm9pZCIsIm9mZnNldCI6MTk5Miwic2FmZSI6ZmFsc2V9LHsibmFtZSI6InZlcmlmeSIsInBhcmFtZXRlcnMiOltdLCJyZXR1cm50eXBlIjoiQm9vbGVhbiIsIm9mZnNldCI6MjA1OCwic2FmZSI6ZmFsc2V9XSwiZXZlbnRzIjpbeyJuYW1lIjoiVHJhbnNmZXIiLCJwYXJhbWV0ZXJzIjpbeyJuYW1lIjoiYXJnMSIsInR5cGUiOiJIYXNoMTYwIn0seyJuYW1lIjoiYXJnMiIsInR5cGUiOiJIYXNoMTYwIn0seyJuYW1lIjoiYXJnMyIsInR5cGUiOiJJbnRlZ2VyIn1dfV19LCJwZXJtaXNzaW9ucyI6W3siY29udHJhY3QiOiIqIiwibWV0aG9kcyI6IioifV0sInRydXN0cyI6W10sImV4dHJhIjp7IkF1dGhvciI6Ik5lbyIsIkVtYWlsIjoiZGV2QG5lby5vcmciLCJEZXNjcmlwdGlvbiI6IlRoaXMgaXMgYSBORVAxNyBleGFtcGxlIn19Df4ITkVGM25lb24tMy4wLjAuMAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAXA7znO4OTpJcbCoGp54UQN2G/OrARpdG9hAgABD/2j+kNG6lMqJY/El92t22Q3yf3/C2dldENvbnRyYWN0AQABD/2j+kNG6lMqJY/El92t22Q3yf3/BnVwZGF0ZQMAAA/9o/pDRupTKiWPxJfdrdtkN8n9/wdkZXN0cm95AAAAD0OkVyMwdLSq1p8g/WmykRNSvvAeBXRlc3QyAAABDwAA/RAIVwICeXBoJgcjBwEAACE1JgMAABC3cWkmIAwbQ29udHJhY3QgaGFzIGJlZW4gZGVwbG95ZWQuOgwcYWZ0ZXIgZ2V0IHRvdGFsU3VwcGx5U3RvcmFnZUHP50eWDAdBQpxJ/RoH2yE1aAMAAAwhYWZ0ZXIgaW5jcmVhc2UgdG90YWxTdXBwbHlzdG9yYWdlQc/nR5YMFG78pd5nDtnbpGznfCdo9+vKgwVMDAdBQpxJ/RoH2yFQNQYDAAAMG2FmdGVyIGluY3JlYXNlIGFzc2V0c3RvcmFnZUHP50eWCwwUbvyl3mcO2dukbOd8J2j368qDBUwMB0FCnEn9GgfbIVMTwAwIVHJhbnNmZXJBlQFvYUBWCAwIAACKXXhFYwHbIWAMB0FCnEn9GgfbIWEMFG78pd5nDtnbpGznfCdo9+vKgwVMYgwEAJQ1d9shYxFkDAVhc3NldGUMCGNvbnRyYWN0ZgwLdG90YWxTdXBwbHlnB0BXAgF4NX4GAAAQs3BoJkEMPFRoZSBwYXJhbWV0ZXJzIGFjY291bnQgU0hPVUxEIGJlIGEgMjAtYnl0ZSBub24temVybyBhZGRyZXNzLjoMFnZhbGlkIGFkZHJlc3MgY29tcGxldGVBz+dHlng18QAAAHFpQFcBAnh52zBQNAVwaEBXAQISw0p4EFDQSnkRUNBwaEAYQFcBAngRznmLcHgQzmhQQS9Yxe1AVwEANcsBAAAQs3BoJhYMEU5vIGF1dGhvcml6YXRpb24uOiE3AwBAQZv2Z84MBWFzc2V0UDSXDAZlbmFibGUQUzWMAwAAQFcBADWFAQAAELNwaCYWDBFObyBhdXRob3JpemF0aW9uLjohNL5AQZv2Z84MBWFzc2V0UDVS////DAZlbmFibGURUzVEAwAAQFcBADU9AQAAELNwaCYWDBFObyBhdXRob3JpemF0aW9uLjohNLtAVwMBQZv2Z84MBWFzc2V0UDUH////eFA1kAAAAHAMF2NoZWNrIGlmIHN0YXR1cyBpcyBudWxsQc/nR5Zo2HFpJgYQciIiIWhK2CYFEFBF2yEaUDcAAEHP50eWaErYJgUQUEXbIXJqQFcDAEGb9mfODAhjb250cmFjdFA1nf7//wwLdG90YWxTdXBwbHlQNBpwaNhxaSYGEHIiDWhK2CYFEFBF2yFyakBXAgJ4Ec55i3B4EM5oUEGSXegxcWlAVwIAQZv2Z84MBWFzc2V0UDVK/v//DAZlbmFibGVQNMxK2CYFEFBF2yFwaBGzcWlAVwACeHg1Dv///3meUDXZAQAAQFcAATVk////eJ415AEAAEBXAAF4NwEA2KpADBRu/KXeZw7Z26Rs53wnaPfryoMFTEH4J+yMQFcGATUu////cGgQtnNrJhsMFkNvbnRyYWN0IG5vdCBkZXBsb3llZC46DAgAAIpdeEVjAdshaJ9xeBC2dGwmGwwWQW1vdW50IGNhbm5vdCBiZSB6ZXJvLjp4abd1bSY/eBpQNwAAQc/nR5ZpGlA3AABBz+dHlgwkSW5zdWZmaWNpZW50IHN1cHBseSBmb3IgbWludCB0b2tlbnMuOiFBLVEIMHJqE854UDUP////eDUc////C2oTznhTE8AMCFRyYW5zZmVyQZUBb2FAVwQDNbn+//9waCefAAAAQTlTbjwMFPVj6kC8KD1NDgXEjqMFs/Kgc0Dvs3FpJikMD21pbnQgbmVvIHRva2Vuc0HP50eWeQwEAJQ1d9shoDXm/v//IlIhQTlTbjwMFM924ovQBixKR47jVWEBExnzz6TSs3JqJhR42KpzayYKeRGgNbT+//8hIh8hDBlXcm9uZyBjYWxsaW5nIHNjcmlwdCBoYXNoOiEiKiEMJFBheW1lbnQgaXMgZGlzYWJsZSBvbiB0aGlzIGNvbnRyYWN0ITpAVwACQZv2Z84MBWFzc2V0UDU0/P//eHlTNC1AVwABQZv2Z84MCGNvbnRyYWN0UDUW/P//DAt0b3RhbFN1cHBseXhTNANAVwEDeBHOeYtweBDOaHpTQeY/GIRAVwICeDXP/P//cGh5s3FpJgd4NBkiCXhoeZ9QNItAVwABNRn9//94nzSZQFcAAUGb9mfODAVhc3NldFA1r/v//3hQNcn7//9ADBxUZXN0Q29udHJhY3RNYW5hZ2VtZW50VXBkYXRlQFcBADcEAHBoQFcAAnh5HxDDVEFifVtSRUA1uvz//0BXBwR4NW4BAAAmDHk1ZgEAABCzIgMRcGgmRQxAVGhlIHBhcmFtZXRlcnMgZnJvbSBhbmQgdG8gU0hPVUxEIGJlIDIwLWJ5dGUgbm9uLXplcm8gYWRkcmVzc2VzLjp6ELZxaSYxDCxUaGUgcGFyYW1ldGVyIGFtb3VudCBNVVNUIGJlIGdyZWF0ZXIgdGhhbiAwLjp4Qfgn7IwkDXhBOVNuPJcQsyIDEHJqJhYMEU5vIGF1dGhvcml6YXRpb24uOng1jfv//3q1c2smGgwVSW5zdWZmaWNpZW50IGJhbGFuY2UuOnh5s3RsJgYRdSJaeHpQNYn+//95elA1Qfz//3h5elMTwAwIVHJhbnNmZXJBlQFvYXk1Sfz//3ZuJil5DA5vbk5FUDE3UGF5bWVudB8Tw0oQeNBKEXrQShJ70FRBYn1bUkURdW1AVwECNRz8//8Qs3BoJhYMEU5vIGF1dGhvcml6YXRpb24uOnh5C1M3AgBAVwABeErZKFDKABSzqyYJeBCzELMiAxBANd37//9AtFz2MBLAHwwGZGVwbG95DBT9o/pDRupTKiWPxJfdrdtkN8n9/0FifVtSAUIMQHefw35eIZrwfE+JXNTEqoXbBmbMyxCK8j07RU26X+GWNle4ynNroOlNlniZ+mWprVE2lybXFAOjtw6hVHX2frooDCECVOanJ/RSNsd2Itpgwx8fHJooXf3zbTYO/4EAs9Oud+FBdHR2qg=="),
        ).await;

		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getblock",
            "params": [12345,0],
            "id": 1
        }"#;

		let result = provider.get_raw_block_by_index(12345).await;
		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_block_header_count() {
		let mock_server = setup_mock_server().await;
		let provider =
			mock_rpc_response(&mock_server, "getblockheadercount", json!([]), json!(256)).await;
		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getblockheadercount",
            "params": [],
            "id": 1
        }"#;

		let result = provider.get_block_header_count().await;
		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_block_count() {
		let mock_server = setup_mock_server().await;
		let provider =
			mock_rpc_response(&mock_server, "getblockcount", json!([]), json!(256)).await;
		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getblockcount",
            "params": [],
            "id": 1
        }"#;

		let result = provider.get_block_count().await;

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_native_contracts() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response(
			&mock_server,
			"getnativecontracts",
			json!([]),
			json!([
				{
					"id": -1,
					"hash": "0xfffdc93764dbaddd97c48f252a53ea4643faa3fd",
					"nef": {
						"magic": 860243278,
						"compiler": "neo-core-v3.0",
						"source": "",
						"tokens": [],
						"script": "EEEa93tnQBBBGvd7Z0AQQRr3e2dAEEEa93tnQBBBGvd7Z0AQQRr3e2dAEEEa93tnQBBBGvd7Z0A=",
						"checksum": 1110259869
					},
					"manifest": {
						"name": "ContractManagement",
						"groups": [],
						"features": {},
						"supportedstandards": [],
						"abi": {
							"methods": [
								{
									"name": "deploy",
									"parameters": [
										{
											"name": "nefFile",
											"type": "ByteArray"
										},
										{
											"name": "manifest",
											"type": "ByteArray"
										}
									],
									"returntype": "Array",
									"offset": 0,
									"safe": false
								},
								{
									"name": "deploy",
									"parameters": [
										{
											"name": "nefFile",
											"type": "ByteArray"
										},
										{
											"name": "manifest",
											"type": "ByteArray"
										},
										{
											"name": "data",
											"type": "Any"
										}
									],
									"returntype": "Array",
									"offset": 7,
									"safe": false
								},
								{
									"name": "destroy",
									"parameters": [],
									"returntype": "Void",
									"offset": 14,
									"safe": false
								},
								{
									"name": "getContract",
									"parameters": [
										{
											"name": "hash",
											"type": "Hash160"
										}
									],
									"returntype": "Array",
									"offset": 21,
									"safe": true
								},
								{
									"name": "getMinimumDeploymentFee",
									"parameters": [],
									"returntype": "Integer",
									"offset": 28,
									"safe": true
								},
								{
									"name": "setMinimumDeploymentFee",
									"parameters": [
										{
											"name": "value",
											"type": "Integer"
										}
									],
									"returntype": "Void",
									"offset": 35,
									"safe": false
								},
								{
									"name": "update",
									"parameters": [
										{
											"name": "nefFile",
											"type": "ByteArray"
										},
										{
											"name": "manifest",
											"type": "ByteArray"
										}
									],
									"returntype": "Void",
									"offset": 42,
									"safe": false
								},
								{
									"name": "update",
									"parameters": [
										{
											"name": "nefFile",
											"type": "ByteArray"
										},
										{
											"name": "manifest",
											"type": "ByteArray"
										},
										{
											"name": "data",
											"type": "Any"
										}
									],
									"returntype": "Void",
									"offset": 49,
									"safe": false
								}
							],
							"events": [
								{
									"name": "Deploy",
									"parameters": [
										{
											"name": "Hash",
											"type": "Hash160"
										}
									]
								},
								{
									"name": "Update",
									"parameters": [
										{
											"name": "Hash",
											"type": "Hash160"
										}
									]
								},
								{
									"name": "Destroy",
									"parameters": [
										{
											"name": "Hash",
											"type": "Hash160"
										}
									]
								}
							]
						},
						"permissions": [
							{
								"contract": "*",
								"methods": "*"
							}
						],
						"trusts": [],
						"extra": null
					},
					"updatehistory": [0]
				}
			]),
		)
		.await;

		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getnativecontracts",
            "params": [],
            "id": 1
        }"#;

		let result = provider.get_native_contracts().await;

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_block_header_index() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response(
            &mock_server,
            "getblockheader",
            json!([12345,1]),
            json!({
        "hash": "0x2240b34669038f82ac492150d391dfc3d7fe5e3c1d34e5b547d50e99c09b468d",
        "size": 1217,
        "version": 0,
        "previousblockhash": "0x045cabde4ecbd50f5e4e1b141eaf0842c1f5f56517324c8dcab8ccac924e3a39",
        "merkleroot": "0x6afa63201b88b55ad2213e5a69a1ad5f0db650bc178fc2bedd2fb301c1278bf7",
        "time": 1539968858,
        "index": 1914006,
        "nextconsensus": "AWZo4qAxhT8fwKL93QATSjCYCgHmCY1XLB",
        "witnesses": [
            {
                "invocation": "DEBJVWapboNkCDlH9uu+tStOgGnwODlolRifxTvQiBkhM0vplSPo4vMj9Jt3jvzztMlwmO75Ss5cptL8wUMxASjZ",
                "verification": "EQwhA/HsPB4oPogN5unEifDyfBkAfFM4WqpMDJF8MgB57a3yEQtBMHOzuw=="
            }
        ],
        "tx": [
            {
                "hash": "0x46eca609a9a8c8340ee56b174b04bc9c9f37c89771c3a8998dc043f5a74ad510",
                "size": 267,
                "version": 0,
                "nonce": 565086327,
                "sender": "AHE5cLhX5NjGB5R2PcdUvGudUoGUBDeHX4",
                "sysfee": "0",
                "netfee": "0",
                "validuntilblock": 2107425,
                "signers": [
                    {
                        "account": "0xf68f181731a47036a99f04dad90043a744edec0f",
                        "scopes": "CalledByEntry"
                    }
                ],
                "attributes": [],
                "script": "AGQMFObBATZUrxE9ipaL3KUsmUioK5U9DBQP7O1Ep0MA2doEn6k2cKQxFxiP9hPADAh0cmFuc2ZlcgwUiXcg2M129PAKv6N8Dt2InCCP3ptBYn1bUjg",
                "witnesses": [
                    {
                        "invocation": "DEBR7EQOb1NUjat1wrINzBNKOQtXoUmRVZU8h5c8K5CLMCUVcGkFVqAAGUJDh3mVcz6sTgXvmMuujWYrBveeM4q+",
                        "verification": "EQwhA/HsPB4oPogN5unEifDyfBkAfFM4WqpMDJF8MgB57a3yEQtBMHOzuw=="
                    }
                ]
            },
            {
                "hash": "0x46eca609a9a8c8340ee56b174b04bc9c9f37c89771c3a8998dc043f5a74ad510",
                "size": 267,
                "version": 0,
                "nonce": 565086327,
                "sender": "AHE5cLhX5NjGB5R2PcdUvGudUoGUBDeHX4",
                "sysfee": "0",
                "netfee": "0",
                "validuntilblock": 2107425,
                "signers": [
                    {
                        "account": "0xf68f181731a47036a99f04dad90043a744edec0f",
                        "scopes": "CalledByEntry"
                    }
                ],
                "attributes": [],
                "script": "AGQMFObBATZUrxE9ipaL3KUsmUioK5U9DBQP7O1Ep0MA2doEn6k2cKQxFxiP9hPADAh0cmFuc2ZlcgwUiXcg2M129PAKv6N8Dt2InCCP3ptBYn1bUjg",
                "witnesses": [
                    {
                        "invocation": "DEBR7EQOb1NUjat1wrINzBNKOQtXoUmRVZU8h5c8K5CLMCUVcGkFVqAAGUJDh3mVcz6sTgXvmMuujWYrBveeM4q+",
                        "verification": "EQwhA/HsPB4oPogN5unEifDyfBkAfFM4WqpMDJF8MgB57a3yEQtBMHOzuw=="
                    }
                ]
            }
        ],
        "confirmations": 7878,
        "nextblockhash": "0x4a97ca89199627f877b6bffe865b8327be84b368d62572ef20953829c3501643"
    }),
        ).await;
		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getblockheader",
            "params": [12345,1],
            "id": 1
        }"#;

		let result = provider.get_block_header_by_index(12345).await;

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_raw_block_header_index() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response(
            &mock_server,
            "getblockheader",
            json!([12345,0]),
            json!("AAAAAFrf0tgylRv20FkZygEC2UDiMHJTukXJPQ/DFP5sezdzm3A7VffHxK0b4rwXh/xR/zV24Mj6+Vhq25qoN1WlxRIBIKp7dwEAAIwAAADitlMicpPpnE8pBtU1U6u0pnLfhgFCDEDGZIUihuWK6RLqloq6UiKxkoW0QFhqGhoQU3cK5IQRATFUY807W/hGmYqP80N8qjKQ/e4o8URTzgRUXJKXf1/sKxEMIQLO1DI5fdxE7boDHAvDuTPyj92Wd3kteyDmwDbdqqzx4hELQRON768A")).await;
		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getblockheader",
            "params": [12345,0],
            "id": 1
        }"#;

		let result = provider.get_raw_block_header_by_index(12345).await;
		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_contract_state() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response(
            &mock_server,
            "getcontractstate",
            json!(["dc675afc61a7c0f7b3d2682bf6e1d8ed865a0e5f"]),
            json!({
        "id": 383,
        "updatecounter": 0,
        "hash": "0xe7f2e74b3498d3a0d80bcbd5925bca32e4acc4f7",
        "nef": {
            "magic": 860243278,
            "compiler": "Neo.Compiler.CSharp 3.1.0",
            "source": "https://github.com/neo-project/neo",
            "tokens": [
                {
                    "hash": "0xfffdc93764dbaddd97c48f252a53ea4643faa3fd",
                    "method": "update",
                    "paramcount": 3,
                    "hasreturnvalue": false,
                    "callflags": "All"
                },
                {
                    "hash": "0xfffdc93764dbaddd97c48f252a53ea4643faa3fd",
                    "method": "destroy",
                    "paramcount": 0,
                    "hasreturnvalue": false,
                    "callflags": "All"
                },
                {
                    "hash": "0xfe924b7cfe89ddd271abaf7210a80a7e11178758",
                    "method": "request",
                    "paramcount": 5,
                    "hasreturnvalue": false,
                    "callflags": "All"
                },
                {
                    "hash": "0xacce6fd80d44e1796aa0c2c625e9e4e0ce39efc0",
                    "method": "itoa",
                    "paramcount": 1,
                    "hasreturnvalue": true,
                    "callflags": "All"
                },
                {
                    "hash": "0xacce6fd80d44e1796aa0c2c625e9e4e0ce39efc0",
                    "method": "jsonDeserialize",
                    "paramcount": 1,
                    "hasreturnvalue": true,
                    "callflags": "All"
                },
                {
                    "hash": "0xfffdc93764dbaddd97c48f252a53ea4643faa3fd",
                    "method": "getContract",
                    "paramcount": 1,
                    "hasreturnvalue": true,
                    "callflags": "All"
                },
                {
                    "hash": "0xda65b600f7124ce6c79950c1772a36403104f2be",
                    "method": "getTransaction",
                    "paramcount": 1,
                    "hasreturnvalue": true,
                    "callflags": "All"
                },
                {
                    "hash": "0xda65b600f7124ce6c79950c1772a36403104f2be",
                    "method": "getTransactionState",
                    "paramcount": 1,
                    "hasreturnvalue": true,
                    "callflags": "All"
                }
            ],
            "script": "WEH4J+yMQEH4J+yMQDTzQFkMBmVuYWJsZUsRzlCLUBDOQZJd6DFK2CYERRDbIRGzQErYJgRFENshQEsRzlCLUBDOQZJd6DFANLiqJhYMEU5vIGF1dGhvcml6YXRpb24uOlkMBmVuYWJsZRESTRHOUYtREM5B5j8YhEASTRHOUYtREM5B5j8YhEA1d////6omFgwRTm8gYXV0aG9yaXphdGlvbi46WQwGZW5hYmxlEBJNEc5Ri1EQzkHmPxiEQFcAAzVP////JgYiKyIpDCRQYXltZW50IGlzIGRpc2FibGUgb24gdGhpcyBjb250cmFjdCE6QFcBAzUJ////qiYWDBFObyBhdXRob3JpemF0aW9uLjoLenlB2/6odBTAcGgfDAh0cmFuc2ZlcnhBYn1bUkXCSnjPSnnPSnrPDAtVbmxvY2tFdmVudEGVAW9hEdsgIgJAQdv+qHRAQWJ9W1JAVwIAEMBwaB8MCGlzUGF1c2VkWtsoStgkCUrKABQoAzpBYn1bUnHCSmnPDA1Jc1BhdXNlZEV2ZW50QZUBb2FpIgJA2yhK2CQJSsoAFCgDOkBXAAJ5JgQiGgwFV29ybGQMBUhlbGxvQZv2Z85B5j8YhEBB5j8YhEBBm/ZnzkBXAAI1If7//6omFgwRTm8gYXV0aG9yaXphdGlvbi46C3l4NwAAQDcAAEA1+v3//6omFgwRTm8gYXV0aG9yaXphdGlvbi46NwEAQDcBAEBXAgMMCGNhbGxiYWNrcAwIdXNlcmRhdGFxemloeHk3AgBANwIAQFcDBEE5U248DBRYhxcRfgqoEHKvq3HS3Yn+fEuS/pgmEgwNVW5hdXRob3JpemVkITp6EJgmLgwiT3JhY2xlIHJlc3BvbnNlIGZhaWx1cmUgd2l0aCBjb2RlIHo3AwCL2yg6ezcEAHBocWkQznIMCnVzZXJkYXRhOiB5i9soQc/nR5YMEHJlc3BvbnNlIHZhbHVlOiBqi9soQc/nR5ZAQTlTbjxADBRYhxcRfgqoEHKvq3HS3Yn+fEuS/kA3BABAQc/nR5ZAVwACeXhBm/ZnzkHmPxiEQFcBABFwIhtZaDcDAGgSTRHOUYtREM5B5j8YhGhKnHBFaAHoA7Uk4kBXAQBB2/6odDcFAHBoFM4VziICQDcFAEBXAQBB2/6odDcFAHBoFM4TziICQFcCAEEtUQgwcGgQznHCSmk3BgDPDBBUcmFuc2FjdGlvblN0YXRlQZUBb2FpNwcAIgJAQS1RCDBANwYAQDcHAEBWAwwUwJjkrPCyCQ3Rbss9WN5CaocVhRtgDBRC5UOC6G3Nygng2ou2fi+sTUmHRGIMBWFzc2V0QZv2Z84SwGFAEsBA",
            "checksum": 1593448136
        },
        "manifest": {
            "name": "TestNetFee",
            "groups": [],
            "features": {},
            "supportedstandards": [
                "NEP-17"
            ],
            "abi": {
                "methods": [
                    {
                        "name": "verify",
                        "parameters": [],
                        "returntype": "Boolean",
                        "offset": 13,
                        "safe": false
                    },
                    {
                        "name": "getPaymentStatus",
                        "parameters": [],
                        "returntype": "Boolean",
                        "offset": 16,
                        "safe": false
                    },
                    {
                        "name": "enablePayment",
                        "parameters": [],
                        "returntype": "Void",
                        "offset": 72,
                        "safe": false
                    },
                    {
                        "name": "disablePayment",
                        "parameters": [],
                        "returntype": "Void",
                        "offset": 137,
                        "safe": false
                    },
                    {
                        "name": "onNEP17Payment",
                        "parameters": [
                            {
                                "name": "from",
                                "type": "Hash160"
                            },
                            {
                                "name": "amount",
                                "type": "Integer"
                            },
                            {
                                "name": "data",
                                "type": "Any"
                            }
                        ],
                        "returntype": "Void",
                        "offset": 190,
                        "safe": false
                    },
                    {
                        "name": "unlock",
                        "parameters": [
                            {
                                "name": "toAssetHash",
                                "type": "Hash160"
                            },
                            {
                                "name": "toAddress",
                                "type": "Hash160"
                            },
                            {
                                "name": "amount",
                                "type": "Integer"
                            }
                        ],
                        "returntype": "Boolean",
                        "offset": 244,
                        "safe": false
                    },
                    {
                        "name": "isPaused",
                        "parameters": [],
                        "returntype": "Boolean",
                        "offset": 351,
                        "safe": false
                    },
                    {
                        "name": "_deploy",
                        "parameters": [
                            {
                                "name": "data",
                                "type": "Any"
                            },
                            {
                                "name": "update",
                                "type": "Boolean"
                            }
                        ],
                        "returntype": "Void",
                        "offset": 431,
                        "safe": false
                    },
                    {
                        "name": "update",
                        "parameters": [
                            {
                                "name": "nefFile",
                                "type": "ByteArray"
                            },
                            {
                                "name": "manifest",
                                "type": "String"
                            }
                        ],
                        "returntype": "Void",
                        "offset": 476,
                        "safe": false
                    },
                    {
                        "name": "destroy",
                        "parameters": [],
                        "returntype": "Void",
                        "offset": 518,
                        "safe": false
                    },
                    {
                        "name": "doRequest",
                        "parameters": [
                            {
                                "name": "filter",
                                "type": "String"
                            },
                            {
                                "name": "url",
                                "type": "String"
                            },
                            {
                                "name": "gasForResponse",
                                "type": "Integer"
                            }
                        ],
                        "returntype": "Void",
                        "offset": 554,
                        "safe": false
                    },
                    {
                        "name": "callback",
                        "parameters": [
                            {
                                "name": "url",
                                "type": "String"
                            },
                            {
                                "name": "userdata",
                                "type": "String"
                            },
                            {
                                "name": "code",
                                "type": "Integer"
                            },
                            {
                                "name": "result",
                                "type": "String"
                            }
                        ],
                        "returntype": "Void",
                        "offset": 592,
                        "safe": false
                    },
                    {
                        "name": "put",
                        "parameters": [
                            {
                                "name": "key",
                                "type": "String"
                            },
                            {
                                "name": "value",
                                "type": "String"
                            }
                        ],
                        "returntype": "Void",
                        "offset": 789,
                        "safe": false
                    },
                    {
                        "name": "putMulti",
                        "parameters": [],
                        "returntype": "Void",
                        "offset": 805,
                        "safe": false
                    },
                    {
                        "name": "testPermission",
                        "parameters": [],
                        "returntype": "Any",
                        "offset": 845,
                        "safe": false
                    },
                    {
                        "name": "testSupportedStandards",
                        "parameters": [],
                        "returntype": "Any",
                        "offset": 869,
                        "safe": false
                    },
                    {
                        "name": "getState",
                        "parameters": [],
                        "returntype": "Any",
                        "offset": 889,
                        "safe": false
                    },
                    {
                        "name": "_initialize",
                        "parameters": [],
                        "returntype": "Void",
                        "offset": 953,
                        "safe": false
                    }
                ],
                "events": [
                    {
                        "name": "UnlockEvent",
                        "parameters": [
                            {
                                "name": "arg1",
                                "type": "Hash160"
                            },
                            {
                                "name": "arg2",
                                "type": "Hash160"
                            },
                            {
                                "name": "arg3",
                                "type": "Integer"
                            }
                        ]
                    },
                    {
                        "name": "IsPausedEvent",
                        "parameters": [
                            {
                                "name": "obj",
                                "type": "Any"
                            }
                        ]
                    },
                    {
                        "name": "TransactionState",
                        "parameters": [
                            {
                                "name": "obj",
                                "type": "Any"
                            }
                        ]
                    }
                ]
            },
            "permissions": [
                {
                    "contract": "0x42e54382e86dcdca09e0da8bb67e2fac4d498744",
                    "methods": [
                        "test"
                    ]
                },
                {
                    "contract": "0xacce6fd80d44e1796aa0c2c625e9e4e0ce39efc0",
                    "methods": [
                        "itoa",
                        "jsonDeserialize"
                    ]
                },
                {
                    "contract": "0xda65b600f7124ce6c79950c1772a36403104f2be",
                    "methods": [
                        "getTransaction",
                        "getTransactionState"
                    ]
                },
                {
                    "contract": "0xfe924b7cfe89ddd271abaf7210a80a7e11178758",
                    "methods": [
                        "request"
                    ]
                },
                {
                    "contract": "0xfffdc93764dbaddd97c48f252a53ea4643faa3fd",
                    "methods": [
                        "destroy",
                        "getContract",
                        "update"
                    ]
                }
            ],
            "trusts": [],
            "extra": {
                "Author": "Neo",
                "Email": "dev@neo.org",
                "Description": "This is a contract example"
            }
        }
    }),
        ).await;
		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getcontractstate",
            "params": ["dc675afc61a7c0f7b3d2682bf6e1d8ed865a0e5f"],
            "id": 1
        }"#;

		let result = provider
			.get_contract_state(H160::from_str("dc675afc61a7c0f7b3d2682bf6e1d8ed865a0e5f").unwrap())
			.await;

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_contract_state_by_name() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response(
			&mock_server,
			"getblockhash",
			json!([16293]),
			json!("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"),
		)
		.await;
		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getcontractstate",
            "params": ["NeoToken"],
            "id": 1
        }"#;

		let result = provider.get_native_contract_state("NeoToken").await;

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	pub async fn test_get_contract_state_by_id() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response(
            &mock_server,
            "getcontractstate",
            json!([-6]),
            json!({
        "id": 383,
        "updatecounter": 0,
        "hash": "0xe7f2e74b3498d3a0d80bcbd5925bca32e4acc4f7",
        "nef": {
            "magic": 860243278,
            "compiler": "Neo.Compiler.CSharp 3.1.0",
            "source": "https://github.com/neo-project/neo",
            "tokens": [
                {
                    "hash": "0xfffdc93764dbaddd97c48f252a53ea4643faa3fd",
                    "method": "update",
                    "paramcount": 3,
                    "hasreturnvalue": false,
                    "callflags": "All"
                },
                {
                    "hash": "0xfffdc93764dbaddd97c48f252a53ea4643faa3fd",
                    "method": "destroy",
                    "paramcount": 0,
                    "hasreturnvalue": false,
                    "callflags": "All"
                },
                {
                    "hash": "0xfe924b7cfe89ddd271abaf7210a80a7e11178758",
                    "method": "request",
                    "paramcount": 5,
                    "hasreturnvalue": false,
                    "callflags": "All"
                },
                {
                    "hash": "0xacce6fd80d44e1796aa0c2c625e9e4e0ce39efc0",
                    "method": "itoa",
                    "paramcount": 1,
                    "hasreturnvalue": true,
                    "callflags": "All"
                },
                {
                    "hash": "0xacce6fd80d44e1796aa0c2c625e9e4e0ce39efc0",
                    "method": "jsonDeserialize",
                    "paramcount": 1,
                    "hasreturnvalue": true,
                    "callflags": "All"
                },
                {
                    "hash": "0xfffdc93764dbaddd97c48f252a53ea4643faa3fd",
                    "method": "getContract",
                    "paramcount": 1,
                    "hasreturnvalue": true,
                    "callflags": "All"
                },
                {
                    "hash": "0xda65b600f7124ce6c79950c1772a36403104f2be",
                    "method": "getTransaction",
                    "paramcount": 1,
                    "hasreturnvalue": true,
                    "callflags": "All"
                },
                {
                    "hash": "0xda65b600f7124ce6c79950c1772a36403104f2be",
                    "method": "getTransactionState",
                    "paramcount": 1,
                    "hasreturnvalue": true,
                    "callflags": "All"
                }
            ],
            "script": "WEH4J+yMQEH4J+yMQDTzQFkMBmVuYWJsZUsRzlCLUBDOQZJd6DFK2CYERRDbIRGzQErYJgRFENshQEsRzlCLUBDOQZJd6DFANLiqJhYMEU5vIGF1dGhvcml6YXRpb24uOlkMBmVuYWJsZRESTRHOUYtREM5B5j8YhEASTRHOUYtREM5B5j8YhEA1d////6omFgwRTm8gYXV0aG9yaXphdGlvbi46WQwGZW5hYmxlEBJNEc5Ri1EQzkHmPxiEQFcAAzVP////JgYiKyIpDCRQYXltZW50IGlzIGRpc2FibGUgb24gdGhpcyBjb250cmFjdCE6QFcBAzUJ////qiYWDBFObyBhdXRob3JpemF0aW9uLjoLenlB2/6odBTAcGgfDAh0cmFuc2ZlcnhBYn1bUkXCSnjPSnnPSnrPDAtVbmxvY2tFdmVudEGVAW9hEdsgIgJAQdv+qHRAQWJ9W1JAVwIAEMBwaB8MCGlzUGF1c2VkWtsoStgkCUrKABQoAzpBYn1bUnHCSmnPDA1Jc1BhdXNlZEV2ZW50QZUBb2FpIgJA2yhK2CQJSsoAFCgDOkBXAAJ5JgQiGgwFV29ybGQMBUhlbGxvQZv2Z85B5j8YhEBB5j8YhEBBm/ZnzkBXAAI1If7//6omFgwRTm8gYXV0aG9yaXphdGlvbi46C3l4NwAAQDcAAEA1+v3//6omFgwRTm8gYXV0aG9yaXphdGlvbi46NwEAQDcBAEBXAgMMCGNhbGxiYWNrcAwIdXNlcmRhdGFxemloeHk3AgBANwIAQFcDBEE5U248DBRYhxcRfgqoEHKvq3HS3Yn+fEuS/pgmEgwNVW5hdXRob3JpemVkITp6EJgmLgwiT3JhY2xlIHJlc3BvbnNlIGZhaWx1cmUgd2l0aCBjb2RlIHo3AwCL2yg6ezcEAHBocWkQznIMCnVzZXJkYXRhOiB5i9soQc/nR5YMEHJlc3BvbnNlIHZhbHVlOiBqi9soQc/nR5ZAQTlTbjxADBRYhxcRfgqoEHKvq3HS3Yn+fEuS/kA3BABAQc/nR5ZAVwACeXhBm/ZnzkHmPxiEQFcBABFwIhtZaDcDAGgSTRHOUYtREM5B5j8YhGhKnHBFaAHoA7Uk4kBXAQBB2/6odDcFAHBoFM4VziICQDcFAEBXAQBB2/6odDcFAHBoFM4TziICQFcCAEEtUQgwcGgQznHCSmk3BgDPDBBUcmFuc2FjdGlvblN0YXRlQZUBb2FpNwcAIgJAQS1RCDBANwYAQDcHAEBWAwwUwJjkrPCyCQ3Rbss9WN5CaocVhRtgDBRC5UOC6G3Nygng2ou2fi+sTUmHRGIMBWFzc2V0QZv2Z84SwGFAEsBA",
            "checksum": 1593448136
        },
        "manifest": {
            "name": "TestNetFee",
            "groups": [],
            "features": {},
            "supportedstandards": [
                "NEP-17"
            ],
            "abi": {
                "methods": [
                    {
                        "name": "verify",
                        "parameters": [],
                        "returntype": "Boolean",
                        "offset": 13,
                        "safe": false
                    },
                    {
                        "name": "getPaymentStatus",
                        "parameters": [],
                        "returntype": "Boolean",
                        "offset": 16,
                        "safe": false
                    },
                    {
                        "name": "enablePayment",
                        "parameters": [],
                        "returntype": "Void",
                        "offset": 72,
                        "safe": false
                    },
                    {
                        "name": "disablePayment",
                        "parameters": [],
                        "returntype": "Void",
                        "offset": 137,
                        "safe": false
                    },
                    {
                        "name": "onNEP17Payment",
                        "parameters": [
                            {
                                "name": "from",
                                "type": "Hash160"
                            },
                            {
                                "name": "amount",
                                "type": "Integer"
                            },
                            {
                                "name": "data",
                                "type": "Any"
                            }
                        ],
                        "returntype": "Void",
                        "offset": 190,
                        "safe": false
                    },
                    {
                        "name": "unlock",
                        "parameters": [
                            {
                                "name": "toAssetHash",
                                "type": "Hash160"
                            },
                            {
                                "name": "toAddress",
                                "type": "Hash160"
                            },
                            {
                                "name": "amount",
                                "type": "Integer"
                            }
                        ],
                        "returntype": "Boolean",
                        "offset": 244,
                        "safe": false
                    },
                    {
                        "name": "isPaused",
                        "parameters": [],
                        "returntype": "Boolean",
                        "offset": 351,
                        "safe": false
                    },
                    {
                        "name": "_deploy",
                        "parameters": [
                            {
                                "name": "data",
                                "type": "Any"
                            },
                            {
                                "name": "update",
                                "type": "Boolean"
                            }
                        ],
                        "returntype": "Void",
                        "offset": 431,
                        "safe": false
                    },
                    {
                        "name": "update",
                        "parameters": [
                            {
                                "name": "nefFile",
                                "type": "ByteArray"
                            },
                            {
                                "name": "manifest",
                                "type": "String"
                            }
                        ],
                        "returntype": "Void",
                        "offset": 476,
                        "safe": false
                    },
                    {
                        "name": "destroy",
                        "parameters": [],
                        "returntype": "Void",
                        "offset": 518,
                        "safe": false
                    },
                    {
                        "name": "doRequest",
                        "parameters": [
                            {
                                "name": "filter",
                                "type": "String"
                            },
                            {
                                "name": "url",
                                "type": "String"
                            },
                            {
                                "name": "gasForResponse",
                                "type": "Integer"
                            }
                        ],
                        "returntype": "Void",
                        "offset": 554,
                        "safe": false
                    },
                    {
                        "name": "callback",
                        "parameters": [
                            {
                                "name": "url",
                                "type": "String"
                            },
                            {
                                "name": "userdata",
                                "type": "String"
                            },
                            {
                                "name": "code",
                                "type": "Integer"
                            },
                            {
                                "name": "result",
                                "type": "String"
                            }
                        ],
                        "returntype": "Void",
                        "offset": 592,
                        "safe": false
                    },
                    {
                        "name": "put",
                        "parameters": [
                            {
                                "name": "key",
                                "type": "String"
                            },
                            {
                                "name": "value",
                                "type": "String"
                            }
                        ],
                        "returntype": "Void",
                        "offset": 789,
                        "safe": false
                    },
                    {
                        "name": "putMulti",
                        "parameters": [],
                        "returntype": "Void",
                        "offset": 805,
                        "safe": false
                    },
                    {
                        "name": "testPermission",
                        "parameters": [],
                        "returntype": "Any",
                        "offset": 845,
                        "safe": false
                    },
                    {
                        "name": "testSupportedStandards",
                        "parameters": [],
                        "returntype": "Any",
                        "offset": 869,
                        "safe": false
                    },
                    {
                        "name": "getState",
                        "parameters": [],
                        "returntype": "Any",
                        "offset": 889,
                        "safe": false
                    },
                    {
                        "name": "_initialize",
                        "parameters": [],
                        "returntype": "Void",
                        "offset": 953,
                        "safe": false
                    }
                ],
                "events": [
                    {
                        "name": "UnlockEvent",
                        "parameters": [
                            {
                                "name": "arg1",
                                "type": "Hash160"
                            },
                            {
                                "name": "arg2",
                                "type": "Hash160"
                            },
                            {
                                "name": "arg3",
                                "type": "Integer"
                            }
                        ]
                    },
                    {
                        "name": "IsPausedEvent",
                        "parameters": [
                            {
                                "name": "obj",
                                "type": "Any"
                            }
                        ]
                    },
                    {
                        "name": "TransactionState",
                        "parameters": [
                            {
                                "name": "obj",
                                "type": "Any"
                            }
                        ]
                    }
                ]
            },
            "permissions": [
                {
                    "contract": "0x42e54382e86dcdca09e0da8bb67e2fac4d498744",
                    "methods": [
                        "test"
                    ]
                },
                {
                    "contract": "0xacce6fd80d44e1796aa0c2c625e9e4e0ce39efc0",
                    "methods": [
                        "itoa",
                        "jsonDeserialize"
                    ]
                },
                {
                    "contract": "0xda65b600f7124ce6c79950c1772a36403104f2be",
                    "methods": [
                        "getTransaction",
                        "getTransactionState"
                    ]
                },
                {
                    "contract": "0xfe924b7cfe89ddd271abaf7210a80a7e11178758",
                    "methods": [
                        "request"
                    ]
                },
                {
                    "contract": "0xfffdc93764dbaddd97c48f252a53ea4643faa3fd",
                    "methods": [
                        "destroy",
                        "getContract",
                        "update"
                    ]
                }
            ],
            "trusts": [],
            "extra": {
                "Author": "Neo",
                "Email": "dev@neo.org",
                "Description": "This is a contract example"
            }
        }
    }),
        ).await;

		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getcontractstate",
            "params": [-6],
            "id": 1
        }"#;

		let result = provider.get_contract_state_by_id(-6).await;
		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_mem_pool() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response(
			&mock_server,
			"getrawmempool",
			json!([1]),
			json!({
			  "height": 5882071,
			  "verified": [
				"0x0c65fbfd2598aee5f30cd18f1264b458f1db137c4a460f4a174facb3f2d59d06",
				"0xc8040c285aa495f5b5e5b3761fd9333899f4ed902951c46d86c3bbb1cb12f2c0"
			  ],
			  "unverified": []
			}),
		)
		.await;

		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getrawmempool",
            "params": [1],
            "id": 1
        }"#;

		let result = provider.get_mem_pool().await;

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_raw_mem_pool() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response(
			&mock_server,
			"getrawmempool",
			json!([]),
			json!([
				"0x9786cce0dddb524c40ddbdd5e31a41ed1f6b5c8a683c122f627ca4a007a7cf4e",
				"0xb488ad25eb474f89d5ca3f985cc047ca96bc7373a6d3da8c0f192722896c1cd7",
				"0xf86f6f2c08fbf766ebe59dc84bc3b8829f1053f0a01deb26bf7960d99fa86cd6"
			]),
		)
		.await;

		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getrawmempool",
            "params": [],
            "id": 1
        }"#;

		let result = provider.get_raw_mem_pool().await;

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_transaction() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response(
            &mock_server,
            "getrawtransaction",
            json!(["7da6ae7ff9d0b7af3d32f3a2feb2aa96c2a27ef8b651f9a132cfaad6ef20724c", 1]),
            json!({
        "hash": "0x7da6ae7ff9d0b7af3d32f3a2feb2aa96c2a27ef8b651f9a132cfaad6ef20724c",
        "size": 386,
        "version": 0,
        "nonce": 246876555,
        "sender": "NikhQp1aAD1YFCiwknhM5LQQebj4464bCJ",
        "sysfee": "0.0999954",
        "netfee": "0.0235316",
        "validuntilblock": 5899,
        "signers": [
            {
                "account": "0xebae4ab3f21765e5f604dfdd590fdf142cfb89fa",
                "scopes": "None"
            },
            {
                "account": "0x86df72a6b4ab5335d506294f9ce993722253b6e2",
                "scopes": "CalledByEntry"
            }
        ],
        "attributes": [],
        "script": "CwMA5AtUAgAAAAwU+on7LBTfD1nd3wT25WUX8rNKrusMFOK2UyJyk+mcTykG1TVTq7Smct+GFMAfDAh0cmFuc2ZlcgwUKLOtq3Jp+cIYHbPLdB6/VRkw4nBBYn1bUjk=",
        "witnesses": [
            {
                "invocation": "DEC31ZE1kiIFPan7qal/h9FYsD2LTk6Lf0m0Kbbh1GExUqTAfye7BDjyEylfR50/AVNQkr+g+jXXMHGcxF4MUYBQ",
                "verification": "DCECztQyOX3cRO26AxwLw7kz8o/dlnd5LXsg5sA23aqs8eILQZVEDXg="
            },
            {
                "invocation": "DED8PagPv03pnEbsxUY7XgFk/qniHcha36hDCzZsmaJkpFg5vbgxk5+QE46K0GFsNpsqDJHNToGD9jeXsPzSvD5T",
                "verification": "EQwhAs7UMjl93ETtugMcC8O5M/KP3ZZ3eS17IObANt2qrPHiEQtBE43vrw=="
            }
        ],
        "blockhash": "0x3d87f53c51c93fc08e5ccc09dbd9e21fcfad4dbea66af454bed334824a90262c",
        "confirmations": 26,
        "blocktime": 1612687482881u64
    }),
        ).await;
		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getrawtransaction",
            "params": ["7da6ae7ff9d0b7af3d32f3a2feb2aa96c2a27ef8b651f9a132cfaad6ef20724c", 1],
            "id": 1
        }"#;

		let result = provider
			.get_transaction(
				H256::from_str(
					"0x7da6ae7ff9d0b7af3d32f3a2feb2aa96c2a27ef8b651f9a132cfaad6ef20724c",
				)
				.unwrap(),
			)
			.await;

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_raw_transaction() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response(
            &mock_server,
            "getrawtransaction",
            json!(["7da6ae7ff9d0b7af3d32f3a2feb2aa96c2a27ef8b651f9a132cfaad6ef20724c",0]),
            json!("AIsJtw60lJgAAAAAAAjoIwAAAAAACxcAAAL6ifssFN8PWd3fBPblZRfys0qu6wDitlMicpPpnE8pBtU1U6u0pnLfhgEAXwsDAOQLVAIAAAAMFPqJ+ywU3w9Z3d8E9uVlF/KzSq7rDBTitlMicpPpnE8pBtU1U6u0pnLfhhTAHwwIdHJhbnNmZXIMFCizratyafnCGB2zy3Qev1UZMOJwQWJ9W1I5AkIMQLfVkTWSIgU9qfupqX+H0ViwPYtOTot/SbQptuHUYTFSpMB/J7sEOPITKV9HnT8BU1CSv6D6NdcwcZzEXgxRgFApDCECztQyOX3cRO26AxwLw7kz8o/dlnd5LXsg5sA23aqs8eILQZVEDXhCDED8PagPv03pnEbsxUY7XgFk/qniHcha36hDCzZsmaJkpFg5vbgxk5+QE46K0GFsNpsqDJHNToGD9jeXsPzSvD5TKxEMIQLO1DI5fdxE7boDHAvDuTPyj92Wd3kteyDmwDbdqqzx4hELQRON768="),
        ).await;

		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getrawtransaction",
            "params": ["7da6ae7ff9d0b7af3d32f3a2feb2aa96c2a27ef8b651f9a132cfaad6ef20724c", 0],
            "id": 1
        }"#;

		let result = provider
			.get_raw_transaction(
				H256::from_str(
					"0x7da6ae7ff9d0b7af3d32f3a2feb2aa96c2a27ef8b651f9a132cfaad6ef20724c",
				)
				.unwrap(),
			)
			.await;

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_storge() {
		let mock_server = setup_mock_server().await;
		let key_hex = "hello".as_bytes().to_hex();
		let key_base64 = key_hex.from_hex().unwrap().to_base64();
		let provider = mock_rpc_response(
			&mock_server,
			"getstorage",
			json!(["99042d380f2b754175717bb932a911bc0bb0ad7d", key_base64]),
			json!("d29ybGQ="),
		)
		.await;

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "getstorage",
			"params": ["99042d380f2b754175717bb932a911bc0bb0ad7d", "{}"],
			"id": 1
		}}"#,
			key_base64
		);

		let result = provider
			.get_storage(
				H160::from_str("0x99042d380f2b754175717bb932a911bc0bb0ad7d").unwrap(),
				key_hex.as_str(),
			)
			.await;
		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_find_storge() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response(
			&mock_server,
			"findstorage",
			json!(["1b468f207a5c5c3ee94e41b4cc606e921b33d160", "{}", 2]),
			json!("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"),
		)
		.await;

		let prefix_base64 = "c3".to_string().to_base64();

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "findstorage",
			"params": ["1b468f207a5c5c3ee94e41b4cc606e921b33d160", "{}", 2],
			"id": 1
		}}"#,
			prefix_base64
		);

		let result = provider
			.find_storage(
				H160::from_str("1b468f207a5c5c3ee94e41b4cc606e921b33d160").unwrap(),
				"c3",
				2,
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_find_storge_with_id() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response(
			&mock_server,
			"getblockhash",
			json!([16293]),
			json!("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"),
		)
		.await;
		let prefix_base64 = "0b".to_string().to_base64();

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "findstorage",
			"params": [-1, "{}", 10],
			"id": 1
		}}"#,
			prefix_base64
		);

		let result = provider.find_storage_with_id(-1, "0b", 10).await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_transaction_height() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response(
			&mock_server,
			"gettransactionheight",
			json!(["57280b29c2f9051af6e28a8662b160c216d57c498ee529e0cf271833f90e1a53"]),
			json!(14),
		)
		.await;
		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "gettransactionheight",
			"params": ["57280b29c2f9051af6e28a8662b160c216d57c498ee529e0cf271833f90e1a53"],
			"id": 1
		}}"#
		);

		let result = provider
			.get_transaction_height(
				H256::from_str(
					"0x57280b29c2f9051af6e28a8662b160c216d57c498ee529e0cf271833f90e1a53",
				)
				.unwrap(),
			)
			.await;
		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_next_block_validators() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response(
			&mock_server,
			"getnextblockvalidators",
			json!([]),
			json!([
			  {
				"publickey": "03aa052fbcb8e5b33a4eefd662536f8684641f04109f1d5e69cdda6f084890286a",
				"votes": "0",
				"active": true
			  }
			]),
		)
		.await;
		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "getnextblockvalidators",
			"params": [],
			"id": 1
		}}"#
		);

		let result = provider.get_next_block_validators().await;
		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_committe() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response(
			&mock_server,
			"getblockhash",
			json!([16293]),
			json!("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"),
		)
		.await;
		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "getcommittee",
			"params": [],
			"id": 1
		}}"#
		);

		let result = provider.get_committee().await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	// Node Methods

	#[tokio::test]
	async fn test_get_connection_count() {
		let mock_server = setup_mock_server().await;
		let provider =
			mock_rpc_response(&mock_server, "getconnectioncount", json!([]), json!(10)).await;

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "getconnectioncount",
			"params": [],
			"id": 1
		}}"#
		);

		let result = provider.get_connection_count().await;
		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_peers() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response(
			&mock_server,
			"getpeers",
			json!([]),
			json!({
				"unconnected": [],
				"bad": [],
				"connected": [
					{
						"address": "47.90.28.99",
						"port": 21333
					},
					{
						"address": "47.90.28.99",
						"port": 22333
					}
				]
			}),
		)
		.await;
		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "getpeers",
			"params": [],
			"id": 1
		}}"#
		);

		let result = provider.get_peers().await;
		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_version() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response(
			&mock_server,
			"getversion",
			json!([]),
			json!( {
				"tcpport": 10333,
				"wsport": 10334,
				"nonce": 1930156121,
				"useragent": "/Neo:3.0.3/",
				"protocol": {
					"addressversion": 53,
					"network": 860833102,
					"validatorscount": 7,
					"msperblock": 15000,
					"maxtraceableblocks": 2102400,
					"maxvaliduntilblockincrement": 5760,
					"maxtransactionsperblock": 512,
					"memorypoolmaxtransactions": 50000,
					"initialgasdistribution": 5200000000000000u64
				}
			}),
		)
		.await;
		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "getversion",
			"params": [],
			"id": 1
		}}"#
		);

		let result = provider.get_version().await;
		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_send_raw_transaction() {
		let mock_server = setup_mock_server().await;
		let tx_hex = "80000001d405ab03e736a01ca277d94b1377113c7e961bb4550511fe1d408f30c77a82650000029b7cffdaa674beae0f930ebe6085af9093e5fe56b34a5c220ccdcf6efc336fc500ca9a3b0000000023ba2703c53263e8d6e522dc32203339dcd8eee99b7cffdaa674beae0f930ebe6085af9093e5fe56b34a5c220ccdcf6efc336fc5001a711802000000295f83f83fc439f56e6e1fb062d89c6f538263d70141403711e366fc99e77a110b6c96b5f8828ef956a6d5cfa5cb63273419149011b0f30dc5458faa59e4867d0ac7537e324c98124bb691feca5c5ddf6ed20f4adb778223210265bf906bf385fbf3f777832e55a87991bcfbe19b097fb7c5ca2e4025a4d5e5d6ac".to_string();
		let tx_base64 = tx_hex.from_hex().unwrap().to_base64();
		let provider = mock_rpc_response(
			&mock_server,
			"sendrawtransaction",
			json!([tx_base64]),
			json!({
				"hash": "0x13ccdb9f7eda95a24aa5a4841b24fed957fe7f1b944996cbc2e92a4fa4f1fa73"
			}),
		)
		.await;
		// Expected request body
		let expected_request_body = format!(
			r#"{{
    "jsonrpc": "2.0",
    "method": "sendrawtransaction",
    "params": ["{}"],
    "id": 1
}}"#,
			tx_base64
		);

		let result = provider.send_raw_transaction(tx_hex).await.expect("TODO: panic message");
		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_submit_block() {
		// TODO:: This one still panic
		let mock_server = setup_mock_server().await;
		let block_base64 = "AAAAACMSKFbGpGl6t7uroMpi2ilhQd84eU/pUrRfQyswXYl76woLOY0oW1z4InfxoKyxFAAB+8FS6cRu2Pm0iaOiD8OMCnLadQEAAMgcAAD6lrDvowCyjK9dBALCmE1fvMuahQFCDEAd8EoEFBcxOLCZfh8w0tUEHHmyn++KzW4I8oeJ1WyMmjHVcolpNzOnAOzXTn/xujwy93gJ9ijvVo6wAF5qC3wCKxEMIQL4L//X3jDpIyMLze0sPNW+yFcufrrL3bnzOipdJpNLixELQRON768CAGUTt7+NSxXGAA7aoUS2kokAAAAAACYcEwAAAAAARzMAAAHNWK7P0zW+HrPTEeHcgAlj39ctnwEAXQMA5AtUAgAAAAwUzViuz9M1vh6z0xHh3IAJY9/XLZ8MFM1Yrs/TNb4es9MR4dyACWPf1y2fE8AMCHRyYW5zZmVyDBS8r0HWhMfUrW7g2Z2pcHudHwyOZkFifVtSOAFCDEADRhUarLK+/BBjhqaWY5ieento21zgkcsUMWNCBWGd+v8a35zatNRgFbUkni4dDNI/BGc3zOgPT6EwroUsgvR+KQwhAv3yei642bBp1hrlpk26E7iWN8VC2MdMXWurST/mONaPC0GVRA14".to_string();
		let block_hex = block_base64.from_base64().unwrap().to_hex();

		let provider = mock_rpc_response(
			&mock_server,
			"submitblock",
			json!([block_base64]),
			json!({"hash": "0xbe153a2ef9e9160906f7054ed8f676aa223a826c4ae662ce0fb3f09d38b093c1"}),
		)
		.await;
		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "submitblock",
			"params": ["{}"],
			"id": 1
		}}"#,
			block_base64
		);

		let result = provider.submit_block(block_hex).await.expect("TODO: panic message");
		// assert!(result, "Result is not okay: {:?}", result);
		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	// SmartContract Methods

	#[tokio::test]
	async fn test_invoke_function() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response(
            &mock_server,
            "invokefunction",
            json!([
    "0xa1a375677dded85db80a852c28c2431cab29e2c4",
    "transfer",
    [
            {
                "type": "Hash160",
                "value": "0xfa03cb7b40072c69ca41f0ad3606a548f1d59966"
            },
            {
                "type": "Hash160",
                "value": "0xebae4ab3f21765e5f604dfdd590fdf142cfb89fa"
            },
            {
                "type": "Integer",
                "value": "10000"
            },
            {
                "type": "String",
                "value": ""
            }
        ],
        [
            {
                "account": "0xfa03cb7b40072c69ca41f0ad3606a548f1d59966",
                "scopes": "CalledByEntry",
                "allowedcontracts": [],
                "allowedgroups": []
            }
        ],
    true
  ]),
            json!({
        "script": "DAABECcMFPqJ+ywU3w9Z3d8E9uVlF/KzSq7rDBRmmdXxSKUGNq3wQcppLAdAe8sD+hTAHwwIdHJhbnNmZXIMFMTiKascQ8IoLIUKuF3Y3n1ndaOhQWJ9W1I=",
        "state": "HALT",
        "gasconsumed": "1490312",
        "exception": null,
        "notifications": [
            {
                "eventname": "Transfer",
                "contract": "0xa1a375677dded85db80a852c28c2431cab29e2c4",
                "state": {
                    "type": "Array",
                    "value": [
                        {
                            "type": "ByteString",
                            "value": "ZpnV8UilBjat8EHKaSwHQHvLA/o="
                        },
                        {
                            "type": "ByteString",
                            "value": "+on7LBTfD1nd3wT25WUX8rNKrus="
                        },
                        {
                            "type": "Integer",
                            "value": "10000"
                        }
                    ]
                }
            }
        ],
        "diagnostics": {
            "invokedcontracts": {
                "hash": "0x9cac876fcc1646f1f017aa49b1fbcf87bd37b043",
                "call": [
                    {
                        "hash": "0xa1a375677dded85db80a852c28c2431cab29e2c4",
                        "call": [
                            {
                                "hash": "0xa1a375677dded85db80a852c28c2431cab29e2c4"
                            },
                            {
                                "hash": "0xa1a375677dded85db80a852c28c2431cab29e2c4"
                            },
                            {
                                "hash": "0xa1a375677dded85db80a852c28c2431cab29e2c4"
                            },
                            {
                                "hash": "0xa1a375677dded85db80a852c28c2431cab29e2c4"
                            },
                            {
                                "hash": "0xa1a375677dded85db80a852c28c2431cab29e2c4"
                            },
                            {
                                "hash": "0xa1a375677dded85db80a852c28c2431cab29e2c4"
                            },
                            {
                                "hash": "0xa1a375677dded85db80a852c28c2431cab29e2c4",
                                "call": [
                                    {
                                        "hash": "0xa1a375677dded85db80a852c28c2431cab29e2c4"
                                    },
                                    {
                                        "hash": "0xa1a375677dded85db80a852c28c2431cab29e2c4"
                                    }
                                ]
                            },
                            {
                                "hash": "0xa1a375677dded85db80a852c28c2431cab29e2c4",
                                "call": [
                                    {
                                        "hash": "0xa1a375677dded85db80a852c28c2431cab29e2c4"
                                    },
                                    {
                                        "hash": "0xa1a375677dded85db80a852c28c2431cab29e2c4"
                                    }
                                ]
                            },
                            {
                                "hash": "0xfffdc93764dbaddd97c48f252a53ea4643faa3fd"
                            }
                        ]
                    }
                ]
            },
            "storagechanges": [
                {
                    "state": "Changed",
                    "key": "BgAAAAEBZpnV8UilBjat8EHKaSwHQHvLA/o=",
                    "value": "8CTJ5wda"
                },
                {
                    "state": "Added",
                    "key": "BgAAAAEB+on7LBTfD1nd3wT25WUX8rNKrus=",
                    "value": "ECc="
                }
            ]
        },
        "stack": [
            {
                "type": "Boolean",
                "value": true
            }
        ],
        "tx": "AOaXOgSIvRYAAAAAAKzgAQAAAAAAesUGAAFmmdXxSKUGNq3wQcppLAdAe8sD+gEAWQwAARAnDBT6ifssFN8PWd3fBPblZRfys0qu6wwUZpnV8UilBjat8EHKaSwHQHvLA/oUwB8MCHRyYW5zZmVyDBTE4imrHEPCKCyFCrhd2N59Z3WjoUFifVtSAUIMQMTS2HRIO9gDxq/U/lqIB77dLBzVHT4cwKdvqoGOZqm4IoGqHbYzBSYHOPHWGNutWvkjCgIQGQFKK1JGyOR16LwoDCEDrQCtTQQyXXSsHZm3oRiqiAzP00uFPaW9tICYC3D7Bm9BVuezJw=="
    }),
        ).await;
		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "invokefunction",
			"params": [
				"af7c7328eee5a275a3bcaee2bf0cf662b5e739be",
				"balanceOf",
				[
					{{
						"type": "Hash160",
						"value": "91b83e96f2a7c4fdf0c1688441ec61986c7cae26"
					}}
				],
				[
					{{
						"account": "cadb3dc2faa3ef14a13b619c9a43124755aa2569",
						"scopes": "CalledByEntry,CustomContracts,CustomGroups,WitnessRules",
						"allowedcontracts": ["ef4073a0f2b305a38ec4050e4d3d28bc40ea63f5"],
						"allowedgroups": [
							"033a4d051b04b7fc0230d2b1aaedfd5a84be279a5361a7358db665ad7857787f1b"
						],
						"rules": [
							{{
								"action": "Allow",
								"condition": {{
									"type": "CalledByContract",
									"hash": "{}"
								}}
							}}
						]
					}}
				]
			],
			"id": 1
		}}"#,
			TestConstants::NEO_TOKEN_HASH
		);

		let public_key = Secp256r1PublicKey::from_bytes(
			&hex::decode(TestConstants::DEFAULT_ACCOUNT_PUBLIC_KEY).unwrap(),
		)
		.unwrap();
		let rule = WitnessRule::new(
			WitnessAction::Allow,
			WitnessCondition::CalledByContract(
				H160::from_hex(TestConstants::NEO_TOKEN_HASH).unwrap(),
			),
		);

		let mut signer = AccountSigner::called_by_entry_hash160(
			H160::from_str("0xcadb3dc2faa3ef14a13b619c9a43124755aa2569").unwrap(),
		)
		.unwrap();
		signer
			.set_allowed_contracts(vec![H160::from_str(TestConstants::NEO_TOKEN_HASH).unwrap()])
			.expect("TODO: panic message");
		signer.set_allowed_groups(vec![public_key]).expect("TODO: panic message");
		signer.set_rules(vec![rule]).expect("TODO: panic message");

		let result = provider
			.invoke_function(
				&H160::from_str("af7c7328eee5a275a3bcaee2bf0cf662b5e739be").unwrap(),
				"balanceOf".to_string(),
				vec![H160::from_hex("91b83e96f2a7c4fdf0c1688441ec61986c7cae26").unwrap().into()],
				Some(vec![Account(signer)]),
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_invoke_function_witnessrules() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response(
			&mock_server,
			"getblockhash",
			json!([16293]),
			json!("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"),
		)
		.await;
		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "invokefunction",
			"params": [
				"af7c7328eee5a275a3bcaee2bf0cf662b5e739be",
				"balanceOf",
				[
					{{
						"type": "Hash160",
						"value": "91b83e96f2a7c4fdf0c1688441ec61986c7cae26"
					}}
				],
				[
					{{
						"account": "cadb3dc2faa3ef14a13b619c9a43124755aa2569",
						"scopes": "CalledByEntry,CustomContracts,CustomGroups,WitnessRules",
						"allowedcontracts": ["ef4073a0f2b305a38ec4050e4d3d28bc40ea63f5"],
						"allowedgroups": [
							"033a4d051b04b7fc0230d2b1aaedfd5a84be279a5361a7358db665ad7857787f1b"
						],
						"rules": [
                    		{{
                        		"action": "Deny",
                        		"condition": {{
                           			"type": "And",
                            		"expressions": [
                                	{{
                                    	"type": "Boolean",
                                    	"expression": true
                                	}},
                                	{{
                                    	"type": "CalledByContract",
                                    	"hash": "{}"
                                	}},
                                	{{
                                    	"type": "CalledByGroup",
                                    	"group": "{}"
                                	}},
                                	{{
                                    	"type": "Group",
                                    	"group": "{}"
                                	}}
                        			]
                        		}}
                    		}},
                    		{{
                        		"action": "Deny",
                        		"condition": {{
                            		"type": "Or",
                            		"expressions": [
                                	{{
                                    	"type": "CalledByGroup",
                                    	"group": "{}"
                                	}},
                                	{{
                                    	"type": "ScriptHash",
                                    	"hash": "{}"
                                	}}
                            		]
                        		}}
                    		}},
                    		{{
                        		"action": "Allow",
                        		"condition": {{
                            		"type": "Not",
                            		"expression": {{
                                		"type": "CalledByEntry"
                            		}}
                        		}}
                    		}}
                		]
					}}
				]
			],
			"id": 1
		}}"#,
			TestConstants::NEO_TOKEN_HASH,
			TestConstants::DEFAULT_ACCOUNT_PUBLIC_KEY,
			TestConstants::DEFAULT_ACCOUNT_PUBLIC_KEY,
			TestConstants::DEFAULT_ACCOUNT_PUBLIC_KEY,
			TestConstants::COMMITTEE_ACCOUNT_SCRIPT_HASH
		);

		let public_key = Secp256r1PublicKey::from_bytes(
			&hex::decode(TestConstants::DEFAULT_ACCOUNT_PUBLIC_KEY).unwrap(),
		)
		.unwrap();
		let rule1 = WitnessRule::new(
			WitnessAction::Deny,
			WitnessCondition::And(vec![
				WitnessCondition::Boolean(true),
				WitnessCondition::CalledByContract(
					H160::from_hex(TestConstants::NEO_TOKEN_HASH).unwrap(),
				),
				WitnessCondition::CalledByGroup(public_key.clone()),
				WitnessCondition::Group(public_key.clone()),
			]),
		);
		let rule2 = WitnessRule::new(
			WitnessAction::Deny,
			WitnessCondition::Or(vec![
				WitnessCondition::CalledByGroup(public_key.clone()),
				WitnessCondition::ScriptHash(
					H160::from_hex(TestConstants::COMMITTEE_ACCOUNT_SCRIPT_HASH).unwrap(),
				),
			]),
		);
		let rule3 = WitnessRule::new(
			WitnessAction::Allow,
			WitnessCondition::Not(Box::new(WitnessCondition::CalledByEntry)),
		);

		let mut signer = AccountSigner::called_by_entry_hash160(
			H160::from_str("0xcadb3dc2faa3ef14a13b619c9a43124755aa2569").unwrap(),
		)
		.unwrap();
		signer
			.set_allowed_contracts(vec![H160::from_str(TestConstants::NEO_TOKEN_HASH).unwrap()])
			.expect("TODO: panic message");
		signer.set_allowed_groups(vec![public_key]).expect("TODO: panic message");
		signer.set_rules(vec![rule1, rule2, rule3]).expect("TODO: panic message");

		let result = provider
			.invoke_function(
				&H160::from_str("af7c7328eee5a275a3bcaee2bf0cf662b5e739be").unwrap(),
				"balanceOf".to_string(),
				vec![H160::from_hex("91b83e96f2a7c4fdf0c1688441ec61986c7cae26").unwrap().into()],
				Some(vec![Account(signer)]),
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_invoke_function_diagnostics() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response(
			&mock_server,
			"invokefunction",
			json!([
				"af7c7328eee5a275a3bcaee2bf0cf662b5e739be",
				"balanceOf",
				[
					{
						"type": "Hash160",
						"value": "91b83e96f2a7c4fdf0c1688441ec61986c7cae26"
					}
				],
				[],
				true
			]),
			json!("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"),
		)
		.await;
		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "invokefunction",
			"params": [
				"af7c7328eee5a275a3bcaee2bf0cf662b5e739be",
				"balanceOf",
				[
					{{
						"type": "Hash160",
						"value": "91b83e96f2a7c4fdf0c1688441ec61986c7cae26"
					}}
				],
				[],
				true
			],
			"id": 1
		}}"#
		);

		let public_key = Secp256r1PublicKey::from_bytes(
			&hex::decode(TestConstants::DEFAULT_ACCOUNT_PUBLIC_KEY).unwrap(),
		)
		.unwrap();
		let rule = WitnessRule::new(
			WitnessAction::Allow,
			WitnessCondition::CalledByContract(
				H160::from_hex(TestConstants::NEO_TOKEN_HASH).unwrap(),
			),
		);

		let result = provider
			.invoke_function_diagnostics(
				H160::from_str("af7c7328eee5a275a3bcaee2bf0cf662b5e739be").unwrap(),
				"balanceOf".to_string(),
				vec![H160::from_hex("91b83e96f2a7c4fdf0c1688441ec61986c7cae26").unwrap().into()],
				vec![],
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_invoke_function_diagnostics_no_params() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = RpcClient::new(http_client);

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "invokefunction",
			"params": [
				"af7c7328eee5a275a3bcaee2bf0cf662b5e739be",
				"symbol",
				[],
				[],
				true
			],
			"id": 1
		}}"#
		);

		let _ = provider
			.invoke_function_diagnostics(
				H160::from_str("af7c7328eee5a275a3bcaee2bf0cf662b5e739be").unwrap(),
				"symbol".to_string(),
				vec![],
				vec![],
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_invoke_function_without_params() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = RpcClient::new(http_client);

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "invokefunction",
			"params": [
				"af7c7328eee5a275a3bcaee2bf0cf662b5e739be",
				"decimals",
				[],
				[]
			],
			"id": 1
		}}"#
		);

		let _ = provider
			.invoke_function(
				&H160::from_str("af7c7328eee5a275a3bcaee2bf0cf662b5e739be").unwrap(),
				"decimals".to_string(),
				vec![],
				None,
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_invoke_script() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = RpcClient::new(http_client);

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "invokescript",
			"params": [
				"EMAMCGRlY2ltYWxzDBQlBZ7LSHjTqHX5HFHO3tMw1Fdf3kFifVtS",
				[]
			],
			"id": 1
		}}"#
		);

		let _ = provider
			.invoke_script(
				"10c00c08646563696d616c730c1425059ecb4878d3a875f91c51ceded330d4575fde41627d5b52"
					.to_string(),
				vec![],
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_invoke_script_diagnostics() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = RpcClient::new(http_client);

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "invokescript",
			"params": [
				"EMAMCGRlY2ltYWxzDBQlBZ7LSHjTqHX5HFHO3tMw1Fdf3kFifVtS",
				[],
				true
			],
			"id": 1
		}}"#
		);

		let _ = provider
			.invoke_script_diagnostics(
				"10c00c08646563696d616c730c1425059ecb4878d3a875f91c51ceded330d4575fde41627d5b52"
					.to_string(),
				vec![],
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_invoke_script_with_signer() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = RpcClient::new(http_client);

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "invokescript",
			"params": [
				"EMAMCGRlY2ltYWxzDBQlBZ7LSHjTqHX5HFHO3tMw1Fdf3kFifVtS",
				[
					{{
						"account": "cc45cc8987b0e35371f5685431e3c8eeea306722",
						"scopes": "CalledByEntry",
						"allowedcontracts": [],
						"allowedgroups": [],
						"rules": []
					}}
				]
			],
			"id": 1
		}}"#
		);

		let signer = AccountSigner::called_by_entry_hash160(
			H160::from_str("0xcc45cc8987b0e35371f5685431e3c8eeea306722").unwrap(),
		)
		.unwrap();

		let _ = provider
			.invoke_script(
				"10c00c08646563696d616c730c1425059ecb4878d3a875f91c51ceded330d4575fde41627d5b52"
					.to_string(),
				vec![Account(signer)],
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_invoke_script_diagnostics_with_signer() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = RpcClient::new(http_client);

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "invokescript",
			"params": [
				"EMAMCGRlY2ltYWxzDBQlBZ7LSHjTqHX5HFHO3tMw1Fdf3kFifVtS",
				[
					{{
						"account": "cc45cc8987b0e35371f5685431e3c8eeea306722",
						"scopes": "CalledByEntry",
						"allowedcontracts": [],
						"allowedgroups": [],
						"rules": []
					}}
				],
				true
			],
			"id": 1
		}}"#
		);

		let signer = AccountSigner::called_by_entry_hash160(
			H160::from_str("0xcc45cc8987b0e35371f5685431e3c8eeea306722").unwrap(),
		)
		.unwrap();

		let _ = provider
			.invoke_script_diagnostics(
				"10c00c08646563696d616c730c1425059ecb4878d3a875f91c51ceded330d4575fde41627d5b52"
					.to_string(),
				vec![Account(signer)],
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_traverse_iterator() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = RpcClient::new(http_client);

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "traverseiterator",
			"params": [
				"127d3320-db35-48d5-b6d3-ca22dca4a370",
				"cb7ef774-1ade-4a83-914b-94373ca92010",
				100
			],
			"id": 1
		}}"#
		);

		let _ = provider
			.traverse_iterator(
				"127d3320-db35-48d5-b6d3-ca22dca4a370".to_string(),
				"cb7ef774-1ade-4a83-914b-94373ca92010".to_string(),
				100,
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_terminate_session() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = RpcClient::new(http_client);

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "terminatesession",
			"params": [
				"127d3320-db35-48d5-b6d3-ca22dca4a370"
			],
			"id": 1
		}}"#
		);

		let _ = provider.terminate_session("127d3320-db35-48d5-b6d3-ca22dca4a370").await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_invoke_contract_verify() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = RpcClient::new(http_client);

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "invokecontractverify",
			"params": [
				"af7c7328eee5a275a3bcaee2bf0cf662b5e739be",
				[
					{{
						"type": "String",
						"value": "a string"
					}},
					{{
						"type": "String",
						"value": "another string"
					}}
				],
				[
					{{
						"account": "cadb3dc2faa3ef14a13b619c9a43124755aa2569",
						"scopes": "CalledByEntry",
						"allowedcontracts": [],
						"allowedgroups": [],
						"rules": []
					}}
				]
			],
			"id": 1
		}}"#
		);

		let signer = AccountSigner::called_by_entry_hash160(
			H160::from_str("0xcadb3dc2faa3ef14a13b619c9a43124755aa2569").unwrap(),
		)
		.unwrap();

		let _ = provider
			.invoke_contract_verify(
				H160::from_str("af7c7328eee5a275a3bcaee2bf0cf662b5e739be").unwrap(),
				vec!["a string".to_string().into(), "another string".to_string().into()],
				vec![Account(signer)],
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_invoke_contract_verify_noparams_nosigners() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = RpcClient::new(http_client);

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "invokecontractverify",
			"params": [
				"af7c7328eee5a275a3bcaee2bf0cf662b5e739be",
				[],
				[]
			],
			"id": 1
		}}"#
		);

		let _ = provider
			.invoke_contract_verify(
				H160::from_str("af7c7328eee5a275a3bcaee2bf0cf662b5e739be").unwrap(),
				vec![],
				vec![],
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	async fn verify_request(
		mock_server: &MockServer,
		expected: &str,
	) -> Result<(), Box<dyn std::error::Error>> {
		// Retrieve the request body from the mock server
		let received_requests = mock_server.received_requests().await.unwrap();
		assert!(!received_requests.is_empty(), "No requests received");

		// Assuming we only have one request
		let request = &received_requests[0];
		let request_body = String::from_utf8_lossy(&request.body);

		// Normalize JSON by removing whitespace and comparing
		let request_json: Value = serde_json::from_str(&request_body).unwrap();
		let expected_json: Value = serde_json::from_str(expected).unwrap();
		assert_eq!(
			request_json, expected_json,
			"The request body does not match the expected body"
		);

		Ok(())
	}

	async fn verify_request_json(
		mock_server: &MockServer,
		expected_json: Value,
	) -> Result<(), Box<dyn std::error::Error>> {
		// Retrieve the request body from the mock server
		let received_requests = mock_server.received_requests().await.unwrap();
		assert!(!received_requests.is_empty(), "No requests received");

		// Assuming we only have one request
		let request = &received_requests[0];
		let request_body = String::from_utf8_lossy(&request.body);

		// Normalize JSON by removing whitespace and comparing
		let request_json: Value = serde_json::from_str(&request_body).unwrap();

		assert_eq!(
			request_json, expected_json,
			"The request body does not match the expected body"
		);

		Ok(())
	}
}