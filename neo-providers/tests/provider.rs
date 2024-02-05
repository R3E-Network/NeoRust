mod neo_tests {
	use lazy_static::lazy_static;
	use neo_crypto::keys::Secp256r1PublicKey;
	use neo_providers::{
		core::transaction::{
			signers::{account_signer::AccountSigner, signer::Signer},
			transaction_send_token::TransactionSendToken,
			witness_rule::{
				witness_action::WitnessAction, witness_condition::WitnessCondition,
				witness_rule::WitnessRule,
			},
		},
		Http, Middleware, Provider,
	};
	use neo_types::{
		address::AddressExtension, block::BlockId, contract_parameter::ContractParameter,
		script_hash::ScriptHashExtension, H256,
	};
	use primitive_types::H160;
	use rustc_serialize::hex::{FromHex, ToHex};
	use serde::Serialize;

	lazy_static! {
		static ref NEO_PROVIDER: Provider<Http> =
			Provider::<Http>::try_from("http://localhost:30333").unwrap();
	}

	#[tokio::test]
	async fn non_existing_data_works() {
		assert!(NEO_PROVIDER.get_transaction(H256::zero()).await.unwrap().is_none());
		assert!(NEO_PROVIDER.get_transaction(H256::zero()).await.unwrap().is_none());
		assert!(NEO_PROVIDER.get_block(H256::zero(), false).await.unwrap().is_none());
		assert!(NEO_PROVIDER
			.get_block_with_txs(BlockId::Hash(H256::zero()))
			.await
			.unwrap()
			.is_none());
	}

	#[tokio::test]
	async fn client_version() {
		assert!(NEO_PROVIDER
			.client_version()
			.await
			.expect("Could not make web3_clientVersion call to provider")
			.starts_with("Geth/v"));
	}

	// Without TLS this would error with "TLS Support not compiled in"
	// #[tokio::test]
	// #[cfg(all(feature = "ws", any(feature = "openssl", feature = "rustls")))]
	// async fn ssl_websocket() {
	// 	let provider = GOERLI.ws().await;
	// 	assert_ne!(NEO_PROVIDER.get_block_number().await.unwrap(), 0.into());
	// }

	// #[tokio::test]
	// #[cfg(feature = "ws")]
	// async fn send_tx_ws() {
	// 	let (provider, anvil) = crate::spawn_anvil_ws().await;
	// 	generic_send_tx_test(provider, anvil.addresses()[0]).await;
	// }

	// #[tokio::test]
	// #[cfg(feature = "ipc")]
	// async fn send_tx_ipc() {
	// 	let (provider, anvil, _ipc) = crate::spawn_anvil_ipc().await;
	// 	generic_send_tx_test(provider, anvil.addresses()[0]).await;
	// }

	// async fn generic_send_tx_test<M: Middleware>(provider: M, who: Address) {
	// 	let tx = Transaction::new().to(who).from(who);
	// 	let pending_tx = provider.send_transaction(tx).await.unwrap();
	// 	let tx_hash = *pending_tx;
	// 	let receipt = pending_tx.confirmations(3).await.unwrap().unwrap();
	// 	assert_eq!(receipt.hash, tx_hash);
	// }

	// MARK: Blockchain Methods
	#[tokio::test]
	async fn test_get_best_block_hash() {
		let expected = r#"{"jsonrpc":"2.0","method":"getbestblockhash","id":1,"params":[]}"#;
		let request = NEO_PROVIDER.get_best_block_hash().await.unwrap();
		verify_request(expected, request);
	}

	#[tokio::test]
	async fn test_get_block_hash_by_index() {
		let expected = r#"{"jsonrpc":"2.0","method":"getblockhash","id":1,"params":[16293]}"#;
		let request = NEO_PROVIDER.get_block_hash(16293).await.unwrap();
		verify_request(expected, request);
	}

	#[tokio::test]
	async fn test_get_block_index() {
		let expected = r#"{"jsonrpc":"2.0","method":"getblock","id":1,"params":[12345,1]}"#;
		let request = NEO_PROVIDER.get_block_by_index(12345, true).await.unwrap();
		verify_request(expected, request);
	}

	#[tokio::test]
	async fn test_get_block_index_only_header() {
		let expected = r#"{"jsonrpc":"2.0","method":"getblockheader","id":1,"params":[12345,1]}"#;
		let request = NEO_PROVIDER.get_block_header_by_index(12345).await.unwrap();
		verify_request(expected, request);
	}

	#[tokio::test]
	async fn test_get_block_hash() {
		let hash =
			H256::from_hex("2240b34669038f82ac492150d391dfc3d7fe5e3c1d34e5b547d50e99c09b468d")
				.unwrap();
		let expected = r#"{"jsonrpc":"2.0","method":"getblock","id":1,"params":["2240b34669038f82ac492150d391dfc3d7fe5e3c1d34e5b547d50e99c09b468d",1]}"#;

		let request = NEO_PROVIDER.get_block(hash, true).await.unwrap();
		verify_request(expected, request);
	}

	#[tokio::test]
	async fn test_get_block_not_full_tx_objects() {
		let hash =
			H256::from_hex("2240b34669038f82ac492150d391dfc3d7fe5e3c1d34e5b547d50e99c09b468d")
				.unwrap();
		let expected = r#"{"jsonrpc":"2.0","method":"getblockheader","id":1,"params":["2240b34669038f82ac492150d391dfc3d7fe5e3c1d34e5b547d50e99c09b468d",1]}"#;

		let request = NEO_PROVIDER.get_block(hash, false).await.unwrap();
		verify_request(expected, request);
	}

	#[tokio::test]
	async fn test_get_raw_block_index() {
		let expected = r#"{"jsonrpc":"2.0","method":"getblock","id":1,"params":[12345,0]}"#;

		let request = NEO_PROVIDER.get_raw_block_by_index(12345).await.unwrap();
		verify_request(expected, request);
	}

	#[tokio::test]
	async fn test_get_block_header_count() {
		let expected = r#"{"jsonrpc":"2.0","method":"getblockheadercount","id":1,"params":[]}"#;

		let request = NEO_PROVIDER.get_block_header_count().await.unwrap();
		verify_request(expected, request);
	}

	#[tokio::test]
	async fn test_get_block_count() {
		let expected = r#"{"jsonrpc":"2.0","method":"getblockcount","id":1,"params":[]}"#;

		let request = NEO_PROVIDER.get_block_count().await.unwrap();
		verify_request(expected, request);
	}

	#[tokio::test]
	async fn test_get_native_contracts() {
		let expected = r#"{"jsonrpc":"2.0","method":"getnativecontracts","id":1,"params":[]}"#;

		let request = NEO_PROVIDER.get_native_contracts().await.unwrap();
		verify_request(expected, request);
	}

	#[tokio::test]
	async fn test_get_block_header_index() {
		let expected = r#"{"jsonrpc":"2.0","method":"getblockheader","id":1,"params":[12345,1]}"#;

		let request = NEO_PROVIDER.get_block_header_by_index(12345).await.unwrap();
		verify_request(expected, request);
	}

	#[tokio::test]
	async fn test_get_contract_state() {
		let hash = H160::from_hex("dc675afc61a7c0f7b3d2682bf6e1d8ed865a0e5f").unwrap();
		let expected = r#"{"jsonrpc":"2.0","method":"getcontractstate","id":1,"params":["dc675afc61a7c0f7b3d2682bf6e1d8ed865a0e5f"]}"#;
		let request = NEO_PROVIDER.get_contract_state(hash).await.unwrap();
		verify_request(expected, request);
	}

	#[tokio::test]
	async fn test_get_contract_state_by_name() {
		let expected =
			r#"{"jsonrpc":"2.0","method":"getcontractstate","id":1,"params":["NeoToken"]}"#;
		let request = NEO_PROVIDER.get_native_contract_state("NeoToken").await.unwrap();
		verify_request(expected, request);
	}

	// Utilities Methods

	#[tokio::test]
	async fn test_list_plugins() {
		let expected = r#"{"jsonrpc":"2.0","method":"listplugins","id":1,"params":[]}"#;
		let request = NEO_PROVIDER.list_plugins().await.unwrap();
		verify_request(expected, request);
	}

	#[tokio::test]
	async fn test_validate_address() {
		let expected = r#"{"jsonrpc":"2.0","method":"validateaddress","id":1,"params":["NTzVAPBpnUUCvrA6tFPxBHGge8Kyw8igxX"]}"#;

		let request = NEO_PROVIDER
			.validate_address("NTzVAPBpnUUCvrA6tFPxBHGge8Kyw8igxX")
			.await
			.unwrap();
		verify_request(expected, request);
	}

	// Wallet Methods

	#[tokio::test]
	async fn test_close_wallet() {
		let expected = r#"{"jsonrpc":"2.0","method":"closewallet","id":1,"params":[]}"#;

		let request = NEO_PROVIDER.close_wallet().await.unwrap();
		verify_request(expected, request);
	}

	#[tokio::test]
	async fn test_open_wallet() {
		let expected =
			r#"{"jsonrpc":"2.0","method":"openwallet","id":1,"params":["wallet.json","one"]}"#;

		let request = NEO_PROVIDER
			.open_wallet("wallet.json".to_string(), "one".to_string())
			.await
			.unwrap();
		verify_request(expected, request);
	}

	#[tokio::test]
	async fn test_dump_priv_key() {
		let hash = H160::from_hex("c11d816956b6682c3406bb99b7ec8a3e93f005c1").unwrap();
		let expected = r#"{"jsonrpc":"2.0","method":"dumpprivkey","id":1,"params":["NdWaiUoBWbPxGsm5wXPjXYJxCyuY1Zw8uW"]}"#;

		let request = NEO_PROVIDER.dump_priv_key(hash).await.unwrap();
		verify_request(expected, request);
	}

	// Application Logs

	#[tokio::test]
	async fn test_get_application_log() {
		let hash =
			H256::from_hex("420d1eb458c707d698c6d2ba0f91327918ddb3b7bae2944df070f3f4e579078b")
				.unwrap();
		let expected = r#"{"jsonrpc":"2.0","method":"getapplicationlog","id":1,"params":["420d1eb458c707d698c6d2ba0f91327918ddb3b7bae2944df070f3f4e579078b"]}"#;

		let request = NEO_PROVIDER.get_application_log(hash).await.unwrap();
		verify_request(expected, request);
	}

	// State Service
	#[tokio::test]
	async fn test_get_state_root() {
		let expected = r#"{"jsonrpc":"2.0","method":"getstateroot","id":1,"params":[52]}"#;

		let request = NEO_PROVIDER.get_state_root(52).await.unwrap();
		verify_request(expected, request);
	}

	#[tokio::test]
	async fn test_get_proof() {
		let state_root = H256::from_slice(
			&"7bf925dbd33af0e00d392b92313da59369ed86c82494d0e02040b24faac0a3ca"
				.from_hex()
				.unwrap(),
		)
		.unwrap();
		let contract = H160::from_hex("79bcd398505eb779df6e67e4be6c14cded08e2f2").unwrap();
		let key = "616e797468696e67";

		let expected = r#"{"jsonrpc":"2.0","method":"getproof","id":1,"params":["7bf925dbd33af0e00d392b92313da59369ed86c82494d0e02040b24faac0a3ca","79bcd398505eb779df6e67e4be6c14cded08e2f2","YW55dGhpbmc="]}"#;
		let request = NEO_PROVIDER.get_proof(state_root, contract, key).await.unwrap();
		verify_request(expected, request);
	}

	// Neo-express tests

	// #[tokio::test]
	// async fn test_express_get_populated_blocks() {
	// 	let expected =
	// 		r#"{"jsonrpc":"2.0","method":"expressgetpopulatedblocks","id":1,"params":[]}"#;
	//
	// 	let NEO_PROVIDER = NeoSwiftExpress::default();
	// 	let request = NEO_PROVIDER.express_get_populated_blocks();
	// 	verify_request(expected, request);
	// }

	// Nep17 tests

	#[tokio::test]
	async fn test_get_nep17_transfers() {
		let hash = H160::from_hex("04457ce4219e462146ac00b09793f81bc5bca2ce").unwrap();
		let expected = r#"{"jsonrpc":"2.0","method":"getnep17transfers","id":1,"params":["NekZLTu93WgrdFHxzBEJUYgLTQMAT85GLi"]}"#;

		let request = NEO_PROVIDER.get_nep17_transfers(hash).await.unwrap();
		verify_request(expected, request);
	}

	#[tokio::test]
	async fn test_get_nep17_transfers_date() {
		let hash = H160::from_hex("04457ce4219e462146ac00b09793f81bc5bca2ce").unwrap();
		let date = chrono::NaiveDateTime::from_timestamp_opt(1553105830, 0).unwrap();
		let expected = r#"{"jsonrpc":"2.0","method":"getnep17transfers","id":1,"params":["NekZLTu93WgrdFHxzBEJUYgLTQMAT85GLi",1553105830]}"#;

		let request = NEO_PROVIDER.get_nep17_transfers(hash).await.unwrap();
		verify_request(expected, request);
	}

	#[tokio::test]
	async fn test_get_nep17_balances() {
		let hash = H160::from_hex("5d75775015b024970bfeacf7c6ab1b0ade974886").unwrap();
		let expected = r#"{"jsonrpc":"2.0","method":"getnep17balances","id":1,"params":["NY9zhKwcmht5cQJ3oRqjJGo3QuVLwXwTzL"]}"#;

		let request = NEO_PROVIDER.get_nep17_balances(hash).await.unwrap();
		verify_request(expected, request);
	}

	// Nep11 tests
	#[tokio::test]
	async fn test_get_nep11_balances() {
		let hash = H160::from_hex("5d75775015b024970bfeacf7c6ab1b0ade974886").unwrap();
		let expected = r#"{"jsonrpc":"2.0","method":"getnep11balances","id":1,"params":["NY9zhKwcmht5cQJ3oRqjJGo3QuVLwXwTzL"]}"#;

		let request = NEO_PROVIDER.get_nep11_balances(hash).await.unwrap();
		verify_request(expected, request);
	}

	#[tokio::test]
	async fn test_get_nep11_transfers() {
		let hash = H160::from_hex("04457ce4219e462146ac00b09793f81bc5bca2ce").unwrap();
		let expected = r#"{"jsonrpc":"2.0","method":"getnep11transfers","id":1,"params":["NekZLTu93WgrdFHxzBEJUYgLTQMAT85GLi"]}"#;

		let request = NEO_PROVIDER.get_nep11_transfers(hash).await.unwrap();
		verify_request(expected, request);
	}

	// Invocation tests

	#[tokio::test]
	async fn test_invoke_function() {
		let contract_hash =
			"af7c7328eee5a275a3bcaee2bf0cf662b5e739be".hex_to_script_hash().unwrap();
		let param = ContractParameter::H160(
			&"91b83e96f2a7c4fdf0c1688441ec61986c7cae26".hex_to_script_hash().unwrap(),
		);
		let allowed_contract =
			"ef4073a0f2b305a38ec4050e4d3d28bc40ea63f5".hex_to_script_hash().unwrap();
		let account = "cadb3dc2faa3ef14a13b619c9a43124755aa2569".hex_to_script_hash().unwrap();
		let allowed_group = Secp256r1PublicKey::from_encoded(
			"033a4d051b04b7fc0230d2b1aaedfd5a84be279a5361a7358db665ad7857787f1b",
		)
		.unwrap();

		let signer = AccountSigner::called_by_entry(&account.into())
			.set_allowed_contracts(vec![allowed_contract])
			.set_allowed_groups(vec![allowed_group])
			.set_rules(vec![WitnessRule {
				action: WitnessAction::Allow,
				condition: WitnessCondition::CalledByContract(contract_hash),
				..Default::default()
			}]);

		let expected = r#"{
        "jsonrpc":"2.0",
        "method":"invokefunction",
        "id":1,
        "params":[
            "af7c7328eee5a275a3bcaee2bf0cf662b5e739be",
            "balanceOf",
            [
                {
                    "type":"Hash160",
                    "value":"91b83e96f2a7c4fdf0c1688441ec61986c7cae26"
                }
            ],
            [
                {
                    "allowedcontracts":[
                        "ef4073a0f2b305a38ec4050e4d3d28bc40ea63f5"
                    ],
                    "account":"cadb3dc2faa3ef14a13b619c9a43124755aa2569",
                    "rules":[
                        {
                            "condition":{
                                "type":"CalledByContract",
                                "hash":"SOME_NEO_TOKEN_HASH"
                            },
                            "action":"Allow"
                        }
                    ],
                    "allowedgroups":[
                        "033a4d051b04b7fc0230d2b1aaedfd5a84be279a5361a7358db665ad7857787f1b"
                    ],
                    "scopes": "CalledByEntry,CustomContracts,CustomGroups,WitnessRules"
                }
            ]
        ]
    }"#;

		let request = NEO_PROVIDER
			.invoke_function(
				&contract_hash,
				"balanceOf".to_string(),
				vec![param],
				Some(vec![Signer::Account(signer)]),
			)
			.await
			.unwrap();

		verify_request(expected, request);
	}

	#[tokio::test]
	async fn test_invoke_script() {
		let script =
			"10c00c08646563696d616c730c1425059ecb4878d3a875f91c51ceded330d4575fde41627d5b52"
				.from_hex()
				.unwrap();
		let expected = r#"{"jsonrpc":"2.0","method":"invokescript","id":1,"params":["EMAMCGRlY2ltYWxzDBQlBZ7LSHjTqHX5HFHO3tMw1Fdf3kFifVtS",[]]}"#;

		let request = NEO_PROVIDER.invoke_script(script.to_hex(), vec![]).await.unwrap();
		verify_request(expected, request);
	}

	#[tokio::test]
	async fn test_invoke_script_with_signer() {
		let script =
			"10c00c08646563696d616c730c1425059ecb4878d3a875f91c51ceded330d4575fde41627d5b52"
				.from_hex()
				.unwrap();
		let signer = AccountSigner::called_by_entry(
			&H160::from_hex("cc45cc8987b0e35371f5685431e3c8eeea306722").unwrap().into(),
		)
		.unwrap();

		let expected = r#"{
        "jsonrpc":"2.0",
        "method":"invokescript",
        "id":1,
        "params":[
            "EMAMCGRlY2ltYWxzDBQlBZ7LSHjTqHX5HFHO3tMw1Fdf3kFifVtS",
            [
                {
                    "allowedcontracts":[],
                    "account":"cc45cc8987b0e35371f5685431e3c8eeea306722",
                    "rules":[],
                    "allowedgroups":[],
                    "scopes":"CalledByEntry"
                }
            ]
        ]
    }"#;

		let request = NEO_PROVIDER
			.invoke_script(script.to_hex(), vec![Signer::Account(signer)])
			.await
			.unwrap();
		verify_request(expected, request);
	}

	// Transaction building tests

	#[tokio::test]
	async fn test_send_many() {
		let expected = r#"{"jsonrpc":"2.0","method":"sendmany","id":1,"params":[[{"asset":"de5f57d430d3dece511cf975a8d37848cb9e0525","value":100,"address":"NRkkHsxkzFxGz77mJtJgYZ3FnBm8baU5Um"},{"asset":"de5f57d430d3dece511cf975a8d37848cb9e0525","value":10,"address":"NNFGNNK1HXSSnA7yKLzRpr8YXwcdgTrsCu"}]]}"#;

		let outputs = vec![
			TransactionSendToken::new(
				H160::from_hex("de5f57d430d3dece511cf975a8d37848cb9e0525").unwrap(),
				100,
				"NRkkHsxkzFxGz77mJtJgYZ3FnBm8baU5Um".to_script_hash().unwrap(),
			),
			TransactionSendToken::new(
				H160::from_hex("de5f57d430d3dece511cf975a8d37848cb9e0525").unwrap(),
				10,
				"NNFGNNK1HXSSnA7yKLzRpr8YXwcdgTrsCu".to_script_hash().unwrap(),
			),
		];

		let request = NEO_PROVIDER
			.send_many("NiVNRW6cBXwkvrZnetZToaHPGSSGgV1HmA".to_script_hash().ok(), outputs)
			.await
			.unwrap();

		verify_request(expected, request);
	}

	// Contract invocation tests

	#[tokio::test]
	async fn test_invoke_contract_verify() {
		let contract_hash = H160::from_hex("af7c7328eee5a275a3bcaee2bf0cf662b5e739be").unwrap();

		let param1 = ContractParameter::string("a string".to_string());
		let param2 = ContractParameter::string("another string".to_string());
		let params = vec![param1, param2];

		let signer = AccountSigner::called_by_entry(
			&H160::from_hex("cadb3dc2faa3ef14a13b619c9a43124755aa2569").unwrap().into(),
		)
		.unwrap();
		let signers = vec![Signer::Account(signer)];

		let expected = r#"{
        "jsonrpc":"2.0",
        "method":"invokecontractverify",
        "id":1,
        "params":[
            "af7c7328eee5a275a3bcaee2bf0cf662b5e739be",
            [
                {"type":"String","value":"a string"},
                {"type":"String","value":"another string"}
            ],
            [
                {
                    "allowedcontracts":[],
                    "account":"cadb3dc2faa3ef14a13b619c9a43124755aa2569",
                    "rules":[],
                    "allowedgroups":[],
                    "scopes":"CalledByEntry"
                }
            ]
        ]
    }"#;

		let request = NEO_PROVIDER
			.invoke_contract_verify(contract_hash, params, signers)
			.await
			.unwrap();
		verify_request(expected, request);
	}

	#[tokio::test]
	async fn test_invoke_contract_verify_no_params_no_signers() {
		let contract_hash = H160::from_hex("af7c7328eee5a275a3bcaee2bf0cf662b5e739be").unwrap();

		let expected = r#"{"jsonrpc":"2.0","method":"invokecontractverify","id":1,"params":["af7c7328eee5a275a3bcaee2bf0cf662b5e739be",[],[]]}"#;

		let request = NEO_PROVIDER
			.invoke_contract_verify(contract_hash, vec![], vec![])
			.await
			.unwrap();
		verify_request(expected, request);
	}

	// Node methods

	#[tokio::test]
	async fn test_get_connection_count() {
		let expected = r#"{"jsonrpc":"2.0","method":"getconnectioncount","id":1,"params":[]}"#;

		let request = NEO_PROVIDER.get_connection_count().await.unwrap();
		verify_request(expected, request);
	}

	#[tokio::test]
	async fn test_get_peers() {
		let expected = r#"{"jsonrpc":"2.0","method":"getpeers","id":1,"params":[]}"#;

		let request = NEO_PROVIDER.get_peers().await.unwrap();
		verify_request(expected, request);
	}

	// Transaction building tests

	#[tokio::test]
	async fn test_send_to_address() {
		let asset = H160::from_hex("de5f57d430d3dece511cf975a8d37848cb9e0525").unwrap();
		let address = "NRCcuUUxKCa3sp45o7bjXetyxUeq58T4ED";
		let amount = 10;

		let expected = r#"{
        "jsonrpc":"2.0",
        "method":"sendtoaddress",
        "id":1,
        "params":[
            "de5f57d430d3dece511cf975a8d37848cb9e0525",
            "NRCcuUUxKCa3sp45o7bjXetyxUeq58T4ED",
            10
        ]
    }"#;

		// let output = TransactionSendToken::new();
		let request =
			NEO_PROVIDER.send_to_address(asset, address.to_string(), amount).await.unwrap();

		verify_request(expected, request);
	}

	#[tokio::test]
	async fn test_send_from() {
		let from = H160::from_hex("de5f57d430d3dece511cf975a8d37848cb9e0525").unwrap();
		let to = H160::from_hex("8cdb257b8873049918fe5a1e7f6289f75d720ba5").unwrap();
		let asset = H160::from_hex("db1acbae4dbae55f8325724cf080ed782925c7a7").unwrap();
		let amount = 10;

		let expected = r#"{
        "jsonrpc":"2.0",
        "method":"sendfrom",
        "id":1,
        "params":[
            "de5f57d430d3dece511cf975a8d37848cb9e0525",
            "NaxePjypvtsQ5GVi6S1jBsSjXribTSUKRu",
            "NbD6be5uYezFZRSBDt6aBfYR9bYsAk8Yui",
            10
        ]
    }"#;

		let request = NEO_PROVIDER
			.send_from(asset, from.to_address(), to.into(), amount)
			.await
			.unwrap();
		verify_request(expected, request);
	}

	#[tokio::test]
	async fn test_send_many_from() {
		let from = "NiVNRW6cBXwkvrZnetZToaHPGSSGgV1HmA";
		let asset = H160::from_hex("de5f57d430d3dece511cf975a8d37848cb9e0525").unwrap();

		let outputs = vec![
			TransactionSendToken::new(
				asset.clone(),
				100,
				"Nhsi2q3hkByxcH2uBQw7cjc2qEpzXSEKTC".address_to_script_hash().unwrap(),
			),
			TransactionSendToken::new(
				asset,
				10,
				"NcwVWxJZh9fxncJ9Sq8msVLotJDsAD3ZD8".address_to_script_hash().unwrap(),
			),
		];

		let expected = r#"{
        "jsonrpc":"2.0",
        "method":"sendmany",
        "id":1,
        "params":[
            "NiVNRW6cBXwkvrZnetZToaHPGSSGgV1HmA",
            [
                {"asset":"de5f57d430d3dece511cf975a8d37848cb9e0525","value":100,"address":"Nhsi2q3hkByxcH2uBQw7cjc2qEpzXSEKTC"},
                {"asset":"de5f57d430d3dece511cf975a8d37848cb9e0525","value":10,"address":"NcwVWxJZh9fxncJ9Sq8msVLotJDsAD3ZD8"}
            ]
        ]
    }"#;
		let request = NEO_PROVIDER.send_many_from(from, outputs);
		verify_request(expected, request);
	}

	// #[tokio::test]
	// async fn test_build_contract_transaction() {
	//
	// 	let operation = SmartContractOperation::new(
	// 		ContractParameter::H160(&H160::from_hex("5b7074e873973a6ed3708862f219a6fbf4d1c411").unwrap()),
	// 		"transfer",
	// 		vec![
	// 			ContractParameter::string("arg1".to_string()),
	// 			ContractParameter::integer(10),
	// 		],
	// 	);
	//
	// 	let signer = AccountSigner::default();
	// 	let network_fee = 0.001;
	//
	// 	let expected = r#"{"jsonrpc":"2.0","method":"sendrawtransaction","id":1,...}"#;
	//
	// 	let request = NEO_PROVIDER
	// 		.build_contract_transaction(operation, vec![signer], network_fee)
	// 		.and_then(|tx| NEO_PROVIDER.send_raw_transaction(tx));
	//
	// 	verify_request(expected, request);
	// }

	fn verify_request<T: Serialize>(expected: &str, request: T) {
		assert!(expected.contains(serde_json::to_string(request).unwrap()));
	}
}
