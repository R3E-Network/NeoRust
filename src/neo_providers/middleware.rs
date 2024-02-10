use async_trait::async_trait;
use auto_impl::auto_impl;
use neo::prelude::*;
use primitive_types::{H160, H256};
use std::{collections::HashMap, fmt::Debug};

#[async_trait]
#[auto_impl(&, Box, Arc)]
pub trait Middleware: Sync + Send + Debug {
	/// Error type returned by most operations
	type Error: MiddlewareError<Inner = <<Self as Middleware>::Inner as Middleware>::Error>;
	/// The JSON-RPC client type at the bottom of the stack
	type Provider: JsonRpcClient;
	/// The next-lower middleware in the middleware stack
	type Inner: Middleware<Provider = Self::Provider>;

	/// Get a reference to the next-lower middleware in the middleware stack
	fn inner(&self) -> &Self::Inner;

	/// Convert a provider error into the associated error type by successively
	/// converting it to every intermediate middleware error
	fn convert_err(p: ProviderError) -> Self::Error {
		Self::Error::from_provider_err(p)
	}

	/// The HTTP or Websocket provider.
	fn provider(&self) -> &Provider<Self::Provider> {
		self.inner().provider()
	}

	fn config(&self) -> &NeoConfig {
		&self.inner().config()
	}

	/// Return the default sender (if any). This will typically be the
	/// connected node's first address, or the address of a Signer in a lower
	/// middleware stack
	fn default_sender(&self) -> Option<Address> {
		self.inner().default_sender()
	}

	/// Returns the current client version using the `web3_clientVersion` RPC.
	async fn client_version(&self) -> Result<String, Self::Error> {
		self.inner().client_version().await.map_err(MiddlewareError::from_err)
	}

	async fn fill_transaction(&self, tx: &mut Transaction) -> Result<(), Self::Error> {
		self.inner().fill_transaction(tx).await.map_err(MiddlewareError::from_err)
	}

	/// Get the block number
	async fn get_block_number(&self) -> Result<u64, Self::Error> {
		self.inner().get_block_number().await.map_err(MiddlewareError::from_err)
	}

	/// Sends the transaction to the entire Neo network and returns the
	/// transaction's hash. This will consume gas from the account that signed
	/// the transaction. This call will fail if no signer is available, and the
	/// RPC node does  not have an unlocked accounts
	async fn send_transaction<T: Into<Transaction> + Send + Sync>(
		&self,
		tx: T,
	) -> Result<PendingTransaction<'_, Self::Provider>, Self::Error> {
		self.inner().send_transaction(tx).await.map_err(MiddlewareError::from_err)
	}

	////// Neo Naming Service
	// The Neo Naming Service (NNS) allows easy to remember and use names to
	// be assigned to Neo addresses. Any provider operation which takes an address
	// may also take an NNS name.
	//
	// NNS also provides the ability for a reverse lookup, which determines the name for an address
	// if it has been configured.

	/// Returns the address that the `nns_name` resolves to (or None if not configured).
	///
	/// # Panics
	///
	/// If the bytes returned from the NNS registrar/resolver cannot be interpreted as
	/// an address. This should theoretically never happen.
	async fn resolve_name(&self, nns_name: &str) -> Result<Address, Self::Error> {
		self.inner().resolve_name(nns_name).await.map_err(MiddlewareError::from_err)
	}

	/// Returns the NNS name the `address` resolves to (or None if not configured).
	///
	/// # Panics
	///
	/// If the bytes returned from the NNS registrar/resolver cannot be interpreted as
	/// a string. This should theoretically never happen.
	async fn lookup_address(&self, address: Address) -> Result<String, Self::Error> {
		self.inner().lookup_address(address).await.map_err(MiddlewareError::from_err)
	}

	/// Fetch a field for the `nns_name` (no None if not configured).
	///
	/// # Panics
	///
	/// If the bytes returned from the NNS registrar/resolver cannot be interpreted as
	/// a string. This should theoretically never happen.
	async fn resolve_field(&self, nns_name: &str, field: &str) -> Result<String, Self::Error> {
		self.inner()
			.resolve_field(nns_name, field)
			.await
			.map_err(MiddlewareError::from_err)
	}

	/// Gets the block at `block_hash_or_number` (full transactions included)
	async fn get_block_with_txs<T: Into<BlockId> + Send + Sync>(
		&self,
		block_hash_or_number: T,
	) -> Result<Option<Block<TransactionResult, Witness>>, Self::Error> {
		self.inner()
			.get_block_with_txs(block_hash_or_number)
			.await
			.map_err(MiddlewareError::from_err)
	}

	/// Sends the read-only (constant) transaction to a single Neo node and return the result
	/// (as bytes) of executing it. This is free, since it does not change any state on the
	/// blockchain.
	async fn call(&self, tx: &Transaction, block: Option<BlockId>) -> Result<Bytes, Self::Error> {
		self.inner().call(tx, block).await.map_err(MiddlewareError::from_err)
	}

	/// Return current client syncing status. If IsFalse sync is over.
	async fn syncing(&self) -> Result<SyncingStatus, Self::Error> {
		self.inner().syncing().await.map_err(MiddlewareError::from_err)
	}

	/// Returns the currently configured network magic, a value used in replay-protected
	/// transaction signing as introduced by EIP-155.
	async fn get_network_magic(&self) -> Result<u32, Self::Error> {
		self.inner().get_network_magic().await.map_err(MiddlewareError::from_err)
	}

	/// Returns the network version.
	async fn get_net_version(&self) -> Result<String, Self::Error> {
		self.inner().get_net_version().await.map_err(MiddlewareError::from_err)
	}

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

	async fn dump_private_key(&self, script_hash: H160) -> Result<String, Self::Error> {
		self.inner()
			.dump_private_key(script_hash)
			.await
			.map_err(MiddlewareError::from_err)
	}

	// Blockchain methods
	async fn get_best_block_hash(&self) -> Result<H256, Self::Error> {
		self.inner().get_best_block_hash().await.map_err(MiddlewareError::from_err)
	}

	async fn get_block_hash(&self, block_index: u32) -> Result<H256, Self::Error> {
		self.inner()
			.get_block_hash(block_index)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_block(&self, block_hash: H256, full_tx: bool) -> Result<NeoBlock, Self::Error> {
		self.inner()
			.get_block(block_hash, full_tx)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_raw_block(&self, block_hash: H256) -> Result<String, Self::Error> {
		self.inner().get_raw_block(block_hash).await.map_err(MiddlewareError::from_err)
	}

	// Node methods
	async fn get_block_header_count(&self) -> Result<u32, Self::Error> {
		self.inner().get_block_count().await.map_err(MiddlewareError::from_err)
	}

	async fn get_block_count(&self) -> Result<u32, Self::Error> {
		self.inner().get_block_count().await.map_err(MiddlewareError::from_err)
	}

	async fn get_block_header(&self, block_hash: H256) -> Result<NeoBlock, Self::Error> {
		self.inner()
			.get_block_header(block_hash)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_block_header_by_index(&self, index: u32) -> Result<NeoBlock, Self::Error> {
		self.inner()
			.get_block_header_by_index(index)
			.await
			.map_err(MiddlewareError::from_err)
	}

	// Smart contract methods

	async fn get_raw_block_header(&self, block_hash: H256) -> Result<String, Self::Error> {
		self.inner()
			.get_raw_block_header(block_hash)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_raw_block_header_by_index(&self, index: u32) -> Result<String, Self::Error> {
		self.inner()
			.get_raw_block_header_by_index(index)
			.await
			.map_err(MiddlewareError::from_err)
	}

	// Utility methods

	async fn get_native_contracts(&self) -> Result<Vec<NativeContractState>, Self::Error> {
		self.inner().get_native_contracts().await.map_err(MiddlewareError::from_err)
	}

	// Wallet methods

	async fn get_contract_state(&self, hash: H160) -> Result<ContractState, Self::Error> {
		self.inner().get_contract_state(hash).await.map_err(MiddlewareError::from_err)
	}

	async fn get_native_contract_state(&self, name: &str) -> Result<ContractState, Self::Error> {
		self.inner()
			.get_native_contract_state(name)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_mem_pool(&self) -> Result<MemPoolDetails, Self::Error> {
		self.inner().get_mem_pool().await.map_err(MiddlewareError::from_err)
	}

	async fn get_raw_mem_pool(&self) -> Result<Vec<H256>, Self::Error> {
		self.inner().get_raw_mem_pool().await.map_err(MiddlewareError::from_err)
	}

	// Application logs

	async fn get_transaction(&self, hash: H256) -> Result<Option<TransactionResult>, Self::Error> {
		self.inner().get_transaction(hash).await.map_err(MiddlewareError::from_err)
	}

	// State service

	async fn get_raw_transaction(&self, tx_hash: H256) -> Result<RawTransaction, Self::Error> {
		self.inner()
			.get_raw_transaction(tx_hash)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_storage(&self, contract_hash: H160, key: &str) -> Result<String, Self::Error> {
		self.inner()
			.get_storage(contract_hash, key)
			.await
			.map_err(MiddlewareError::from_err)
	}
	// Blockchain methods

	async fn get_transaction_height(&self, tx_hash: H256) -> Result<u32, Self::Error> {
		self.inner()
			.get_transaction_height(tx_hash)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_next_block_validators(&self) -> Result<Vec<Validator>, Self::Error> {
		self.inner()
			.get_next_block_validators()
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_committee(&self) -> Result<Vec<String>, Self::Error> {
		self.inner().get_committee().await.map_err(MiddlewareError::from_err)
	}

	async fn get_connection_count(&self) -> Result<u32, Self::Error> {
		self.inner().get_connection_count().await.map_err(MiddlewareError::from_err)
	}

	async fn get_peers(&self) -> Result<Peers, Self::Error> {
		self.inner().get_peers().await.map_err(MiddlewareError::from_err)
	}

	// Smart contract method
	async fn get_version(&self) -> Result<NeoVersion, Self::Error> {
		self.inner().get_version().await.map_err(MiddlewareError::from_err)
	}

	async fn send_raw_transaction(&self, hex: String) -> Result<RawTransaction, Self::Error> {
		self.inner().send_raw_transaction(hex).await.map_err(MiddlewareError::from_err)
	}

	async fn submit_block(&self, hex: String) -> Result<bool, Self::Error> {
		self.inner().submit_block(hex).await.map_err(MiddlewareError::from_err)
	}

	// Blockchain methods
	async fn invoke_function(
		&self,
		contract_hash: &H160,
		method: String,
		params: Vec<ContractParameter>,
		signers: Option<Vec<Signer>>,
	) -> Result<InvocationResult, Self::Error> {
		self.inner()
			.invoke_function(contract_hash, method, params, signers)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn invoke_script(
		&self,
		hex: String,
		signers: Vec<Signer>,
	) -> Result<InvocationResult, Self::Error> {
		self.inner()
			.invoke_script(hex, signers)
			.await
			.map_err(MiddlewareError::from_err)
	}

	// More smart contract methods

	async fn get_unclaimed_gas(&self, hash: H160) -> Result<UnclaimedGas, Self::Error> {
		self.inner().get_unclaimed_gas(hash).await.map_err(MiddlewareError::from_err)
	}

	async fn list_plugins(&self) -> Result<Vec<Plugin>, Self::Error> {
		self.inner().list_plugins().await.map_err(MiddlewareError::from_err)
	}

	async fn validate_address(&self, address: &str) -> Result<ValidateAddress, Self::Error> {
		self.inner().validate_address(address).await.map_err(MiddlewareError::from_err)
	}

	// Wallet methods
	async fn close_wallet(&self) -> Result<bool, Self::Error> {
		self.inner().close_wallet().await.map_err(MiddlewareError::from_err)
	}

	async fn dump_priv_key(&self, script_hash: H160) -> Result<String, Self::Error> {
		self.inner().dump_priv_key(script_hash).await.map_err(MiddlewareError::from_err)
	}

	async fn get_wallet_balance(&self, token_hash: H160) -> Result<Balance, Self::Error> {
		self.inner()
			.get_wallet_balance(token_hash)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_new_address(&self) -> Result<String, Self::Error> {
		self.inner().get_new_address().await.map_err(MiddlewareError::from_err)
	}

	async fn get_wallet_unclaimed_gas(&self) -> Result<String, Self::Error> {
		self.inner().get_wallet_unclaimed_gas().await.map_err(MiddlewareError::from_err)
	}

	async fn import_priv_key(&self, priv_key: String) -> Result<NeoAddress, Self::Error> {
		self.inner().import_priv_key(priv_key).await.map_err(MiddlewareError::from_err)
	}

	async fn calculate_network_fee(&self, hex: String) -> Result<u64, Self::Error> {
		self.inner().calculate_network_fee(hex).await.map_err(MiddlewareError::from_err)
	}

	async fn list_address(&self) -> Result<Vec<NeoAddress>, Self::Error> {
		self.inner().list_address().await.map_err(MiddlewareError::from_err)
	}
	async fn open_wallet(&self, path: String, password: String) -> Result<bool, Self::Error> {
		self.inner()
			.open_wallet(path, password)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn send_from(
		&self,
		token_hash: H160,
		from: Address,
		to: Address,
		amount: u32,
	) -> Result<Transaction, Self::Error> {
		self.inner()
			.send_from(token_hash, from, to, amount)
			.await
			.map_err(MiddlewareError::from_err)
	}

	// Transaction methods

	async fn send_many(
		&self,
		from: Option<H160>,
		send_tokens: Vec<TransactionSendToken>,
	) -> Result<Transaction, Self::Error> {
		self.inner()
			.send_many(from, send_tokens)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn send_to_address(
		&self,
		token_hash: H160,
		to: Address,
		amount: u32,
	) -> Result<Transaction, Self::Error> {
		self.inner()
			.send_to_address(token_hash, to, amount)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_application_log(&self, tx_hash: H256) -> Result<ApplicationLog, Self::Error> {
		self.inner()
			.get_application_log(tx_hash)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_nep17_balances(&self, script_hash: H160) -> Result<Nep17Balances, Self::Error> {
		self.inner()
			.get_nep17_balances(script_hash)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_nep17_transfers(&self, script_hash: H160) -> Result<Nep17Transfers, Self::Error> {
		self.inner()
			.get_nep17_transfers(script_hash)
			.await
			.map_err(MiddlewareError::from_err)
	}

	// NEP-17 methods

	async fn get_nep17_transfers_from(
		&self,
		script_hash: H160,
		from: u64,
	) -> Result<Nep17Transfers, Self::Error> {
		self.inner()
			.get_nep17_transfers_from(script_hash, from)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_nep17_transfers_range(
		&self,
		script_hash: H160,
		from: u64,
		to: u64,
	) -> Result<Nep17Transfers, Self::Error> {
		self.inner()
			.get_nep17_transfers_range(script_hash, from, to)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_nep11_balances(&self, script_hash: H160) -> Result<Nep11Balances, Self::Error> {
		self.inner()
			.get_nep11_balances(script_hash)
			.await
			.map_err(MiddlewareError::from_err)
	}

	// NEP-11 methods

	async fn get_nep11_transfers(&self, script_hash: H160) -> Result<Nep11Transfers, Self::Error> {
		self.inner()
			.get_nep11_transfers(script_hash)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_nep11_transfers_from(
		&self,
		script_hash: H160,
		from: u64,
	) -> Result<Nep11Transfers, Self::Error> {
		self.inner()
			.get_nep11_transfers_from(script_hash, from)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_nep11_transfers_range(
		&self,
		script_hash: H160,
		from: u64,
		to: u64,
	) -> Result<Nep11Transfers, Self::Error> {
		self.inner()
			.get_nep11_transfers_range(script_hash, from, to)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_nep11_properties(
		&self,
		script_hash: H160,
		token_id: &str,
	) -> Result<HashMap<String, String>, Self::Error> {
		self.inner()
			.get_nep11_properties(script_hash, token_id)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_state_root(&self, block_index: u32) -> Result<StateRoot, Self::Error> {
		self.inner()
			.get_state_root(block_index)
			.await
			.map_err(MiddlewareError::from_err)
	}

	// State service methods
	async fn get_proof(
		&self,
		root_hash: H256,
		contract_hash: H160,
		key: &str,
	) -> Result<String, Self::Error> {
		self.inner()
			.get_proof(root_hash, contract_hash, key)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn verify_proof(&self, root_hash: H256, proof: &str) -> Result<bool, Self::Error> {
		self.inner()
			.verify_proof(root_hash, proof)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_state_height(&self) -> Result<StateHeight, Self::Error> {
		self.inner().get_state_height().await.map_err(MiddlewareError::from_err)
	}

	async fn get_state(
		&self,
		root_hash: H256,
		contract_hash: H160,
		key: &str,
	) -> Result<String, Self::Error> {
		self.inner()
			.get_state(root_hash, contract_hash, key)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn find_states(
		&self,
		root_hash: H256,
		contract_hash: H160,
		key_prefix: &str,
		start_key: Option<&str>,
		count: Option<u32>,
	) -> Result<States, Self::Error> {
		self.inner()
			.find_states(root_hash, contract_hash, key_prefix, start_key, count)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_block_by_index(&self, index: u32, full_tx: bool) -> Result<NeoBlock, Self::Error> {
		self.inner()
			.get_block_by_index(index, full_tx)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_raw_block_by_index(&self, index: u32) -> Result<String, Self::Error> {
		self.inner()
			.get_raw_block_by_index(index)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn invoke_function_diagnostics(
		&self,
		contract_hash: H160,
		name: String,
		params: Vec<ContractParameter>,
		signers: Vec<Signer>,
	) -> Result<InvocationResult, Self::Error> {
		self.inner()
			.invoke_function_diagnostics(contract_hash, name, params, signers)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn invoke_script_diagnostics(
		&self,
		hex: String,
		signers: Vec<Signer>,
	) -> Result<InvocationResult, Self::Error> {
		self.inner()
			.invoke_script_diagnostics(hex, signers)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn traverse_iterator(
		&self,
		session_id: String,
		iterator_id: String,
		count: u32,
	) -> Result<Vec<StackItem>, Self::Error> {
		self.inner()
			.traverse_iterator(session_id, iterator_id, count)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn terminate_session(&self, session_id: &str) -> Result<bool, Self::Error> {
		self.inner()
			.terminate_session(session_id)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn invoke_contract_verify(
		&self,
		hash: H160,
		params: Vec<ContractParameter>,
		signers: Vec<Signer>,
	) -> Result<InvocationResult, Self::Error> {
		self.inner()
			.invoke_contract_verify(hash, params, signers)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn get_raw_mempool(&self) -> Result<MemPoolDetails, Self::Error> {
		self.inner().get_raw_mempool().await.map_err(MiddlewareError::from_err)
	}

	async fn import_private_key(&self, wif: String) -> Result<NeoAddress, Self::Error> {
		self.inner().import_private_key(wif).await.map_err(MiddlewareError::from_err)
	}

	async fn get_block_header_hash(&self, hash: H256) -> Result<NeoBlock, Self::Error> {
		self.inner()
			.get_block_header_hash(hash)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn send_to_address_send_token(
		&self,
		send_token: &TransactionSendToken,
	) -> Result<Transaction, Self::Error> {
		self.inner()
			.send_to_address_send_token(send_token)
			.await
			.map_err(MiddlewareError::from_err)
	}

	async fn send_from_send_token(
		&self,
		send_token: &TransactionSendToken,
		from: Address,
	) -> Result<Transaction, Self::Error> {
		self.inner()
			.send_from_send_token(send_token, from)
			.await
			.map_err(MiddlewareError::from_err)
	}
}
