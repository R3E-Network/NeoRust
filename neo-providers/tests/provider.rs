mod neo_tests {
	use lazy_static::lazy_static;
	use primitive_types::H160;
	use rustc_serialize::hex::FromHex;
	use neo_providers::{core::transaction::transaction::Transaction, Middleware, Provider};
	use neo_types::{block::BlockId, H256};
	use neo_types::address::Address;
	use neo_types::script_hash::ScriptHashExtension;

	lazy_static!(
		static ref NeoProvider: Provider<Http> = Provider::<Http>::try_from("http://localhost:30333").unwrap();
	);
	)
	#[tokio::test]
	async fn non_existing_data_works() {

		assert!(NeoProvider.get_transaction(H256::zero()).await.unwrap().is_none());
		assert!(provider.get_transaction(H256::zero()).await.unwrap().is_none());
		assert!(provider.get_block(H256::zero(), false).await.unwrap().is_none());
		assert!(provider
			.get_block_with_txs(BlockId::Hash(H256::zero()))
			.await
			.unwrap()
			.is_none());
	}

	#[tokio::test]
	async fn client_version() {
		let provider = GOERLI.provider();

		assert!(provider
			.client_version()
			.await
			.expect("Could not make web3_clientVersion call to provider")
			.starts_with("Geth/v"));
	}

	// Without TLS this would error with "TLS Support not compiled in"
	#[tokio::test]
	#[cfg(all(feature = "ws", any(feature = "openssl", feature = "rustls")))]
	async fn ssl_websocket() {
		let provider = GOERLI.ws().await;
		assert_ne!(provider.get_block_number().await.unwrap(), 0.into());
	}

	#[tokio::test]
	#[cfg(feature = "ws")]
	async fn send_tx_ws() {
		let (provider, anvil) = crate::spawn_anvil_ws().await;
		generic_send_tx_test(provider, anvil.addresses()[0]).await;
	}

	#[tokio::test]
	#[cfg(feature = "ipc")]
	async fn send_tx_ipc() {
		let (provider, anvil, _ipc) = crate::spawn_anvil_ipc().await;
		generic_send_tx_test(provider, anvil.addresses()[0]).await;
	}

	async fn generic_send_tx_test<M: Middleware>(provider: M, who: Address) {
		let tx = Transaction::new().to(who).from(who);
		let pending_tx = provider.send_transaction(tx).await.unwrap();
		let tx_hash = *pending_tx;
		let receipt = pending_tx.confirmations(3).await.unwrap().unwrap();
		assert_eq!(receipt.hash, tx_hash);
	}

	// MARK: Blockchain Methods
	#[test]
	fn test_get_best_block_hash() {
		let expected = r#"{"jsonrpc":"2.0","method":"getbestblockhash","id":1,"params":[]}"#;
		let neo_swift = Provider::new();
		let request = neo_swift.get_best_block_hash();
		verify_request(expected, request);
	}

	#[test]
	fn test_get_block_hash() {
		let expected = r#"{"jsonrpc":"2.0","method":"getblockhash","id":1,"params":[16293]}"#;
		let neo_swift = NeoSwift::default();
		let request = neo_swift.get_block_hash(16293);
		verify_request(expected, request);
	}

	#[test]
	fn test_get_block_index() {
		let expected = r#"{"jsonrpc":"2.0","method":"getblock","id":1,"params":[12345,1]}"#;
		let neo_swift = NeoSwift::default();
		let request = neo_swift.get_block(12345, true);
		verify_request(expected, request);
	}

	#[test]
	fn test_get_block_index_only_header() {
		let expected = r#"{"jsonrpc":"2.0","method":"getblockheader","id":1,"params":[12345,1]}"#;
		let neo_swift = NeoSwift::default();
		let request = neo_swift.get_block(12345, false);
		verify_request(expected, request);
	}

	#[test]
	fn test_get_block_hash() {
		let hash = H256::from_hex("2240b34669038f82ac492150d391dfc3d7fe5e3c1d34e5b547d50e99c09b468d").unwrap();
		let expected = r#"{"jsonrpc":"2.0","method":"getblock","id":1,"params":["2240b34669038f82ac492150d391dfc3d7fe5e3c1d34e5b547d50e99c09b468d",1]}"#;
		let neo_swift = NeoSwift::default();
		let request = neo_swift.get_block(hash, true);
		verify_request(expected, request);
	}

	#[test]
	fn test_get_block_not_full_tx_objects() {
		let hash = H256::from_hex("2240b34669038f82ac492150d391dfc3d7fe5e3c1d34e5b547d50e99c09b468d").unwrap();
		let expected = r#"{"jsonrpc":"2.0","method":"getblockheader","id":1,"params":["2240b34669038f82ac492150d391dfc3d7fe5e3c1d34e5b547d50e99c09b468d",1]}"#;
		let neo_swift = NeoSwift::default();
		let request = neo_swift.get_block(hash, false);
		verify_request(expected, request);
	}

	#[test]
	fn test_get_raw_block_index() {
		let expected = r#"{"jsonrpc":"2.0","method":"getblock","id":1,"params":[12345,0]}"#;
		let neo_swift = NeoSwift::default();
		let request = neo_swift.get_raw_block(12345);
		verify_request(expected, request);
	}

	#[test]
	fn test_get_block_header_count() {
		let expected = r#"{"jsonrpc":"2.0","method":"getblockheadercount","id":1,"params":[]}"#;
		let neo_swift = NeoSwift::default();
		let request = neo_swift.get_block_header_count();
		verify_request(expected, request);
	}

	#[test]
	fn test_get_block_count() {
		let expected = r#"{"jsonrpc":"2.0","method":"getblockcount","id":1,"params":[]}"#;
		let neo_swift = NeoSwift::default();
		let request = neo_swift.get_block_count();
		verify_request(expected, request);
	}

	#[test]
	fn test_get_native_contracts() {
		let expected = r#"{"jsonrpc":"2.0","method":"getnativecontracts","id":1,"params":[]}"#;
		let neo_swift = NeoSwift::default();
		let request = neo_swift.get_native_contracts();
		verify_request(expected, request);
	}

	#[test]
	fn test_get_block_header_index() {
		let expected = r#"{"jsonrpc":"2.0","method":"getblockheader","id":1,"params":[12345,1]}"#;
		let neo_swift = NeoSwift::default();
		let request = neo_swift.get_block_header(12345);
		verify_request(expected, request);
	}

	#[test]
	fn test_get_contract_state() {
		let hash = H160::from_hex("dc675afc61a7c0f7b3d2682bf6e1d8ed865a0e5f").unwrap();
		let expected = r#"{"jsonrpc":"2.0","method":"getcontractstate","id":1,"params":["dc675afc61a7c0f7b3d2682bf6e1d8ed865a0e5f"]}"#;
		let neo_swift = NeoSwift::default();
		let request = neo_swift.get_contract_state(hash);
		verify_request(expected, request);
	}

	#[test]
	fn test_get_contract_state_by_name() {
		let expected = r#"{"jsonrpc":"2.0","method":"getcontractstate","id":1,"params":["NeoToken"]}"#;
		let neo_swift = NeoSwift::default();
		let request = neo_swift.get_native_contract_state("NeoToken");
		verify_request(expected, request);
	}

// Utilities Methods

	#[test]
	fn test_list_plugins() {
		let expected = r#"{"jsonrpc":"2.0","method":"listplugins","id":1,"params":[]}"#;

		let neo_swift = NeoSwift::default();
		let request = neo_swift.list_plugins();
		verify_request(expected, request);
	}

	#[test]
	fn test_validate_address() {
		let expected = r#"{"jsonrpc":"2.0","method":"validateaddress","id":1,"params":["NTzVAPBpnUUCvrA6tFPxBHGge8Kyw8igxX"]}"#;

		let neo_swift = NeoSwift::default();
		let request = neo_swift.validate_address("NTzVAPBpnUUCvrA6tFPxBHGge8Kyw8igxX");
		verify_request(expected, request);
	}

// Wallet Methods

	#[test]
	fn test_close_wallet() {
		let expected = r#"{"jsonrpc":"2.0","method":"closewallet","id":1,"params":[]}"#;

		let neo_swift = NeoSwift::default();
		let request = neo_swift.close_wallet();
		verify_request(expected, request);
	}

	#[test]
	fn test_open_wallet() {
		let expected = r#"{"jsonrpc":"2.0","method":"openwallet","id":1,"params":["wallet.json","one"]}"#;

		let neo_swift = NeoSwift::default();
		let request = neo_swift.open_wallet("wallet.json", "one");
		verify_request(expected, request);
	}

	#[test]
	fn test_dump_priv_key() {
		let hash = H160::from_hex("c11d816956b6682c3406bb99b7ec8a3e93f005c1").unwrap();
		let expected = r#"{"jsonrpc":"2.0","method":"dumpprivkey","id":1,"params":["NdWaiUoBWbPxGsm5wXPjXYJxCyuY1Zw8uW"]}"#;

		let neo_swift = NeoSwift::default();
		let request = neo_swift.dump_priv_key(hash);
		verify_request(expected, request);
	}

// Application Logs

	#[test]
	fn test_get_application_log() {
		let hash = H256::from_hex("420d1eb458c707d698c6d2ba0f91327918ddb3b7bae2944df070f3f4e579078b").unwrap();
		let expected = r#"{"jsonrpc":"2.0","method":"getapplicationlog","id":1,"params":["420d1eb458c707d698c6d2ba0f91327918ddb3b7bae2944df070f3f4e579078b"]}"#;

		let neo_swift = NeoSwift::default();
		let request = neo_swift.get_application_log(hash);
		verify_request(expected, request);
	}

// State Service 

	#[test]
	fn test_get_state_root() {
		let expected = r#"{"jsonrpc":"2.0","method":"getstateroot","id":1,"params":[52]}"#;

		let neo_swift = NeoSwift::default();
		let request = neo_swift.get_state_root(52);
		verify_request(expected, request);
	}

	#[test]
	fn test_get_proof() {
		let state_root = H256::from_slice(&"7bf925dbd33af0e00d392b92313da59369ed86c82494d0e02040b24faac0a3ca".from_hex().unwrap()).unwrap();
		let key = H160::from_hex("79bcd398505eb779df6e67e4be6c14cded08e2f2").unwrap();
		let value = "616e797468696e67".from_hex().unwrap();

		let expected = r#"{"jsonrpc":"2.0","method":"getproof","id":1,"params":["7bf925dbd33af0e00d392b92313da59369ed86c82494d0e02040b24faac0a3ca","79bcd398505eb779df6e67e4be6c14cded08e2f2","YW55dGhpbmc="]}"#;

		let neo_swift = NeoSwift::default();
		let request = neo_swift.get_proof(state_root, key, value);
		verify_request(expected, request);
	}

// Neo-express tests

	#[test]
	fn test_express_get_populated_blocks() {

		let expected = r#"{"jsonrpc":"2.0","method":"expressgetpopulatedblocks","id":1,"params":[]}"#;

		let neo_swift = NeoSwiftExpress::default();
		let request = neo_swift.express_get_populated_blocks();
		verify_request(expected, request);
	}

// Abbreviated...

}
