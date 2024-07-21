use std::{
	collections::HashMap, convert::TryFrom, fmt::Debug, future::Future, net::Ipv4Addr,
	str::FromStr, sync::Arc, time::Duration,
};

use async_trait::async_trait;
use futures_util::lock::Mutex;
use primitive_types::{H160, H256};
use rustc_serialize::{base64, base64::ToBase64, hex::FromHex};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::value::Value;
use tracing::trace;
use tracing_futures::Instrument;
use url::{Host, ParseError, Url};

use neo::prelude::*;

use crate::{neo_providers::rpc::provider::sealed::Sealed, prelude::Base64Encode};

use crate::neo_types::ScriptHashExtension;
use serde_json::json;

/// Node Clients
#[derive(Copy, Clone)]
pub enum NodeClient {
	/// RNEO
	NEO,
}

impl FromStr for NodeClient {
	type Err = ProviderError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.split('/').next().unwrap().to_lowercase().as_str() {
			"NEO" => Ok(NodeClient::NEO),
			_ => Err(ProviderError::UnsupportedNodeClient),
		}
	}
}

/// An abstract provider for interacting with the [Neo JSON RPC
/// API](https://github.com/neo/wiki/wiki/JSON-RPC). Must be instantiated
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
pub struct Provider<P> {
	inner: P,
	nns: Option<Address>,
	interval: Option<Duration>,
	from: Option<Address>,
	_node_client: Arc<Mutex<Option<NeoVersion>>>,
}

impl<P> AsRef<P> for Provider<P> {
	fn as_ref(&self) -> &P {
		&self.inner
	}
}

// JSON RPC bindings
impl<P: JsonRpcClient> Provider<P> {
	/// Instantiate a new provider with a backend.
	pub fn new(provider: P) -> Self {
		Self {
			inner: provider,
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
			let fetched = self.inner.fetch(method, params).await;
			let res: R = fetched.map_err(Into::into)?;
			trace!(rx = ?serde_json::to_string(&res)?);
			Ok::<_, ProviderError>(res)
		}
		.instrument(span)
		.await?;
		Ok(res)
	}
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<P: JsonRpcClient> Middleware for Provider<P> {
	type Error = ProviderError;
	type Provider = P;
	type Inner = Self;

	fn inner(&self) -> &Self::Inner {
		unreachable!("There is no inner provider here")
	}

	fn convert_err(p: ProviderError) -> Self::Error {
		p
	}

	fn provider(&self) -> &Provider<Self::Provider> {
		self
	}

	async fn network(&self) -> u32 {
		self.get_version().await.unwrap().protocol.unwrap().network
		// if self.config().network == None {
		// 	let network = self.inner().get_version().await.unwrap().protocol.unwrap().network;
		// 	network
		// } else {
		// 	self.config().network.unwrap()
		// }
	}

	//////////////////////// Neo methods////////////////////////////

	fn nns_resolver(&self) -> H160 {
		H160::from(self.config().nns_resolver.clone())
	}

	fn block_interval(&self) -> u32 {
		self.config().block_interval
	}

	fn polling_interval(&self) -> u32 {
		self.config().polling_interval
	}

	fn max_valid_until_block_increment(&self) -> u32 {
		self.config().max_valid_until_block_increment
	}

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
		return Ok(if full_tx {
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
	async fn get_transaction(
		&self,
		hash: H256,
	) -> Result<Option<TransactionResult>, ProviderError> {
		self.request("getrawtransaction", vec![hash.to_value(), 1.to_value()]).await
	}

	/// Gets the corresponding transaction information based on the specified transaction hash.
	/// - Parameter txHash: The transaction hash
	/// - Returns: The request object
	async fn get_raw_transaction(&self, tx_hash: H256) -> Result<RawTransaction, ProviderError> {
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
	async fn submit_block(&self, hex: String) -> Result<bool, ProviderError> {
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
			}
            None => {
				let signers: Vec<TransactionSigner> = vec![];
				self.request(
					"invokefunction",
					json!([
						ScriptHashExtension::to_hex_big_endian(contract_hash),
						method,
						params,
						signers
					]),
				)
				.await
			}
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
		let scriptBase64 = serde_json::to_value(hex.from_hex().unwrap().to_base64()).unwrap();
		let signersJson = serde_json::to_value(&signers).unwrap();
		self.request("invokescript", [scriptBase64, signersJson]).await
	}

	/// Gets the unclaimed GAS of the account with the specified script hash.
	/// - Parameter scriptHash: The account's script hash
	/// - Returns: The request object
	async fn get_unclaimed_gas(&self, hash: H160) -> Result<UnclaimedGas, ProviderError> {
		self.request("getunclaimedgas", [serialize(&hash)]).await
	}

	/// Gets a list of plugins loaded by the node.
	/// - Returns: The request object
	async fn list_plugins(&self) -> Result<Vec<Plugin>, ProviderError> {
		self.request("listplugins", ()).await
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
		self.request("closewallet", ()).await
	}

	/// Exports the private key of the specified script hash.
	/// - Parameter scriptHash: The account's script hash
	/// - Returns: The request object
	async fn dump_priv_key(&self, script_hash: H160) -> Result<String, ProviderError> {
		let params = [script_hash.to_value()].to_vec();
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
		self.request("getnewaddress", ()).await
	}

	/// Gets the amount of unclaimed GAS in the wallet.
	/// - Returns: The request object
	async fn get_wallet_unclaimed_gas(&self) -> Result<String, ProviderError> {
		self.request("getwalletunclaimedgas", ()).await
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
		self.request("calculatenetworkfee", vec![txBase64.to_value()]).await
	}

	/// Lists all the addresses in the current wallet.
	/// - Returns: The request object
	async fn list_address(&self) -> Result<Vec<NeoAddress>, ProviderError> {
		self.request("listaddress", ()).await
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
		from: Address,
		to: Address,
		amount: u32,
	) -> Result<Transaction, ProviderError> {
		let params =
			[token_hash.to_value(), from.to_value(), to.to_value(), amount.to_value()].to_vec();
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

	async fn send_to_address_send_token(
		&self,
		send_token: &TransactionSendToken,
	) -> Result<Transaction, ProviderError> {
		let params = [send_token.to_value()].to_vec();
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
		})
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
		let params = vec![hex.to_value(), signers.to_value(), true.to_value()];
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
		self.request(
			"invokecontractverify",
			vec![hash.to_value(), params.to_value(), signers.to_value()],
		)
		.await
	}

	async fn send_from_send_token(
		&self,
		send_token: &TransactionSendToken,
		from: Address,
	) -> Result<Transaction, ProviderError> {
		let params = [from.to_value(), vec![send_token.to_value()].into()].to_vec();
		self.request("sendmany", params).await
	}
}

impl<P: JsonRpcClient> Provider<P> {
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
impl Provider<crate::Ipc> {
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

impl Provider<Http> {
	/// The Url to which requests are made
	pub fn url(&self) -> &Url {
		self.inner.url()
	}

	/// Mutable access to the Url to which requests are made
	pub fn url_mut(&mut self) -> &mut Url {
		self.inner.url_mut()
	}
}

impl<Read, Write> Provider<RwClient<Read, Write>>
where
	Read: JsonRpcClient + 'static,
	<Read as JsonRpcClient>::Error: Sync + Send + 'static,
	Write: JsonRpcClient + 'static,
	<Write as JsonRpcClient>::Error: Sync + Send + 'static,
{
	/// Creates a new [Provider] with a [RwClient]
	pub fn rw(r: Read, w: Write) -> Self {
		Self::new(RwClient::new(r, w))
	}
}

impl Provider<MockProvider> {
	/// Returns a `Provider` instantiated with an internal "mock" transport.
	///
	/// # Example
	///
	/// ```
	/// # use NeoRust::prelude::Provider;
	///  async fn foo() -> Result<(), Box<dyn std::error::Error>> {
	/// // Instantiate the provider
	/// let (provider, mock) = Provider::mocked();
	/// // Push the mock response
	/// mock.push(u64::from(12))?;
	/// // Make the call
	/// let blk = provider.get_block_number().await.unwrap();
	/// // The response matches
	/// assert_eq!(blk.as_u64(), 12);
	/// // and the request as well!
	/// mock.assert_request("neo_blockNumber", ()).unwrap();
	/// # Ok(())
	/// # }
	/// ```
	pub fn mocked() -> (Self, MockProvider) {
		let mock = MockProvider::new();
		let mock_clone = mock.clone();
		(Self::new(mock), mock_clone)
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

impl TryFrom<&str> for Provider<Http> {
	type Error = ParseError;

	fn try_from(src: &str) -> Result<Self, Self::Error> {
		Ok(Provider::new(Http::new(Url::parse(src)?)))
	}
}

impl TryFrom<String> for Provider<Http> {
	type Error = ParseError;

	fn try_from(src: String) -> Result<Self, Self::Error> {
		Provider::try_from(src.as_str())
	}
}

impl<'a> TryFrom<&'a String> for Provider<Http> {
	type Error = ParseError;

	fn try_from(src: &'a String) -> Result<Self, Self::Error> {
		Provider::try_from(src.as_str())
	}
}

#[cfg(not(target_arch = "wasm32"))]
impl Provider<RetryClient<Http>> {
	/// Create a new [`RetryClient`] by connecting to the provided URL. Errors
	/// if `src` is not a valid URL
	pub fn new_client(src: &str, max_retry: u32, initial_backoff: u64) -> Result<Self, ParseError> {
		Ok(Provider::new(RetryClient::new(
			Http::new(Url::parse(src)?),
			Box::new(HttpRateLimitRetryPolicy),
			max_retry,
			initial_backoff,
		)))
	}
}

mod sealed {
	use neo::prelude::{Http, Provider};

	/// private trait to ensure extension trait is not implement outside of this crate
	pub trait Sealed {}
	impl Sealed for Provider<Http> {}
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
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
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

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl ProviderExt for Provider<Http> {
	type Error = ParseError;

	async fn try_connect(url: &str) -> Result<Self, Self::Error>
	where
		Self: Sized,
	{
		let mut provider = Provider::try_from(url)?;
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
			}
		}
	}
	false
}

#[cfg(test)]
mod tests {
	use std::str::FromStr;

	use crate::neo_types::Base64Encode;
	use lazy_static::lazy_static;
	use neo::prelude::{
		AccountSigner, BodyRegexMatcher, HttpProvider, Middleware, Provider, ProviderError,
		ScriptHashExtension, Secp256r1PublicKey, Signer::Account, SignerTrait, TestConstants,
		WitnessAction, WitnessCondition, WitnessRule,
	};
	use primitive_types::{H160, H256};
	use reqwest::Client;
	use serde_json::{json, Value};
	use tokio;
	use url::Url;
	use wiremock::{
		matchers::{method, path},
		Mock, MockServer, ResponseTemplate,
	};

	#[tokio::test]
	async fn test_get_best_block_hash() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = Provider::new(http_client);

		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getbestblockhash",
            "params": [],
            "id": 1
        }"#;

		provider.get_best_block_hash().await;

		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_block_hash() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = Provider::new(http_client);

		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getblockhash",
            "params": [16293],
            "id": 1
        }"#;

		provider.get_block_hash(16293).await;

		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_block_index() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = Provider::new(http_client);

		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getblock",
            "params": [12345,1],
            "id": 1
        }"#;

		provider.get_block_by_index(12345, true).await;

		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_block_index_only_header() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = Provider::new(http_client);

		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getblockheader",
            "params": [12345,1],
            "id": 1
        }"#;

		provider.get_block_by_index(12345, false).await;

		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_block_by_hash() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = Provider::new(http_client);

		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getblock",
            "params": ["2240b34669038f82ac492150d391dfc3d7fe5e3c1d34e5b547d50e99c09b468d",1],
            "id": 1
        }"#;

		provider
			.get_block(
				H256::from_str(
					"0x2240b34669038f82ac492150d391dfc3d7fe5e3c1d34e5b547d50e99c09b468d",
				)
				.unwrap(),
				true,
			)
			.await;

		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_block_not_full_Tx_objects() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = Provider::new(http_client);

		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getblockheader",
            "params": ["2240b34669038f82ac492150d391dfc3d7fe5e3c1d34e5b547d50e99c09b468d",1],
            "id": 1
        }"#;

		provider
			.get_block(
				H256::from_str(
					"0x2240b34669038f82ac492150d391dfc3d7fe5e3c1d34e5b547d50e99c09b468d",
				)
				.unwrap(),
				false,
			)
			.await;

		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_raw_block_index() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = Provider::new(http_client);

		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getblock",
            "params": [12345,0],
            "id": 1
        }"#;

		provider.get_raw_block_by_index(12345).await;

		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_block_header_count() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = Provider::new(http_client);

		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getblockheadercount",
            "params": [],
            "id": 1
        }"#;

		provider.get_block_header_count().await;

		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_block_count() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = Provider::new(http_client);

		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getblockcount",
            "params": [],
            "id": 1
        }"#;

		provider.get_block_count().await;

		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_native_contracts() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = Provider::new(http_client);

		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getnativecontracts",
            "params": [],
            "id": 1
        }"#;

		provider.get_native_contracts().await;

		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_block_header_index() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = Provider::new(http_client);

		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getblockheader",
            "params": [12345,1],
            "id": 1
        }"#;

		provider.get_block_header_by_index(12345).await;

		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_raw_block_header_index() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = Provider::new(http_client);

		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getblockheader",
            "params": [12345,0],
            "id": 1
        }"#;

		provider.get_raw_block_header_by_index(12345).await;

		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_contract_state() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = Provider::new(http_client);

		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getcontractstate",
            "params": ["dc675afc61a7c0f7b3d2682bf6e1d8ed865a0e5f"],
            "id": 1
        }"#;

		provider
			.get_contract_state(H160::from_str("dc675afc61a7c0f7b3d2682bf6e1d8ed865a0e5f").unwrap())
			.await;

		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_contract_state_by_name() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = Provider::new(http_client);

		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getcontractstate",
            "params": ["NeoToken"],
            "id": 1
        }"#;

		provider.get_native_contract_state("NeoToken").await;

		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_contract_state_by_Id() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = Provider::new(http_client);

		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getcontractstate",
            "params": [-6],
            "id": 1
        }"#;

		provider.get_contract_state_by_id(-6).await;

		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_mem_pool() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = Provider::new(http_client);

		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getrawmempool",
            "params": [1],
            "id": 1
        }"#;

		provider.get_mem_pool().await;

		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_raw_mem_pool() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = Provider::new(http_client);

		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getrawmempool",
            "params": [],
            "id": 1
        }"#;

		provider.get_raw_mem_pool().await;

		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_transaction() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = Provider::new(http_client);

		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getrawtransaction",
            "params": ["1f31821787b0a53df0ff7d6e0e7ecba3ac19dd517d6d2ea5aaf00432c20831d6", 1],
            "id": 1
        }"#;

		provider
			.get_transaction(
				H256::from_str(
					"0x1f31821787b0a53df0ff7d6e0e7ecba3ac19dd517d6d2ea5aaf00432c20831d6",
				)
				.unwrap(),
			)
			.await;

		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_raw_transaction() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = Provider::new(http_client);

		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getrawtransaction",
            "params": ["1f31821787b0a53df0ff7d6e0e7ecba3ac19dd517d6d2ea5aaf00432c20831d6", 0],
            "id": 1
        }"#;

		provider
			.get_raw_transaction(
				H256::from_str(
					"0x1f31821787b0a53df0ff7d6e0e7ecba3ac19dd517d6d2ea5aaf00432c20831d6",
				)
				.unwrap(),
			)
			.await;

		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_storge() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = Provider::new(http_client);

		let key = "616e797468696e67";
		let key_base64 = key.to_string().to_base64();

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "getstorage",
			"params": ["03febccf81ac85e3d795bc5cbd4e84e907812aa3", "{}"],
			"id": 1
		}}"#,
			key_base64
		);

		provider
			.get_storage(H160::from_str("03febccf81ac85e3d795bc5cbd4e84e907812aa3").unwrap(), key)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_find_storge() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = Provider::new(http_client);

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

		provider
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
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = Provider::new(http_client);

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

		provider.find_storage_with_id(-1, "0b", 10).await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_transaction_height() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = Provider::new(http_client);

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "gettransactionheight",
			"params": ["793f560ae7058a50c672890e69c9292391dd159ce963a33462059d03b9573d6a"],
			"id": 1
		}}"#
		);

		provider
			.get_transaction_height(
				H256::from_str(
					"0x793f560ae7058a50c672890e69c9292391dd159ce963a33462059d03b9573d6a",
				)
				.unwrap(),
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_next_block_validators() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = Provider::new(http_client);

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "getnextblockvalidators",
			"params": [],
			"id": 1
		}}"#
		);

		provider.get_next_block_validators().await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_committe() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = Provider::new(http_client);

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "getcommittee",
			"params": [],
			"id": 1
		}}"#
		);

		provider.get_committee().await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	// Node Methods

	#[tokio::test]
	async fn test_get_connection_count() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = Provider::new(http_client);

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "getconnectioncount",
			"params": [],
			"id": 1
		}}"#
		);

		provider.get_connection_count().await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_peers() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = Provider::new(http_client);

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "getpeers",
			"params": [],
			"id": 1
		}}"#
		);

		provider.get_peers().await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_version() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = Provider::new(http_client);

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "getversion",
			"params": [],
			"id": 1
		}}"#
		);

		provider.get_version().await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_send_raw_transaction() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = Provider::new(http_client);

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "sendrawtransaction",
			"params": ["gAAAAdQFqwPnNqAconfZSxN3ETx+lhu0VQUR/h1AjzDHeoJlAAACm3z/2qZ0vq4Pkw6+YIWvkJPl/lazSlwiDM3Pbvwzb8UAypo7AAAAACO6JwPFMmPo1uUi3DIgMznc2O7pm3z/2qZ0vq4Pkw6+YIWvkJPl/lazSlwiDM3Pbvwzb8UAGnEYAgAAAClfg/g/xDn1bm4fsGLYnG9TgmPXAUFANxHjZvyZ53oRC2yWtfiCjvlWptXPpctjJzQZFJARsPMNxUWPqlnkhn0Kx1N+MkyYEku2kf7KXF3fbtIPStt3giMhAmW/kGvzhfvz93eDLlWoeZG8++GbCX+3xcouQCWk1eXWrA=="],
			"id": 1
		}}"#
		);

		provider.send_raw_transaction("80000001d405ab03e736a01ca277d94b1377113c7e961bb4550511fe1d408f30c77a82650000029b7cffdaa674beae0f930ebe6085af9093e5fe56b34a5c220ccdcf6efc336fc500ca9a3b0000000023ba2703c53263e8d6e522dc32203339dcd8eee99b7cffdaa674beae0f930ebe6085af9093e5fe56b34a5c220ccdcf6efc336fc5001a711802000000295f83f83fc439f56e6e1fb062d89c6f538263d70141403711e366fc99e77a110b6c96b5f8828ef956a6d5cfa5cb63273419149011b0f30dc5458faa59e4867d0ac7537e324c98124bb691feca5c5ddf6ed20f4adb778223210265bf906bf385fbf3f777832e55a87991bcfbe19b097fb7c5ca2e4025a4d5e5d6ac".to_string()).await.expect("TODO: panic message");

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_submit_block() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = Provider::new(http_client);

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "submitblock",
			"params": ["00000000000000000000000000000000"],
			"id": 1
		}}"#
		);

		provider
			.submit_block("00000000000000000000000000000000".to_string())
			.await
			.expect("TODO: panic message");

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	// SmartContract Methods

	#[tokio::test]
	async fn test_invoke_function() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = Provider::new(http_client);

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

		let _ = provider
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
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = Provider::new(http_client);

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

		let _ = provider
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
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url);
		let provider = Provider::new(http_client);

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

		let _ = provider
			.invoke_function_diagnostics(
				H160::from_str("af7c7328eee5a275a3bcaee2bf0cf662b5e739be").unwrap(),
				"balanceOf".to_string(),
				vec![H160::from_hex("91b83e96f2a7c4fdf0c1688441ec61986c7cae26").unwrap().into()],
				vec![],
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	async fn setup_mock_server() -> MockServer {
		let server = MockServer::start().await;
		Mock::given(method("POST"))
			.and(path("/"))
			.respond_with(ResponseTemplate::new(200).set_body_string("hello"))
			.mount(&server)
			.await;
		server
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

		// assert_eq!(
		// 	request_json, expected_json,
		// 	"The request body does not match the expected body"
		// );

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
