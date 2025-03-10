use async_trait::async_trait;
use futures_util::lock::Mutex;
use getset::{Getters, Setters};
use primitive_types::{H160, H256};
use rustc_serialize::{
	base64,
	base64::ToBase64,
	hex::{FromHex, ToHex},
};
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
use tracing::{debug, trace};
use tracing_futures::Instrument;
use url::{Host, ParseError, Url};

// Replace the generic import with specific imports
use crate::{
	neo_builder::{
		CallFlags, InteropService, ScriptBuilder, TransactionBuilder, TransactionSigner,
	},
	neo_clients::{
		APITrait, Http, HttpProvider, HttpRateLimitRetryPolicy, JsonRpcProvider, ProviderError,
		RetryClient, RwClient,
	},
};

use crate::{
	builder::{Signer, Transaction, TransactionSendToken},
	codec::NeoSerializable,
	config::NEOCONFIG,
	neo_clients::rpc::rpc_client::sealed::Sealed,
	neo_protocol::*,
	neo_types::ScriptHashExtension,
	prelude::Base64Encode,
	Address, ContractManifest, ContractParameter, ContractState, InvocationResult,
	NativeContractState, NefFile, StackItem, ValueExtension,
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
		let first_segment = s
			.split('/')
			.next()
			.ok_or(ProviderError::ParseError("Invalid client string format".to_string()))?;
		match first_segment.to_lowercase().as_str() {
			"neo" => Ok(NeoClient::NEO),
			_ => Err(ProviderError::UnsupportedNodeClient),
		}
	}
}

/// An abstract provider for interacting with the [Neo JSON RPC
/// API](https://github.com/neo/wiki/JSON-RPC). Must be instantiated
/// with a data transport which implements the `JsonRpcClient` trait
/// (e.g. HTTP, Websockets etc.)
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
#[derive(Clone, Debug, Getters)]
pub struct RpcClient<P> {
	provider: P,
	nns: Option<Address>,
	interval: Option<Duration>,
	from: Option<Address>,
	_node_client: Arc<Mutex<Option<NeoVersion>>>,
	// #[getset(get = "pub")]
	// allow_transmission_on_fault: bool,
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
			// allow_transmission_on_fault: false,
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
		let span = tracing::trace_span!("rpc: ", method = method, params = ?serde_json::to_string(&params)?);
		// https://docs.rs/tracing/0.1.22/tracing/span/struct.Span.html#in-asynchronous-code
		let res = async move {
			// trace!("tx");
			let fetched = self.provider.fetch(method, params).await;
			let res: R = fetched.map_err(Into::into)?;
			// debug!("Response: = {:?}", res);
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

	async fn network(&self) -> Result<u32, ProviderError> {
		// trace!("network = {:?}", self.get_version().await.unwrap());
		if NEOCONFIG.lock().map_err(|_| ProviderError::LockError)?.network.is_none() {
			let version = self.get_version().await?;
			let protocol = version.protocol.ok_or(ProviderError::ProtocolNotFound)?;
			return Ok(protocol.network);
		}
		NEOCONFIG
			.lock()
			.map_err(|_| ProviderError::LockError)?
			.network
			.ok_or(ProviderError::NetworkNotFound)
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

	/// Gets the block by hash string.
	/// - Parameters:
	///   - hash: The block hash as a string
	///   - full_tx: Whether to get block information with all transaction objects or just the block header
	/// - Returns: The request object
	async fn get_block_by_hash(
		&self,
		hash: &str,
		full_tx: bool,
	) -> Result<NeoBlock, ProviderError> {
		let block_hash = H256::from_str(hash)
			.map_err(|e| ProviderError::ParseError(format!("Invalid block hash: {}", e)))?;
		self.get_block(block_hash, full_tx).await
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
	async fn get_transaction(&self, hash: H256) -> Result<RTransaction, ProviderError> {
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

	/// Sends a transaction to the network
	///
	/// # Arguments
	///
	/// * `tx` - The transaction to send
	///
	/// # Returns
	///
	/// A `Result` containing the transaction hash or a `ProviderError`
	async fn send_transaction<'a>(&self, tx: Transaction<'a, P>) -> Result<H256, ProviderError> {
		let tx_hex = hex::encode(tx.to_array());
		let result = self.send_raw_transaction(tx_hex).await?;

		// Convert the transaction hash to H256
		let tx_hash = H256::from_str(&result.hash.to_string()).map_err(|e| {
			ProviderError::ParseError(format!("Failed to parse transaction hash: {}", e))
		})?;

		Ok(tx_hash)
	}

	/// Broadcasts a new block over the NEO network.
	/// - Parameter serializedBlockAsHex: The block in hexadecimal
	/// - Returns: The request object
	async fn submit_block(&self, hex: String) -> Result<SubmitBlock, ProviderError> {
		self.request("submitblock", vec![hex.to_value()]).await
	}

	/// Broadcasts the node's address to the network
	async fn broadcast_address(&self) -> Result<bool, ProviderError> {
		self.request("broadcastaddr", Vec::<String>::new()).await
	}

	/// Broadcasts a block to the network
	async fn broadcast_block(&self, block: NeoBlock) -> Result<bool, ProviderError> {
		let block_json = serde_json::to_string(&block)
			.map_err(|e| ProviderError::ParseError(format!("Failed to serialize block: {}", e)))?;

		self.request("broadcastblock", vec![block_json.to_value()]).await
	}

	/// Broadcasts a request for blocks to the network
	///
	/// # Arguments
	///
	/// * `hash` - The hash of the block to start from
	/// * `count` - The number of blocks to request
	///
	/// # Returns
	///
	/// A `Result` containing a boolean indicating success or a `ProviderError`
	async fn broadcast_get_blocks(&self, hash: &str, count: u32) -> Result<bool, ProviderError> {
		let hash_obj = H256::from_str(hash)
			.map_err(|e| ProviderError::ParseError(format!("Invalid block hash: {}", e)))?;

		self.request("broadcastgetblocks", vec![hash_obj.to_value(), count.to_value()])
			.await
	}

	/// Broadcasts a transaction to the network
	///
	/// # Arguments
	///
	/// * `tx` - The transaction to broadcast
	///
	/// # Returns
	///
	/// A `Result` containing a boolean indicating success or a `ProviderError`
	async fn broadcast_transaction(&self, tx: RTransaction) -> Result<bool, ProviderError> {
		let tx_json = serde_json::to_string(&tx).map_err(|e| {
			ProviderError::ParseError(format!("Failed to serialize transaction: {}", e))
		})?;

		self.request("broadcasttransaction", vec![tx_json.to_value()]).await
	}

	/// Creates a contract deployment transaction
	async fn create_contract_deployment_transaction(
		&self,
		nef: NefFile,
		manifest: ContractManifest,
		signers: Vec<Signer>,
	) -> Result<TransactionBuilder<P>, ProviderError> {
		let nef_bytes = nef.to_array();
		let manifest_json = serde_json::to_string(&manifest).map_err(|e| {
			ProviderError::ParseError(format!("Failed to serialize manifest: {}", e))
		})?;

		let mut script_builder = ScriptBuilder::new();
		script_builder
			.push_data(manifest_json.as_bytes().to_vec())
			.push_data(nef_bytes)
			.sys_call(InteropService::SystemContractCall);

		let mut builder = TransactionBuilder::new();
		builder.extend_script(script_builder.to_bytes());

		// Add signers to the transaction
		// Note: Signers will be added when the transaction is built

		Ok(builder)
	}

	/// Creates a contract update transaction
	async fn create_contract_update_transaction(
		&self,
		contract_hash: H160,
		nef: NefFile,
		manifest: ContractManifest,
		signers: Vec<Signer>,
	) -> Result<TransactionBuilder<P>, ProviderError> {
		let nef_bytes = nef.to_array();
		let manifest_json = serde_json::to_string(&manifest).map_err(|e| {
			ProviderError::ParseError(format!("Failed to serialize manifest: {}", e))
		})?;

		let mut script_builder = ScriptBuilder::new();
		script_builder
			.push_data(manifest_json.as_bytes().to_vec())
			.push_data(nef_bytes)
			.push_data(contract_hash.to_vec())
			.sys_call(InteropService::SystemContractCall);

		let mut builder = TransactionBuilder::new();
		builder.extend_script(script_builder.to_bytes());

		// Add signers to the transaction
		// Note: Signers will be added when the transaction is built

		Ok(builder)
	}

	/// Creates an invocation transaction
	async fn create_invocation_transaction(
		&self,
		contract_hash: H160,
		method: &str,
		parameters: Vec<ContractParameter>,
		signers: Vec<Signer>,
	) -> Result<TransactionBuilder<P>, ProviderError> {
		let mut script_builder = ScriptBuilder::new();
		script_builder
			.contract_call(&contract_hash, method, &parameters, None)
			.map_err(|e| {
				ProviderError::ParseError(format!("Failed to create contract call: {}", e))
			})?;

		let mut builder = TransactionBuilder::new();
		builder.extend_script(script_builder.to_bytes());

		// Add signers to the transaction
		// Note: Signers will be added when the transaction is built

		Ok(builder)
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
		let hex_bytes = hex
			.from_hex()
			.map_err(|e| ProviderError::ParseError(format!("Failed to parse hex: {}", e)))?;
		let script_base64 = serde_json::to_value(hex_bytes.to_base64())?;
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
	async fn calculate_network_fee(
		&self,
		txBase64: String,
	) -> Result<NeoNetworkFee, ProviderError> {
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
	) -> Result<RTransaction, ProviderError> {
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
	) -> Result<RTransaction, ProviderError> {
		let params = match from {
			Some(f) => json!([f.to_address(), send_tokens]),
			None => json!([send_tokens]),
		};
		//let params = [from.unwrap().to_value(), send_tokens.to_value()].to_vec();
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
		to: H160,
		amount: u32,
	) -> Result<RTransaction, ProviderError> {
		let params = json!([token_hash.to_hex(), to.to_address(), amount]);
		self.request("sendtoaddress", params).await
	}

	async fn cancel_transaction(
		&self,
		txHash: H256,
		signers: Vec<H160>,
		extra_fee: Option<u64>,
	) -> Result<RTransaction, ProviderError> {
		//to be implemented
		if signers.is_empty() {
			return Err(ProviderError::CustomError("signers must not be empty".into()));
		}
		let signer_addresses: Vec<String> =
			signers.into_iter().map(|signer| signer.to_address()).collect();
		let params = json!([
			txHash.0.to_hex(),
			signer_addresses,
			extra_fee.map_or("".to_string(), |fee| fee.to_string())
		]);
		// let params = [from.to_value(), vec![send_token.to_value()].into()].to_vec();
		self.request("canceltransaction", params).await
	}

	/// Gets the application logs of the specified transaction hash.
	/// - Parameter txHash: The transaction hash
	/// - Returns: The request object
	async fn get_application_log(&self, tx_hash: H256) -> Result<ApplicationLog, ProviderError> {
		self.request("getapplicationlog", vec![tx_hash.0.to_hex().to_value()]).await
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
		let params = json!([script_hash.to_address()]);
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
		self.request("getnep17transfers", json!([script_hash.to_address(), from])).await
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
		let params = json!([script_hash.to_address(), from, to]);
		self.request("getnep17transfers", params).await
	}

	/// Gets all NEP-11 balances of the specified account.
	/// - Parameter scriptHash: The account's script hash
	/// - Returns: The request object
	async fn get_nep11_balances(&self, script_hash: H160) -> Result<Nep11Balances, ProviderError> {
		let params = json!([script_hash.to_address()]);
		self.request("getnep11balances", params).await
	}

	/// Gets all NEP-11 transaction of the given account.
	/// - Parameter scriptHash: The account's script hash
	/// - Returns: The request object
	async fn get_nep11_transfers(
		&self,
		script_hash: H160,
	) -> Result<Nep11Transfers, ProviderError> {
		let params = json!([script_hash.to_address()]);
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
		let params = json!([script_hash.to_address(), from]);
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
		let params = json!([script_hash.to_address(), from, to]);
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
		let params = json!([script_hash.to_address(), token_id]);
		self.request("getnep11properties", params).await
	}

	/// Gets the state root by the block height.
	/// - Parameter blockIndex: The block index
	/// - Returns: The request object
	async fn get_state_root(&self, block_index: u32) -> Result<StateRoot, ProviderError> {
		let params = json!([block_index]);
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
			json!([
				root_hash.0.to_hex(),
				contract_hash.to_hex(),
				Base64Encode::to_base64(&key.to_string())
			]),
		)
		.await
	}

	/// Verifies the proof data and gets the value of the storage corresponding to the key.
	/// - Parameters:
	///   - rootHash: The root hash
	///   - proof: The proof data of the state root
	/// - Returns: The request object
	async fn verify_proof(&self, root_hash: H256, proof: &str) -> Result<String, ProviderError> {
		let params = json!([root_hash.0.to_hex(), Base64Encode::to_base64(&proof.to_string())]);
		self.request("verifyproof", params).await
	}

	/// Gets the state root height.
	/// - Returns: The request object
	async fn get_state_height(&self) -> Result<StateHeight, ProviderError> {
		self.request("getstateheight", Vec::<StateHeight>::new()).await
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
			json!([
				root_hash.0.to_hex(),
				contract_hash.to_hex(),
				Base64Encode::to_base64(&key.to_string())
			]), //key.to_base64()],
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
		let mut params = json!([
			root_hash.0.to_hex(),
			contract_hash.to_hex(),
			Base64Encode::to_base64(&key_prefix.to_string())
		]);
		if let (Some(start_key), Some(count)) = (start_key, count) {
			params = json!([
				root_hash.0.to_hex(),
				contract_hash.to_hex(),
				Base64Encode::to_base64(&key_prefix.to_string()),
				Base64Encode::to_base64(&start_key.to_string()),
				count,
			]);
		} else if let Some(count) = count {
			params = json!([
				root_hash.0.to_hex(),
				contract_hash.to_hex(),
				Base64Encode::to_base64(&key_prefix.to_string()),
				"".to_string(),
				count,
			]);
		} else if let Some(start_key) = start_key {
			params = json!([
				root_hash.0.to_hex(),
				contract_hash.to_hex(),
				Base64Encode::to_base64(&key_prefix.to_string()),
				Base64Encode::to_base64(&start_key.to_string()),
			]);
		}

		// if let Some(start_key) = start_key {
		// 	params.push(start_key.to_value())
		// }
		// if let Some(count) = count {
		// 	params.push(count.to_value())
		// }
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
		let hex_bytes = hex
			.from_hex()
			.map_err(|e| ProviderError::ParseError(format!("Failed to parse hex: {}", e)))?;
		let script_base64 = serde_json::to_value(hex_bytes.to_base64())?;
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
		Box::pin(async move { self.get_mem_pool().await })
	}

	fn import_private_key<'life0, 'async_trait>(
		&'life0 self,
		wif: String,
	) -> Pin<Box<dyn Future<Output = Result<NeoAddress, Self::Error>> + Send + 'async_trait>>
	where
		'life0: 'async_trait,
		Self: 'async_trait,
	{
		Box::pin(async move { self.import_priv_key(wif).await })
	}

	fn get_block_header_hash<'life0, 'async_trait>(
		&'life0 self,
		hash: H256,
	) -> Pin<Box<dyn Future<Output = Result<NeoBlock, Self::Error>> + Send + 'async_trait>>
	where
		'life0: 'async_trait,
		Self: 'async_trait,
	{
		Box::pin(async move { self.get_block_header(hash).await })
	}

	async fn send_to_address_send_token(
		&self,
		send_token: &TransactionSendToken,
	) -> Result<RTransaction, ProviderError> {
		// let params = [send_token.to_value()].to_vec();
		let params = json!([send_token.token.to_hex(), send_token.address, send_token.value,]);
		self.request("sendtoaddress", params).await
	}

	async fn send_from_send_token(
		&self,
		send_token: &TransactionSendToken,
		from: H160,
	) -> Result<RTransaction, ProviderError> {
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
		Ok(RpcClient::new(Http::new(src)?))
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
			Http::new(src)?,
			Box::new(HttpRateLimitRetryPolicy),
			max_retry,
			initial_backoff,
		)))
	}
}

mod sealed {
	use crate::neo_clients::{Http, RpcClient};

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
		Self::try_connect(url).await.expect("Failed to connect to provider. This is a fatal error as connect() is designed to exit on error.")
	}

	/// Try to create a new `Provider`
	async fn try_connect(url: &str) -> Result<Self, Self::Error>
	where
		Self: Sized;

	/// Customize `Provider` settings for chain.
	///
	/// E.g. the average block time can be used to tune the polling interval.
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
		let network = provider.get_version().await.map_err(|e| ParseError::InvalidPort)?;
		let protocol = network.protocol.ok_or(ParseError::InvalidPort)?;
		provider.set_network(protocol.network);

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
						|| ipv4.is_link_local()
						|| ipv4.is_loopback()
						|| ipv4.is_private(),
				Host::Ipv6(ipv6) => ipv6.is_loopback(),
			};
		}
	}
	false
}

#[cfg(test)]
mod tests {
	use base64::{engine::general_purpose, Engine};
	use blake2::digest::Mac;
	use lazy_static::lazy_static;
	use log::debug;
	use primitive_types::{H160, H256};
	use rustc_serialize::{
		base64::FromBase64,
		hex::{FromHex, ToHex},
	};
	use serde_json::{json, Value};
	use std::{any::Any, hash::Hash, str::FromStr, sync::Arc};
	use tokio::{
		self,
		sync::{Mutex, OnceCell},
	};
	use tracing::field::debug;
	use url::Url;
	use wiremock::{
		matchers::{body_json, method as http_method, method, path},
		Mock, MockServer, ResponseTemplate,
	};

	use crate::{
		builder::{
			AccountSigner as AccountSignerType, OracleResponseCode, Signer::AccountSigner,
			SignerTrait, TransactionSendToken, WitnessAction, WitnessCondition, WitnessRule,
			WitnessScope,
		},
		config::TestConstants,
		crypto::Secp256r1PublicKey,
		neo_clients::{api_trait::APITrait, HttpProvider, MockClient, ProviderError},
		neo_protocol::{
			AddressEntry, ConflictsAttribute, HighPriorityAttribute, NeoWitness, Nep11Balance,
			Nep11Token, Nep11Transfer, Nep17Balance, Nep17Transfer, NotValidBeforeAttribute,
			OracleResponse, OracleResponseAttribute, RTransaction, RTransactionSigner, StateResult,
			States, TransactionAttributeEnum, Validator,
		},
		neo_types::{
			Base64Encode, ContractABI, ContractMethod, ContractParameter2, ContractPermission,
			Diagnostics, InvokedContract, NodePluginType, StorageChange, ToBase64,
		},
		providers::RpcClient,
		ContractManifest, ContractNef, ContractParameterType, ContractState, InvocationResult,
		NeoVMStateType, ScriptHashExtension, StackItem, TypeError, VMState,
	};

	async fn setup_mock_server() -> MockServer {
		MockServer::start().await
	}

	async fn mock_rpc_response(
		mock_server: &MockServer,
		rpc_method: &str,
		params: serde_json::Value,
		result: serde_json::Value,
	) -> RpcClient<HttpProvider> {
		// Create the JSON response
		// let json_response = json!({
		// 	"jsonrpc": "2.0",
		// 	"id": 1,
		// 	"result": result
		// });

		// // Convert JSON response to a pretty-printed string
		// let json_response_str = serde_json::to_string_pretty(&json_response)
		// 	.expect("Failed to serialize JSON");

		// // Print the JSON response
		// println!("Formed JSON response:\n{}", json_response_str);

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
		let http_client = HttpProvider::new(url).unwrap();
		RpcClient::new(http_client)
	}

	async fn mock_rpc_response_with_id(
		mock_server: &MockServer,
		rpc_method: &str,
		params: serde_json::Value,
		result: serde_json::Value,
		id: u32,
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
				"id": id,
				"result": result
			})))
			.mount(mock_server)
			.await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url).unwrap();
		RpcClient::new(http_client)
	}

	async fn mock_rpc_response_without_request(
		mock_server: &MockServer,
		result: serde_json::Value,
	) -> RpcClient<HttpProvider> {
		Mock::given(http_method("POST"))
			.and(path("/"))
			.respond_with(ResponseTemplate::new(200).set_body_json(json!({
				"jsonrpc": "2.0",
				"id": 1,
				"result": result
			})))
			.mount(mock_server)
			.await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url).unwrap();
		RpcClient::new(http_client)
	}

	#[tokio::test]
	async fn test_error_reponse() {
		let _ = env_logger::builder().is_test(true).try_init();

		let mock_provider = Arc::new(Mutex::new(MockClient::new().await));
		{
			let mut mock_provider_guard = mock_provider.lock().await; // Lock the mock_provider once
			mock_provider_guard
				.mock_response_error(json!({
					"code": -32602,
					"message": "Invalid address length, expected 40 got 64 bytes",
					"data": null
				}))
				.await;
			mock_provider_guard.mount_mocks().await;
		}

		let client = {
			let mock_provider = mock_provider.lock().await;
			Arc::new(mock_provider.into_client())
		};
		let result = client.get_best_block_hash().await;

		// Assert that the error is a JsonRpcError
		assert!(matches!(result, Err(ProviderError::JsonRpcError(_))), "Expected a JsonRpcError.");
		if let ProviderError::JsonRpcError(json_rpc_error) = result.unwrap_err() {
			assert_eq!(json_rpc_error.code, -32602);
			assert_eq!(json_rpc_error.message, "Invalid address length, expected 40 got 64 bytes");
		}
	}

	#[tokio::test]
	async fn test_error_reponse_complex_data() {
		let _ = env_logger::builder().is_test(true).try_init();

		let mock_provider = Arc::new(Mutex::new(MockClient::new().await));
		{
			let mut mock_provider_guard = mock_provider.lock().await; // Lock the mock_provider once
			mock_provider_guard
				.mock_response_error(json!({
					"code": -32602,
					"message": "Invalid address length, expected 40 got 64 bytes",
					"data": {
						"foo": "bar"
					}
				}))
				.await;
			mock_provider_guard.mount_mocks().await;
		}

		let client = {
			let mock_provider = mock_provider.lock().await;
			Arc::new(mock_provider.into_client())
		};
		let result = client.get_best_block_hash().await;

		// Assert that the error is a JsonRpcError
		assert!(matches!(result, Err(ProviderError::JsonRpcError(_))), "Expected a JsonRpcError.");
		if let ProviderError::JsonRpcError(json_rpc_error) = result.unwrap_err() {
			assert_eq!(json_rpc_error.code, -32602);
			assert_eq!(json_rpc_error.message, "Invalid address length, expected 40 got 64 bytes");
			assert_eq!(json_rpc_error.data, Some(json!({ "foo": "bar" })));
		}
	}

	// Blockchain Methods

	#[tokio::test]
	async fn test_get_best_block_hash() {
		let _ = env_logger::builder().is_test(false).try_init();

		let mock_provider = Arc::new(Mutex::new(MockClient::new().await));

		// Set the mock response before using the client
		{
			let mut mock_provider_guard = mock_provider.lock().await; // Lock the mock_provider once
			mock_provider_guard
				.mock_response(
					"getbestblockhash",
					json!([]),
					json!("0x3d1e051247f246f60dd2ba4f90f799578b5d394157b1f2b012c016b29536b899"),
				)
				.await;
			mock_provider_guard.mount_mocks().await;
		}

		let client = {
			let mock_provider = mock_provider.lock().await;
			Arc::new(mock_provider.into_client())
		};

		let result = client.get_best_block_hash().await;

		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getbestblockhash",
            "params": [],
            "id": 1
        }"#;

		verify_request(mock_provider.lock().await.server(), expected_request_body)
			.await
			.unwrap();

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		assert_eq!(
			result.unwrap(),
			H256::from_str("0x3d1e051247f246f60dd2ba4f90f799578b5d394157b1f2b012c016b29536b899")
				.unwrap()
		);
	}

	#[tokio::test]
	async fn test_get_block_hash() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response(
			&mock_server,
			"getblockhash",
			json!([16293]),
			json!("0x147ad6a26f1d5a9bb2bea3f0b2ca9fab3824873beaf8887e87d08c8fd98a81b3"),
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
		assert_eq!(
			result.unwrap(),
			H256::from_str("0x147ad6a26f1d5a9bb2bea3f0b2ca9fab3824873beaf8887e87d08c8fd98a81b3")
				.unwrap()
		);
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
		"nonce": "7F8EEE652D4BC959",
        "index": 1914006,
		"primary": 1,
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
            "method": "getblock",
            "params": [12345,1],
            "id": 1
        }"#;

		let result = provider.get_block_by_index(12345, true).await;

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		let neo_block = result.unwrap();
		assert_eq!(
			neo_block.hash,
			H256::from_str("0x1de7e5eaab0f74ac38f5191c038e009d3c93ef5c392d1d66fa95ab164ba308b8")
				.unwrap()
		);
		assert_eq!(neo_block.size, 1217);
		assert_eq!(neo_block.version, 0);
		assert_eq!(
			neo_block.prev_block_hash,
			H256::from_str("0x045cabde4ecbd50f5e4e1b141eaf0842c1f5f56517324c8dcab8ccac924e3a39")
				.unwrap()
		);
		assert_eq!(
			neo_block.merkle_root_hash,
			H256::from_str("0x6afa63201b88b55ad2213e5a69a1ad5f0db650bc178fc2bedd2fb301c1278bf7")
				.unwrap()
		);
		assert_eq!(neo_block.time, 1539968858);
		assert_eq!(neo_block.nonce, "7F8EEE652D4BC959");
		assert_eq!(neo_block.get_nonce_as_u64().unwrap(), 9191546007828810073);
		assert_eq!(neo_block.index, 1914006);
		assert_eq!(neo_block.primary.unwrap(), 1);
		assert_eq!(neo_block.next_consensus, "AWZo4qAxhT8fwKL93QATSjCYCgHmCY1XLB");
		assert!(neo_block.witnesses.is_some());
		assert_eq!(neo_block.witnesses.clone().unwrap().len(), 1);
		assert!(neo_block.witnesses.clone().unwrap().contains(
			&NeoWitness::new("DEBJVWapboNkCDlH9uu+tStOgGnwODlolRifxTvQiBkhM0vplSPo4vMj9Jt3jvzztMlwmO75Ss5cptL8wUMxASjZ".to_string(),
							"EQwhA/HsPB4oPogN5unEifDyfBkAfFM4WqpMDJF8MgB57a3yEQtBMHOzuw==".to_string()
			)
		));
		assert!(neo_block.transactions.is_some());
		assert_eq!(neo_block.transactions.clone().unwrap().len(), 2);

		let expected_transactions = vec![
			RTransaction::new(
				H256::from_str("0x46eca609a9a8c8340ee56b174b04bc9c9f37c89771c3a8998dc043f5a74ad510").unwrap(),
				267,
				0,
				565086327,
				"AHE5cLhX5NjGB5R2PcdUvGudUoGUBDeHX4".to_string(),
				"0".to_string(),
				"0".to_string(),
				2107425,
				vec![
					RTransactionSigner::new(H160::from_str("0xf68f181731a47036a99f04dad90043a744edec0f").unwrap(),
					vec![
						WitnessScope::CalledByEntry
					]
					)
				],
				Vec::new(),
				"AGQMFObBATZUrxE9ipaL3KUsmUioK5U9DBQP7O1Ep0MA2doEn6k2cKQxFxiP9hPADAh0cmFuc2ZlcgwUiXcg2M129PAKv6N8Dt2InCCP3ptBYn1bUjg".to_string(),
				vec![
					NeoWitness::new("DEBR7EQOb1NUjat1wrINzBNKOQtXoUmRVZU8h5c8K5CLMCUVcGkFVqAAGUJDh3mVcz6sTgXvmMuujWYrBveeM4q+".to_string(),
							"EQwhA/HsPB4oPogN5unEifDyfBkAfFM4WqpMDJF8MgB57a3yEQtBMHOzuw==".to_string()
					)
				]
			),
			RTransaction::new(
				H256::from_str("0x46eca609a9a8c8340ee56b174b04bc9c9f37c89771c3a8998dc043f5a74ad510").unwrap(),
				267,
				0,
				565086327,
				"AHE5cLhX5NjGB5R2PcdUvGudUoGUBDeHX4".to_string(),
				"0".to_string(),
				"0".to_string(),
				2107425,
				vec![
					RTransactionSigner::new(H160::from_str("0xf68f181731a47036a99f04dad90043a744edec0f").unwrap(),
					vec![
						WitnessScope::CalledByEntry
					]
					)
				],
				Vec::new(),
				"AGQMFObBATZUrxE9ipaL3KUsmUioK5U9DBQP7O1Ep0MA2doEn6k2cKQxFxiP9hPADAh0cmFuc2ZlcgwUiXcg2M129PAKv6N8Dt2InCCP3ptBYn1bUjg".to_string(),
				vec![
					NeoWitness::new("DEBR7EQOb1NUjat1wrINzBNKOQtXoUmRVZU8h5c8K5CLMCUVcGkFVqAAGUJDh3mVcz6sTgXvmMuujWYrBveeM4q+".to_string(),
							"EQwhA/HsPB4oPogN5unEifDyfBkAfFM4WqpMDJF8MgB57a3yEQtBMHOzuw==".to_string()
					)
				]
			)
		];

		assert_eq!(neo_block.transactions.clone().unwrap(), expected_transactions);
		assert_eq!(neo_block.confirmations, 7878);
		assert_eq!(
			neo_block.next_block_hash.unwrap(),
			H256::from_str("0x4a97ca89199627f877b6bffe865b8327be84b368d62572ef20953829c3501643")
				.unwrap()
		);

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
		"nonce": "7F8EEE652D4BC95A",
        "index": 1914006,
        "nextconsensus": "AWZo4qAxhT8fwKL93QATSjCYCgHmCY1XLB",
        "witnesses": [
            {
                "invocation": "DEBJVWapboNkCDlH9uu+tStOgGnwODlolRifxTvQiBkhM0vplSPo4vMj9Jt3jvzztMlwmO75Ss5cptL8wUMxASjZ",
                "verification": "EQwhA/HsPB4oPogN5unEifDyfBkAfFM4WqpMDJF8MgB57a3yEQtBMHOzuw=="
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
		let neo_block = result.unwrap();
		assert_eq!(
			neo_block.hash,
			H256::from_str("0x1de7e5eaab0f74ac38f5191c038e009d3c93ef5c392d1d66fa95ab164ba308b8")
				.unwrap()
		);
		assert_eq!(neo_block.size, 1217);
		assert_eq!(neo_block.version, 0);
		assert_eq!(
			neo_block.prev_block_hash,
			H256::from_str("0x045cabde4ecbd50f5e4e1b141eaf0842c1f5f56517324c8dcab8ccac924e3a39")
				.unwrap()
		);
		assert_eq!(
			neo_block.merkle_root_hash,
			H256::from_str("0x6afa63201b88b55ad2213e5a69a1ad5f0db650bc178fc2bedd2fb301c1278bf7")
				.unwrap()
		);
		assert_eq!(neo_block.time, 1539968858);
		assert_eq!(neo_block.nonce, "7F8EEE652D4BC95A");
		assert_eq!(neo_block.get_nonce_as_u64().unwrap(), 9191546007828810074);
		assert_eq!(neo_block.index, 1914006);
		assert_eq!(neo_block.next_consensus, "AWZo4qAxhT8fwKL93QATSjCYCgHmCY1XLB");
		assert!(neo_block.witnesses.is_some());
		assert_eq!(neo_block.witnesses.clone().unwrap().len(), 1);
		assert!(neo_block.witnesses.clone().unwrap().contains(
			&NeoWitness::new("DEBJVWapboNkCDlH9uu+tStOgGnwODlolRifxTvQiBkhM0vplSPo4vMj9Jt3jvzztMlwmO75Ss5cptL8wUMxASjZ".to_string(),
							"EQwhA/HsPB4oPogN5unEifDyfBkAfFM4WqpMDJF8MgB57a3yEQtBMHOzuw==".to_string()
			)
		));
		assert!(neo_block.transactions.is_some());
		assert_eq!(neo_block.transactions.clone().unwrap().len(), 0);
		assert_eq!(neo_block.confirmations, 7878);
		assert_eq!(
			neo_block.next_block_hash.unwrap(),
			H256::from_str("0x4a97ca89199627f877b6bffe865b8327be84b368d62572ef20953829c3501643")
				.unwrap()
		);

		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_block_by_hash() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response(
			&mock_server,
			"getblock",
			json!(["2240b34669038f82ac492150d391dfc3d7fe5e3c1d34e5b547d50e99c09b468d", 1]),
			json!([]),
		)
		.await;

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
		let provider = mock_rpc_response_with_id(
            &mock_server,
            "getblock",
            json!([12345, 0]),
            json!("00000000ebaa4ed893333db1ed556bb24145f4e7fe40b9c7c07ff2235c7d3d361ddb27e603da9da4c7420d090d0e29c588cfd701b3f81819375e537c634bd779ddc7e2e2c436cc5ba53f00001952d428256ad0cdbe48d3a3f5d10013ab9ffee489706078714f1ea201c340c44387d762d1bcb2ab0ec650628c7c674021f333ee7666e2a03805ad86df3b826b5dbf5ac607a361807a047d43cf6bba726dcb06a42662aee7e78886c72faef940e6cef9abab82e1e90c6683ac8241b3bf51a10c908f01465f19c3df1099ef5de5d43a648a6e4ab63cc7d5e88146bddbe950e8041e44a2b0b81f21ad706e88258540fd19314f46ad452b4cbedf58bf9d266c0c808374cd33ef18d9a0575b01e47f6bb04abe76036619787c457c49288aeb91ff23cdb85771c0209db184801d5bdd348b532102103a7f7dd016558597f7960d27c516a4394fd968b9e65155eb4b013e4040406e2102a7bc55fe8684e0119768d104ba30795bdcc86619e864add26156723ed185cd622102b3622bf4017bdfe317c58aed5f4c753f206b7db896046fa7d774bbc4bf7f8dc22103d90c07df63e690ce77912e10ab51acc944b66860237b608c4f8f8309e71ee69954ae0100001952d42800000000"),
			67,
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
		assert_eq!(result.unwrap(), "00000000ebaa4ed893333db1ed556bb24145f4e7fe40b9c7c07ff2235c7d3d361ddb27e603da9da4c7420d090d0e29c588cfd701b3f81819375e537c634bd779ddc7e2e2c436cc5ba53f00001952d428256ad0cdbe48d3a3f5d10013ab9ffee489706078714f1ea201c340c44387d762d1bcb2ab0ec650628c7c674021f333ee7666e2a03805ad86df3b826b5dbf5ac607a361807a047d43cf6bba726dcb06a42662aee7e78886c72faef940e6cef9abab82e1e90c6683ac8241b3bf51a10c908f01465f19c3df1099ef5de5d43a648a6e4ab63cc7d5e88146bddbe950e8041e44a2b0b81f21ad706e88258540fd19314f46ad452b4cbedf58bf9d266c0c808374cd33ef18d9a0575b01e47f6bb04abe76036619787c457c49288aeb91ff23cdb85771c0209db184801d5bdd348b532102103a7f7dd016558597f7960d27c516a4394fd968b9e65155eb4b013e4040406e2102a7bc55fe8684e0119768d104ba30795bdcc86619e864add26156723ed185cd622102b3622bf4017bdfe317c58aed5f4c753f206b7db896046fa7d774bbc4bf7f8dc22103d90c07df63e690ce77912e10ab51acc944b66860237b608c4f8f8309e71ee69954ae0100001952d42800000000");
		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_block_header_count() {
		let mock_server = setup_mock_server().await;
		let provider =
			mock_rpc_response(&mock_server, "getblockheadercount", json!([]), json!(543)).await;
		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getblockheadercount",
            "params": [],
            "id": 1
        }"#;

		let result = provider.get_block_header_count().await;
		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		assert_eq!(result.unwrap(), 543);
		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_block_count() {
		let mock_server = setup_mock_server().await;
		let provider =
			mock_rpc_response_with_id(&mock_server, "getblockcount", json!([]), json!(1234), 67)
				.await;
		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getblockcount",
            "params": [],
            "id": 1
        }"#;

		let result = provider.get_block_count().await;

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		assert_eq!(result.unwrap(), 1234);
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
					"id": -6,
					"hash": "0xd2a4cff31913016155e38e474a2c06d08be276cf",
					"nef": {
						"magic": 860243278,
						"compiler": "neo-core-v3.0",
						"source": "variable-size-source-gastoken",
						"tokens": [],
						"script": "EEEa93tnQBBBGvd7Z0AQQRr3e2dAEEEa93tnQBBBGvd7Z0A=",
						"checksum": 2663858513i64
					},
					"manifest": {
						"name": "GasToken",
						"groups": [],
						"supportedstandards": ["NEP-17"],
						"abi": {
							"methods": [
								{
									"name": "balanceOf",
									"parameters": [
										{
											"name": "account",
											"type": "Hash160"
										},
										{
											"name": "manifest",
											"type": "ByteArray"
										}
									],
									"returntype": "Integer",
									"offset": 0,
									"safe": true
								},
								{
									"name": "decimals",
									"parameters": [],
									"returntype": "Integer",
									"offset": 7,
									"safe": true
								},
								{
									"name": "symbol",
									"parameters": [],
									"returntype": "String",
									"offset": 14,
									"safe": true
								},
								{
									"name": "totalSupply",
									"parameters": [],
									"returntype": "Integer",
									"offset": 21,
									"safe": true
								},
								{
									"name": "transfer",
									"parameters": [
										{
											"name": "from",
											"type": "Hash160"
										},
										{
											"name": "to",
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
									"returntype": "Boolean",
									"offset": 28,
									"safe": false
								}
							],
							"events": [
								{
									"name": "Transfer",
									"parameters": [
										{
											"name": "from",
											"type": "Hash160"
										},
										{
											"name": "to",
											"type": "Hash160"
										},
										{
											"name": "amount",
											"type": "Integer"
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
				},
				{
					"id": -8,
					"hash": "0x49cf4e5378ffcd4dec034fd98a174c5491e395e2",
					"nef": {
						"magic": 860243278,
						"compiler": "neo-core-v3.0",
						"source": "variable-size-source-rolemanagement",
						"tokens": [],
						"script": "EEEa93tnQBBBGvd7Z0A=",
						"checksum": 983638438
					},
					"manifest": {
						"name": "RoleManagement",
						"groups": [],
						"supportedstandards": [],
						"abi": {
							"methods": [
								{
									"name": "designateAsRole",
									"parameters": [
										{
											"name": "role",
											"type": "Integer"
										},
										{
											"name": "nodes",
											"type": "Array"
										}
									],
									"returntype": "Void",
									"offset": 0,
									"safe": false
								},
								{
									"name": "getDesignatedByRole",
									"parameters": [
										{
											"name": "role",
											"type": "Integer"
										},
										{
											"name": "index",
											"type": "Integer"
										}
									],
									"returntype": "Array",
									"offset": 7,
									"safe": true
								}
							],
							"events": []
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
				},
				{
					"id": -9,
					"hash": "0xfe924b7cfe89ddd271abaf7210a80a7e11178758",
					"nef": {
						"magic": 860243278,
						"compiler": "neo-core-v3.0",
						"source": "variable-size-source-oraclecontract",
						"tokens": [],
						"script": "EEEa93tnQBBBGvd7Z0AQQRr3e2dAEEEa93tnQBBBGvd7Z0A=",
						"checksum": 2663858513i64
					},
					"manifest": {
						"name": "OracleContract",
						"groups": [],
						"supportedstandards": [],
						"abi": {
							"methods": [
								{
									"name": "finish",
									"parameters": [],
									"returntype": "Void",
									"offset": 0,
									"safe": false
								},
								{
									"name": "getPrice",
									"parameters": [],
									"returntype": "Integer",
									"offset": 7,
									"safe": true
								},
								{
									"name": "request",
									"parameters": [
										{
											"name": "url",
											"type": "String"
										},
										{
											"name": "filter",
											"type": "String"
										},
										{
											"name": "callback",
											"type": "String"
										},
										{
											"name": "userData",
											"type": "Any"
										},
										{
											"name": "gasForResponse",
											"type": "Integer"
										}
									],
									"returntype": "Void",
									"offset": 14,
									"safe": false
								},
								{
									"name": "setPrice",
									"parameters": [
										{
											"name": "price",
											"type": "Integer"
										}
									],
									"returntype": "Void",
									"offset": 21,
									"safe": false
								},
								{
									"name": "verify",
									"parameters": [],
									"returntype": "Boolean",
									"offset": 28,
									"safe": true
								}
							],
							"events": [
								{
									"name": "OracleRequest",
									"parameters": [
										{
											"name": "Id",
											"type": "Integer"
										},
										{
											"name": "RequestContract",
											"type": "Hash160"
										},
										{
											"name": "Url",
											"type": "String"
										},
										{
											"name": "Filter",
											"type": "String"
										}
									]
								},
								{
									"name": "OracleResponse",
									"parameters": [
										{
											"name": "Id",
											"type": "Integer"
										},
										{
											"name": "OriginalTx",
											"type": "Hash256"
										}
									]
								},
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
		let native_contracts = result.unwrap();
		assert_eq!(native_contracts.len(), 3);
		let c1 = native_contracts.get(0).unwrap();
		assert_eq!(c1.id, -6);
		assert_eq!(
			c1.hash(),
			&H160::from_str("0xd2a4cff31913016155e38e474a2c06d08be276cf").unwrap()
		);
		let nef1 = &c1.nef;
		assert_eq!(nef1.magic, 860243278);
		assert_eq!(nef1.compiler, "neo-core-v3.0".to_string());
		assert_eq!(nef1.source, "variable-size-source-gastoken".to_string());
		assert_eq!(nef1.tokens.len(), 0);
		let mut result = nef1.get_first_token();
		assert!(matches!(result, Err(TypeError::IndexOutOfBounds(_))));
		if let Err(TypeError::IndexOutOfBounds(msg)) = result {
			assert!(msg.contains("does not have any method tokens"));
		}
		let mut result = nef1.get_token(0);
		assert!(matches!(result, Err(TypeError::IndexOutOfBounds(_))));
		if let Err(TypeError::IndexOutOfBounds(msg)) = result {
			assert!(msg.contains("only has 0 method tokens"));
		}
		assert_eq!(nef1.script, "EEEa93tnQBBBGvd7Z0AQQRr3e2dAEEEa93tnQBBBGvd7Z0A=".to_string());
		assert_eq!(nef1.checksum, 2663858513);
		let manifest1 = c1.manifest();
		assert_eq!(manifest1.name.clone().unwrap(), "GasToken".to_string());
		assert_eq!(manifest1.groups.len(), 0);
		assert_eq!(manifest1.supported_standards.len(), 1);
		assert_eq!(manifest1.supported_standards.get(0).unwrap(), &"NEP-17".to_string());
		assert_eq!(*manifest1.get_supported_standard(0).unwrap(), manifest1.supported_standards[0]);
		let mut result = manifest1.get_supported_standard(1);
		assert!(matches!(result, Err(TypeError::IndexOutOfBounds(_))));
		if let Err(TypeError::IndexOutOfBounds(msg)) = result {
			assert!(msg.contains("only supports 1 standards"));
		}
		assert_eq!(manifest1.abi.clone().unwrap().methods.len(), 5);
		assert_eq!(manifest1.abi.clone().unwrap().events.len(), 1);
		assert_eq!(
			manifest1.abi.clone().unwrap().get_first_event(),
			manifest1.abi.clone().unwrap().get_event(0)
		);
		let binding = manifest1.abi.clone().unwrap();
		let mut result = binding.get_event(1);
		assert!(matches!(result, Err(TypeError::IndexOutOfBounds(_))));
		if let Err(TypeError::IndexOutOfBounds(msg)) = result {
			assert!(msg.contains("only has 1 events"));
		}

		let c2 = native_contracts.get(1).unwrap();
		assert_eq!(c2.id, -8);
		assert_eq!(
			c2.hash(),
			&H160::from_str("0x49cf4e5378ffcd4dec034fd98a174c5491e395e2").unwrap()
		);
		let nef2 = &c2.nef;
		assert_eq!(nef2.magic, 860243278);
		assert_eq!(nef2.compiler, "neo-core-v3.0".to_string());
		assert_eq!(nef2.source, "variable-size-source-rolemanagement".to_string());
		assert_eq!(nef2.tokens.len(), 0);
		assert_eq!(nef2.script, "EEEa93tnQBBBGvd7Z0A=".to_string());
		assert_eq!(nef2.checksum, 983638438);
		let manifest2 = c2.manifest();
		assert_eq!(manifest2.name.clone().unwrap(), "RoleManagement".to_string());
		assert_eq!(manifest2.groups.len(), 0);
		assert_eq!(manifest2.supported_standards.len(), 0);
		let mut result = manifest2.get_first_supported_standard();
		assert!(matches!(result, Err(TypeError::IndexOutOfBounds(_))));
		if let Err(TypeError::IndexOutOfBounds(msg)) = result {
			assert!(msg.contains("does not support any standard"));
		}
		assert_eq!(manifest2.abi.clone().unwrap().methods.len(), 2);
		assert_eq!(manifest2.abi.clone().unwrap().events.len(), 0);
		let binding2 = manifest2.abi.clone().unwrap();
		let mut result = binding2.get_first_event();
		assert!(matches!(result, Err(TypeError::IndexOutOfBounds(_))));
		if let Err(TypeError::IndexOutOfBounds(msg)) = result {
			assert!(msg.contains("does not have any events"));
		}

		let c3 = native_contracts.get(2).unwrap();
		assert_eq!(c3.id, -9);
		assert_eq!(
			c3.hash(),
			&H160::from_str("0xfe924b7cfe89ddd271abaf7210a80a7e11178758").unwrap()
		);
		let nef3 = &c3.nef;
		assert_eq!(nef3.magic, 860243278);
		assert_eq!(nef3.compiler, "neo-core-v3.0".to_string());
		assert_eq!(nef3.source, "variable-size-source-oraclecontract".to_string());
		assert_eq!(nef3.tokens.len(), 0);
		assert_eq!(nef3.script, "EEEa93tnQBBBGvd7Z0AQQRr3e2dAEEEa93tnQBBBGvd7Z0A=".to_string());
		assert_eq!(nef3.checksum, 2663858513);
		let manifest3 = c3.manifest();
		assert_eq!(manifest3.name.clone().unwrap(), "OracleContract".to_string());
		assert_eq!(manifest3.groups.len(), 0);
		assert_eq!(manifest3.supported_standards.len(), 0);
		assert_eq!(manifest3.abi.clone().unwrap().methods.len(), 5);
		assert_eq!(manifest3.abi.clone().unwrap().events.len(), 2);

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

		// assert!(result.is_ok(), "Result is not okay: {:?}", result);
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
				"id": -4,
				"updatecounter": 0,
				"hash": "0xda65b600f7124ce6c79950c1772a36403104f2be",
				"nef": {
					"magic": 860243278,
					"compiler": "neo-core-v3.0",
					"source": "variable-size-source-ledgercontract",
					"tokens": [],
					"script": "EEEa93tnQBBBGvd7Z0AQQRr3e2dAEEEa93tnQBBBGvd7Z0AQQRr3e2dA",
					"checksum": 529571427
				},
				"manifest": {
					"name": "LedgerContract",
					"groups": [],
					"features": {},
					"supportedstandards": [],
					"abi": {
						"methods": [
							{
								"name": "currentHash",
								"parameters": [],
								"returntype": "Hash256",
								"offset": 0,
								"safe": true
							},
							{
								"name": "getTransactionHeight",
								"parameters": [
									{
										"name": "hash",
										"type": "Hash256"
									}
								],
								"returntype": "Integer",
								"offset": 35,
								"safe": true
							}
						],
						"events": []
					},
					"permissions": [
						{
							"contract": "*",
							"methods": "*"
						}
					],
					"trusts": [],
					"extra": null
				}
			}),
		)
		.await;
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
		let contract_state = result.unwrap();
		assert_eq!(contract_state.id, -4);
		assert_eq!(contract_state.update_counter, 0);
		assert_eq!(
			contract_state.hash,
			H160::from_str("0xda65b600f7124ce6c79950c1772a36403104f2be").unwrap()
		);
		let nef = contract_state.clone().nef;
		assert_eq!(nef.magic, 860243278);
		assert_eq!(nef.compiler, "neo-core-v3.0".to_string());
		assert_eq!(nef.source, "variable-size-source-ledgercontract".to_string());
		assert_eq!(
			nef.script,
			"EEEa93tnQBBBGvd7Z0AQQRr3e2dAEEEa93tnQBBBGvd7Z0AQQRr3e2dA".to_string()
		);
		assert_eq!(nef.tokens.len(), 0);
		assert_eq!(nef.checksum, 529571427);

		let manifest = contract_state.clone().manifest;
		assert_eq!(manifest.name.clone().unwrap(), "LedgerContract".to_string());
		assert_eq!(manifest.groups.len(), 0);
		assert_eq!(manifest.supported_standards.len(), 0);

		let abi_o = manifest.abi.clone();
		assert!(abi_o.is_some());

		let abi = abi_o.unwrap();
		assert_eq!(abi.methods.len(), 2);
		assert_eq!(abi.get_first_method().unwrap().name, "currentHash".to_string());
		assert_eq!(abi.get_method(0).unwrap().parameters.len(), 0);
		assert_eq!(abi.get_method(1).unwrap().name, "getTransactionHeight".to_string());
		assert_eq!(abi.get_method(1).unwrap().parameters.len(), 1);
		assert_eq!(abi.get_method(1).unwrap().parameters[0].name, "hash".to_string());
		assert_eq!(abi.get_method(1).unwrap().parameters[0].typ, ContractParameterType::H256);
		assert_eq!(abi.get_method(1).unwrap().return_type, ContractParameterType::Integer);
		let mut result = abi.get_method(2);
		assert!(matches!(result, Err(TypeError::IndexOutOfBounds(_))));
		if let Err(TypeError::IndexOutOfBounds(msg)) = result {
			assert!(msg.contains("only contains 2 methods"));
		}

		assert_eq!(abi.events.len(), 0);

		assert_eq!(manifest.permissions.len(), 1);
		let mut result = manifest.get_permission(1);
		assert!(matches!(result, Err(TypeError::IndexOutOfBounds(_))));
		if let Err(TypeError::IndexOutOfBounds(msg)) = result {
			assert!(msg.contains("only has permission for 1 contracts"));
		}
		assert_eq!(manifest.get_first_permission().unwrap().contract, "*".to_string());
		assert_eq!(manifest.get_permission(0).unwrap().methods.len(), 1);
		assert_eq!(manifest.get_permission(0).unwrap().methods[0], "*".to_string());

		assert_eq!(manifest.trusts.len(), 0);
		let mut result = manifest.get_first_trust();
		assert!(matches!(result, Err(TypeError::IndexOutOfBounds(_))));
		if let Err(TypeError::IndexOutOfBounds(msg)) = result {
			assert!(msg.contains("does not trust any other contracts"));
		}

		assert!(manifest.extra.is_none());
		let id = -4;
		let update_counter = 0;
		let hash = H160::from_str("0xda65b600f7124ce6c79950c1772a36403104f2be").unwrap();
		let nef2 = ContractNef::new(
			860243278,
			"neo-core-v3.0".to_string(),
			Some("variable-size-source-ledgercontract".to_string()),
			Vec::new(),
			"EEEa93tnQBBBGvd7Z0AQQRr3e2dAEEEa93tnQBBBGvd7Z0AQQRr3e2dA".to_string(),
			529571427,
		);
		let method1 = ContractMethod::new(
			"currentHash".to_string(),
			Some(Vec::new()),
			0,
			ContractParameterType::H256,
			true,
		);
		let method2 = ContractMethod::new(
			"getTransactionHeight".to_string(),
			Some(vec![ContractParameter2::new("hash".to_string(), ContractParameterType::H256)]),
			35,
			ContractParameterType::Integer,
			true,
		);
		let contract_abi = ContractABI::new(Some(vec![method1, method2]), Some(vec![]));
		let contract_permission = ContractPermission::new("*".to_string(), vec!["*".to_string()]);
		let contract_manifest = ContractManifest::new(
			Some("LedgerContract".to_string()),
			vec![],
			None,
			vec![],
			Some(contract_abi),
			vec![contract_permission],
			vec![],
			None,
		);
		let expected_equal = ContractState::new(id, update_counter, hash, nef2, contract_manifest);
		assert_eq!(contract_state, expected_equal);
		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_contract_state_missing_array_values_should_be_empty() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response(
			&mock_server,
			"getcontractstate",
			json!(["dc675afc61a7c0f7b3d2682bf6e1d8ed865a0e5f"]),
			json!({
				"nef": {
					"tokens": [],
				},
				"manifest": {
					"groups": [],
					"supportedstandards": [],
					"abi": {
						"methods": [],
						"events": []
					},
					"permissions": [],
					"trusts": [],
					"extra": null
				}
			}),
		)
		.await;
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
		let contract_state = result.unwrap();
		let nef = contract_state.clone().nef;
		assert_eq!(nef.tokens.len(), 0);

		let manifest = contract_state.clone().manifest;
		assert_eq!(manifest.groups.len(), 0);
		assert_eq!(manifest.supported_standards.len(), 0);

		let abi_o = manifest.abi.clone();
		assert!(abi_o.is_some());

		let abi = abi_o.unwrap();
		assert_eq!(abi.events.len(), 0);

		assert_eq!(manifest.permissions.len(), 0);

		assert_eq!(manifest.trusts.len(), 0);
		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_contract_state_by_name() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response(
			&mock_server,
			"getcontractstate",
			json!(["NeoToken"]),
			json!({
				"id": -4,
				"updatecounter": 0,
				"hash": "0xda65b600f7124ce6c79950c1772a36403104f2be",
				"nef": {
					"magic": 860243278,
					"compiler": "neo-core-v3.0",
					"source": "variable-size-source-ledgercontract",
					"tokens": [],
					"script": "EEEa93tnQBBBGvd7Z0AQQRr3e2dAEEEa93tnQBBBGvd7Z0AQQRr3e2dA",
					"checksum": 529571427
				},
				"manifest": {
					"name": "LedgerContract",
					"groups": [],
					"features": {},
					"supportedstandards": [],
					"abi": {
						"methods": [
							{
								"name": "currentHash",
								"parameters": [],
								"returntype": "Hash256",
								"offset": 0,
								"safe": true
							},
							{
								"name": "getTransactionHeight",
								"parameters": [
									{
										"name": "hash",
										"type": "Hash256"
									}
								],
								"returntype": "Integer",
								"offset": 35,
								"safe": true
							}
						],
						"events": []
					},
					"permissions": [
						{
							"contract": "*",
							"methods": "*"
						}
					],
					"trusts": [],
					"extra": null
				}
			}),
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
			  "height": 5492,
			  "verified": [
				"0x9786cce0dddb524c40ddbdd5e31a41ed1f6b5c8a683c122f627ca4a007a7cf4e",
				"0xb488ad25eb474f89d5ca3f985cc047ca96bc7373a6d3da8c0f192722896c1cd7"
			  ],
			  "unverified": [
				"0x9786cce0dddb524c40ddbdd5e31a41ed1f6b5c8a683c122f627ca4a007a7cf4e",
				"0xb488ad25eb474f89d5ca3f985cc047ca96bc7373a6d3da8c0f192722896c1cd7"
			  ]
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
		let mem_pool_details = result.unwrap();
		assert_eq!(mem_pool_details.height, 5492);
		assert_eq!(mem_pool_details.verified.len(), 2);
		assert_eq!(
			mem_pool_details.verified,
			vec![
				H256::from_str(
					"0x9786cce0dddb524c40ddbdd5e31a41ed1f6b5c8a683c122f627ca4a007a7cf4e"
				)
				.unwrap(),
				H256::from_str(
					"0xb488ad25eb474f89d5ca3f985cc047ca96bc7373a6d3da8c0f192722896c1cd7"
				)
				.unwrap()
			]
		);
		assert_eq!(mem_pool_details.unverified.len(), 2);
		assert_eq!(
			mem_pool_details.unverified,
			vec![
				H256::from_str(
					"0x9786cce0dddb524c40ddbdd5e31a41ed1f6b5c8a683c122f627ca4a007a7cf4e"
				)
				.unwrap(),
				H256::from_str(
					"0xb488ad25eb474f89d5ca3f985cc047ca96bc7373a6d3da8c0f192722896c1cd7"
				)
				.unwrap()
			]
		);

		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_mem_pool_empty() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response_with_id(
			&mock_server,
			"getrawmempool",
			json!([1]),
			json!({
			  "height": 5492,
			  "verified": [],
			  "unverified": []
			}),
			67,
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
		let mem_pool_details = result.unwrap();
		assert_eq!(mem_pool_details.height, 5492);
		assert_eq!(mem_pool_details.verified.len(), 0);
		assert_eq!(mem_pool_details.unverified.len(), 0);

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
		assert_eq!(
			result.unwrap(),
			vec![
				H256::from_str(
					"0x9786cce0dddb524c40ddbdd5e31a41ed1f6b5c8a683c122f627ca4a007a7cf4e"
				)
				.unwrap(),
				H256::from_str(
					"0xb488ad25eb474f89d5ca3f985cc047ca96bc7373a6d3da8c0f192722896c1cd7"
				)
				.unwrap(),
				H256::from_str(
					"0xf86f6f2c08fbf766ebe59dc84bc3b8829f1053f0a01deb26bf7960d99fa86cd6"
				)
				.unwrap()
			]
		);

		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_raw_mem_pool_empty() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response(&mock_server, "getrawmempool", json!([]), json!([])).await;

		// Expected request body
		let expected_request_body = r#"{
            "jsonrpc": "2.0",
            "method": "getrawmempool",
            "params": [],
            "id": 1
        }"#;

		let result = provider.get_raw_mem_pool().await;

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		assert_eq!(result.unwrap().len(), 0);

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
        "hash": "0x8b8b222ba4ae17eaf37d444210920690d0981b02c368f4f1973c8fd662438d89",
        "size": 267,
        "version": 0,
        "nonce": 1046354582,
        "sender": "AHE5cLhX5NjGB5R2PcdUvGudUoGUBDeHX4",
        "sysfee": "9007810",
        "netfee": "1267450",
        "validuntilblock": 2103622,
        "signers": [
            {
                "account": "0x69ecca587293047be4c59159bf8bc399985c160d",
                "scopes": "CustomContracts,CustomGroups, WitnessRules",
				"allowedcontracts": [
					"0xd2a4cff31913016155e38e474a2c06d08be276cf",
					"0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5"
				],
				"allowedgroups": [
					"033a4d051b04b7fc0230d2b1aaedfd5a84be279a5361a7358db665ad7857787f1b"
				],
				"rules": [
					{
						"action": "Allow",
						"condition": {
							"type": "ScriptHash",
							"hash": "0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5"
						}
					}
				]
            }
        ],
        "attributes": [
			{
				"type": "HighPriority"
			},
			{
				"type": "OracleResponse",
				"id": 0,
				"code": "Success",
				"result": "EQwhA/HsPB4oPogN5unEifDyfBkAfFM4WqpMDJF8MgB57a3yEQtBMHOzuw=="
			},
			{
				"type": "NotValidBefore",
				"height": "10500"
			},
			{
				"type": "Conflicts",
				"hash": "0x8529cf7301d13cc13d85913b8367700080a6e96db045687b8db720e91e80321a"
			},
			{
				"type": "Conflicts",
				"hash": "0x8529cf7301d13cc13d85913b8367700080a6e96db045687b8db720e91e80321b"
			}
		],
        "script": "AGQMFObBATZUrxE9ipaL3KUsmUioK5U9DBQP7O1Ep0MA2doEn6k2cKQxFxiP9hPADAh0cmFuc2ZlcgwUiXcg2M129PAKv6N8Dt2InCCP3ptBYn1bUjg=",
        "witnesses": [
            {
                "invocation": "DEBhsuS9LxQ2PKpx2XJJ/aGEr/pZ7qfZy77OyhDmWx+BobkQAnDPLg6ohOa9SSHa0OMDavUl7zpmJip3r8T5Dr1L",
                "verification": "EQwhA/HsPB4oPogN5unEifDyfBkAfFM4WqpMDJF8MgB57a3yEQtBMHOzuw=="
            }
        ],
        "blockhash": "0x8529cf7301d13cc13d85913b8367700080a6e96db045687b8db720e91e803299",
        "confirmations": 1388,
        "blocktime": 1589019142879i64,
		"vmstate": "HALT"
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
		let transaction = result.unwrap();
		assert_eq!(
			*transaction.hash(),
			H256::from_str("0x8b8b222ba4ae17eaf37d444210920690d0981b02c368f4f1973c8fd662438d89")
				.unwrap()
		);
		assert_eq!(*transaction.size(), 267);
		assert_eq!(*transaction.version(), 0);
		assert_eq!(*transaction.nonce(), 1046354582);
		assert_eq!(*transaction.sender(), "AHE5cLhX5NjGB5R2PcdUvGudUoGUBDeHX4".to_string());
		assert_eq!(*transaction.sys_fee(), "9007810".to_string());
		assert_eq!(*transaction.net_fee(), "1267450".to_string());
		assert_eq!(*transaction.valid_until_block(), 2103622);

		let signers = transaction.signers();
		assert_eq!(signers.len(), 1);
		let mut result = transaction.get_signer(1);
		assert!(matches!(result, Err(TypeError::IndexOutOfBounds(_))));
		if let Err(TypeError::IndexOutOfBounds(msg)) = result {
			assert!(msg.contains("only has 1 signers"));
		}

		let first_signer = transaction.get_first_signer().unwrap();
		assert_eq!(
			first_signer.account,
			H160::from_str("69ecca587293047be4c59159bf8bc399985c160d").unwrap()
		);
		assert_eq!(first_signer.get_scopes().len(), 3);
		assert_eq!(*first_signer.get_first_scope().unwrap(), WitnessScope::CustomContracts);
		assert_eq!(*first_signer.get_scope(1).unwrap(), WitnessScope::CustomGroups);
		assert_eq!(*first_signer.get_scope(2).unwrap(), WitnessScope::WitnessRules);
		let mut result = first_signer.get_scope(3);
		assert!(matches!(result, Err(TypeError::IndexOutOfBounds(_))));
		if let Err(TypeError::IndexOutOfBounds(msg)) = result {
			assert!(msg.contains("only has 3 witness scopes"));
		}
		assert_eq!(first_signer.allowed_contracts.len(), 2);
		assert_eq!(
			first_signer.get_first_allowed_contract().unwrap(),
			&H160::from_str("0xd2a4cff31913016155e38e474a2c06d08be276cf").unwrap()
		);
		assert_eq!(
			first_signer.get_allowed_contract(1).unwrap(),
			&H160::from_str("0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5").unwrap()
		);
		let mut result = first_signer.get_allowed_contract(2);
		assert!(matches!(result, Err(TypeError::IndexOutOfBounds(_))));
		if let Err(TypeError::IndexOutOfBounds(msg)) = result {
			assert!(msg.contains("only allows 2 contracts"));
		}
		assert_eq!(
			first_signer.allowed_groups[0],
			"033a4d051b04b7fc0230d2b1aaedfd5a84be279a5361a7358db665ad7857787f1b".to_string()
		);
		let mut result = first_signer.get_allowed_group(1);
		assert!(matches!(result, Err(TypeError::IndexOutOfBounds(_))));
		if let Err(TypeError::IndexOutOfBounds(msg)) = result {
			assert!(msg.contains("only allows 1 groups"));
		}
		assert_eq!(
			first_signer.get_first_allowed_group().unwrap(),
			&"033a4d051b04b7fc0230d2b1aaedfd5a84be279a5361a7358db665ad7857787f1b".to_string()
		);

		let rule = first_signer.get_rules().get(0).unwrap();
		let first_rule = first_signer.get_first_rule().unwrap();
		let mut result = first_signer.get_rule(1);
		assert!(matches!(result, Err(TypeError::IndexOutOfBounds(_))));
		if let Err(TypeError::IndexOutOfBounds(msg)) = result {
			assert!(msg.contains("only has 1 witness rules"));
		}
		assert_eq!(rule, first_rule);
		assert_eq!(rule.action, WitnessAction::Allow);
		assert_eq!(
			rule.condition,
			WitnessCondition::ScriptHash(
				H160::from_str("0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5").unwrap()
			)
		);

		let attributes = transaction.attributes();
		assert_eq!(attributes.len(), 5);
		let mut result = transaction.get_attribute(5);
		assert!(matches!(result, Err(TypeError::IndexOutOfBounds(_))));
		if let Err(TypeError::IndexOutOfBounds(msg)) = result {
			assert!(msg.contains("only has 5 attributes"));
		}

		assert_eq!(
			transaction.get_first_attribute().unwrap(),
			&TransactionAttributeEnum::HighPriority(HighPriorityAttribute {})
		);

		assert_eq!(
			attributes.get(1).unwrap(),
			&TransactionAttributeEnum::OracleResponse(OracleResponseAttribute {
				oracle_response: OracleResponse {
					id: 0,
					response_code: OracleResponseCode::Success,
					result: "EQwhA/HsPB4oPogN5unEifDyfBkAfFM4WqpMDJF8MgB57a3yEQtBMHOzuw=="
						.to_string()
				}
			})
		);

		assert_eq!(
			attributes.get(2).unwrap(),
			&TransactionAttributeEnum::NotValidBefore(NotValidBeforeAttribute { height: 10500 })
		);

		let conflict_hash1 =
			H256::from_str("0x8529cf7301d13cc13d85913b8367700080a6e96db045687b8db720e91e80321a")
				.unwrap();
		assert_eq!(
			attributes.get(3).unwrap(),
			&TransactionAttributeEnum::Conflicts(ConflictsAttribute { hash: conflict_hash1 })
		);

		let conflict_hash2 =
			H256::from_str("0x8529cf7301d13cc13d85913b8367700080a6e96db045687b8db720e91e80321b")
				.unwrap();
		assert_eq!(
			attributes.get(4).unwrap(),
			&TransactionAttributeEnum::Conflicts(ConflictsAttribute { hash: conflict_hash2 })
		);

		assert_eq!(transaction.script(), &"AGQMFObBATZUrxE9ipaL3KUsmUioK5U9DBQP7O1Ep0MA2doEn6k2cKQxFxiP9hPADAh0cmFuc2ZlcgwUiXcg2M129PAKv6N8Dt2InCCP3ptBYn1bUjg=".to_string());
		assert_eq!(transaction.witnesses().len(), 1);
		assert_eq!(transaction.witnesses(), &vec![
			NeoWitness::new("DEBhsuS9LxQ2PKpx2XJJ/aGEr/pZ7qfZy77OyhDmWx+BobkQAnDPLg6ohOa9SSHa0OMDavUl7zpmJip3r8T5Dr1L".to_string(),"EQwhA/HsPB4oPogN5unEifDyfBkAfFM4WqpMDJF8MgB57a3yEQtBMHOzuw==".to_string())
		]);
		assert_eq!(
			transaction.block_hash(),
			&H256::from_str("0x8529cf7301d13cc13d85913b8367700080a6e96db045687b8db720e91e803299")
				.unwrap()
		);
		assert_eq!(transaction.confirmations(), &1388);
		assert_eq!(transaction.block_time(), &1589019142879);
		assert_eq!(transaction.vmstate(), &VMState::Halt);

		verify_request(&mock_server, expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_raw_transaction() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response(
            &mock_server,
            "getrawtransaction",
            json!(["7da6ae7ff9d0b7af3d32f3a2feb2aa96c2a27ef8b651f9a132cfaad6ef20724c",0]),
            json!("00961a5e3e0feced44a74300d9da049fa93670a43117188ff6c272890000000000fa561300000000004619200000010feced44a74300d9da049fa93670a43117188ff6015600640c14e6c1013654af113d8a968bdca52c9948a82b953d0c140feced44a74300d9da049fa93670a43117188ff613c00c087472616e736665720c14897720d8cd76f4f00abfa37c0edd889c208fde9b41627d5b523801420c4061b2e4bd2f14363caa71d97249fda184affa59eea7d9cbbececa10e65b1f81a1b9100270cf2e0ea884e6bd4921dad0e3036af525ef3a66262a77afc4f90ebd4b2b110c2103f1ec3c1e283e880de6e9c489f0f27c19007c53385aaa4c0c917c320079edadf2110b413073b3bb"),
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
		assert_eq!(result.unwrap(), "00961a5e3e0feced44a74300d9da049fa93670a43117188ff6c272890000000000fa561300000000004619200000010feced44a74300d9da049fa93670a43117188ff6015600640c14e6c1013654af113d8a968bdca52c9948a82b953d0c140feced44a74300d9da049fa93670a43117188ff613c00c087472616e736665720c14897720d8cd76f4f00abfa37c0edd889c208fde9b41627d5b523801420c4061b2e4bd2f14363caa71d97249fda184affa59eea7d9cbbececa10e65b1f81a1b9100270cf2e0ea884e6bd4921dad0e3036af525ef3a66262a77afc4f90ebd4b2b110c2103f1ec3c1e283e880de6e9c489f0f27c19007c53385aaa4c0c917c320079edadf2110b413073b3bb".to_string());
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
			json!("4c696e"),
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
		assert_eq!(result.unwrap(), "4c696e".to_string());
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
			json!(1223),
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
		assert_eq!(result.unwrap(), 1223);
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
				"publickey": "03f1ec3c1e283e880de6e9c489f0f27c19007c53385aaa4c0c917c320079edadf2",
				"votes": "0",
				"active": false
			  },
			  {
				"publickey": "02494f3ff953e45ca4254375187004f17293f90a1aa4b1a89bc07065bc1da521f6",
				"votes": "91600000",
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
		let next_validators = result.unwrap();
		assert_eq!(next_validators.len(), 2);
		assert_eq!(
			next_validators,
			vec![
				Validator::new(
					"03f1ec3c1e283e880de6e9c489f0f27c19007c53385aaa4c0c917c320079edadf2"
						.to_string(),
					"0".to_string(),
					false
				),
				Validator::new(
					"02494f3ff953e45ca4254375187004f17293f90a1aa4b1a89bc07065bc1da521f6"
						.to_string(),
					"91600000".to_string(),
					true
				)
			]
		);
		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_next_block_validators_empty() {
		let mock_server = setup_mock_server().await;
		let provider =
			mock_rpc_response(&mock_server, "getnextblockvalidators", json!([]), json!([])).await;
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
		let next_validators = result.unwrap();
		assert_eq!(next_validators.len(), 0);
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
			mock_rpc_response(&mock_server, "getconnectioncount", json!([]), json!(2)).await;

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
		assert_eq!(result.unwrap(), 2);
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
				"unconnected": [
					{
						"address": "127.0.0.1",
						"port": 20335
					},
					{
						"address": "127.0.0.1",
						"port": 20336
					},
					{
						"address": "127.0.0.1",
						"port": 20337
					}
				],
				"bad": [
					{
						"address": "127.0.0.1",
						"port": 20333
					}
				],
				"connected": [
					{
						"address": "172.18.0.3",
						"port": 40333
					},
					{
						"address": "172.18.0.4",
						"port": 20333
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
		let peers = result.unwrap();
		assert_eq!(peers.unconnected.len(), 3);
		assert_eq!(
			peers.unconnected,
			vec![
				AddressEntry::new("127.0.0.1".to_string(), 20335),
				AddressEntry::new("127.0.0.1".to_string(), 20336),
				AddressEntry::new("127.0.0.1".to_string(), 20337)
			]
		);
		assert_eq!(peers.bad.len(), 1);
		assert_eq!(peers.bad, vec![AddressEntry::new("127.0.0.1".to_string(), 20333)]);
		assert_eq!(
			peers.connected,
			vec![
				AddressEntry::new("172.18.0.3".to_string(), 40333),
				AddressEntry::new("172.18.0.4".to_string(), 20333)
			]
		);

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_peers_empty() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response(
			&mock_server,
			"getpeers",
			json!([]),
			json!({
				"unconnected": [],
				"bad": [],
				"connected": []
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
		let peers = result.unwrap();
		assert_eq!(peers.unconnected.len(), 0);
		assert_eq!(peers.bad.len(), 0);
		assert_eq!(peers.connected.len(), 0);

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
				"tcpport": 40333,
				"wsport": 40334,
				"nonce": 224036820,
				"useragent": "/Neo:3.0.0/",
				"protocol": {
					"network": 769,
					"validatorscount": 7,
					"msperblock": 15000,
					"maxvaliduntilblockincrement": 1,
					"maxtraceableblocks": 3,
					"addressversion": 22,
					"maxtransactionsperblock": 150000,
					"memorypoolmaxtransactions": 34000,
					"initialgasdistribution": 14,
					"hardforks": [
						{
							"name": "HF_Aspidochelone",
							"blockheight": 0
						}
					]
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
		let version = result.unwrap();
		assert_eq!(version.tcp_port.unwrap(), 40333);
		assert_eq!(version.ws_port.unwrap(), 40334);
		assert_eq!(version.nonce, 224036820);
		assert_eq!(version.user_agent, "/Neo:3.0.0/".to_string());

		let protocol = version.protocol.unwrap();
		assert_eq!(protocol.address_version, 22);
		assert_eq!(protocol.network, 769);
		assert_eq!(protocol.ms_per_block, 15000);
		assert_eq!(protocol.max_traceable_blocks, 3);
		assert_eq!(protocol.max_valid_until_block_increment, 1);
		assert_eq!(protocol.max_transactions_per_block, 150000);
		assert_eq!(protocol.initial_gas_distribution, 14);
		assert_eq!(protocol.hard_forks.len(), 1);

		let hard_forks = protocol.hard_forks[0].clone();
		assert_eq!(hard_forks.name, "HF_Aspidochelone".to_string());
		assert_eq!(hard_forks.block_height, 0);
		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_version_network_long() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response(
			&mock_server,
			"getversion",
			json!([]),
			json!( {
				"tcpport": 40333,
				"wsport": 40334,
				"nonce": 224036820,
				"useragent": "/Neo:3.0.0/",
				"protocol": {
					"network": 4232068425u32,
					"validatorscount": 3,
					"msperblock": 15000,
					"maxvaliduntilblockincrement": 1,
					"maxtraceableblocks": 3,
					"addressversion": 22,
					"maxtransactionsperblock": 150000,
					"memorypoolmaxtransactions": 34000,
					"initialgasdistribution": 14,
					"hardforks": []
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
		let version = result.unwrap();
		assert_eq!(version.tcp_port.unwrap(), 40333);
		assert_eq!(version.ws_port.unwrap(), 40334);
		assert_eq!(version.nonce, 224036820);
		assert_eq!(version.user_agent, "/Neo:3.0.0/".to_string());

		let protocol = version.protocol.unwrap();
		assert_eq!(protocol.address_version, 22);
		assert_eq!(protocol.network, 4232068425);
		assert_eq!(protocol.validators_count, Some(3));
		assert_eq!(protocol.ms_per_block, 15000);
		assert_eq!(protocol.max_traceable_blocks, 3);
		assert_eq!(protocol.max_valid_until_block_increment, 1);
		assert_eq!(protocol.max_transactions_per_block, 150000);
		assert_eq!(protocol.initial_gas_distribution, 14);
		assert_eq!(protocol.hard_forks.len(), 0);
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
				"hash": "0xb0748d216c9c0d0498094cdb50407035917b350fc0338c254b78f944f723b770"
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

		let result = provider.send_raw_transaction(tx_hex).await;
		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		assert_eq!(
			result.unwrap().hash,
			H256::from_str("0xb0748d216c9c0d0498094cdb50407035917b350fc0338c254b78f944f723b770")
				.unwrap()
		);
		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_submit_block() {
		let mock_server = setup_mock_server().await;
		// let block_base64 = "AAAAACMSKFbGpGl6t7uroMpi2ilhQd84eU/pUrRfQyswXYl76woLOY0oW1z4InfxoKyxFAAB+8FS6cRu2Pm0iaOiD8OMCnLadQEAAMgcAAD6lrDvowCyjK9dBALCmE1fvMuahQFCDEAd8EoEFBcxOLCZfh8w0tUEHHmyn++KzW4I8oeJ1WyMmjHVcolpNzOnAOzXTn/xujwy93gJ9ijvVo6wAF5qC3wCKxEMIQL4L//X3jDpIyMLze0sPNW+yFcufrrL3bnzOipdJpNLixELQRON768CAGUTt7+NSxXGAA7aoUS2kokAAAAAACYcEwAAAAAARzMAAAHNWK7P0zW+HrPTEeHcgAlj39ctnwEAXQMA5AtUAgAAAAwUzViuz9M1vh6z0xHh3IAJY9/XLZ8MFM1Yrs/TNb4es9MR4dyACWPf1y2fE8AMCHRyYW5zZmVyDBS8r0HWhMfUrW7g2Z2pcHudHwyOZkFifVtSOAFCDEADRhUarLK+/BBjhqaWY5ieento21zgkcsUMWNCBWGd+v8a35zatNRgFbUkni4dDNI/BGc3zOgPT6EwroUsgvR+KQwhAv3yei642bBp1hrlpk26E7iWN8VC2MdMXWurST/mONaPC0GVRA14".to_string();
		// let block_hex = block_base64.from_base64().unwrap().to_hex();
		let block_hex = "00000000000000000000000000000000".to_string();

		let provider =
			mock_rpc_response(&mock_server, "submitblock", json!([block_hex]), json!(true)).await;
		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "submitblock",
			"params": ["{}"],
			"id": 1
		}}"#,
			block_hex
		);

		let result = provider.submit_block(block_hex).await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		assert_eq!(result.unwrap().get_submit_block(), true);
	}

	// SmartContract Methods

	#[tokio::test]
	async fn test_invoke_function() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response_without_request(
			&mock_server,
			json!({
				"script": "wh8MFnRva2Vuc1dpdGhXaXRuZXNzQ2hlY2sMFFdiWCF05OK8ywVb+rl30RPV3+zlQWJ9W1I=",
				"state": "HALT",
				"gasconsumed": "12908980",
				"exception": null,
				"notifications": [
					{
						"eventname": "Mint",
						"contract": "0xe5ecdfd513d177b9fa5b05cbbce2e47421586257",
						"state": {
							"type": "Array",
							"value": [
								{
									"type": "Integer",
									"value": "1"
								},
								{
									"type": "ByteString",
									"value": "dG9rZW4x"
								},
								{
									"type": "Integer",
									"value": "10000"
								}
							]
						}
					},
					{
						"eventname": "StorageUpdate",
						"contract": "0xe5ecdfd513d177b9fa5b05cbbce2e47421586257",
						"state": {
							"type": "Array",
							"value": [
								{
									"type": "ByteString",
									"value": "dG9rZW4x"
								},
								{
									"type": "ByteString",
									"value": "Y3JlYXRl"
								}
							]
						}
					}
				],
				"stack": [
					{
						"type": "InteropInterface",
						"interface": "IIterator",
						"id": "fcf7b800-192a-488f-95d3-c40ac7b30ef1"
					}
				],
				"session": "6ecb0e24-ce7f-4550-9838-aeb8c9e08570"
			}),
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

		let mut signer = AccountSignerType::called_by_entry_hash160(
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
				Some(vec![AccountSigner(signer)]),
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		let invocation_result = result.unwrap();
		assert_eq!(
			invocation_result.script,
			"wh8MFnRva2Vuc1dpdGhXaXRuZXNzQ2hlY2sMFFdiWCF05OK8ywVb+rl30RPV3+zlQWJ9W1I=".to_string()
		);
		assert_eq!(invocation_result.state, NeoVMStateType::Halt);
		assert_eq!(invocation_result.gas_consumed, "12908980".to_string());
		assert!(invocation_result.exception.is_none());

		let notifications = invocation_result.notifications.clone().unwrap();
		assert_eq!(notifications.len(), 2);

		let mut result = invocation_result.get_notification(2);
		assert!(matches!(result, Err(TypeError::IndexOutOfBounds(_))));
		if let Err(TypeError::IndexOutOfBounds(msg)) = result {
			assert!(msg.contains("Only 2 notifications have been sent in this invocation"));
		}
		assert_eq!(
			invocation_result.get_first_notification(),
			invocation_result.get_notification(0)
		);
		assert_eq!(
			invocation_result.get_notification(0).unwrap().contract,
			H160::from_str("0xe5ecdfd513d177b9fa5b05cbbce2e47421586257").unwrap()
		);
		assert_eq!(invocation_result.get_notification(0).unwrap().event_name, "Mint".to_string());
		assert!(
			matches!(notifications.get(0).unwrap().state, StackItem::Array { .. }),
			"The stack item type is not Array as expected"
		);

		let stack_item_list = notifications.get(0).unwrap().state.as_array().unwrap();
		let item0 = stack_item_list[0].as_int();
		assert_eq!(item0, Some(1));
		assert_eq!(stack_item_list[1].as_string().unwrap(), "token1".to_string());
		assert_eq!(
			notifications.get(1).unwrap().contract,
			H160::from_str("0xe5ecdfd513d177b9fa5b05cbbce2e47421586257").unwrap()
		);
		assert_eq!(notifications.get(1).unwrap().event_name, "StorageUpdate".to_string());
		assert!(
			matches!(notifications.get(1).unwrap().state, StackItem::Array { .. }),
			"The stack item type is not Array as expected"
		);
		let stack_item_list1 = notifications.get(1).unwrap().state.as_array().unwrap();
		assert_eq!(stack_item_list1[1].as_string().unwrap(), "create".to_string());

		assert_eq!(invocation_result.stack.len(), 1);
		assert_eq!(invocation_result.get_stack_item(0), invocation_result.get_first_stack_item());
		assert_eq!(
			invocation_result.stack.get(0).unwrap().get_iterator_id().unwrap(),
			&"fcf7b800-192a-488f-95d3-c40ac7b30ef1".to_string()
		);
		assert_eq!(
			invocation_result.stack.get(0).unwrap().get_interface_name().unwrap(),
			&"IIterator".to_string()
		);
		let mut result = invocation_result.get_stack_item(1);
		assert!(matches!(result, Err(TypeError::IndexOutOfBounds(_))));
		if let Err(TypeError::IndexOutOfBounds(msg)) = result {
			assert!(msg.contains("only 1 items left on the NeoVM stack after"));
		}
		assert_eq!(
			invocation_result.session_id,
			Some("6ecb0e24-ce7f-4550-9838-aeb8c9e08570".to_string())
		);
	}

	#[tokio::test]
	async fn test_stack_item_invoke_function() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response_without_request(
            &mock_server,
            json!({
        "script": "0c14e6c1013654af113d8a968bdca52c9948a82b953d11c00c0962616c616e63654f660c14897720d8cd76f4f00abfa37c0edd889c208fde9b41627d5b52",
        "state": "HALT",
        "gasconsumed": "2007570",
        "exception": null,
        "notifications": [],
        "stack": [
            {
                "type": "Buffer",
                "value": "dHJhbnNmZXI="
            },
			{
                "type": "Buffer",
                "value": "lBNDI5IT+g52XxAnznQvSNt3mpY="
            },
			{
                "type": "Buffer",
                "value": "wWq="
            },
			{
                "type": "Pointer",
                "value": "123"
            },
			{
                "type": "Map",
                "value": [
					{
						"key": {
							"type": "ByteString",
							"value": "lBNDI5IT+g52XxAnznQvSNt3mpY="
						},
						"value": {
							"type": "Pointer",
							"value": 12
						}
					}
				]
            },
        ],
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

		let mut signer = AccountSignerType::called_by_entry_hash160(
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
				Some(vec![AccountSigner(signer)]),
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();

		// Response verification
		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		let invocation_result = result.unwrap();
		assert_eq!(invocation_result.script, "0c14e6c1013654af113d8a968bdca52c9948a82b953d11c00c0962616c616e63654f660c14897720d8cd76f4f00abfa37c0edd889c208fde9b41627d5b52".to_string());
		assert_eq!(invocation_result.state, NeoVMStateType::Halt);
		assert_eq!(invocation_result.gas_consumed, "2007570".to_string());
		assert!(invocation_result.exception.is_none());

		let notifications = invocation_result.notifications.clone().unwrap();
		assert_eq!(notifications.len(), 0);

		let mut result = invocation_result.get_first_notification();
		assert!(matches!(result, Err(TypeError::IndexOutOfBounds(_))));
		if let Err(TypeError::IndexOutOfBounds(msg)) = result {
			assert!(msg.contains("No notifications have been sent in this invocation"));
		}

		assert_eq!(invocation_result.stack.len(), 5);
		let item0 = invocation_result.stack.get(0).unwrap();
		assert!(
			matches!(item0, StackItem::Buffer { .. }),
			"The stack item type is not Buffer as expected"
		);
		assert_eq!(item0.as_string().unwrap(), "transfer".to_string());

		let item1 = invocation_result.stack.get(1).unwrap();
		assert!(
			matches!(item1, StackItem::Buffer { .. }),
			"The stack item type is not Buffer as expected"
		);
		assert_eq!(item1.as_address().unwrap(), "NZQvGWfSupuUAYtCH6pje72hdkWJH1jAZP".to_string());

		// TODO: Base64 decode failed on "wWo=", which is a valid string
		// let try_base =general_purpose::STANDARD.decode("wWo=".trim_end()).expect(&"Failed to decode the string: wWq=".to_string());
		// let item2 = invocation_result.stack.get(2).unwrap();
		// assert!(
		// 	matches!(item2, StackItem::Buffer { .. }),
		// 	"The stack item type is not Buffer as expected"
		// );
		// assert_eq!(item2.as_bytes().unwrap(), hex::decode("c16a").unwrap());
		// assert_eq!(item2.as_int().unwrap(), 27329);

		let item3 = invocation_result.stack.get(3).unwrap();
		assert!(
			matches!(item3, StackItem::Pointer { .. }),
			"The stack item type is not Pointer as expected"
		);
		assert_eq!(item3.as_int().unwrap(), 123);

		let item4 = invocation_result.stack.get(4).unwrap();
		assert!(
			matches!(item4, StackItem::Map { .. }),
			"The stack item type is not Pointer as expected"
		);
		let stack_item_map = item4.as_map().unwrap();
		assert_eq!(stack_item_map.len(), 1);
		let value = stack_item_map
			.get(&StackItem::new_byte_string(
				hex::decode("941343239213fa0e765f1027ce742f48db779a96").unwrap(),
			))
			.unwrap()
			.as_int();
		assert_eq!(value, Some(12));
	}

	#[tokio::test]
	async fn test_invoke_function_empty_stack() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response_without_request(
            &mock_server,
            json!({
        "script": "0c14e6c1013654af113d8a968bdca52c9948a82b953d11c00c0962616c616e63654f660c14897720d8cd76f4f00abfa37c0edd889c208fde9b41627d5b52",
        "state": "HALT",
        "gasconsumed": "2007570",
        "exception": null,
        "notifications": [],
        "stack": [],
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

		let mut signer = AccountSignerType::called_by_entry_hash160(
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
				Some(vec![AccountSigner(signer)]),
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();

		// Response verification
		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		let invocation_result = result.unwrap();
		assert_eq!(invocation_result.stack.len(), 0);
		let mut result = invocation_result.get_first_stack_item();
		assert!(matches!(result, Err(TypeError::IndexOutOfBounds(_))));
		if let Err(TypeError::IndexOutOfBounds(msg)) = result {
			assert!(msg.contains("no items were left on the NeoVM stack after this invocation"));
		}
	}

	#[tokio::test]
	async fn test_invoke_function_pending_signatures() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response_without_request(
            &mock_server,
            json!({
        "script": "00046e616d65675f0e5a86edd8e1f62b68d2b3f7c0a761fc5a67dc",
        "state": "HALT",
        "gasconsumed": "2.489",
        "stack": [],
		"pendingsignature": {
			"type": "Transaction",
			"data": "base64 string of the tx bytes",
			"network": 305419896,
			"items": {
				"0x69ecca587293047be4c59159bf8bc399985c160d": {
					"script": "base64 script",
					"parameters": [
						{
							"type": "Signature",
							"value": ""
						}
					],
					"signatures": {
						"<033a4d051b04b7fc0230d2b1aaedfd5a84be279a5361a7358db665ad7857787f1b>": "base64 string of signature"
					}
				},
				"0x05859de95ccbbd5668e0f055b208273634d4657f": {
					"script": "base64 script",
					"parameters": [
						{
							"type": "Signature",
						},
						{
							"type": "Signature",
						}
					],
					"signatures": {
						"033a1d0a3b04b7fc0230d2b1aaedfd5a84be279a5361a7358db665ad7957783f81": "base64 string of signature",
						"033a4c051b09b77c0230d2b1aaedfd5a84be279a5361a7358db665ad7d57787f10": "base64 string of signature"
					}
				}
			}
		}
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

		let mut signer = AccountSignerType::called_by_entry_hash160(
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
				Some(vec![AccountSigner(signer)]),
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();

		// Response verification
		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		let invocation_result = result.unwrap();
		assert_eq!(
			invocation_result.script,
			"00046e616d65675f0e5a86edd8e1f62b68d2b3f7c0a761fc5a67dc".to_string()
		);
		assert_eq!(invocation_result.state, NeoVMStateType::Halt);
		assert_eq!(invocation_result.gas_consumed, "2.489".to_string());
		assert_eq!(invocation_result.stack.len(), 0);

		let pending_sig = invocation_result.pending_signature.unwrap();
		assert_eq!(pending_sig.typ, "Transaction".to_string());
		assert_eq!(pending_sig.data, "base64 string of the tx bytes".to_string());
		assert_eq!(pending_sig.network, 305419896);
		let items = pending_sig.items;
		let item = items.get(&"0x05859de95ccbbd5668e0f055b208273634d4657f".to_string()).unwrap();
		assert_eq!(item.script, "base64 script".to_string());
		assert_eq!(item.parameters[1].typ(), &ContractParameterType::Signature);
		assert_eq!(
			item.signatures.get(
				&"033a1d0a3b04b7fc0230d2b1aaedfd5a84be279a5361a7358db665ad7957783f81".to_string()
			),
			Some(&"base64 string of signature".to_string())
		);
	}

	#[tokio::test]
	async fn test_invoke_function_without_or_empty_params() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response_without_request(
			&mock_server,
			json!({
			"script": "10c00c0962616c616e63654f660c14897720d8cd76f4f00abfa37c0edd889c208fde9b41627d5b52",
			"state": "FAULT",
			"gasconsumed": "2007390",
			"exception": null,
			"notifications": [],
			"stack": []
			}),
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

		let mut signer = AccountSignerType::called_by_entry_hash160(
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
				Some(vec![AccountSigner(signer)]),
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();

		// Response verification
		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		let invocation_result = result.unwrap();
		assert_eq!(
			invocation_result.script,
			"10c00c0962616c616e63654f660c14897720d8cd76f4f00abfa37c0edd889c208fde9b41627d5b52"
				.to_string()
		);
		assert_eq!(invocation_result.state, NeoVMStateType::Fault);
		assert_eq!(invocation_result.gas_consumed, "2007390".to_string());
		assert_eq!(invocation_result.stack.len(), 0);
		let expected_result = InvocationResult::new(
			"10c00c0962616c616e63654f660c14897720d8cd76f4f00abfa37c0edd889c208fde9b41627d5b52"
				.to_string(),
			NeoVMStateType::Fault,
			"2007390".to_string(),
			None,
			Some(vec![]),
			None,
			vec![],
			None,
			None,
			None,
		);
	}

	#[tokio::test]
	async fn test_invoke_function_empty_state() {
		let mock_server = setup_mock_server().await;
		let provider = mock_rpc_response_without_request(
			&mock_server,
			json!({
			"script": "10c00c0962616c616e63654f660c14897720d8cd76f4f00abfa37c0edd889c208fde9b41627d5b52",
			"state": "",
			"gasconsumed": "2007390",
			"stack": []
			}),
		)
		.await;

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

		let mut signer = AccountSignerType::called_by_entry_hash160(
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
				Some(vec![AccountSigner(signer)]),
			)
			.await;

		// Response verification
		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		let invocation_result = result.unwrap();
		assert_eq!(
			invocation_result.script,
			"10c00c0962616c616e63654f660c14897720d8cd76f4f00abfa37c0edd889c208fde9b41627d5b52"
				.to_string()
		);
		assert_eq!(invocation_result.state, NeoVMStateType::None);
		assert_eq!(invocation_result.gas_consumed, "2007390".to_string());
		assert_eq!(invocation_result.stack.len(), 0);
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

		let mut signer = AccountSignerType::called_by_entry_hash160(
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
				Some(vec![AccountSigner(signer)]),
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
			json!({
				"script": "wh8MC2NhbGxTeW1ib2xzDBQ35AiF8REp1Iy5N6DbcAjECghSDkFifVtS",
				"state": "HALT",
				"gasconsumed": "4845600",
				"exception": null,
				"notifications": [],
				"diagnostics": {
					"invokedcontracts": {
						"hash": "0x7df45ba2d3a0c0520ceef7a73f8d1c404cc59a48",
						"call": [
							{
								"hash": "0x0e52080ac40870dba037b98cd42911f18508e437",
								"call": [
									{
										"hash": "0x0e52080ac40870dba037b98cd42911f18508e437"
									},
									{
										"hash": "0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5"
									},
									{
										"hash": "0xd2a4cff31913016155e38e474a2c06d08be276cf"
									}
								]
							}
						]
					},
					"storagechanges": [
						{
							"state": "Deleted",
							"key": "BgAAAP8=",
							"value": "DRZcmJnDi79ZkcXkewSTcljK7Gk="
						},
						{
							"state": "Changed",
							"key": "+v///xQNFlyYmcOLv1mRxeR7BJNyWMrsaQ==",
							"value": "QQEhBQAb1mAS"
						},
						{
							"state": "Added",
							"key": "+v///xRjv+9gkFzYfFbaQGRkS+b3ro7EiA==",
							"value": "QQEhAQo="
						}
					]
				},
				"stack": [
					{
						"type": "Array",
						"value": [
							{
								"type": "ByteString",
								"value": "TkVP"
							},
							{
								"type": "ByteString",
								"value": "R0FT"
							},
							{
								"type": "ByteString",
								"value": "TkVP"
							}
						]
					}
				]
			}),
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
		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		let invocation_result = result.unwrap();
		let dignostics = invocation_result.diagnostics.unwrap();

		let invoked_contracts = dignostics.clone().invoked_contracts;
		assert_eq!(
			invoked_contracts.hash,
			H160::from_str("0x7df45ba2d3a0c0520ceef7a73f8d1c404cc59a48").unwrap()
		);

		let calls = invoked_contracts.invoked_contracts;
		assert_eq!(calls.len(), 1);
		assert_eq!(
			calls.get(0).unwrap().hash,
			H160::from_str("0x0e52080ac40870dba037b98cd42911f18508e437").unwrap()
		);

		let nested_invoked_contrats = &calls.get(0).unwrap().invoked_contracts;
		assert_eq!(nested_invoked_contrats.len(), 3);
		assert_eq!(
			nested_invoked_contrats.get(0).unwrap().hash,
			H160::from_str("0x0e52080ac40870dba037b98cd42911f18508e437").unwrap()
		);
		assert_eq!(nested_invoked_contrats.get(0).unwrap().invoked_contracts.len(), 0);
		assert_eq!(
			nested_invoked_contrats.get(1).unwrap().hash,
			H160::from_str("0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5").unwrap()
		);
		assert_eq!(nested_invoked_contrats.get(1).unwrap().invoked_contracts.len(), 0);
		assert_eq!(
			nested_invoked_contrats.get(2).unwrap().hash,
			H160::from_str("0xd2a4cff31913016155e38e474a2c06d08be276cf").unwrap()
		);
		assert_eq!(nested_invoked_contrats.get(2).unwrap().invoked_contracts.len(), 0);

		let storage_changes = dignostics.clone().storage_changes;
		assert_eq!(storage_changes.len(), 3);
		let storage_change1 = storage_changes.get(0).unwrap();
		let expected_storage_change1 = StorageChange::new(
			"Deleted".to_string(),
			"BgAAAP8=".to_string(),
			"DRZcmJnDi79ZkcXkewSTcljK7Gk=".to_string(),
		);
		assert_eq!(storage_change1, &expected_storage_change1);
		let storage_change2 = storage_changes.get(1).unwrap();
		let expected_storage_change2 = StorageChange::new(
			"Changed".to_string(),
			"+v///xQNFlyYmcOLv1mRxeR7BJNyWMrsaQ==".to_string(),
			"QQEhBQAb1mAS".to_string(),
		);
		assert_eq!(storage_change2, &expected_storage_change2);
		let storage_change3 = storage_changes.get(2).unwrap();
		let expected_storage_change3 = StorageChange::new(
			"Added".to_string(),
			"+v///xRjv+9gkFzYfFbaQGRkS+b3ro7EiA==".to_string(),
			"QQEhAQo=".to_string(),
		);
		assert_eq!(storage_change3, &expected_storage_change3);

		let called_contract_call = InvokedContract::new_hash(
			H160::from_str("0x0e52080ac40870dba037b98cd42911f18508e437").unwrap(),
		);
		let neo_token_call = InvokedContract::new_hash(
			H160::from_str("0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5").unwrap(),
		);
		let gas_token_call = InvokedContract::new_hash(
			H160::from_str("0xd2a4cff31913016155e38e474a2c06d08be276cf").unwrap(),
		);
		let call2 = vec![called_contract_call, neo_token_call, gas_token_call];
		let call1 = InvokedContract::new(
			H160::from_str("0x0e52080ac40870dba037b98cd42911f18508e437").unwrap(),
			call2,
		);
		let expected_invoked_contract = InvokedContract::new(
			H160::from_str("0x7df45ba2d3a0c0520ceef7a73f8d1c404cc59a48").unwrap(),
			vec![call1],
		);
		let expected_diagnostics = Diagnostics::new(
			expected_invoked_contract,
			vec![expected_storage_change1, expected_storage_change2, expected_storage_change3],
		);
		assert_eq!(dignostics, expected_diagnostics);
	}

	#[tokio::test]
	async fn test_invoke_function_diagnostics_no_params() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url).unwrap();
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
		let http_client = HttpProvider::new(url).unwrap();
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

		let provider = mock_rpc_response(
			&mock_server,
			"invokescript",
			json!(["EMAMCGRlY2ltYWxzDBQlBZ7LSHjTqHX5HFHO3tMw1Fdf3kFifVtS", []]),
			json!({
				"script": "10c00c08646563696d616c730c1425059ecb4878d3a875f91c51ceded330d4575fde41627d5b52",
				"state": "HALT",
				"gasconsumed": "0.161",
				"stack": [
					{
						"type": "ByteString",
						"value": "VHJhbnNmZXI="
					}
				]
			}),
		)
		.await;

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

		let result = provider
			.invoke_script(
				"10c00c08646563696d616c730c1425059ecb4878d3a875f91c51ceded330d4575fde41627d5b52"
					.to_string(),
				vec![],
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		let invocation_result = result.unwrap();
		assert_eq!(
			invocation_result.script,
			"10c00c08646563696d616c730c1425059ecb4878d3a875f91c51ceded330d4575fde41627d5b52"
				.to_string()
		);
		assert_eq!(invocation_result.state, NeoVMStateType::Halt);
		assert_eq!(invocation_result.gas_consumed, "0.161".to_string());
		assert_eq!(invocation_result.stack.len(), 1);
		assert_eq!(
			invocation_result.stack,
			vec![StackItem::new_byte_string("Transfer".as_bytes().to_vec())]
		);
	}

	#[tokio::test]
	async fn test_invoke_script_diagnostics() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url).unwrap();
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
		let http_client = HttpProvider::new(url).unwrap();
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

		let signer = AccountSignerType::called_by_entry_hash160(
			H160::from_str("0xcc45cc8987b0e35371f5685431e3c8eeea306722").unwrap(),
		)
		.unwrap();

		let _ = provider
			.invoke_script(
				"10c00c08646563696d616c730c1425059ecb4878d3a875f91c51ceded330d4575fde41627d5b52"
					.to_string(),
				vec![AccountSigner(signer)],
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_invoke_script_diagnostics_with_signer() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url).unwrap();
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

		let signer = AccountSignerType::called_by_entry_hash160(
			H160::from_str("0xcc45cc8987b0e35371f5685431e3c8eeea306722").unwrap(),
		)
		.unwrap();

		let _ = provider
			.invoke_script_diagnostics(
				"10c00c08646563696d616c730c1425059ecb4878d3a875f91c51ceded330d4575fde41627d5b52"
					.to_string(),
				vec![AccountSigner(signer)],
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_traverse_iterator() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let provider = mock_rpc_response_without_request(
			&mock_server,
			json!([
				{
					"type": "ByteString",
					"value": "dG9rZW5PbmU="
				},
				{
					"type": "ByteString",
					"value": "dG9rZW5Ud28="
				},
				{
					"type": "ByteString",
					"value": "dG9rZW5UaHJlZQ="
				},
				{
					"type": "ByteString",
					"value": "dG9rZW5Gb3Vy"
				}
			]),
		)
		.await;

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

		let result = provider
			.traverse_iterator(
				"127d3320-db35-48d5-b6d3-ca22dca4a370".to_string(),
				"cb7ef774-1ade-4a83-914b-94373ca92010".to_string(),
				100,
			)
			.await;
		verify_request(&mock_server, &expected_request_body).await.unwrap();

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		let iterator_list = result.unwrap();
		assert_eq!(iterator_list.len(), 4);
		assert_eq!(iterator_list.get(3).unwrap().as_string(), Some("tokenFour".to_string()));
	}

	#[tokio::test]
	async fn test_terminate_session() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let provider = mock_rpc_response_without_request(&mock_server, json!(true)).await;
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

		let result = provider.terminate_session("127d3320-db35-48d5-b6d3-ca22dca4a370").await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		assert_eq!(result.unwrap(), true);
	}

	#[tokio::test]
	async fn test_invoke_contract_verify() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let provider = mock_rpc_response_without_request(
			&mock_server,
			json!({
				"script": "VgEMFJOtFXKks1xLklSDzhcBt4dC3EYPYEBXAAIhXwAhQfgn7IxA",
				"state": "FAULT",
				"gasconsumed": "0.0103542",
				"exception": "Specified argument was out of the range of valid values. (Parameter 'index')",
				"stack": [
					{
						"type": "Buffer",
						"value": "dHJhbnNmZXI="
					}
				]
			}),
		)
		.await;

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

		let signer = AccountSignerType::called_by_entry_hash160(
			H160::from_str("0xcadb3dc2faa3ef14a13b619c9a43124755aa2569").unwrap(),
		)
		.unwrap();

		let result = provider
			.invoke_contract_verify(
				H160::from_str("af7c7328eee5a275a3bcaee2bf0cf662b5e739be").unwrap(),
				vec!["a string".to_string().into(), "another string".to_string().into()],
				vec![AccountSigner(signer)],
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		let invocation_result = result.unwrap();
		assert_eq!(
			invocation_result.script,
			"VgEMFJOtFXKks1xLklSDzhcBt4dC3EYPYEBXAAIhXwAhQfgn7IxA".to_string()
		);
		assert_eq!(invocation_result.state, NeoVMStateType::Fault);
		assert_eq!(invocation_result.gas_consumed, "0.0103542".to_string());
		assert_eq!(
			invocation_result.exception,
			Some(
				"Specified argument was out of the range of valid values. (Parameter 'index')"
					.to_string()
			)
		);
		assert_eq!(invocation_result.stack.len(), 1);
		let stack_item = invocation_result.stack.get(0).unwrap();
		assert!(
			matches!(stack_item, StackItem::Buffer { .. }),
			"The stack item type is not Buffer as expected"
		);
		assert_eq!(stack_item.as_string(), Some("transfer".to_string()));
	}

	#[tokio::test]
	async fn test_invoke_contract_verify_noparams_nosigners() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url).unwrap();
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

	// Utility methods

	#[tokio::test]
	async fn test_list_plugins() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let provider = mock_rpc_response_without_request(
			&mock_server,
			json!([
				{
					"name": "ApplicationLogs",
					"version": "3.0.0.0",
					"interfaces": [
						"IPersistencePlugin"
					]
				},
				{
					"name": "LevelDBStore",
					"version": "3.0.0.0",
					"interfaces": []
				},
				{
					"name": "RocksDBStore",
					"version": "3.0.0.0",
					"interfaces": []
				},
				{
					"name": "RpcNep17Tracker",
					"version": "3.0.0.0",
					"interfaces": [
						"IPersistencePlugin"
					]
				},
				{
					"name": "RpcServerPlugin",
					"version": "3.0.0.0",
					"interfaces": []
				},
				{
					"name": "StatesDumper",
					"version": "3.0.0.0",
					"interfaces": [
						"IPersistencePlugin"
					]
				},
				{
					"name": "SystemLog",
					"version": "3.0.0.0",
					"interfaces": [
						"ILogPlugin"
					]
				}
			]),
		)
		.await;

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "listplugins",
			"params": [],
			"id": 1
		}}"#
		);

		let result = provider.list_plugins().await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		let list_plugins = result.unwrap();
		assert_eq!(list_plugins.len(), 7);

		let mut plugin = list_plugins.get(0).unwrap();
		assert_eq!(
			NodePluginType::value_of_name(&plugin.name),
			Ok(NodePluginType::ApplicationLogs)
		);
		assert_eq!(plugin.version, "3.0.0.0".to_string());
		assert_eq!(plugin.interfaces.len(), 1);
		assert_eq!(plugin.interfaces, vec!["IPersistencePlugin"]);

		let mut plugin = list_plugins.get(1).unwrap();
		assert_eq!(NodePluginType::value_of_name(&plugin.name), Ok(NodePluginType::LevelDbStore));
		assert_eq!(plugin.version, "3.0.0.0".to_string());
		assert_eq!(plugin.interfaces.len(), 0);

		let mut plugin = list_plugins.get(2).unwrap();
		assert_eq!(NodePluginType::value_of_name(&plugin.name), Ok(NodePluginType::RocksDbStore));
		assert_eq!(plugin.version, "3.0.0.0".to_string());
		assert_eq!(plugin.interfaces.len(), 0);

		let mut plugin = list_plugins.get(3).unwrap();
		assert_eq!(
			NodePluginType::value_of_name(&plugin.name),
			Ok(NodePluginType::RpcNep17Tracker)
		);
		assert_eq!(plugin.version, "3.0.0.0".to_string());
		assert_eq!(plugin.interfaces.len(), 1);
		assert_eq!(plugin.interfaces, vec!["IPersistencePlugin"]);

		let mut plugin = list_plugins.get(4).unwrap();
		assert_eq!(
			NodePluginType::value_of_name(&plugin.name),
			Ok(NodePluginType::RpcServerPlugin)
		);
		assert_eq!(plugin.version, "3.0.0.0".to_string());
		assert_eq!(plugin.interfaces.len(), 0);

		let mut plugin = list_plugins.get(5).unwrap();
		assert_eq!(NodePluginType::value_of_name(&plugin.name), Ok(NodePluginType::StatesDumper));
		assert_eq!(plugin.version, "3.0.0.0".to_string());
		assert_eq!(plugin.interfaces.len(), 1);
		assert_eq!(plugin.interfaces, vec!["IPersistencePlugin"]);

		let mut plugin = list_plugins.get(6).unwrap();
		assert_eq!(NodePluginType::value_of_name(&plugin.name), Ok(NodePluginType::SystemLog));
		assert_eq!(plugin.version, "3.0.0.0".to_string());
		assert_eq!(plugin.interfaces.len(), 1);
		assert_eq!(plugin.interfaces, vec!["ILogPlugin"]);
	}

	#[tokio::test]
	async fn test_validate_address() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let provider = mock_rpc_response_without_request(
			&mock_server,
			json!(
			{
				"address": "AQVh2pG732YvtNaxEGkQUei3YA4cvo7d2i",
				"isvalid": true
			}
			),
		)
		.await;

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "validateaddress",
			"params": ["NTzVAPBpnUUCvrA6tFPxBHGge8Kyw8igxX"],
			"id": 1
		}}"#
		);

		let result = provider.validate_address("NTzVAPBpnUUCvrA6tFPxBHGge8Kyw8igxX").await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		let validate_address = result.unwrap();
		assert_eq!(validate_address.address, "AQVh2pG732YvtNaxEGkQUei3YA4cvo7d2i".to_string());
		assert_eq!(validate_address.is_valid, true);
	}

	// Wallet Methods

	#[tokio::test]
	async fn test_close_wallet() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let provider = mock_rpc_response_without_request(&mock_server, json!(true)).await;

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "closewallet",
			"params": [],
			"id": 1
		}}"#
		);

		let result = provider.close_wallet().await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		assert_eq!(result.unwrap(), true);
	}

	#[tokio::test]
	async fn test_open_wallet() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let provider = mock_rpc_response_without_request(&mock_server, json!(true)).await;

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "openwallet",
			"params": ["wallet.json","one"],
			"id": 1
		}}"#
		);

		let result = provider.open_wallet("wallet.json".to_string(), "one".to_string()).await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		assert_eq!(result.unwrap(), true);
	}

	#[tokio::test]
	async fn test_dump_priv_key() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let provider = mock_rpc_response_without_request(
			&mock_server,
			json!("L1ZW4aRmy4MMG3x3wk9S6WEJJxcaZi72YxPx854Lspdo9jNFxEoJ"),
		)
		.await;

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "dumpprivkey",
			"params": ["NdWaiUoBWbPxGsm5wXPjXYJxCyuY1Zw8uW"],
			"id": 1
		}}"#
		);

		let result = provider
			.dump_priv_key(H160::from_str("c11d816956b6682c3406bb99b7ec8a3e93f005c1").unwrap())
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		assert_eq!(
			result.unwrap(),
			"L1ZW4aRmy4MMG3x3wk9S6WEJJxcaZi72YxPx854Lspdo9jNFxEoJ".to_string()
		);
	}

	#[tokio::test]
	async fn test_get_wallet_balance() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let provider = mock_rpc_response_without_request(
			&mock_server,
			json!(
				{
					"balance": "200"
				}
			),
		)
		.await;

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "getwalletbalance",
			"params": ["de5f57d430d3dece511cf975a8d37848cb9e0525"],
			"id": 1
		}}"#
		);

		let result = provider
			.get_wallet_balance(H160::from_str("de5f57d430d3dece511cf975a8d37848cb9e0525").unwrap())
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		assert_eq!(result.unwrap().balance, "200".to_string())
	}

	#[tokio::test]
	async fn test_get_wallet_balance_upper_case() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let provider = mock_rpc_response_without_request(
			&mock_server,
			json!(
				{
					"Balance": "199999990.0"
				}
			),
		)
		.await;

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "getwalletbalance",
			"params": ["de5f57d430d3dece511cf975a8d37848cb9e0525"],
			"id": 1
		}}"#
		);

		let result = provider
			.get_wallet_balance(H160::from_str("de5f57d430d3dece511cf975a8d37848cb9e0525").unwrap())
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		assert_eq!(result.unwrap().balance, "199999990.0".to_string())
	}

	#[tokio::test]
	async fn test_get_new_address() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let provider = mock_rpc_response_without_request(
			&mock_server,
			json!("APuGosNQYQoRYYMxvay3yZsragzvfBMdNs"),
		)
		.await;

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "getnewaddress",
			"params": [],
			"id": 1
		}}"#
		);
		let result = provider.get_new_address().await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		assert_eq!(result.unwrap(), "APuGosNQYQoRYYMxvay3yZsragzvfBMdNs".to_string())
	}

	#[tokio::test]
	async fn test_get_wallet_unclaimed_gas() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let provider = mock_rpc_response_without_request(&mock_server, json!("289799420400")).await;

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "getwalletunclaimedgas",
			"params": [],
			"id": 1
		}}"#
		);
		let result = provider.get_wallet_unclaimed_gas().await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		assert_eq!(result.unwrap(), "289799420400".to_string())
	}

	#[tokio::test]
	async fn test_get_unclaimed_gas() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let provider = mock_rpc_response_without_request(
			&mock_server,
			json!(
				{
					"unclaimed": "79199824176",
					"address": "AGZLEiwUyCC4wiL5sRZA3LbxWPs9WrZeyN"
				}
			),
		)
		.await;

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "getunclaimedgas",
			"params": ["NaQ6Kj6qYinh1frv1wrn53wbPFe5BH5T7g"],
			"id": 1
		}}"#
		);
		let result = provider
			.get_unclaimed_gas(H160::from_str("ffa6adbb5f82ad2a1aafa22ce6aaf05dad5de39e").unwrap())
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		let unclaimed_gas = result.unwrap();
		assert_eq!(unclaimed_gas.unclaimed, "79199824176".to_string());
		assert_eq!(unclaimed_gas.address, "AGZLEiwUyCC4wiL5sRZA3LbxWPs9WrZeyN".to_string());
	}

	#[tokio::test]
	async fn test_import_priv_key() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let provider = mock_rpc_response_without_request(
			&mock_server,
			json!(
				{
					"address": "AYhJaF5oqUscfNfH87KQHm1YwuKrwPgkMA",
					"haskey": true,
					"label": null,
					"watchonly": false
				}
			),
		)
		.await;

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "importprivkey",
			"params": ["L5c6jz6Rh8arFJW3A5vg7Suaggo28ApXVF2EPzkAXbm94ThqaA6r"],
			"id": 1
		}}"#
		);
		let result = provider
			.import_priv_key("L5c6jz6Rh8arFJW3A5vg7Suaggo28ApXVF2EPzkAXbm94ThqaA6r".to_string())
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		let address = result.unwrap();
		assert_eq!(address.address, "AYhJaF5oqUscfNfH87KQHm1YwuKrwPgkMA".to_string());
		assert_eq!(address.has_key, true);
		assert!(address.label.is_none());
		assert_eq!(address.watch_only, false);
	}

	#[tokio::test]
	async fn test_calculate_network_fee() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let provider = mock_rpc_response_without_request(
			&mock_server,
			json!(
				{
					"networkfee": 1230610
				}

			),
		)
		.await;

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "calculatenetworkfee",
			"params": ["bmVvdzNq"],
			"id": 1
		}}"#
		);
		let result = provider.calculate_network_fee("6e656f77336a".to_string()).await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		assert_eq!(result.unwrap().network_fee, 1230610);
	}

	#[tokio::test]
	async fn test_list_address() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let provider = mock_rpc_response_without_request(
			&mock_server,
			json!(
				[
					{
						"address": "AK5AmzrrM3sw3kbCHXpHNeuK3kkjnneUrb",
						"haskey": true,
						"label": "hodl",
						"watchonly": false
					},
					{
						"address": "AHE5cLhX5NjGB5R2PcdUvGudUoGUBDeHX4",
						"haskey": false,
						"label": null,
						"watchonly": true
					}
				]
			),
		)
		.await;

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "listaddress",
			"params": [],
			"id": 1
		}}"#
		);
		let result = provider.list_address().await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		let list_address = result.unwrap();
		assert_eq!(list_address.len(), 2);

		let mut address = list_address.get(0).unwrap();
		assert_eq!(address.address, "AK5AmzrrM3sw3kbCHXpHNeuK3kkjnneUrb".to_string());
		assert_eq!(address.has_key, true);
		assert_eq!(address.label, Some("hodl".to_string()));
		assert_eq!(address.watch_only, false);

		let mut address = list_address.get(1).unwrap();
		assert_eq!(address.address, "AHE5cLhX5NjGB5R2PcdUvGudUoGUBDeHX4".to_string());
		assert_eq!(address.has_key, false);
		assert_eq!(address.label, None);
		assert_eq!(address.watch_only, true);
	}

	#[tokio::test]
	async fn test_send_from() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let provider = mock_rpc_response_without_request(
			&mock_server,
			json!(
				{
					"hash": "0x6818f446c2e503998ac766a8a175f86d9a89885423f6b055aa123c984625833e",
					"size": 266,
					"version": 0,
					"nonce": 1762654532,
					"sender": "AHE5cLhX5NjGB5R2PcdUvGudUoGUBDeHX4",
					"sysfee": "9007810",
					"netfee": "1266450",
					"validuntilblock": 2106392,
					"signers": [
						{
							"account": "0xf68f181731a47036a99f04dad90043a744edec0f",
							"scopes": "CalledByEntry"
						}
					],
					"attributes": [],
					"script": "GgwU5sEBNlSvET2KlovcpSyZSKgrlT0MFA/s7USnQwDZ2gSfqTZwpDEXGI/2E8AMCHRyYW5zZmVyDBSJdyDYzXb08Aq/o3wO3YicII/em0FifVtSOA==",
					"witnesses": [
						{
							"invocation": "DEAZaoPvbyaQyUYqIBc4MyDCGxGhxlPCuBbcHn5cYMpHPi2JD4PX2I1EsDPNtrEESPo//WBnsKyl5o5ViR5YDcJR",
							"verification": "EQwhA/HsPB4oPogN5unEifDyfBkAfFM4WqpMDJF8MgB57a3yEQtBMHOzuw=="
						}
					],
				}
			)
		).await;
		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "sendfrom",
			"params": ["de5f57d430d3dece511cf975a8d37848cb9e0525", "NaxePjypvtsQ5GVi6S1jBsSjXribTSUKRu", "NbD6be5uYezFZRSBDt6aBfYR9bYsAk8Yui", 10],
			"id": 1
		}}"#
		);
		let result = provider
			.send_from(
				H160::from_str("0xde5f57d430d3dece511cf975a8d37848cb9e0525").unwrap(),
				H160::from_str("8cdb257b8873049918fe5a1e7f6289f75d720ba5").unwrap(),
				H160::from_str("db1acbae4dbae55f8325724cf080ed782925c7a7").unwrap(),
				10,
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		let tx = result.unwrap();
		assert_eq!(
			tx.hash(),
			&H256::from_str("0x6818f446c2e503998ac766a8a175f86d9a89885423f6b055aa123c984625833e")
				.unwrap()
		);
		assert_eq!(tx.size(), &266);
		assert_eq!(tx.nonce(), &1762654532);
		assert_eq!(tx.sender(), &"AHE5cLhX5NjGB5R2PcdUvGudUoGUBDeHX4".to_string());
		assert_eq!(tx.sys_fee(), &"9007810".to_string());
		assert_eq!(tx.net_fee(), &"1266450".to_string());
		assert_eq!(tx.valid_until_block(), &2106392);

		assert_eq!(tx.signers().len(), 1);
		let signer = tx.signers().get(0).unwrap();
		assert_eq!(
			signer.account,
			H160::from_str("0xf68f181731a47036a99f04dad90043a744edec0f").unwrap()
		);
		assert_eq!(signer.scopes.len(), 1);
		assert_eq!(signer.scopes[0], WitnessScope::CalledByEntry);

		assert_eq!(tx.attributes().len(), 0);
		let mut result = tx.get_first_attribute();
		assert!(matches!(result, Err(TypeError::IndexOutOfBounds(_))));
		if let Err(TypeError::IndexOutOfBounds(msg)) = result {
			assert!(msg.contains("does not have any attributes"));
		}

		assert_eq!(tx.script(), &"GgwU5sEBNlSvET2KlovcpSyZSKgrlT0MFA/s7USnQwDZ2gSfqTZwpDEXGI/2E8AMCHRyYW5zZmVyDBSJdyDYzXb08Aq/o3wO3YicII/em0FifVtSOA==".to_string());

		assert_eq!(tx.witnesses().len(), 1);
		assert_eq!(tx.witnesses(), &vec![
			NeoWitness::new("DEAZaoPvbyaQyUYqIBc4MyDCGxGhxlPCuBbcHn5cYMpHPi2JD4PX2I1EsDPNtrEESPo//WBnsKyl5o5ViR5YDcJR".to_string(), "EQwhA/HsPB4oPogN5unEifDyfBkAfFM4WqpMDJF8MgB57a3yEQtBMHOzuw==".to_string())
		]);
	}

	#[tokio::test]
	async fn test_send_from_transaction_send_asset() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url).unwrap();
		let provider = RpcClient::new(http_client);

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "sendfrom",
			"params": ["de5f57d430d3dece511cf975a8d37848cb9e0525", "Ng9E3D4DpM6JrgSxizhanJ6zm6BjvZ2XkM", "NUokBS9rfH8qncwFdfByBTT9yJjxQv8h2h", 10],
			"id": 1
		}}"#
		);
		let _ = provider
			.send_from_send_token(
				&TransactionSendToken::new(
					H160::from_str("0xde5f57d430d3dece511cf975a8d37848cb9e0525").unwrap(),
					10,
					"NUokBS9rfH8qncwFdfByBTT9yJjxQv8h2h".to_string(),
				),
				H160::from_str("44b159ceed1bfbd753748227309428f54f52e4dd").unwrap(),
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_send_many() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let provider = mock_rpc_response_without_request(
			&mock_server,
			json!(
				{
					"hash": "0xf60ec3b0810fb8c17a9a05eaeb3b361ead889a38d3fd1bf2d561a6e7001bb2f5",
					"size": 352,
					"version": 0,
					"nonce": 1256822346,
					"sender": "AHE5cLhX5NjGB5R2PcdUvGudUoGUBDeHX4",
					"sysfee": "18015620",
					"netfee": "1352450",
					"validuntilblock": 2106840,
					"signers": [
						{
							"account": "0xbe175fb771d5782282b7598b56c26a2f5ebf2d24",
							"scopes": "CalledByEntry"
						},
						{
							"account": "0xf68f181731a47036a99f04dad90043a744edec0f",
							"scopes": "CalledByEntry"
						}
					],
					"attributes": [],
					"script": "AGQMFObBATZUrxE9ipaL3KUsmUioK5U9DBQP7O1Ep0MA2doEn6k2cKQxFxiP9hPADAh0cmFuc2ZlcgwUiXcg2M129PAKv6N8Dt2InCCP3ptBYn1bUjgaDBQP7O1Ep0MA2doEn6k2cKQxFxiP9gwUD+ztRKdDANnaBJ+pNnCkMRcYj/YTwAwIdHJhbnNmZXIMFIl3INjNdvTwCr+jfA7diJwgj96bQWJ9W1I4",
					"witnesses": [
						{
							"invocation": "DEDjHdgTfdXKx1R9f4D1lRklhisjDOkkMt7t1fO1CPO31gVQZUiWJc7GvJqjkR35iDjJjGIwd3s/Lm7q71rwdVC4",
							"verification": "EQwhA/HsPB4oPogN5unEifDyfBkAfFM4WqpMDJF8MgB57a3yEQtBMHOzuw=="
						}
					],
				}
			)
		).await;

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "sendmany",
			"params": [
				[
					{{
						"asset": "de5f57d430d3dece511cf975a8d37848cb9e0525",
						"value": 100,
						"address": "NRkkHsxkzFxGz77mJtJgYZ3FnBm8baU5Um"
					}},
					{{
						"asset": "de5f57d430d3dece511cf975a8d37848cb9e0525",
						"value": 10,
						"address": "NNFGNNK1HXSSnA7yKLzRpr8YXwcdgTrsCu"
					}}
				]
			],
			"id": 1
		}}"#
		);
		let result = provider
			.send_many(
				None,
				vec![
					TransactionSendToken::new(
						H160::from_str("0xde5f57d430d3dece511cf975a8d37848cb9e0525").unwrap(),
						100,
						"NRkkHsxkzFxGz77mJtJgYZ3FnBm8baU5Um".to_string(),
					),
					TransactionSendToken::new(
						H160::from_str("0xde5f57d430d3dece511cf975a8d37848cb9e0525").unwrap(),
						10,
						"NNFGNNK1HXSSnA7yKLzRpr8YXwcdgTrsCu".to_string(),
					),
				],
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		let tx = result.unwrap();
		assert_eq!(
			tx.hash(),
			&H256::from_str("0xf60ec3b0810fb8c17a9a05eaeb3b361ead889a38d3fd1bf2d561a6e7001bb2f5")
				.unwrap()
		);
		assert_eq!(tx.size(), &352);
		assert_eq!(tx.version(), &0);
		assert_eq!(tx.nonce(), &1256822346);
		assert_eq!(tx.sender(), &"AHE5cLhX5NjGB5R2PcdUvGudUoGUBDeHX4".to_string());
		assert_eq!(tx.sys_fee(), &"18015620".to_string());
		assert_eq!(tx.net_fee(), &"1352450".to_string());
		assert_eq!(tx.valid_until_block(), &2106840);

		assert_eq!(tx.signers().len(), 2);
		let signer = tx.signers().get(0).unwrap();
		assert_eq!(
			signer.account,
			H160::from_str("0xbe175fb771d5782282b7598b56c26a2f5ebf2d24").unwrap()
		);
		assert_eq!(signer.scopes.len(), 1);
		assert_eq!(signer.scopes.get(0), Some(&WitnessScope::CalledByEntry));
		assert_eq!(
			tx.signers(),
			&vec![
				RTransactionSigner::new(
					H160::from_str("0xbe175fb771d5782282b7598b56c26a2f5ebf2d24").unwrap(),
					vec![WitnessScope::CalledByEntry]
				),
				RTransactionSigner::new(
					H160::from_str("0xf68f181731a47036a99f04dad90043a744edec0f").unwrap(),
					vec![WitnessScope::CalledByEntry]
				)
			]
		);

		let mut result = tx.get_first_signer().clone().unwrap().get_first_allowed_contract();
		assert!(matches!(result, Err(TypeError::IndexOutOfBounds(_))));
		if let Err(TypeError::IndexOutOfBounds(msg)) = result {
			assert!(msg.contains("does not allow any specific contract"));
		}

		let mut result = tx.get_first_signer().clone().unwrap().get_first_allowed_group();
		assert!(matches!(result, Err(TypeError::IndexOutOfBounds(_))));
		if let Err(TypeError::IndexOutOfBounds(msg)) = result {
			assert!(msg.contains("does not allow any specific group"));
		}

		assert_eq!(tx.attributes().len(), 0);

		assert_eq!(tx.script(), &"AGQMFObBATZUrxE9ipaL3KUsmUioK5U9DBQP7O1Ep0MA2doEn6k2cKQxFxiP9hPADAh0cmFuc2ZlcgwUiXcg2M129PAKv6N8Dt2InCCP3ptBYn1bUjgaDBQP7O1Ep0MA2doEn6k2cKQxFxiP9gwUD+ztRKdDANnaBJ+pNnCkMRcYj/YTwAwIdHJhbnNmZXIMFIl3INjNdvTwCr+jfA7diJwgj96bQWJ9W1I4".to_string());

		assert_eq!(tx.witnesses().len(), 1);
		assert_eq!(tx.witnesses(), &vec![
			NeoWitness::new(
				"DEDjHdgTfdXKx1R9f4D1lRklhisjDOkkMt7t1fO1CPO31gVQZUiWJc7GvJqjkR35iDjJjGIwd3s/Lm7q71rwdVC4".to_string(),
				"EQwhA/HsPB4oPogN5unEifDyfBkAfFM4WqpMDJF8MgB57a3yEQtBMHOzuw==".to_string()
			)
		]);
	}

	#[tokio::test]
	async fn test_send_many_empty_transaction() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url).unwrap();
		let provider = RpcClient::new(http_client);

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "sendmany",
			"params": [
				[]
			],
			"id": 1
		}}"#
		);
		let _ = provider.send_many(None, Vec::<TransactionSendToken>::new()).await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();

		let _ = env_logger::builder().is_test(true).try_init();

		let mock_provider = Arc::new(Mutex::new(MockClient::new().await));
		{
			let mut mock_provider_guard = mock_provider.lock().await; // Lock the mock_provider once
			mock_provider_guard
				.mock_response_error(json!({
					"code": -32602,
					"message": "Invalid params"
				}))
				.await;
			mock_provider_guard.mount_mocks().await;
		}

		let client = {
			let mock_provider = mock_provider.lock().await;
			Arc::new(mock_provider.into_client())
		};
		let result = client.send_many(None, Vec::<TransactionSendToken>::new()).await;

		// Assert that the error is a JsonRpcError
		assert!(matches!(result, Err(ProviderError::JsonRpcError(_))), "Expected a JsonRpcError.");
	}

	#[tokio::test]
	async fn test_send_many_with_from() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url).unwrap();
		let provider = RpcClient::new(http_client);

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "sendmany",
			"params": [
				"NiVNRW6cBXwkvrZnetZToaHPGSSGgV1HmA",
				[
					{{
						"asset": "de5f57d430d3dece511cf975a8d37848cb9e0525",
						"value": 100,
						"address": "Nhsi2q3hkByxcH2uBQw7cjc2qEpzXSEKTC"
					}},
					{{
						"asset": "de5f57d430d3dece511cf975a8d37848cb9e0525",
						"value": 10,
						"address": "NcwVWxJZh9fxncJ9Sq8msVLotJDsAD3ZD8"
					}}
				]
			],
			"id": 1
		}}"#
		);
		let _ = provider
			.send_many(
				Some(H160::from_address("NiVNRW6cBXwkvrZnetZToaHPGSSGgV1HmA").unwrap()),
				vec![
					TransactionSendToken::new(
						H160::from_str("0xde5f57d430d3dece511cf975a8d37848cb9e0525").unwrap(),
						100,
						"Nhsi2q3hkByxcH2uBQw7cjc2qEpzXSEKTC".to_string(),
					),
					TransactionSendToken::new(
						H160::from_str("0xde5f57d430d3dece511cf975a8d37848cb9e0525").unwrap(),
						10,
						"NcwVWxJZh9fxncJ9Sq8msVLotJDsAD3ZD8".to_string(),
					),
				],
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_send_to_address() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let provider = mock_rpc_response_without_request(
			&mock_server,
			json!(
				{
					"hash": "0xabd78548399bbe684fac50b6a71d0ce3f689497d4e79cb26a2b4dfb211782c39",
					"size": 375,
					"version": 0,
					"nonce": 1509730265,
					"sender": "AK5AmzrrM3sw3kbCHXpHNeuK3kkjnneUrb",
					"sysfee": "9007810",
					"netfee": "2375840",
					"validuntilblock": 2106930,
					"signers": [
						{
							"account": "0xf68f181731a47036a99f04dad90043a744edec0f",
							"scopes": "CalledByEntry"
						}
					],
					"attributes": [],
					"script": "GgwU5sEBNlSvET2KlovcpSyZSKgrlT0MFA/s7USnQwDZ2gSfqTZwpDEXGI/2E8AMCHRyYW5zZmVyDBSJdyDYzXb08Aq/o3wO3YicII/em0FifVtSOA==",
					"witnesses": [
						{
							"invocation": "DECstBmb75AW65NjA35fFlSxszuLRDUzd0nnbfyH8MlnSA02f6B1XlvItpZQBsAd7Pvqa7S+olPAKDO0qtq3ZtOB",
							"verification": "DCED8ew8Hig+iA3m6cSJ8PJ8GQB8UzhaqkwMkXwyAHntrfILQQqQatQ="
						},
						{
							"invocation": "DED6KQHcomjUhyLcmIcPwM1iWkbOlgDnidWZP+PLDWLQRk2rKLg5B/sY1YD1bqylF0zmtDCSIQKeMivAGJyOSXi4",
							"verification": "EQwhA/HsPB4oPogN5unEifDyfBkAfFM4WqpMDJF8MgB57a3yEQtBMHOzuw=="
						}
					],
				}
			)
		).await;

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "sendtoaddress",
			"params": ["de5f57d430d3dece511cf975a8d37848cb9e0525", "NRCcuUUxKCa3sp45o7bjXetyxUeq58T4ED", 10],
			"id": 1
		}}"#
		);
		let result = provider
			.send_to_address(
				H160::from_str("0xde5f57d430d3dece511cf975a8d37848cb9e0525").unwrap(),
				H160::from_str("674231bd321880fc5c4a73994c87870e52c5fe39").unwrap(),
				10,
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		let tx = result.unwrap();
		assert_eq!(
			tx.hash(),
			&H256::from_str("0xabd78548399bbe684fac50b6a71d0ce3f689497d4e79cb26a2b4dfb211782c39")
				.unwrap()
		);
		assert_eq!(tx.size(), &375);
		assert_eq!(tx.version(), &0);
		assert_eq!(tx.nonce(), &1509730265);
		assert_eq!(tx.sender(), &"AK5AmzrrM3sw3kbCHXpHNeuK3kkjnneUrb".to_string());
		assert_eq!(tx.sys_fee(), &"9007810".to_string());
		assert_eq!(tx.net_fee(), &"2375840".to_string());
		assert_eq!(tx.valid_until_block(), &2106930);

		assert_eq!(tx.signers().len(), 1);
		let signer = tx.signers().get(0).unwrap();
		assert_eq!(
			signer.account,
			H160::from_str("0xf68f181731a47036a99f04dad90043a744edec0f").unwrap()
		);
		assert_eq!(signer.scopes.len(), 1);
		assert_eq!(signer.scopes.get(0), Some(&WitnessScope::CalledByEntry));
		assert_eq!(tx.attributes().len(), 0);

		assert_eq!(tx.script(), &"GgwU5sEBNlSvET2KlovcpSyZSKgrlT0MFA/s7USnQwDZ2gSfqTZwpDEXGI/2E8AMCHRyYW5zZmVyDBSJdyDYzXb08Aq/o3wO3YicII/em0FifVtSOA==".to_string());

		assert_eq!(tx.witnesses().len(), 2);
		assert_eq!(tx.witnesses(), &vec![
			NeoWitness::new(
				"DECstBmb75AW65NjA35fFlSxszuLRDUzd0nnbfyH8MlnSA02f6B1XlvItpZQBsAd7Pvqa7S+olPAKDO0qtq3ZtOB".to_string(),
				"DCED8ew8Hig+iA3m6cSJ8PJ8GQB8UzhaqkwMkXwyAHntrfILQQqQatQ=".to_string()
			),
			NeoWitness::new(
				"DED6KQHcomjUhyLcmIcPwM1iWkbOlgDnidWZP+PLDWLQRk2rKLg5B/sY1YD1bqylF0zmtDCSIQKeMivAGJyOSXi4".to_string(),
				"EQwhA/HsPB4oPogN5unEifDyfBkAfFM4WqpMDJF8MgB57a3yEQtBMHOzuw==".to_string()
			)
		]);
	}

	#[tokio::test]
	async fn test_send_to_address_transaction_send_asset() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url).unwrap();
		let provider = RpcClient::new(http_client);

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "sendtoaddress",
			"params": ["de5f57d430d3dece511cf975a8d37848cb9e0525", "NaCsFrmoJepqCJSxnTyb41CXVSjr3dMjuL", 10],
			"id": 1
		}}"#
		);
		let _ = provider
			.send_to_address_send_token(&TransactionSendToken::new(
				H160::from_str("0xde5f57d430d3dece511cf975a8d37848cb9e0525").unwrap(),
				10,
				"NaCsFrmoJepqCJSxnTyb41CXVSjr3dMjuL".to_string(),
			))
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_cancel_transaction() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let provider = mock_rpc_response_without_request(
			&mock_server,
			json!(
				{
					"hash": "0x8916c94bb7c4a63f117ca873c9224955bf9ad00b1149a0173e33614559713425",
					"size": 192,
					"version": 0,
					"nonce": 1749194511,
					"sender": "NM7Aky765FG8NhhwtxjXRx7jEL1cnw7PBP",
					"sysfee": "0",
					"netfee": "2343101",
					"validuntilblock": 86401,
					"signers": [
						{
							"account": "0x69ecca587293047be4c59159bf8bc399985c160d",
							"scopes": "None"
						}
					],
					"attributes": [
						{
							"type": "Conflicts",
							"hash": "0x830869b930477c50c91a11569821a4d665d4a61af78a092e3c1a8690f846fc82"
						}
					],
					"script": "QA==",
					"witnesses": [
						{
							"invocation": "DEBoureAXZRM9wyjX/xZP3Di8Q006H3mb\\u002B6DqWbjAjnIVP/Zd/uG57SecDqO2mZAX\\u002B5eoHQYAGRI6/nrOuwP\\u002BrLU",
							"verification": "DCEDOk0FGwS3/AIw0rGq7f1ahL4nmlNhpzWNtmWteFd4fxtBVuezJw=="
						}
					],
				}
			)
		).await;

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "canceltransaction",
			"params": [
				"0000000000000000000000000000000000000000000000000000000000000000", 
				[
					"NKuyBkoGdZZSLyPbJEetheRhMjeznFZszf",
					"NKuyBkoGdZZSLyPbJEetheRhMjeznFZszf"
				], 
				"3333"
			],
			"id": 1
		}}"#
		);
		let result = provider
			.cancel_transaction(H256::zero(), vec![H160::zero(), H160::zero()], Some(3333))
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		let tx = result.unwrap();
		assert_eq!(
			tx.hash(),
			&H256::from_str("0x8916c94bb7c4a63f117ca873c9224955bf9ad00b1149a0173e33614559713425")
				.unwrap()
		);
	}

	// TokenTracker: Nep17

	#[tokio::test]
	async fn test_get_nep17_transfers() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let provider = mock_rpc_response_without_request(
			&mock_server,
			json!({
				"sent": [
					{
						"timestamp": 1554283931,
						"assethash": "1aada0032aba1ef6d1f07bbd8bec1d85f5380fb3",
						"transferaddress": "AYwgBNMepiv5ocGcyNT4mA8zPLTQ8pDBis",
						"amount": "100000000000",
						"blockindex": 368082,
						"transfernotifyindex": 0,
						"txhash": "240ab1369712ad2782b99a02a8f9fcaa41d1e96322017ae90d0449a3ba52a564"
					},
					{
						"timestamp": 1554880287,
						"assethash": "1aada0032aba1ef6d1f07bbd8bec1d85f5380fb3",
						"transferaddress": "AYwgBNMepiv5ocGcyNT4mA8zPLTQ8pDBis",
						"amount": "100000000000",
						"blockindex": 397769,
						"transfernotifyindex": 0,
						"txhash": "12fdf7ce8b2388d23ab223854cb29e5114d8288c878de23b7924880f82dfc834"
					}
				],
				"received": [
					{
						"timestamp": 1555651816,
						"assethash": "600c4f5200db36177e3e8a09e9f18e2fc7d12a0f",
						"transferaddress": "AYwgBNMepiv5ocGcyNT4mA8zPLTQ8pDBis",
						"amount": "1000000",
						"blockindex": 436036,
						"transfernotifyindex": 0,
						"txhash": "df7683ece554ecfb85cf41492c5f143215dd43ef9ec61181a28f922da06aba58"
					}
				],
				"address": "AbHgdBaWEnHkCiLtDZXjhvhaAK2cwFh5pF"
			}),
		)
		.await;

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "getnep17transfers",
			"params": ["NekZLTu93WgrdFHxzBEJUYgLTQMAT85GLi"],
			"id": 1
		}}"#
		);
		let result = provider
			.get_nep17_transfers(
				H160::from_str("04457ce4219e462146ac00b09793f81bc5bca2ce").unwrap(),
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();

		assert!(result.is_ok(), "Result is not okay: {:?}", result);

		let transfers = result.unwrap();
		let sent = transfers.sent;
		assert_eq!(sent.len(), 2);
		assert_eq!(
			sent,
			vec![
				Nep17Transfer::new(
					1554283931,
					H160::from_str("1aada0032aba1ef6d1f07bbd8bec1d85f5380fb3").unwrap(),
					"AYwgBNMepiv5ocGcyNT4mA8zPLTQ8pDBis".to_string(),
					100000000000,
					368082,
					0,
					H256::from_str(
						"240ab1369712ad2782b99a02a8f9fcaa41d1e96322017ae90d0449a3ba52a564"
					)
					.unwrap()
				),
				Nep17Transfer::new(
					1554880287,
					H160::from_str("1aada0032aba1ef6d1f07bbd8bec1d85f5380fb3").unwrap(),
					"AYwgBNMepiv5ocGcyNT4mA8zPLTQ8pDBis".to_string(),
					100000000000,
					397769,
					0,
					H256::from_str(
						"12fdf7ce8b2388d23ab223854cb29e5114d8288c878de23b7924880f82dfc834"
					)
					.unwrap()
				)
			]
		);

		let received = transfers.received;
		assert_eq!(received.len(), 1);
		assert_eq!(
			received,
			vec![Nep17Transfer::new(
				1555651816,
				H160::from_str("600c4f5200db36177e3e8a09e9f18e2fc7d12a0f").unwrap(),
				"AYwgBNMepiv5ocGcyNT4mA8zPLTQ8pDBis".to_string(),
				1000000,
				436036,
				0,
				H256::from_str("df7683ece554ecfb85cf41492c5f143215dd43ef9ec61181a28f922da06aba58")
					.unwrap()
			)]
		);
	}

	#[tokio::test]
	async fn test_get_nep17_transfers_date() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url).unwrap();
		let provider = RpcClient::new(http_client);

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "getnep17transfers",
			"params": ["NSH1UeM96PKhjuzVBKcyWeNNuQkT3sHGmA", 1553105830],
			"id": 1
		}}"#
		);
		let _ = provider
			.get_nep17_transfers_from(
				H160::from_str("8bed27d0e88266807a6339270f0593510967cb45").unwrap(),
				1553105830,
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_nep17_transfers_date_from_to() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url).unwrap();
		let provider = RpcClient::new(http_client);

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "getnep17transfers",
			"params": ["NfWL3Kx7qtZzXrajmggAD4b6r2kGzajbaJ", 1553105830, 1557305830],
			"id": 1
		}}"#
		);
		let _ = provider
			.get_nep17_transfers_range(
				H160::from_str("2eeda865e7824c71b3fe14bed35d04d0f2f0e9d6").unwrap(),
				1553105830,
				1557305830,
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_nep17_balances() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let provider = mock_rpc_response_without_request(
			&mock_server,
			json!({
				"address": "NXXazKH39yNFWWZF5MJ8tEN98VYHwzn7g3",
				"balance": [
					{
						"assethash": "a48b6e1291ba24211ad11bb90ae2a10bf1fcd5a8",
						"name": "SomeToken",
						"symbol": "SOTO",
						"decimals": "4",
						"amount": "50000000000",
						"lastupdatedblock": 251604
					},
					{
						"assethash": "1aada0032aba1ef6d1f07bbd8bec1d85f5380fb3",
						"name": "RandomToken",
						"symbol": "RATO",
						"decimals": "2",
						"amount": "100000000",
						"lastupdatedblock": 251600
					}
				]
			}),
		)
		.await;

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "getnep17balances",
			"params": ["NY9zhKwcmht5cQJ3oRqjJGo3QuVLwXwTzL"],
			"id": 1
		}}"#
		);
		let result = provider
			.get_nep17_balances(H160::from_str("5d75775015b024970bfeacf7c6ab1b0ade974886").unwrap())
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();

		assert!(result.is_ok(), "Result is not okay: {:?}", result);

		let nep17_balances = result.unwrap();
		assert_eq!(nep17_balances.address, "NXXazKH39yNFWWZF5MJ8tEN98VYHwzn7g3".to_string());
		let balance_list = nep17_balances.balances;
		assert_eq!(balance_list.len(), 2);
		assert_eq!(
			balance_list,
			vec![
				Nep17Balance::new(
					H160::from_str("a48b6e1291ba24211ad11bb90ae2a10bf1fcd5a8").unwrap(),
					Some("SomeToken".to_string()),
					Some("SOTO".to_string()),
					Some("4".to_string()),
					"50000000000".to_string(),
					251604
				),
				Nep17Balance::new(
					H160::from_str("1aada0032aba1ef6d1f07bbd8bec1d85f5380fb3").unwrap(),
					Some("RandomToken".to_string()),
					Some("RATO".to_string()),
					Some("2".to_string()),
					"100000000".to_string(),
					251600
				),
			]
		);
	}

	// ApplicationLogs

	#[tokio::test]
	async fn test_get_application_log() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let provider = mock_rpc_response_without_request(
			&mock_server,
			json!({
				"txid": "0x01bcf2edbd27abb8d660b6a06113b84d02f635fed836ce46a38b4d67eae80109",
				"executions": [
					{
						"trigger": "Application",
						"vmstate": "HALT",
						"exception": "asdf",
						"gasconsumed": "9007810",
						"stack": [
							{
								"type": "Integer",
								"value": "1"
							}
						],
						"notifications": [
							{
								"contract": "0x70e2301955bf1e74cbb31d18c2f96972abadb328",
								"eventname": "Transfer",
								"state": {
									"type": "Array",
									"value": [
										{
											"type": "Any"
										},
										{
											"type": "ByteString",
											"value": "ev0gMlXLKXK9CmqCfnTjh+0yK+w="
										},
										{
											"type": "Integer",
											"value": "600000000"
										}
									]
								}
							},
							{
								"contract": "0xf61eebf573ea36593fd43aa150c055ad7906ab83",
								"eventname": "Transfer",
								"state": {
									"type": "Array",
									"value": [
										{
											"type": "ByteString",
											"value": "VHJhbnNmZXI="
										},
										{
											"type": "ByteString",
											"value": "CaVYdMLaS4bl1J/1MKGxU+sSx9Y="
										},
										{
											"type": "Integer",
											"value": "100"
										}
									]
								}
							}
						]
					}
				]
			}),
		)
		.await;

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "getapplicationlog",
			"params": ["420d1eb458c707d698c6d2ba0f91327918ddb3b7bae2944df070f3f4e579078b"],
			"id": 1
		}}"#
		);
		let result = provider
			.get_application_log(
				H256::from_str("420d1eb458c707d698c6d2ba0f91327918ddb3b7bae2944df070f3f4e579078b")
					.unwrap(),
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();

		assert!(result.is_ok(), "Result is not okay: {:?}", result);

		let app_log = result.unwrap();
		assert_eq!(
			app_log.transaction_id,
			H256::from_str("0x01bcf2edbd27abb8d660b6a06113b84d02f635fed836ce46a38b4d67eae80109")
				.unwrap()
		);
		assert_eq!(app_log.executions.len(), 1);
		assert_eq!(app_log.get_execution(0), app_log.get_first_execution());
		let mut result = app_log.get_execution(1);
		assert!(matches!(result, Err(TypeError::IndexOutOfBounds(_))));
		if let Err(TypeError::IndexOutOfBounds(msg)) = result {
			assert!(msg.contains("has only 1 executions"));
		}
		let execution = app_log.executions.get(0).unwrap();
		assert_eq!(execution.trigger, "Application".to_string());
		assert_eq!(execution.state, VMState::Halt);
		assert_eq!(execution.gas_consumed, "9007810".to_string());
		assert_eq!(execution.stack.len(), 1);
		assert_eq!(execution.get_first_stack_item(), execution.get_stack_item(0));
		let stack_item = execution.get_stack_item(0).unwrap();
		assert!(
			matches!(stack_item, StackItem::Integer { .. }),
			"The stack item type is not Integer as expected"
		);
		assert_eq!(stack_item.as_int(), Some(1));
		let mut result = execution.get_stack_item(1);
		assert!(matches!(result, Err(TypeError::IndexOutOfBounds(_))));
		if let Err(TypeError::IndexOutOfBounds(msg)) = result {
			assert!(msg.contains("only 1 items left on the NeoVM stack after"));
		}

		assert_eq!(execution.notifications.len(), 2);
		assert_eq!(execution.get_first_notification(), execution.get_notification(0));
		let mut result = execution.get_notification(2);
		assert!(matches!(result, Err(TypeError::IndexOutOfBounds(_))));
		if let Err(TypeError::IndexOutOfBounds(msg)) = result {
			assert!(msg.contains("only sent 2 notifications"));
		}

		let notifcation0 = execution.get_notification(0).unwrap();
		assert_eq!(
			notifcation0.contract,
			H160::from_str("0x70e2301955bf1e74cbb31d18c2f96972abadb328").unwrap()
		);
		assert!(
			matches!(notifcation0.state, StackItem::Array { .. }),
			"The stack item type is not Array as expected"
		);
		assert_eq!(notifcation0.event_name, "Transfer".to_string());

		let notifcation_0_array = notifcation0.state.as_array().unwrap();

		let from0 = notifcation_0_array.get(0);
		let to0 = notifcation_0_array.get(1).unwrap().as_address().unwrap();
		let amount0 = notifcation_0_array.get(2).unwrap().as_int().unwrap();

		assert!(from0.is_some());
		assert_eq!(to0, "NX8GreRFGFK5wpGMWetpX93HmtrezGogzk".to_string());
		assert_eq!(amount0, 600000000);

		let notifcation1 = execution.get_notification(1).unwrap();
		assert_eq!(
			notifcation1.contract,
			H160::from_str("0xf61eebf573ea36593fd43aa150c055ad7906ab83").unwrap()
		);
		assert!(
			matches!(notifcation1.state, StackItem::Array { .. }),
			"The stack item type is not Array as expected"
		);
		assert_eq!(notifcation1.event_name, "Transfer".to_string());

		let notifcation_1_array = notifcation1.state.as_array().unwrap();

		let event_name1 = notifcation_1_array.get(0).unwrap().as_string().unwrap();
		let from1 = notifcation_1_array.get(1).unwrap().as_address().unwrap();
		let amount1 = notifcation_1_array.get(2).unwrap().as_int().unwrap();

		assert_eq!(event_name1, "Transfer".to_string());
		assert_eq!(from1, "NLnyLtep7jwyq1qhNPkwXbJpurC4jUT8ke".to_string());
		assert_eq!(amount1, 100);
	}

	#[tokio::test]
	async fn test_get_application_log_empty_stack() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let provider = mock_rpc_response_without_request(
			&mock_server,
			json!({
				"txid": "0x01bcf2edbd27abb8d660b6a06113b84d02f635fed836ce46a38b4d67eae80109",
				"executions": [
					{
						"trigger": "Application",
						"vmstate": "HALT",
						"exception": "asdf",
						"gasconsumed": "9007810",
						"stack": [],
						"notifications": []
					}
				]
			}),
		)
		.await;

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "getapplicationlog",
			"params": ["420d1eb458c707d698c6d2ba0f91327918ddb3b7bae2944df070f3f4e579078b"],
			"id": 1
		}}"#
		);
		let result = provider
			.get_application_log(
				H256::from_str("420d1eb458c707d698c6d2ba0f91327918ddb3b7bae2944df070f3f4e579078b")
					.unwrap(),
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();

		assert!(result.is_ok(), "Result is not okay: {:?}", result);

		let app_log = result.unwrap();
		let execution = app_log.executions.get(0).unwrap();
		let mut result = execution.get_first_stack_item();
		assert!(matches!(result, Err(TypeError::IndexOutOfBounds(_))));
		if let Err(TypeError::IndexOutOfBounds(msg)) = result {
			assert!(msg.contains("no items were left on the NeoVM stack after"));
		}

		let mut result = execution.get_first_notification();
		assert!(matches!(result, Err(TypeError::IndexOutOfBounds(_))));
		if let Err(TypeError::IndexOutOfBounds(msg)) = result {
			assert!(msg.contains("did not send any notifications"));
		}
	}
	// StateService

	#[tokio::test]
	async fn test_get_state_root() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let provider = mock_rpc_response_without_request(
			&mock_server,
			json!({
				"version": 0,
				"index": 160,
				"roothash": "0x28870d1ed61ef167e99354249c622504b0d81d814eaa87dbf8612c91b9b303b7",
				"witnesses": [
					{
						"invocation": "DEDN8o6cmOUt/pfRIexVzO2shhX2vTYFd+cU8vZDQ2Dvn3pe/vHcYOSlY3lPRKecb5zBuLCqaKSvZsC1FAbT00dWDEDoPojyFw66R+pKQsOy0MFmeBBgaC6Z1XGLAigVDHi2VuhAxfpwFpXSTUv3Uv5cIOY+V5g40+2zpU19YQIAWyOJDEDPfitQTjK90KnrloPXKvgTNFPn1520dxDCzQxhl/Wfp7S8dW91/3x3GrF1EaIi32aJtF8W8jUH1Spr/ma66ISs",
						"verification": "EwwhAwAqLhjDnN7Qb8Yd2UoHuOnz+gNqcFvu+HZCUpVOgtDXDCECAM1gQDlYokm5qzKbbAjI/955zDMJc2eji/a1GIEJU2EMIQKXhyDsbFxYdeA0d+FsbZj5AQhamA13R64ysGgh19j6UwwhA8klCeQozdf3pP3UqXxniRC0DxRl3d5PBJ9zJa8zgHkpFAtBE43vrw=="
					}
				]
			})
		).await;

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "getstateroot",
			"params": [52],
			"id": 1
		}}"#
		);
		let result = provider.get_state_root(52).await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		let state_root = result.unwrap();
		assert_eq!(state_root.version, 0);
		assert_eq!(state_root.index, 160);
		assert_eq!(
			state_root.root_hash,
			H256::from_str("0x28870d1ed61ef167e99354249c622504b0d81d814eaa87dbf8612c91b9b303b7")
				.unwrap()
		);
		assert_eq!(state_root.witnesses.len(), 1);
		let witness = state_root.witnesses.get(0).unwrap();
		assert_eq!(witness.invocation, "DEDN8o6cmOUt/pfRIexVzO2shhX2vTYFd+cU8vZDQ2Dvn3pe/vHcYOSlY3lPRKecb5zBuLCqaKSvZsC1FAbT00dWDEDoPojyFw66R+pKQsOy0MFmeBBgaC6Z1XGLAigVDHi2VuhAxfpwFpXSTUv3Uv5cIOY+V5g40+2zpU19YQIAWyOJDEDPfitQTjK90KnrloPXKvgTNFPn1520dxDCzQxhl/Wfp7S8dW91/3x3GrF1EaIi32aJtF8W8jUH1Spr/ma66ISs".to_string());
		assert_eq!(witness.verification, "EwwhAwAqLhjDnN7Qb8Yd2UoHuOnz+gNqcFvu+HZCUpVOgtDXDCECAM1gQDlYokm5qzKbbAjI/955zDMJc2eji/a1GIEJU2EMIQKXhyDsbFxYdeA0d+FsbZj5AQhamA13R64ysGgh19j6UwwhA8klCeQozdf3pP3UqXxniRC0DxRl3d5PBJ9zJa8zgHkpFAtBE43vrw==".to_string());
	}

	#[tokio::test]
	async fn test_get_proof() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let provider = mock_rpc_response_without_request(
			&mock_server,
			json!("Bfv///8XBiQBAQ8DRzb6Vkdw0r5nxMBp6Z5nvbyXiupMvffwm0v5GdB6jHvyAAQEBAQEBAQEA7l84HFtRI5V11s58vA+8CZ5GArFLkGUYLO98RLaMaYmA5MEnx0upnVI45XTpoUDRvwrlPD59uWy9aIrdS4T0D2cA6Rwv/l3GmrctRzL1me+iTUFdDgooaz+esFHFXJdDANfA2bdshZMp5ox2goVAOMjvoxNIWWOqjJoRPu6ZOw2kdj6A8xovEK1Mp6cAG9z/jfFDrSEM60kuo97MNaVOP/cDZ1wA1nf4WdI+jksYz0EJgzBukK8rEzz8jE2cb2Zx2fytVyQBANC7v2RaLMCRF1XgLpSri12L2IwL9Zcjz5LZiaB5nHKNgQpAQYPDw8PDw8DggFffnsVMyqAfZjg+4gu97N/gKpOsAK8Q27s56tijRlSAAMm26DYxOdf/IjEgkE/u/CoRL6dDnzvs1dxCg/00esMvgPGioeOqQCkDOTfliOnCxYjbY/0XvVUOXkceuDm1W0FzQQEBAQEBAQEBAQEBAQEBJIABAPH1PnX/P8NOgV4KHnogwD7xIsD8KvNhkTcDxgCo7Ec6gPQs1zD4igSJB4M9jTREq+7lQ5PbTH/6d138yUVvtM8bQP9Df1kh7asXrYjZolKhLcQ1NoClQgEzbcJfYkCHXv6DQQEBAOUw9zNl/7FJrWD7rCv0mbOoy6nLlHWiWuyGsA12ohRuAQEBAQEBAQEBAYCBAIAAgA=")
		).await;

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "getproof",
			"params": [
				"7bf925dbd33af0e00d392b92313da59369ed86c82494d0e02040b24faac0a3ca", 
				"79bcd398505eb779df6e67e4be6c14cded08e2f2",
				"YW55dGhpbmc="
			],
			"id": 1
		}}"#
		);
		let result = provider
			.get_proof(
				H256::from_str(
					"0x7bf925dbd33af0e00d392b92313da59369ed86c82494d0e02040b24faac0a3ca",
				)
				.unwrap(),
				H160::from_str("0x79bcd398505eb779df6e67e4be6c14cded08e2f2").unwrap(),
				"616e797468696e67",
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		assert_eq!(result.unwrap(), "Bfv///8XBiQBAQ8DRzb6Vkdw0r5nxMBp6Z5nvbyXiupMvffwm0v5GdB6jHvyAAQEBAQEBAQEA7l84HFtRI5V11s58vA+8CZ5GArFLkGUYLO98RLaMaYmA5MEnx0upnVI45XTpoUDRvwrlPD59uWy9aIrdS4T0D2cA6Rwv/l3GmrctRzL1me+iTUFdDgooaz+esFHFXJdDANfA2bdshZMp5ox2goVAOMjvoxNIWWOqjJoRPu6ZOw2kdj6A8xovEK1Mp6cAG9z/jfFDrSEM60kuo97MNaVOP/cDZ1wA1nf4WdI+jksYz0EJgzBukK8rEzz8jE2cb2Zx2fytVyQBANC7v2RaLMCRF1XgLpSri12L2IwL9Zcjz5LZiaB5nHKNgQpAQYPDw8PDw8DggFffnsVMyqAfZjg+4gu97N/gKpOsAK8Q27s56tijRlSAAMm26DYxOdf/IjEgkE/u/CoRL6dDnzvs1dxCg/00esMvgPGioeOqQCkDOTfliOnCxYjbY/0XvVUOXkceuDm1W0FzQQEBAQEBAQEBAQEBAQEBJIABAPH1PnX/P8NOgV4KHnogwD7xIsD8KvNhkTcDxgCo7Ec6gPQs1zD4igSJB4M9jTREq+7lQ5PbTH/6d138yUVvtM8bQP9Df1kh7asXrYjZolKhLcQ1NoClQgEzbcJfYkCHXv6DQQEBAOUw9zNl/7FJrWD7rCv0mbOoy6nLlHWiWuyGsA12ohRuAQEBAQEBAQEBAYCBAIAAgA=".to_string());
	}

	#[tokio::test]
	async fn test_verify_proof() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let provider = mock_rpc_response_without_request(
			&mock_server,
			json!("QAFBAighAhY5RqEz49Lg2Yf7kMsBsGDtF4DxcY4too7fE7ll/StgIQA="),
		)
		.await;

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "verifyproof",
			"params": [
				"7bf925dbd33af0e00d392b92313da59369ed86c82494d0e02040b24faac0a3ca", 
				"Bfv///8XBiQBAQ8DRzb6Vkdw0r5nxMBp6Z5nvbyXiupMvffwm0v5GdB6jHvyAAQEBAQEBAQEA7l84HFtRI5V11s58vA+8CZ5GArFLkGUYLO98RLaMaYmA5MEnx0upnVI45XTpoUDRvwrlPD59uWy9aIrdS4T0D2cA6Rwv/l3GmrctRzL1me+iTUFdDgooaz+esFHFXJdDANfA2bdshZMp5ox2goVAOMjvoxNIWWOqjJoRPu6ZOw2kdj6A8xovEK1Mp6cAG9z/jfFDrSEM60kuo97MNaVOP/cDZ1wA1nf4WdI+jksYz0EJgzBukK8rEzz8jE2cb2Zx2fytVyQBANC7v2RaLMCRF1XgLpSri12L2IwL9Zcjz5LZiaB5nHKNgQpAQYPDw8PDw8DggFffnsVMyqAfZjg+4gu97N/gKpOsAK8Q27s56tijRlSAAMm26DYxOdf/IjEgkE/u/CoRL6dDnzvs1dxCg/00esMvgPGioeOqQCkDOTfliOnCxYjbY/0XvVUOXkceuDm1W0FzQQEBAQEBAQEBAQEBAQEBJIABAPH1PnX/P8NOgV4KHnogwD7xIsD8KvNhkTcDxgCo7Ec6gPQs1zD4igSJB4M9jTREq+7lQ5PbTH/6d138yUVvtM8bQP9Df1kh7asXrYjZolKhLcQ1NoClQgEzbcJfYkCHXv6DQQEBAOUw9zNl/7FJrWD7rCv0mbOoy6nLlHWiWuyGsA12ohRuAQEBAQEBAQEBAYCBAIAAgA="
			],
			"id": 1
		}}"#
		);
		let result = provider.verify_proof(
			H256::from_str("0x7bf925dbd33af0e00d392b92313da59369ed86c82494d0e02040b24faac0a3ca").unwrap(),
			"05fbffffff17062401010f034736fa564770d2be67c4c069e99e67bdbc978aea4cbdf7f09b4bf919d07a8c7bf200040404040404040403b97ce0716d448e55d75b39f2f03ef02679180ac52e419460b3bdf112da31a6260393049f1d2ea67548e395d3a6850346fc2b94f0f9f6e5b2f5a22b752e13d03d9c03a470bff9771a6adcb51ccbd667be893505743828a1acfe7ac14715725d0c035f0366ddb2164ca79a31da0a1500e323be8c4d21658eaa326844fbba64ec3691d8fa03cc68bc42b5329e9c006f73fe37c50eb48433ad24ba8f7b30d69538ffdc0d9d700359dfe16748fa392c633d04260cc1ba42bcac4cf3f2313671bd99c767f2b55c90040342eefd9168b302445d5780ba52ae2d762f62302fd65c8f3e4b662681e671ca36042901060f0f0f0f0f0f0382015f7e7b15332a807d98e0fb882ef7b37f80aa4eb002bc436eece7ab628d1952000326dba0d8c4e75ffc88c482413fbbf0a844be9d0e7cefb357710a0ff4d1eb0cbe03c68a878ea900a40ce4df9623a70b16236d8ff45ef55439791c7ae0e6d56d05cd04040404040404040404040404040492000403c7d4f9d7fcff0d3a05782879e88300fbc48b03f0abcd8644dc0f1802a3b11cea03d0b35cc3e22812241e0cf634d112afbb950e4f6d31ffe9dd77f32515bed33c6d03fd0dfd6487b6ac5eb62366894a84b710d4da02950804cdb7097d89021d7bfa0d0404040394c3dccd97fec526b583eeb0afd266cea32ea72e51d6896bb21ac035da8851b804040404040404040406020402000200"
		).await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		assert_eq!(
			result.unwrap(),
			"QAFBAighAhY5RqEz49Lg2Yf7kMsBsGDtF4DxcY4too7fE7ll/StgIQA=".to_string()
		);
	}

	#[tokio::test]
	async fn test_get_state_height() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let provider = mock_rpc_response_without_request(
			&mock_server,
			json!({
				"localrootindex": 212,
				"validatedrootindex": 211
			}),
		)
		.await;

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "getstateheight",
			"params": [],
			"id": 1
		}}"#
		);
		let result = provider.get_state_height().await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		let state_height = result.unwrap();
		assert_eq!(state_height.local_root_index, 212);
		assert_eq!(state_height.validated_root_index, 211);
	}

	#[tokio::test]
	async fn test_get_state() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let provider =
			mock_rpc_response_without_request(&mock_server, json!("QQEhBwAA1VhfeRI=")).await;

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "getstate",
			"params": [
				"7bf925dbd33af0e00d392b92313da59369ed86c82494d0e02040b24faac0a3ca", 
				"7c5832ba81fd0af40ec11e96b1c26613466dae02",
				"QQEhB4DRxWFfeRI="
			],
			"id": 1
		}}"#
		);
		let result = provider
			.get_state(
				H256::from_str(
					"0x7bf925dbd33af0e00d392b92313da59369ed86c82494d0e02040b24faac0a3ca",
				)
				.unwrap(),
				H160::from_str("7c5832ba81fd0af40ec11e96b1c26613466dae02").unwrap(),
				"4101210780d1c5615f7912",
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		assert_eq!(result.unwrap(), "QQEhBwAA1VhfeRI=".to_string());
	}

	#[tokio::test]
	async fn test_find_states() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let provider = mock_rpc_response_without_request(
			&mock_server,
			json!({
				"firstProof": "Gfr///8UDRZcmJnDi79ZkcXkewSTcljK7GkIJAEBDwOakA9CYtxDPpx00gKk0RCLmrtNtpTsY2rXB/RqfGHIPLIABAQEBAQEBAMe+jlFz2/5ZKl+ycxczvmS75mO9ssmFZef+WHov7XIHQQDiT6zHh/siCZ0c2bfBEymPmRNTiXSAKFIammjmnnBnJYDh0IX5YfZdqNZkfFN/6VaLZ6kX+N+bBGdlNVUyP7pwJ4DrpFUvhWA+kXVxDLE8qKtLcQimKQY1RcWw14bsjURuRYEBAQDsyA6/WuQyV98xH99kDVz3bhQHmUNBIQqJd0x0R/+TGwEKQEGDw8PDw8PAzKhCJmqIIilFwEfMQJDUEMXInq+AbRk8Jfnoi1weu8aUgADo6udX84sFVzKZLdtwtJ6TIMgQOrYZQ+7yKG+5TlliscDzboXdiwLKASBJeAVtNTl7NHqclD6UBe4XrwJQQYJIDQEBAQEBAQEBAQEBAQEBAQkAQEEA6pd1tKBerO8Qub4cvuKEpXDlGCJsktZ4Vk0xT+D6Av5UgADBr2ExYHjKsB15w2Ra40oWm7iPwdhWEVf6nHV6St/W8gEBAQEBAQDufefqjG8jPxPHOFpyF8LE16aXEzlFeuts4vaQ+wGCL4EBAQEBAQEBARKAScNAQYFDAkICQkMAwgLCw8FCQkBDAUOBAcLAAQJAwcCBQgMCg4MBgkDzXuGD6B7eZe7+IxNOv1j48vZn5A9qz4nzzvdSqSQRr8LAglBASEFACrPdwI=",
				"lastProof": "Gfr///8Uf2XUNDYnCLJV8OBoVr3LXOmdhQUIJAEBDwOakA9CYtxDPpx00gKk0RCLmrtNtpTsY2rXB/RqfGHIPLIABAQEBAQEBAMe+jlFz2/5ZKl+ycxczvmS75mO9ssmFZef+WHov7XIHQQDiT6zHh/siCZ0c2bfBEymPmRNTiXSAKFIammjmnnBnJYDh0IX5YfZdqNZkfFN/6VaLZ6kX+N+bBGdlNVUyP7pwJ4DrpFUvhWA+kXVxDLE8qKtLcQimKQY1RcWw14bsjURuRYEBAQDsyA6/WuQyV98xH99kDVz3bhQHmUNBIQqJd0x0R/+TGwEKQEGDw8PDw8PAzKhCJmqIIilFwEfMQJDUEMXInq+AbRk8Jfnoi1weu8aUgADo6udX84sFVzKZLdtwtJ6TIMgQOrYZQ+7yKG+5TlliscDzboXdiwLKASBJeAVtNTl7NHqclD6UBe4XrwJQQYJIDQEBAQEBAQEBAQEBAQEBAQkAQEEA6pd1tKBerO8Qub4cvuKEpXDlGCJsktZ4Vk0xT+D6Av5UgADBr2ExYHjKsB15w2Ra40oWm7iPwdhWEVf6nHV6St/W8gEBAQEBAQDufefqjG8jPxPHOFpyF8LE16aXEzlFeuts4vaQ+wGCL4EBAQEBAQEBARKAScPBgUNBAMEAwYCBwAICwIFBQ8ADgAGCAUGCw0MCwUMDgkJDQgFAAUDkvma2Sek54h+A0fdKAxoUjETufDdw3bX/Crnad92qPUNAgtBASEHAADVWF95Eg==",
				"truncated": false,
				"results": [
					{
						"key": "FA0WXJiZw4u/WZHF5HsEk3JYyuxp",
						"value": "QQEhBQAqz3cC"
					},
					{
						"key": "FH9l1DQ2JwiyVfDgaFa9y1zpnYUF",
						"value": "QQEhBwAA1VhfeRI="
					}
				]
			})
		).await;

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "findstates",
			"params": [
				"76d6bddf6d9b5979d532877f0617bf31abd03d663c73357dfb2e2417a287b09f", 
				"d2a4cff31913016155e38e474a2c06d08be276cf",
				"C/4=",
				"Cw==",
				2
			],
			"id": 1
		}}"#
		);
		let result = provider
			.find_states(
				H256::from_str(
					"0x76d6bddf6d9b5979d532877f0617bf31abd03d663c73357dfb2e2417a287b09f",
				)
				.unwrap(),
				H160::from_str("0xd2a4cff31913016155e38e474a2c06d08be276cf").unwrap(),
				"0bfe",
				Some("0b"),
				Some(2),
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		let states = result.unwrap();
		let first_proof = "Gfr///8UDRZcmJnDi79ZkcXkewSTcljK7GkIJAEBDwOakA9CYtxDPpx00gKk0RCLmrtNtpTsY2rXB\
		/RqfGHIPLIABAQEBAQEBAMe+jlFz2/5ZKl+ycxczvmS75mO9ssmFZef+WHov7XIHQQDiT6zHh\
		/siCZ0c2bfBEymPmRNTiXSAKFIammjmnnBnJYDh0IX5YfZdqNZkfFN/6VaLZ6kX+N+bBGdlNVUyP7pwJ4DrpFUvhWA\
		+kXVxDLE8qKtLcQimKQY1RcWw14bsjURuRYEBAQDsyA6/WuQyV98xH99kDVz3bhQHmUNBIQqJd0x0R\
		/+TGwEKQEGDw8PDw8PAzKhCJmqIIilFwEfMQJDUEMXInq+AbRk8Jfnoi1weu8aUgADo6udX84sFVzKZLdtwtJ6TIMgQOrYZQ+7yKG\
		+5TlliscDzboXdiwLKASBJeAVtNTl7NHqclD6UBe4XrwJQQYJIDQEBAQEBAQEBAQEBAQEBAQkAQEEA6pd1tKBerO8Qub4cvuKEpXDlGCJsktZ4Vk0xT+D6Av5UgADBr2ExYHjKsB15w2Ra40oWm7iPwdhWEVf6nHV6St/W8gEBAQEBAQDufefqjG8jPxPHOFpyF8LE16aXEzlFeuts4vaQ+wGCL4EBAQEBAQEBARKAScNAQYFDAkICQkMAwgLCw8FCQkBDAUOBAcLAAQJAwcCBQgMCg4MBgkDzXuGD6B7eZe7+IxNOv1j48vZn5A9qz4nzzvdSqSQRr8LAglBASEFACrPdwI=";
		assert_eq!(states.first_proof, Some(first_proof.to_string()));
		let last_proof = "Gfr///8Uf2XUNDYnCLJV8OBoVr3LXOmdhQUIJAEBDwOakA9CYtxDPpx00gKk0RCLmrtNtpTsY2rXB\
		/RqfGHIPLIABAQEBAQEBAMe+jlFz2/5ZKl+ycxczvmS75mO9ssmFZef+WHov7XIHQQDiT6zHh\
		/siCZ0c2bfBEymPmRNTiXSAKFIammjmnnBnJYDh0IX5YfZdqNZkfFN/6VaLZ6kX+N+bBGdlNVUyP7pwJ4DrpFUvhWA\
		+kXVxDLE8qKtLcQimKQY1RcWw14bsjURuRYEBAQDsyA6/WuQyV98xH99kDVz3bhQHmUNBIQqJd0x0R\
		/+TGwEKQEGDw8PDw8PAzKhCJmqIIilFwEfMQJDUEMXInq+AbRk8Jfnoi1weu8aUgADo6udX84sFVzKZLdtwtJ6TIMgQOrYZQ+7yKG\
		+5TlliscDzboXdiwLKASBJeAVtNTl7NHqclD6UBe4XrwJQQYJIDQEBAQEBAQEBAQEBAQEBAQkAQEEA6pd1tKBerO8Qub4cvuKEpXDlGCJsktZ4Vk0xT+D6Av5UgADBr2ExYHjKsB15w2Ra40oWm7iPwdhWEVf6nHV6St/W8gEBAQEBAQDufefqjG8jPxPHOFpyF8LE16aXEzlFeuts4vaQ+wGCL4EBAQEBAQEBARKAScPBgUNBAMEAwYCBwAICwIFBQ8ADgAGCAUGCw0MCwUMDgkJDQgFAAUDkvma2Sek54h+A0fdKAxoUjETufDdw3bX/Crnad92qPUNAgtBASEHAADVWF95Eg==";
		assert_eq!(states.last_proof, Some(last_proof.to_string()));
		assert_eq!(states.truncated, false);
		let results = states.results.clone();
		assert_eq!(results.len(), 2);
		let key1 = "FA0WXJiZw4u/WZHF5HsEk3JYyuxp".to_string();
		let value1 = "QQEhBQAqz3cC".to_string();
		let key2 = "FH9l1DQ2JwiyVfDgaFa9y1zpnYUF".to_string();
		let value2 = "QQEhBwAA1VhfeRI=".to_string();
		let result0 = results.get(0).clone().unwrap();
		assert_eq!(result0.key, key1);
		assert_eq!(result0.value, value1);

		let expected_result1 = StateResult { key: key1, value: value1 };
		let expected_result2 = StateResult { key: key2, value: value2 };
		let expected_results = vec![expected_result1, expected_result2];
		let expected_states = States {
			first_proof: Some(first_proof.to_string()),
			last_proof: Some(last_proof.to_string()),
			truncated: false,
			results: expected_results,
		};
		assert_eq!(states, expected_states);
	}

	#[tokio::test]
	async fn test_find_states_single_result() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let provider = mock_rpc_response_without_request(
			&mock_server,
			json!({
				"firstProof": "Bfr///8LBiQBAQ8DqDawCFNqYkkQC+no3z6WbmuP8DJmy9e4MMK\
				+QzHITdGyAAQEBAQEBAQDHvo5Rc9v+WSpfsnMXM75ku+ZjvbLJhWXn/lh6L+1yB0EA4k\
				+sx4f7IgmdHNm3wRMpj5kTU4l0gChSGppo5p5wZyWA7QRkH8fw1R6WnCQfRWk96ZKPBPSeOU\
				+gvwQuwjznHjfA66RVL4VgPpF1cQyxPKirS3EIpikGNUXFsNeG7I1EbkWBAQEA7MgOv1rkMlffMR\
				/fZA1c924UB5lDQSEKiXdMdEf/kxsBCkBBg8PDw8PDwMJqhMyRWjael2lcsob2BXims/yMjMrrSkkWY\
				/MsReC7lIAAzP6dmF3DTZHkfcXYHO6On6KQucSwUv9UryMqImoBKrLA27ebHC45rpr3EGcLJ7D7EAm\
				/JihcES3pIzYVxgh6hSrBAQEBAQEBAQEBAQEBAQEJAEBCwMWm2J/uEa8sf+ET9RUiBXqOLuLQ\
				/dr4V494mGlwcp9DAkCBwCY0uJieRI=",
				"truncated": true,
				"results": [
					{
						"key": "Cw==",
						"value": "AJjS4mJ5Eg=="
					}
				]
			}),
		)
		.await;

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "findstates",
			"params": [
				"76d6bddf6d9b5979d532877f0617bf31abd03d663c73357dfb2e2417a287b09f", 
				"d2a4cff31913016155e38e474a2c06d08be276cf",
				"C/4=",
				"Cw==",
				2
			],
			"id": 1
		}}"#
		);
		let result = provider
			.find_states(
				H256::from_str(
					"0x76d6bddf6d9b5979d532877f0617bf31abd03d663c73357dfb2e2417a287b09f",
				)
				.unwrap(),
				H160::from_str("0xd2a4cff31913016155e38e474a2c06d08be276cf").unwrap(),
				"0bfe",
				Some("0b"),
				Some(2),
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		let states = result.unwrap();
		let first_proof = "Bfr///8LBiQBAQ8DqDawCFNqYkkQC+no3z6WbmuP8DJmy9e4MMK\
				+QzHITdGyAAQEBAQEBAQDHvo5Rc9v+WSpfsnMXM75ku+ZjvbLJhWXn/lh6L+1yB0EA4k\
				+sx4f7IgmdHNm3wRMpj5kTU4l0gChSGppo5p5wZyWA7QRkH8fw1R6WnCQfRWk96ZKPBPSeOU\
				+gvwQuwjznHjfA66RVL4VgPpF1cQyxPKirS3EIpikGNUXFsNeG7I1EbkWBAQEA7MgOv1rkMlffMR\
				/fZA1c924UB5lDQSEKiXdMdEf/kxsBCkBBg8PDw8PDwMJqhMyRWjael2lcsob2BXims/yMjMrrSkkWY\
				/MsReC7lIAAzP6dmF3DTZHkfcXYHO6On6KQucSwUv9UryMqImoBKrLA27ebHC45rpr3EGcLJ7D7EAm\
				/JihcES3pIzYVxgh6hSrBAQEBAQEBAQEBAQEBAQEJAEBCwMWm2J/uEa8sf+ET9RUiBXqOLuLQ\
				/dr4V494mGlwcp9DAkCBwCY0uJieRI=";
		assert_eq!(states.first_proof, Some(first_proof.to_string()));
		assert!(states.last_proof.is_none());
		assert_eq!(states.results.len(), 1);
	}

	#[tokio::test]
	async fn test_find_states_empty_results() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let provider = mock_rpc_response_without_request(
			&mock_server,
			json!({
				"truncated": true,
				"results": []
			}),
		)
		.await;

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "findstates",
			"params": [
				"76d6bddf6d9b5979d532877f0617bf31abd03d663c73357dfb2e2417a287b09f", 
				"d2a4cff31913016155e38e474a2c06d08be276cf",
				"C/4=",
				"Cw==",
				2
			],
			"id": 1
		}}"#
		);
		let result = provider
			.find_states(
				H256::from_str(
					"0x76d6bddf6d9b5979d532877f0617bf31abd03d663c73357dfb2e2417a287b09f",
				)
				.unwrap(),
				H160::from_str("0xd2a4cff31913016155e38e474a2c06d08be276cf").unwrap(),
				"0bfe",
				Some("0b"),
				Some(2),
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		let states = result.unwrap();
		assert!(states.first_proof.is_none());
		assert!(states.last_proof.is_none());
		assert_eq!(states.truncated, true);
		assert_eq!(states.results.len(), 0);
	}

	#[tokio::test]
	async fn test_find_states_nocount() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url).unwrap();
		let provider = RpcClient::new(http_client);

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "findstates",
			"params": [
				"76d6bddf6d9b5979d532877f0617bf31abd03d663c73357dfb2e2417a287b09f", 
				"d2a4cff31913016155e38e474a2c06d08be276cf",
				"C/4=",
				"Cw=="
			],
			"id": 1
		}}"#
		);
		let _ = provider
			.find_states(
				H256::from_str(
					"0x76d6bddf6d9b5979d532877f0617bf31abd03d663c73357dfb2e2417a287b09f",
				)
				.unwrap(),
				H160::from_str("0xd2a4cff31913016155e38e474a2c06d08be276cf").unwrap(),
				"0bfe",
				Some("0b"),
				None,
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_find_states_nostartkey_withcount() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url).unwrap();
		let provider = RpcClient::new(http_client);

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "findstates",
			"params": [
				"76d6bddf6d9b5979d532877f0617bf31abd03d663c73357dfb2e2417a287b09f", 
				"d2a4cff31913016155e38e474a2c06d08be276cf",
				"C/4=",
				"",
				53
			],
			"id": 1
		}}"#
		);
		let _ = provider
			.find_states(
				H256::from_str(
					"0x76d6bddf6d9b5979d532877f0617bf31abd03d663c73357dfb2e2417a287b09f",
				)
				.unwrap(),
				H160::from_str("0xd2a4cff31913016155e38e474a2c06d08be276cf").unwrap(),
				"0bfe",
				None,
				Some(53),
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_find_states_nostartkey_nocount() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url).unwrap();
		let provider = RpcClient::new(http_client);

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "findstates",
			"params": [
				"76d6bddf6d9b5979d532877f0617bf31abd03d663c73357dfb2e2417a287b09f", 
				"d2a4cff31913016155e38e474a2c06d08be276cf",
				"C/4="
			],
			"id": 1
		}}"#
		);
		let _ = provider
			.find_states(
				H256::from_str(
					"0x76d6bddf6d9b5979d532877f0617bf31abd03d663c73357dfb2e2417a287b09f",
				)
				.unwrap(),
				H160::from_str("0xd2a4cff31913016155e38e474a2c06d08be276cf").unwrap(),
				"0bfe",
				None,
				None,
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	// Neo-express related tests
	// #[tokio::test]
	// async fn test_express_get_populated_blocks() {
	//     // Access the global mock server
	//     let mock_server = setup_mock_server().await;

	// 	let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
	// 	let http_client = HttpProvider::new(url).unwrap();
	// 	let provider = RpcClient::new(http_client);

	//     // Expected request body
	// 	let expected_request_body = format!(r#"{{
	// 		"jsonrpc": "2.0",
	// 		"method": "expressgetpopulatedblocks",
	// 		"params": [],
	// 		"id": 1
	// 	}}"#);
	//     let _ = provider.express().await;

	//     verify_request(&mock_server, &expected_request_body).await.unwrap();
	// }

	// TokenTracker: Nep11
	#[tokio::test]
	async fn test_get_nep11_balance() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let provider = mock_rpc_response_without_request(
			&mock_server,
			json!({
				"address": "NXXazKH39yNFWWZF5MJ8tEN98VYHwzn7g3",
				"balance": [
					{
						"assethash": "a48b6e1291ba24211ad11bb90ae2a10bf1fcd5a8",
						"name": "FunnyCats",
						"symbol": "FCS",
						"decimals": "0",
						"tokens": [
							{
								"tokenid": "1",
								"amount": "1",
								"lastupdatedblock": 12345
							},
							{
								"tokenid": "2",
								"amount": "1",
								"lastupdatedblock": 123456
							}
						]
					},
					{
						"assethash": "1aada0032aba1ef6d1f07bbd8bec1d85f5380fb3",
						"name": "CuteNeoKittens",
						"symbol": "CNKS",
						"decimals": "4",
						"tokens": [
							{
								"tokenid": "4",
								"amount": "10000",
								"lastupdatedblock": 12345
							},
							{
								"tokenid": "10",
								"amount": "6500",
								"lastupdatedblock": 654321
							}
						]
					}
				]
			}),
		)
		.await;

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "getnep11balances",
			"params": [
				"NY9zhKwcmht5cQJ3oRqjJGo3QuVLwXwTzL"
			],
			"id": 1
		}}"#
		);
		let result = provider
			.get_nep11_balances(H160::from_str("5d75775015b024970bfeacf7c6ab1b0ade974886").unwrap())
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		let nep11_balances = result.unwrap();
		assert_eq!(nep11_balances.address, "NXXazKH39yNFWWZF5MJ8tEN98VYHwzn7g3".to_string());
		let nep11_balances_list = nep11_balances.balances;
		assert_eq!(nep11_balances_list.len(), 2);
		assert_eq!(
			nep11_balances_list,
			vec![
				Nep11Balance::new(
					H160::from_str("a48b6e1291ba24211ad11bb90ae2a10bf1fcd5a8").unwrap(),
					"FunnyCats".to_string(),
					"FCS".to_string(),
					"0".to_string(),
					vec![
						Nep11Token::new("1".to_string(), "1".to_string(), 12345),
						Nep11Token::new("2".to_string(), "1".to_string(), 123456)
					]
				),
				Nep11Balance::new(
					H160::from_str("1aada0032aba1ef6d1f07bbd8bec1d85f5380fb3").unwrap(),
					"CuteNeoKittens".to_string(),
					"CNKS".to_string(),
					"4".to_string(),
					vec![
						Nep11Token::new("4".to_string(), "10000".to_string(), 12345),
						Nep11Token::new("10".to_string(), "6500".to_string(), 654321)
					]
				)
			]
		)
	}

	#[tokio::test]
	async fn test_get_nep11_transfers() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let provider = mock_rpc_response_without_request(
			&mock_server,
			json!({
				"sent": [
					{
						"tokenid": "1",
						"timestamp": 1554283931,
						"assethash": "1aada0032aba1ef6d1f07bbd8bec1d85f5380fb3",
						"transferaddress": "AYwgBNMepiv5ocGcyNT4mA8zPLTQ8pDBis",
						"amount": "100000000000",
						"blockindex": 368082,
						"transfernotifyindex": 0,
						"txhash": "240ab1369712ad2782b99a02a8f9fcaa41d1e96322017ae90d0449a3ba52a564"
					},
					{
						"tokenid": "2",
						"timestamp": 1554880287,
						"assethash": "1aada0032aba1ef6d1f07bbd8bec1d85f5380fb3",
						"transferaddress": "AYwgBNMepiv5ocGcyNT4mA8zPLTQ8pDBis",
						"amount": "100000000000",
						"blockindex": 397769,
						"transfernotifyindex": 0,
						"txhash": "12fdf7ce8b2388d23ab223854cb29e5114d8288c878de23b7924880f82dfc834"
					}
				],
				"received": [
					{
						"tokenid": "3",
						"timestamp": 1555651816,
						"assethash": "600c4f5200db36177e3e8a09e9f18e2fc7d12a0f",
						"transferaddress": "AYwgBNMepiv5ocGcyNT4mA8zPLTQ8pDBis",
						"amount": "1000000",
						"blockindex": 436036,
						"transfernotifyindex": 0,
						"txhash": "df7683ece554ecfb85cf41492c5f143215dd43ef9ec61181a28f922da06aba58"
					}
				],
				"address": "AbHgdBaWEnHkCiLtDZXjhvhaAK2cwFh5pF"
			}),
		)
		.await;

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "getnep11transfers",
			"params": [
				"NekZLTu93WgrdFHxzBEJUYgLTQMAT85GLi"
			],
			"id": 1
		}}"#
		);
		let result = provider
			.get_nep11_transfers(
				H160::from_str("04457ce4219e462146ac00b09793f81bc5bca2ce").unwrap(),
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();

		assert!(result.is_ok(), "Result is not okay: {:?}", result);
		let transfers = result.unwrap();
		let sent = transfers.sent;
		assert_eq!(sent.len(), 2);
		assert_eq!(
			sent,
			vec![
				Nep11Transfer::new(
					1554283931,
					H160::from_str("1aada0032aba1ef6d1f07bbd8bec1d85f5380fb3").unwrap(),
					"AYwgBNMepiv5ocGcyNT4mA8zPLTQ8pDBis".to_string(),
					100000000000,
					368082,
					0,
					H256::from_str(
						"240ab1369712ad2782b99a02a8f9fcaa41d1e96322017ae90d0449a3ba52a564"
					)
					.unwrap(),
					"1".to_string()
				),
				Nep11Transfer::new(
					1554880287,
					H160::from_str("1aada0032aba1ef6d1f07bbd8bec1d85f5380fb3").unwrap(),
					"AYwgBNMepiv5ocGcyNT4mA8zPLTQ8pDBis".to_string(),
					100000000000,
					397769,
					0,
					H256::from_str(
						"12fdf7ce8b2388d23ab223854cb29e5114d8288c878de23b7924880f82dfc834"
					)
					.unwrap(),
					"2".to_string()
				)
			]
		);

		let received = transfers.received;
		assert_eq!(received.len(), 1);
		assert_eq!(
			received,
			vec![Nep11Transfer::new(
				1555651816,
				H160::from_str("600c4f5200db36177e3e8a09e9f18e2fc7d12a0f").unwrap(),
				"AYwgBNMepiv5ocGcyNT4mA8zPLTQ8pDBis".to_string(),
				1000000,
				436036,
				0,
				H256::from_str("df7683ece554ecfb85cf41492c5f143215dd43ef9ec61181a28f922da06aba58")
					.unwrap(),
				"3".to_string()
			)]
		);
	}

	#[tokio::test]
	async fn test_get_nep11_transfers_date() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url).unwrap();
		let provider = RpcClient::new(http_client);

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "getnep11transfers",
			"params": [
				"NSH1UeM96PKhjuzVBKcyWeNNuQkT3sHGmA",
				1553105830
			],
			"id": 1
		}}"#
		);
		let _ = provider
			.get_nep11_transfers_from(
				H160::from_str("8bed27d0e88266807a6339270f0593510967cb45").unwrap(),
				1553105830,
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_nep11_transfers_from_to() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let url = Url::parse(&mock_server.uri()).expect("Invalid mock server URL");
		let http_client = HttpProvider::new(url).unwrap();
		let provider = RpcClient::new(http_client);

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "getnep11transfers",
			"params": [
				"NfWL3Kx7qtZzXrajmggAD4b6r2kGzajbaJ",
				1553105830,
				1557305830
			],
			"id": 1
		}}"#
		);
		let _ = provider
			.get_nep11_transfers_range(
				H160::from_str("2eeda865e7824c71b3fe14bed35d04d0f2f0e9d6").unwrap(),
				1553105830,
				1557305830,
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();
	}

	#[tokio::test]
	async fn test_get_nep11_properties() {
		// Access the global mock server
		let mock_server = setup_mock_server().await;

		let provider = mock_rpc_response_without_request(
			&mock_server,
			json!({
				"keyProp1": "valueProp1",
				"keyProp2": "valueProp2"
			}),
		)
		.await;

		// Expected request body
		let expected_request_body = format!(
			r#"{{
			"jsonrpc": "2.0",
			"method": "getnep11properties",
			"params": [
				"NfWL3Kx7qtZzXrajmggAD4b6r2kGzajbaJ",
				"12345"
			],
			"id": 1
		}}"#
		);
		let result = provider
			.get_nep11_properties(
				H160::from_str("2eeda865e7824c71b3fe14bed35d04d0f2f0e9d6").unwrap(),
				"12345",
			)
			.await;

		verify_request(&mock_server, &expected_request_body).await.unwrap();

		assert!(result.is_ok(), "Result is not okay: {:?}", result);

		let properties = result.unwrap();
		assert_eq!(properties.len(), 2);
		assert_eq!(properties.get("keyProp1"), Some(&"valueProp1".to_string()));
		assert_eq!(properties.get("keyProp2"), Some(&"valueProp2".to_string()));
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
