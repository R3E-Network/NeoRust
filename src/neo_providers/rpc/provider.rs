use async_trait::async_trait;
use futures_util::lock::Mutex;
use neo::prelude::*;
use primitive_types::{H160, H256 as TxHash, H256, U256};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
	collections::HashMap, convert::TryFrom, fmt::Debug, future::Future, net::Ipv4Addr,
	str::FromStr, sync::Arc, time::Duration,
};
use tracing::trace;
use tracing_futures::Instrument;
use url::{Host, ParseError, Url};

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
///  use NeoRust::prelude::{Http, Middleware, NeoConstants, Provider};
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
	_node_client: Arc<Mutex<Option<NodeClient>>>,
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
	pub async fn node_client(&self) -> Result<NodeClient, ProviderError> {
		let mut node_client = self._node_client.lock().await;

		if let Some(node_client) = *node_client {
			Ok(node_client)
		} else {
			let client_version = self.client_version().await?;
			let client_version = match client_version.parse::<NodeClient>() {
				Ok(res) => res,
				Err(_) => return Err(ProviderError::UnsupportedNodeClient),
			};
			*node_client = Some(client_version);
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
			let res: R = self.inner.fetch(method, params).await.map_err(Into::into)?;
			trace!(rx = ?serde_json::to_string(&res)?);
			Ok::<_, ProviderError>(res)
		}
		.instrument(span)
		.await?;
		Ok(res)
	}

	pub fn call_raw<'a>(&'a self, tx: &'a Transaction) -> CallBuilder<'a, P> {
		CallBuilder::new(self, tx)
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

	fn default_sender(&self) -> Option<Address> {
		self.from.clone()
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
	async fn get_best_block_hash(&self) -> Result<H256, ProviderError> {
		self.request("getbestblockhash", ()).await
	}

	async fn get_block_hash(&self, block_index: u32) -> Result<H256, ProviderError> {
		self.request("getblockhash", [block_index.to_value()].to_vec()).await
	}

	async fn get_block(&self, block_hash: H256, full_tx: bool) -> Result<NeoBlock, ProviderError> {
		return Ok(if full_tx {
			self.request("getblock", [block_hash.to_value(), 1.to_value()].to_vec()).await?
		} else {
			self.get_block_header_hash(block_hash).await?
		})
	}

	async fn get_raw_block(&self, block_hash: H256) -> Result<String, ProviderError> {
		self.request("getblock", [block_hash.to_value(), 0.to_value()]).await
	}

	// Node methods

	async fn get_block_header_count(&self) -> Result<u32, ProviderError> {
		self.request("getblockheadercount", ()).await
	}

	async fn get_block_count(&self) -> Result<u32, ProviderError> {
		self.request("getblockcount", ()).await
	}

	async fn get_block_header(&self, block_hash: H256) -> Result<NeoBlock, ProviderError> {
		self.request("getblockheader", vec![block_hash.to_value(), 1.to_value()]).await
	}

	async fn get_block_header_by_index(&self, index: u32) -> Result<NeoBlock, ProviderError> {
		self.request("getblockheader", vec![index.to_value(), 1.to_value()]).await
	}

	// Smart contract methods

	async fn get_raw_block_header(&self, block_hash: H256) -> Result<String, ProviderError> {
		self.request("getblockheader", vec![block_hash.to_value(), 0.to_value()]).await
	}

	async fn get_raw_block_header_by_index(&self, index: u32) -> Result<String, ProviderError> {
		self.request("getblockheader", vec![index.to_value(), 0.to_value()]).await
	}

	// Utility methods

	async fn get_native_contracts(&self) -> Result<Vec<NativeContractState>, ProviderError> {
		self.request("getnativecontracts", ()).await
	}

	// Wallet methods

	async fn get_contract_state(&self, hash: H160) -> Result<ContractState, ProviderError> {
		self.request("getcontractstate", vec![hash.to_value()]).await
	}

	async fn get_native_contract_state(&self, name: &str) -> Result<ContractState, ProviderError> {
		self.request("getcontractstate", vec![name.to_value()]).await
	}

	async fn get_mem_pool(&self) -> Result<MemPoolDetails, ProviderError> {
		self.request("getrawmempool", vec![1.to_value()]).await
	}

	async fn get_raw_mem_pool(&self) -> Result<Vec<H256>, ProviderError> {
		self.request("getrawmempool", ()).await
	}

	// Application logs

	async fn get_transaction(
		&self,
		hash: H256,
	) -> Result<Option<TransactionResult>, ProviderError> {
		self.request("getrawtransaction", vec![hash.to_value(), 1.to_value()]).await
	}

	// State service

	async fn get_raw_transaction(&self, tx_hash: H256) -> Result<RawTransaction, ProviderError> {
		self.request("getrawtransaction", vec![tx_hash.to_value(), 0.to_value()]).await
	}

	async fn get_storage(&self, contract_hash: H160, key: &str) -> Result<String, ProviderError> {
		let params = [contract_hash.to_value(), key.to_value()];
		self.request("getstorage", params.to_vec()).await
	}
	// Blockchain methods

	async fn get_transaction_height(&self, tx_hash: H256) -> Result<u32, ProviderError> {
		let params = [tx_hash.to_value()];
		self.request("gettransactionheight", params.to_vec()).await
	}

	async fn get_next_block_validators(&self) -> Result<Vec<Validator>, ProviderError> {
		self.request("getnextblockvalidators", ()).await
	}

	async fn get_committee(&self) -> Result<Vec<String>, ProviderError> {
		self.request("getcommittee", ()).await
	}

	async fn get_connection_count(&self) -> Result<u32, ProviderError> {
		self.request("getconnectioncount", ()).await
	}

	async fn get_peers(&self) -> Result<Peers, ProviderError> {
		self.request("getpeers", ()).await
	}

	// Smart contract methods

	async fn get_version(&self) -> Result<NeoVersion, ProviderError> {
		self.request("getversion", ()).await
	}

	async fn send_raw_transaction(&self, hex: String) -> Result<RawTransaction, ProviderError> {
		self.request("sendrawtransaction", vec![hex.to_value()]).await
	}
	// More node methods

	async fn submit_block(&self, hex: String) -> Result<bool, ProviderError> {
		self.request("submitblock", vec![hex.to_value()]).await
	}

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
					[
						contract_hash.to_value(),
						method.to_value(),
						params.to_value(),
						signers.to_value(),
					],
				)
				.await
			},
			None =>
				self.request(
					"invokefunction",
					[contract_hash.to_value(), method.to_value(), params.to_value()],
				)
				.await,
		}
	}

	async fn invoke_script(
		&self,
		hex: String,
		signers: Vec<Signer>,
	) -> Result<InvocationResult, ProviderError> {
		let signers: Vec<TransactionSigner> =
			signers.into_iter().map(|signer| signer.into()).collect::<Vec<_>>();
		self.request("invokescript", [hex.to_value(), signers.to_value()]).await
	}

	// More smart contract methods

	async fn get_unclaimed_gas(&self, hash: H160) -> Result<UnclaimedGas, ProviderError> {
		self.request("getunclaimedgas", [utils::serialize(&hash)]).await
	}

	async fn list_plugins(&self) -> Result<Vec<Plugin>, ProviderError> {
		self.request("listplugins", ()).await
	}

	// More utility methods

	async fn validate_address(&self, address: &str) -> Result<ValidateAddress, ProviderError> {
		self.request("validateaddress", vec![address.to_value()]).await
	}

	// More wallet methods

	async fn close_wallet(&self) -> Result<bool, ProviderError> {
		self.request("closewallet", ()).await
	}

	async fn dump_priv_key(&self, script_hash: H160) -> Result<String, ProviderError> {
		let params = [script_hash.to_value()].to_vec();
		self.request("dumpprivkey", params).await
	}

	async fn get_wallet_balance(&self, token_hash: H160) -> Result<Balance, ProviderError> {
		self.request("getwalletbalance", vec![token_hash.to_value()]).await
	}

	async fn get_new_address(&self) -> Result<String, ProviderError> {
		self.request("getnewaddress", ()).await
	}

	async fn get_wallet_unclaimed_gas(&self) -> Result<String, ProviderError> {
		self.request("getwalletunclaimedgas", ()).await
	}

	async fn import_priv_key(&self, priv_key: String) -> Result<NeoAddress, ProviderError> {
		let params = [priv_key.to_value()].to_vec();
		self.request("importprivkey", params).await
	}

	async fn calculate_network_fee(&self, hex: String) -> Result<u64, ProviderError> {
		self.request("calculatenetworkfee", vec![hex.to_value()]).await
	}

	async fn list_address(&self) -> Result<Vec<NeoAddress>, ProviderError> {
		self.request("listaddress", ()).await
	}

	async fn open_wallet(&self, path: String, password: String) -> Result<bool, ProviderError> {
		self.request("openwallet", vec![path.to_value(), password.to_value()]).await
	}

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

	// Transaction methods

	async fn send_many(
		&self,
		from: Option<H160>,
		send_tokens: Vec<TransactionSendToken>,
	) -> Result<Transaction, ProviderError> {
		let params = [from.unwrap().to_value(), send_tokens.to_value()].to_vec();
		self.request("sendmany", params).await
	}

	async fn send_to_address(
		&self,
		token_hash: H160,
		to: Address,
		amount: u32,
	) -> Result<Transaction, ProviderError> {
		let params = [token_hash.to_value(), to.to_value(), amount.to_value()].to_vec();
		self.request("sendtoaddress", params).await
	}

	async fn get_application_log(&self, tx_hash: H256) -> Result<ApplicationLog, ProviderError> {
		self.request("getapplicationlog", vec![tx_hash.to_value()]).await
	}

	async fn get_nep17_balances(&self, script_hash: H160) -> Result<Nep17Balances, ProviderError> {
		self.request("getnep17balances", [script_hash.to_value()].to_vec()).await
	}

	async fn get_nep17_transfers(
		&self,
		script_hash: H160,
	) -> Result<Nep17Transfers, ProviderError> {
		let params = [script_hash.to_value()].to_vec();
		self.request("getnep17transfers", params).await
	}

	// NEP-17 methods

	async fn get_nep17_transfers_from(
		&self,
		script_hash: H160,
		from: u64,
	) -> Result<Nep17Transfers, ProviderError> {
		// let params = [script_hash.to_value(), from.to_value()].to_vec();
		self.request("getnep17transfers", [script_hash.to_value(), from.to_value()])
			.await
	}

	async fn get_nep17_transfers_range(
		&self,
		script_hash: H160,
		from: u64,
		to: u64,
	) -> Result<Nep17Transfers, ProviderError> {
		let params = [script_hash.to_value(), from.to_value(), to.to_value()].to_vec();
		self.request("getnep17transfers", params).await
	}

	async fn get_nep11_balances(&self, script_hash: H160) -> Result<Nep11Balances, ProviderError> {
		let params = [script_hash.to_value()].to_vec();
		self.request("getnep11balances", params).await
	}

	// NEP-11 methods

	async fn get_nep11_transfers(
		&self,
		script_hash: H160,
	) -> Result<Nep11Transfers, ProviderError> {
		let params = [script_hash.to_value()].to_vec();
		self.request("getnep11transfers", params).await
	}

	async fn get_nep11_transfers_from(
		&self,
		script_hash: H160,
		from: u64,
	) -> Result<Nep11Transfers, ProviderError> {
		let params = [script_hash.to_value(), from.to_value()].to_vec();
		self.request("getnep11transfers", params).await
	}

	async fn get_nep11_transfers_range(
		&self,
		script_hash: H160,
		from: u64,
		to: u64,
	) -> Result<Nep11Transfers, ProviderError> {
		let params = [script_hash.to_value(), from.to_value(), to.to_value()].to_vec();
		self.request("getnep11transfers", params).await
	}

	async fn get_nep11_properties(
		&self,
		script_hash: H160,
		token_id: &str,
	) -> Result<HashMap<String, String>, ProviderError> {
		let params = [script_hash.to_value(), token_id.to_value()].to_vec();
		self.request("getnep11properties", params).await
	}

	async fn get_state_root(&self, block_index: u32) -> Result<StateRoot, ProviderError> {
		let params = [block_index.to_value()].to_vec();
		self.request("getstateroot", params).await
	}

	// State service methods

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

	async fn verify_proof(&self, root_hash: H256, proof: &str) -> Result<bool, ProviderError> {
		let params = [root_hash.to_value(), proof.to_value()].to_vec();
		self.request("verifyproof", params).await
	}

	async fn get_state_height(&self) -> Result<StateHeight, ProviderError> {
		self.request("getstateheight", ()).await
	}

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
		let full_tx = if full_tx { 1 } else { 0 };
		self.request("getblock", vec![index.to_value(), full_tx.to_value()]).await
	}

	async fn get_raw_block_by_index(&self, index: u32) -> Result<String, ProviderError> {
		self.request("getblock", vec![index.to_value(), 0.to_value()]).await
	}

	async fn invoke_function_diagnostics(
		&self,
		contract_hash: H160,
		name: String,
		params: Vec<ContractParameter>,
		signers: Vec<Signer>,
	) -> Result<InvocationResult, ProviderError> {
		let params = vec![
			contract_hash.to_value(),
			name.to_value(),
			serde_json::to_string(&params).unwrap().to_value(),
			serde_json::to_string(&signers).unwrap().to_value(),
			true.to_value(),
		];
		self.request("invokefunction", params).await
	}

	async fn invoke_script_diagnostics(
		&self,
		hex: String,
		signers: Vec<Signer>,
	) -> Result<InvocationResult, ProviderError> {
		let params = vec![hex.to_value(), signers.to_value(), true.to_value()];
		self.request("invokescript", params).await
	}

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

	async fn get_raw_mempool(&self) -> Result<MemPoolDetails, ProviderError> {
		self.request("getrawmempool", ()).await
	}

	async fn import_private_key(&self, wif: String) -> Result<NeoAddress, ProviderError> {
		self.request("importprivkey", vec![wif.to_value()]).await
	}

	async fn get_block_header_hash(&self, hash: H256) -> Result<NeoBlock, ProviderError> {
		self.request("getblockheader", vec![hash.to_value(), 1.to_value()]).await
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

impl Provider<HttpProvider> {
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
	/// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
	/// use neo_providers::{Middleware, Provider};
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

impl TryFrom<&str> for Provider<HttpProvider> {
	type Error = ParseError;

	fn try_from(src: &str) -> Result<Self, Self::Error> {
		Ok(Provider::new(HttpProvider::new(Url::parse(src)?)))
	}
}

impl TryFrom<String> for Provider<HttpProvider> {
	type Error = ParseError;

	fn try_from(src: String) -> Result<Self, Self::Error> {
		Provider::try_from(src.as_str())
	}
}

impl<'a> TryFrom<&'a String> for Provider<HttpProvider> {
	type Error = ParseError;

	fn try_from(src: &'a String) -> Result<Self, Self::Error> {
		Provider::try_from(src.as_str())
	}
}

#[cfg(not(target_arch = "wasm32"))]
impl Provider<RetryClient<HttpProvider>> {
	/// Create a new [`RetryClient`] by connecting to the provided URL. Errors
	/// if `src` is not a valid URL
	pub fn new_client(src: &str, max_retry: u32, initial_backoff: u64) -> Result<Self, ParseError> {
		Ok(Provider::new(RetryClient::new(
			HttpProvider::new(Url::parse(src)?),
			Box::new(HttpRateLimitRetryPolicy),
			max_retry,
			initial_backoff,
		)))
	}
}

mod sealed {
	use crate::{Http, Provider};
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
///  # use neo_providers::{Http, Provider, ProviderExt};
///  # async fn t() {
/// let http_provider = Provider::<Http>::connect("https://eth.llamarpc.com").await;
/// # }
/// ```
///
/// This is essentially short for
///
/// ```no_run
/// use std::convert::TryFrom;
/// use neo_config::NeoNetwork;
/// use neo_providers::{Http, Provider, ProviderExt};
/// let http_provider = Provider::<Http>::try_from("https://eth.llamarpc.com").unwrap().set_network(NeoNetwork::MainNet.to_magic());
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
impl ProviderExt for Provider<HttpProvider> {
	type Error = ParseError;

	async fn try_connect(url: &str) -> Result<Self, Self::Error>
	where
		Self: Sized,
	{
		let mut provider = Provider::try_from(url)?;
		let Some(chain) = provider.get_net_version().await.ok() else { panic!("") };
		provider.set_network(u32::from_str(&chain).unwrap());

		Ok(provider)
	}

	fn set_network(&mut self, network: u32) -> &mut Self {
		// if let Some(blocktime) = chain {
		// use half of the block time
		self.set_interval(Duration::from_millis(network as u64 / 2));
		// }
		self
	}
}

/// Returns true if the endpoint is local
///
/// # Example
///
/// ```
/// use neo_providers::is_local_endpoint;
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
			match host {
				Host::Domain(domain) =>
					return domain.contains("localhost") || domain.contains("localdev.me"),
				Host::Ipv4(ipv4) =>
					return ipv4 == Ipv4Addr::LOCALHOST
						|| ipv4.is_link_local() || ipv4.is_loopback()
						|| ipv4.is_private(),
				Host::Ipv6(ipv6) => return ipv6.is_loopback(),
			}
		}
	}
	false
}
