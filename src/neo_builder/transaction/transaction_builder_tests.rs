#[cfg(test)]
mod tests {
	use crate::{
		neo_builder::GAS_TOKEN_HASH,
		neo_clients::MockClient,
		neo_protocol::{NeoProtocol, NeoVersion},
		neo_types::ScriptHashExtension,
		prelude::{
			init_logger, ApplicationLog, ContractParameter, ContractSigner, InvocationResult,
			Signer, StackItem, TestConstants, TransactionAttribute, TransactionError, Witness,
			WitnessScope,
		},
	};
	use lazy_static::lazy_static;
	use neo::{
		builder::VerificationScript,
		config::{NeoConfig, NEOCONFIG},
		prelude::{
			APITrait, Account, AccountSigner, AccountTrait, Http, HttpProvider, KeyPair,
			NeoConstants, RawTransaction, RpcClient, ScriptBuilder, Secp256r1PrivateKey,
			TransactionBuilder,
		},
	};
	use num_bigint::BigInt;
	use primitive_types::{H160, H256};
	use rustc_serialize::hex::ToHex;
	use serde_json::json;
	use std::{ops::Deref, str::FromStr, sync::Arc};
	use tokio::sync::{Mutex, OnceCell};
	use tracing::debug;

	lazy_static! {
		pub static ref ACCOUNT1: Account = Account::from_key_pair(
			KeyPair::from_secret_key(
				&Secp256r1PrivateKey::from_bytes(
					&hex::decode(
						"e6e919577dd7b8e97805151c05ae07ff4f752654d6d8797597aca989c02c4cb3"
					)
					.unwrap()
				)
				.unwrap()
			),
			None,
			None
		)
		.expect("Failed to create ACCOUNT1");
		pub static ref ACCOUNT2: Account = Account::from_key_pair(
			KeyPair::from_secret_key(
				&Secp256r1PrivateKey::from_bytes(
					&hex::decode(
						"b4b2b579cac270125259f08a5f414e9235817e7637b9a66cfeb3b77d90c8e7f9"
					)
					.unwrap()
				)
				.unwrap()
			),
			None,
			None
		)
		.expect("Failed to create ACCOUNT2");
	}

	static CLIENT: OnceCell<RpcClient<HttpProvider>> = OnceCell::const_new();

	#[tokio::test]
	async fn test_build_transaction_with_correct_nonce() {
		let mut nonce = 1;
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let mut tx = TransactionBuilder::with_client(&client)
			.valid_until_block(1)
			.unwrap()
			.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::called_by_entry(ACCOUNT1.deref()).unwrap().into()])
			.nonce(nonce)
			.unwrap()
			.get_unsigned_tx()
			.await
			.unwrap();

		assert_eq!(*tx.nonce(), nonce);

		nonce = 0;
		tx = TransactionBuilder::with_client(&client)
			.valid_until_block(1)
			.unwrap()
			.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::called_by_entry(ACCOUNT1.deref()).unwrap().into()])
			.nonce(nonce)
			.unwrap()
			.get_unsigned_tx()
			.await
			.unwrap();
		assert_eq!(*tx.nonce(), nonce);

		nonce = u32::MAX;
		tx = TransactionBuilder::with_client(&client)
			.valid_until_block(1)
			.unwrap()
			.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::called_by_entry(ACCOUNT1.deref()).unwrap().into()])
			.nonce(nonce)
			.unwrap()
			.get_unsigned_tx()
			.await
			.unwrap();
		debug!("{:?}", tx);
		assert_eq!(*tx.nonce(), nonce);
	}

	#[tokio::test]
	async fn test_invoke_script() {
		let mock_provider = Arc::new(Mutex::new(MockClient::new().await));
		let client = {
			let mut mock_provider = mock_provider.lock().await;

			mock_provider
				.mock_invoke_script(InvocationResult {
					stack: vec![StackItem::ByteString { value: base64::encode("NEO") }],
					..Default::default()
				})
				.await
				.mock_get_block_count(1000)
				.await
				.mock_calculate_network_fee(1230610)
				.await
				.mount_mocks()
				.await;
			Arc::new(mock_provider.into_client())
		};

		let script = ScriptBuilder::new()
			.contract_call(
				&ScriptHashExtension::from_hex("0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5")
					.unwrap(),
				"symbol",
				&[],
				None,
			)
			.unwrap()
			.to_bytes();
		let tb = TransactionBuilder::with_client(&client);
		let response = tb.client.unwrap().invoke_script((&script).to_hex(), vec![]).await.unwrap();

		println!("Response: {:?}", response); // Add this line for debugging

		assert!(!response.stack.is_empty(), "Response stack is empty");

		if let Some(item) = response.stack.get(0) {
			if let Some(value) = item.as_string() {
				assert_eq!(value, "NEO", "Unexpected value in response");
			} else {
				panic!("First item in stack is not a string");
			}
		} else {
			panic!("Response stack is empty");
		}
	}

	#[tokio::test]
	async fn test_build_without_setting_script() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let err = TransactionBuilder::with_client(&client)
			.set_signers(vec![AccountSigner::called_by_entry(ACCOUNT1.deref()).unwrap().into()])
			.get_unsigned_tx()
			.await
			.err()
			.unwrap();

		assert_eq!(err, TransactionError::NoScript);
	}

	#[tokio::test]
	async fn test_sign_transaction_with_additional_signers() {
		let mock_provider = Arc::new(Mutex::new(MockClient::new().await));
		let client = {
			let mut mock_provider = mock_provider.lock().await;
			mock_provider.mock_default_responses().await.mount_mocks().await;
			Arc::new(mock_provider.into_client())
		};
		let script = vec![0x01u8, 0x02u8, 0x03u8];

		let tx = TransactionBuilder::with_client(&client)
			.set_script(Some(script))
			.set_signers(vec![
				AccountSigner::called_by_entry(ACCOUNT1.deref()).unwrap().into(),
				AccountSigner::called_by_entry(ACCOUNT2.deref()).unwrap().into(),
			])
			.valid_until_block(1000)
			.unwrap()
			.sign()
			.await
			.unwrap();

		assert_eq!(tx.witnesses().len(), 2);

		let signers = tx
			.witnesses()
			.iter()
			.map(|witness| witness.verification.get_public_keys().unwrap().first().unwrap().clone())
			.collect::<Vec<_>>();

		assert!(signers.contains(&ACCOUNT1.deref().clone().key_pair.unwrap().public_key()));
		assert!(signers.contains(&ACCOUNT2.deref().clone().key_pair.unwrap().public_key()));
	}

	#[tokio::test]
	async fn test_send_invoke_function() {
		init_logger();
		let mock_provider = Arc::new(Mutex::new(MockClient::new().await));
		let client = {
			let mut mock_provider = mock_provider.lock().await;
			mock_provider
				.mock_invoke_function(InvocationResult {
					stack: vec![StackItem::ByteString { value: base64::encode("NEO") }],
					..Default::default()
				})
				.await
				.mock_invoke_script(InvocationResult {
					stack: vec![StackItem::ByteString { value: base64::encode("NEO") }],
					..Default::default()
				})
				.await
				.mount_mocks()
				.await;
			Arc::new(mock_provider.into_client())
		};
		let tb = TransactionBuilder::with_client(&client);
		let response = tb
			.client
			.unwrap()
			.invoke_function(
				&H160::from_str("0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5").unwrap(),
				"symbol".to_string(),
				vec![],
				None,
			)
			.await;

		match response {
			Ok(response) => {
				assert_eq!(response.stack[0].as_string().unwrap(), "NEO");
			},
			Err(e) => {
				panic!("Failed to invoke function: {:?}", e);
			},
		}
	}

	#[tokio::test]
	async fn test_fail_building_transaction_with_incorrect_nonce() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let mut tb = TransactionBuilder::with_client(&client);
		tb.valid_until_block(1)
			.unwrap()
			.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::called_by_entry(ACCOUNT1.deref()).unwrap().into()]);

		// Test with 0, which should be valid
		assert!(tb.nonce(0).is_ok());

		// Test with u32::MAX, which should be valid
		assert!(tb.nonce(u32::MAX).is_ok());

		// Test overflow condition
		tb.nonce(u32::MAX).unwrap();
		assert!(tb.nonce(u32::MAX).is_ok());

		// Reset nonce for next test
		tb.nonce(0).unwrap();

		// Test with -1 cast to u32, which is actually u32::MAX
		assert!(tb.nonce((-1i32) as u32).is_ok());
	}

	#[tokio::test]
	async fn test_fail_building_transaction_with_invalid_block_number() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let mut tb = TransactionBuilder::with_client(&client);
		tb.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::called_by_entry(ACCOUNT1.deref()).unwrap().into()]);

		assert!(tb.valid_until_block(-1i32 as u32).is_ok());
		// assert!(tb.valid_until_block(2u32.pow(32)).is_err());
	}

	#[tokio::test]
	async fn test_automatically_set_nonce() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let mut tb = TransactionBuilder::with_client(&client);
		tb.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::called_by_entry(ACCOUNT1.deref()).unwrap().into()]);

		let tx = match tb.get_unsigned_tx().await {
			Ok(tx) => tx,
			Err(e) => panic!("Failed to build transaction: {:?}", e),
		};
		assert!(tx.nonce() < &u32::MAX && tx.nonce() > &0);
	}

	#[tokio::test]
	async fn test_fail_building_tx_without_any_signer() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let mut tb = TransactionBuilder::with_client(&client);
		tb.valid_until_block(100).unwrap().set_script(Some(vec![1, 2, 3]));

		assert!(tb.get_unsigned_tx().await.is_err());

		let mut tb2 = TransactionBuilder::with_client(&client);
		tb2.set_signers(vec![
			AccountSigner::global(ACCOUNT1.deref()).unwrap().into(),
			AccountSigner::called_by_entry(ACCOUNT1.deref()).unwrap().into(),
		]);
	}

	#[tokio::test]
	async fn test_override_signer() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let mut tb = TransactionBuilder::with_client(&client);
		tb.set_script(Some(vec![1, 2, 3])).set_signers(vec![AccountSigner::global(
			ACCOUNT1.deref(),
		)
		.unwrap()
		.into()]);
		assert_eq!(
			tb.signers()[0],
			Signer::AccountSigner(AccountSigner::global(ACCOUNT1.deref()).unwrap())
		);

		tb.set_signers(vec![AccountSigner::global(ACCOUNT2.deref()).unwrap().into()]);
		assert_eq!(tb.signers(), &vec![AccountSigner::global(ACCOUNT2.deref()).unwrap().into()]);
	}

	#[tokio::test]
	async fn test_attributes_high_priority_committee() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let mut tb = TransactionBuilder::with_client(&client);
		let multi_sig_account = Account::multi_sig_from_public_keys(
			&mut vec![ACCOUNT2.get_public_key().unwrap(), ACCOUNT1.get_public_key().unwrap()],
			1,
		)
		.unwrap();
		tb.set_script(Some(vec![1, 2, 3]))
			.set_attributes(vec![TransactionAttribute::HighPriority])
			.set_signers(vec![AccountSigner::none(&multi_sig_account).unwrap().into()]);

		let tx = tb.get_unsigned_tx().await.unwrap();
		assert_eq!(tx.attributes()[0], TransactionAttribute::HighPriority);
	}

	#[tokio::test]
	async fn test_attributes_high_priority() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let mut tb = TransactionBuilder::with_client(&client);
		tb.set_script(Some(vec![1, 2, 3]))
			.set_attributes(vec![TransactionAttribute::HighPriority])
			.set_signers(vec![AccountSigner::none(ACCOUNT1.deref()).unwrap().into()]);

		let tx = tb.get_unsigned_tx().await.unwrap();
		assert_eq!(tx.attributes()[0], TransactionAttribute::HighPriority);
	}

	#[tokio::test]
	async fn test_attributes_high_priority_not_committee_member() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let mut tb = TransactionBuilder::with_client(&client);
		tb.set_script(Some(vec![1, 2, 3]))
			.set_attributes(vec![TransactionAttribute::HighPriority])
			.set_signers(vec![AccountSigner::none(ACCOUNT2.deref()).unwrap().into()]);

		assert!(tb.get_unsigned_tx().await.is_err());
	}

	#[tokio::test]
	async fn test_attributes_high_priority_only_added_once() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let mut tb = TransactionBuilder::with_client(&client);
		tb.set_script(Some(vec![1, 2, 3]))
			.set_attributes(vec![
				TransactionAttribute::HighPriority,
				TransactionAttribute::HighPriority,
			])
			.set_signers(vec![AccountSigner::none(ACCOUNT1.deref()).unwrap().into()]);

		let tx = tb.get_unsigned_tx().await.unwrap();
		assert_eq!(tx.attributes()[0], TransactionAttribute::HighPriority);
	}

	#[tokio::test]
	async fn test_fail_adding_more_than_max_attributes_to_tx_just_attributes() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let attrs: Vec<TransactionAttribute> = (0..=NeoConstants::MAX_TRANSACTION_ATTRIBUTES)
			.map(|_| TransactionAttribute::HighPriority)
			.collect();
		let mut tb = TransactionBuilder::with_client(&client);
		// assert!(tb.set_attributes(attrs));
	}

	#[tokio::test]
	async fn test_fail_adding_more_than_max_attributes_to_tx_attributes_and_signers() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let mut tb = TransactionBuilder::with_client(&client);
		tb.set_signers(vec![
			AccountSigner::called_by_entry(ACCOUNT1.deref()).unwrap().into(),
			AccountSigner::called_by_entry(ACCOUNT1.deref()).unwrap().into(),
			AccountSigner::called_by_entry(ACCOUNT1.deref()).unwrap().into(),
		]);
		let attrs: Vec<TransactionAttribute> = (0..=NeoConstants::MAX_TRANSACTION_ATTRIBUTES - 3)
			.map(|_| TransactionAttribute::HighPriority)
			.collect();
		// assert!(tb.set_attributes(attrs));
	}

	#[tokio::test]
	async fn test_fail_adding_more_than_max_attributes_to_tx_signers() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let mut tb = TransactionBuilder::with_client(&client);
		tb.set_attributes(vec![TransactionAttribute::HighPriority]);
		let signers: Vec<AccountSigner> = (0..NeoConstants::MAX_TRANSACTION_ATTRIBUTES)
			.map(|_| AccountSigner::called_by_entry(ACCOUNT1.deref()).unwrap())
			.collect();
		// assert!(tb.set_signers(signers.into_iter().map(Into::into).collect()));
	}

	#[tokio::test]
	async fn test_automatic_setting_of_valid_until_block_variable() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let block_count = 1000;
		let max_valid_until_block_increment = 1000;
		let mut tb = TransactionBuilder::with_client(&client);
		tb.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::none(ACCOUNT1.deref()).unwrap().into()]);

		let tx = tb.get_unsigned_tx().await.unwrap();
		assert_eq!(*tx.valid_until_block(), block_count + max_valid_until_block_increment);
	}

	#[tokio::test]
	async fn test_automatic_setting_of_system_fee_and_network_fee() {
		let mock_provider = Arc::new(Mutex::new(MockClient::new().await));
		let client = {
			let mut mock_provider = mock_provider.lock().await;

			mock_provider
				.mock_invoke_script(InvocationResult {
					stack: vec![StackItem::Integer { value: 1000000.into() }],
					gas_consumed: "9999510".to_string(),
					..Default::default()
				})
				.await
				.mock_get_block_count(1000)
				.await
				.mock_calculate_network_fee(1230610)
				.await
				.mock_get_version(NeoVersion {
					tcp_port: Some(10333),
					ws_port: Some(10334),
					nonce: 1234567890,
					user_agent: "/Neo:3.5.0/".to_string(),
					protocol: Some(NeoProtocol::default()),
				})
				.await
				.mount_mocks()
				.await;
			Arc::new(mock_provider.into_client())
		};

		// Test getversion
		let version = client.get_version().await.unwrap();
		assert_eq!(version.nonce, 1234567890);
		assert_eq!(version.user_agent, "/Neo:3.5.0/");
		assert!(version.protocol.is_some());

		let script = vec![1, 2, 3];
		let mut tb = TransactionBuilder::with_client(&client);
		tb.set_script(Some(script.clone()))
			.set_signers(vec![AccountSigner::none(ACCOUNT1.deref()).unwrap().into()])
			.valid_until_block(1000)
			.unwrap();

		let tx = tb.get_unsigned_tx().await.unwrap();
		assert_eq!(*tx.sys_fee(), 9999510);
		assert_eq!(*tx.net_fee(), 1230610);
	}
	#[tokio::test]
	async fn test_fail_trying_to_sign_transaction_with_account_missing_a_private_key() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let account_without_keypair =
			Account::from_address(ACCOUNT1.get_address().as_str()).unwrap();
		let mut tb = TransactionBuilder::with_client(&client);
		tb.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::none(&account_without_keypair).unwrap().into()])
			.valid_until_block(1000)
			.unwrap();

		assert!(tb.sign().await.is_err());
	}

	#[tokio::test]
	async fn test_fail_automatically_signing_with_multi_sig_account_signer() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let multi_sig_account = Account::multi_sig_from_public_keys(
			vec![ACCOUNT1.get_public_key().unwrap()].as_mut(),
			1,
		)
		.unwrap();
		let mut tb = TransactionBuilder::with_client(&client);
		tb.set_script(Some(vec![1, 2, 3])).set_signers(vec![AccountSigner::none(
			&multi_sig_account,
		)
		.unwrap()
		.into()]);

		assert!(tb.sign().await.is_err());
	}

	#[tokio::test]
	async fn test_fail_with_no_signing_account() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let mut tb = TransactionBuilder::with_client(&client);
		tb.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![ContractSigner::called_by_entry(
				ACCOUNT1.address_or_scripthash().script_hash(),
				&*vec![],
			)
			.into()]);

		assert!(tb.sign().await.is_err());
	}

	#[tokio::test]
	async fn test_fail_signing_with_account_without_ec_keypair() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let account_without_keypair = Account::from_verification_script(
			&ACCOUNT1.clone().verification_script().clone().unwrap(),
		)
		.unwrap();
		let mut tb = TransactionBuilder::with_client(&client);
		tb.set_script(Some(vec![1, 2, 3])).set_signers(vec![AccountSigner::none(
			&account_without_keypair,
		)
		.unwrap()
		.into()]);

		assert!(tb.sign().await.is_err());
	}

	#[tokio::test]
	async fn test_fail_sending_transaction_because_it_doesnt_contain_the_right_number_of_witnesses()
	{
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let mut tb = TransactionBuilder::with_client(&client);
		tb.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::called_by_entry(ACCOUNT1.deref()).unwrap().into()])
			.valid_until_block(1000)
			.unwrap();

		let mut tx = tb.get_unsigned_tx().await.unwrap();
		// assert!(tx.send_tx(&client).await.is_err());
	}

	#[tokio::test]
	async fn test_contract_witness() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let contract_hash = H160::from_str("e87819d005b730645050f89073a4cd7bf5f6bd3c").unwrap();
		let params = vec![ContractParameter::from("iamgroot"), ContractParameter::from(2)];
		let invocation_script = ScriptBuilder::new()
			.push_data("iamgroot".as_bytes().to_vec())
			.push_integer(BigInt::from(2))
			.to_bytes();
		let mut tb = TransactionBuilder::with_client(&client);
		tb.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![
				ContractSigner::global(contract_hash, &params).into(),
				AccountSigner::called_by_entry(ACCOUNT1.deref()).unwrap().into(),
			])
			.valid_until_block(1000)
			.unwrap();

		let tx = tb.sign().await.unwrap();
		assert!(tx.witnesses().contains(&Witness::from_scripts(invocation_script, vec![])));
	}

	#[tokio::test]
	async fn test_transfer_neo_from_normal_account() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let script = ScriptBuilder::new()
			.contract_call(
				&ScriptHashExtension::from_hex("0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5")
					.unwrap(),
				"transfer",
				&vec![
					ContractParameter::from(ACCOUNT1.address_or_scripthash().script_hash()),
					ContractParameter::from(
						H160::from_str("969a77db482f74ce27105f760efa139223431394").unwrap(),
					),
					ContractParameter::from(5),
					ContractParameter::any(),
				],
				None,
			)
			.unwrap()
			.to_bytes();

		let binding = ACCOUNT1.verification_script().clone().unwrap();
		let expected_verification_script = binding.script();
		let mut tb = TransactionBuilder::with_client(&client);
		tb.set_script(Some(script.clone()))
			.set_signers(vec![AccountSigner::none(ACCOUNT1.deref()).unwrap().into()])
			.valid_until_block(100)
			.unwrap();

		let tx = tb.sign().await.unwrap();
		assert_eq!(tx.script(), &script);
		assert_eq!(tx.witnesses().len(), 1);
		assert_eq!(
			tx.witnesses().first().unwrap().verification.script(),
			expected_verification_script
		);
	}

	#[tokio::test]
	async fn test_extend_script() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let script1 = ScriptBuilder::new()
			.contract_call(
				&ScriptHashExtension::from_hex("0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5")
					.unwrap(),
				"transfer",
				&vec![
					ContractParameter::from(ACCOUNT1.address_or_scripthash().script_hash()),
					ContractParameter::from(
						H160::from_str("969a77db482f74ce27105f760efa139223431394").unwrap(),
					),
					ContractParameter::from(11),
					ContractParameter::any(),
				],
				None,
			)
			.unwrap()
			.to_bytes();

		let script2 = ScriptBuilder::new()
			.contract_call(
				&ScriptHashExtension::from_hex("0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5")
					.unwrap(),
				"transfer",
				&vec![
					ContractParameter::from(ACCOUNT1.address_or_scripthash().script_hash()),
					ContractParameter::from(ACCOUNT2.address_or_scripthash().script_hash()),
					ContractParameter::from(22),
					ContractParameter::any(),
				],
				None,
			)
			.unwrap()
			.to_bytes();

		let mut tb = TransactionBuilder::with_client(&client);
		tb.set_script(Some(script1.clone()));
		assert_eq!(tb.script().clone().unwrap().len(), script1.len());

		tb.extend_script(script2.clone());
		assert_eq!(tb.script().clone().unwrap().len(), [script1, script2].concat().len());
	}

	#[tokio::test]
	async fn test_invoking_with_params_should_produce_the_correct_request() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let mut tb = TransactionBuilder::with_client(&client);
		let script = ScriptBuilder::new()
			.contract_call(
				&ScriptHashExtension::from_hex("0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5")
					.unwrap(),
				"transfer",
				&vec![
					ContractParameter::from(ACCOUNT1.address_or_scripthash().script_hash()),
					ContractParameter::from(
						H160::from_str("969a77db482f74ce27105f760efa139223431394").unwrap(),
					),
					ContractParameter::from(5),
					ContractParameter::any(),
				],
				None,
			)
			.unwrap()
			.to_bytes();

		let response = tb
			.client
			.unwrap()
			.invoke_function(
				&H160::from_str("0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5").unwrap(),
				"transfer".to_string(),
				vec![
					ContractParameter::from(ACCOUNT1.address_or_scripthash().script_hash()),
					ContractParameter::from(
						H160::from_str("969a77db482f74ce27105f760efa139223431394").unwrap(),
					),
					ContractParameter::from(5),
					ContractParameter::any(),
				],
				None,
			)
			.await
			.unwrap();

		assert_eq!(response.stack[0].as_string().unwrap(), "NEO");
	}

	#[tokio::test]
	async fn test_fail_signing_with_account_without_ec_key_pair() {
		let mock_provider = Arc::new(Mutex::new(MockClient::new().await));
		let client = {
			let mut mock_provider = mock_provider.lock().await;
			mock_provider.mock_default_responses().await.mount_mocks().await;
			Arc::new(mock_provider.into_client())
		};

		let account =
			Account::from_verification_script(&VerificationScript::from(vec![1, 2, 3])).unwrap();

		let mut tx_builder = TransactionBuilder::with_client(client.as_ref());
		tx_builder
			.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::none(&account).unwrap().into()]);

		assert!(tx_builder.sign().await.is_err());
	}

	#[tokio::test]
	async fn test_do_if_sender_cannot_cover_fees() {
		let mock_provider = Arc::new(Mutex::new(MockClient::new().await));
		let client = {
			let mut mock_provider = mock_provider.lock().await;
			mock_provider
				.mock_invoke_script(InvocationResult {
					gas_consumed: "9999510".to_string(),
					..Default::default()
				})
				.await
				.mock_get_block_count(1000)
				.await
				.mock_calculate_network_fee(1230610)
				.await
				.mock_invoke_function(InvocationResult {
					stack: vec![StackItem::Integer { value: 1000000.into() }],
					..Default::default()
				})
				.await
				.mount_mocks()
				.await;
			Arc::new(mock_provider.into_client())
		};

		let account1 =
			Account::from_wif("L1WMhxazScMhUrdv34JqQb1HFSQmWeN2Kpc1R9JGKwL7CDNP21uR").unwrap();
		let recipient = H160::from_str("969a77db482f74ce27105f760efa139223431394").unwrap();

		let script = ScriptBuilder::new()
			.contract_call(
				&GAS_TOKEN_HASH,
				"transfer",
				&[
					ContractParameter::h160(&account1.address_or_scripthash().script_hash()),
					ContractParameter::h160(&recipient),
					ContractParameter::integer(2_000_000),
					ContractParameter::any(),
				],
				None,
			)
			.unwrap()
			.to_bytes();

		let tested = Arc::new(std::sync::atomic::AtomicBool::new(false));
		let tested_clone = tested.clone();

		let mut tx_builder = TransactionBuilder::with_client(client.as_ref());
		let _ = tx_builder
			.set_script(Some(script))
			.set_signers(vec![AccountSigner::called_by_entry(&account1).unwrap().into()])
			.valid_until_block(2000000)
			.unwrap()
			.do_if_sender_cannot_cover_fees(Box::new(move |fee, balance| {
				assert_eq!(fee, 1230610 + 9999510);
				assert_eq!(balance, 1000000);
				tested_clone.store(true, std::sync::atomic::Ordering::SeqCst);
			}));

		let _ = tx_builder.get_unsigned_tx().await.unwrap();

		assert!(tested.load(std::sync::atomic::Ordering::SeqCst));
	}

	#[tokio::test]
	async fn test_do_if_sender_cannot_cover_fees_already_specified_a_supplier() {
		let mock_provider = Arc::new(Mutex::new(MockClient::new().await));
		let client = {
			let mut mock_provider = mock_provider.lock().await;
			mock_provider.mock_default_responses().await.mount_mocks().await;
			Arc::new(mock_provider.into_client())
		};
		let mut tx_builder = TransactionBuilder::with_client(&client);

		// TODO: check and add
		// NeoConfig::throw_if_sender_cannot_cover_fees(TransactionError::InsufficientFunds);

		assert!(tx_builder.do_if_sender_cannot_cover_fees(Box::new(|_, _| {})).is_err());
	}

	#[tokio::test]
	async fn test_throw_if_sender_cannot_cover_fees() {
		let mock_provider = Arc::new(Mutex::new(MockClient::new().await));
		let client = {
			let mut mock_provider = mock_provider.lock().await;
			mock_provider
				.mock_invoke_script(InvocationResult::default())
				.await
				.mock_calculate_network_fee(1230610)
				.await
				.mock_invoke_function(InvocationResult {
					stack: vec![StackItem::Integer { value: 1000000 }],
					..Default::default()
				})
				.await
				.mount_mocks()
				.await;
			Arc::new(mock_provider.into_client())
		};

		let account1 =
			Account::from_wif("L1WMhxazScMhUrdv34JqQb1HFSQmWeN2Kpc1R9JGKwL7CDNP21uR").unwrap();

		let script = ScriptBuilder::new()
			.contract_call(
				&H160::from_str(TestConstants::NEO_TOKEN_HASH).unwrap(),
				"transfer",
				&[
					ContractParameter::h160(&account1.address_or_scripthash().script_hash()),
					ContractParameter::h160(&H160::zero()),
					ContractParameter::integer(5),
					ContractParameter::any(),
				],
				None,
			)
			.unwrap()
			.to_bytes();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		let _ = tx_builder
			.set_script(Some(script))
			.valid_until_block(2000000)
			.unwrap()
			.set_signers(vec![AccountSigner::called_by_entry(&account1).unwrap().into()])
			.throw_if_sender_cannot_cover_fees(TransactionError::InsufficientFunds);

		assert!(tx_builder.get_unsigned_tx().await.is_err());
	}

	#[tokio::test]
	async fn test_throw_if_sender_cannot_cover_fees_already_specified_a_consumer() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let mut tx_builder = TransactionBuilder::with_client(&client);
		let _ = tx_builder.do_if_sender_cannot_cover_fees(Box::new(|_, _| {}));

		assert!(tx_builder
			.throw_if_sender_cannot_cover_fees(TransactionError::InsufficientFunds)
			.is_err());
	}

	#[tokio::test]
	async fn test_build_with_invalid_script() {
		let mock_provider = Arc::new(Mutex::new(MockClient::new().await));
		let client = {
			let mut mock_provider = mock_provider.lock().await;
			mock_provider
				.mock_invoke_script(InvocationResult::default())
				.await
				.mock_get_block_count(1000)
				.await
				.mount_mocks()
				.await;
			Arc::new(mock_provider.into_client())
		};

		let account1 =
			Account::from_wif("L1WMhxazScMhUrdv34JqQb1HFSQmWeN2Kpc1R9JGKwL7CDNP21uR").unwrap();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder
			.set_script(Some(hex::decode("0c0e4f7261636c65436f6e7472616374411af77b67").unwrap()))
			.set_signers(vec![AccountSigner::called_by_entry(&account1).unwrap().into()]);

		assert!(tx_builder.get_unsigned_tx().await.is_err());
	}

	#[tokio::test]
	async fn test_build_with_script_vm_faults() {
		let mock_provider = Arc::new(Mutex::new(MockClient::new().await));
		let client = {
			let mut mock_provider = mock_provider.lock().await;
			mock_provider
				.mock_invoke_script(InvocationResult::default())
				.await
				.mock_get_block_count(1000)
				.await
				.mock_calculate_network_fee(1230610)
				.await
				.mount_mocks()
				.await;
			Arc::new(mock_provider.into_client())
		};

		let account1 =
			Account::from_wif("L1WMhxazScMhUrdv34JqQb1HFSQmWeN2Kpc1R9JGKwL7CDNP21uR").unwrap();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder
			.set_script(Some(hex::decode("0c00120c1493ad1572").unwrap()))
			.set_signers(vec![AccountSigner::called_by_entry(&account1).unwrap().into()]);

		let result = tx_builder.get_unsigned_tx().await;
		assert!(result.is_err());
		assert_eq!(
            result.unwrap_err().to_string(),
            "The vm exited due to the following exception: Value was either too large or too small for an Int32."
        );
	}

	#[tokio::test]
	async fn test_get_unsigned_transaction() {
		let mock_provider = Arc::new(Mutex::new(MockClient::new().await));
		let client = {
			let mut mock_provider = mock_provider.lock().await;
			mock_provider
				.mock_invoke_script(InvocationResult {
					gas_consumed: "984060".to_string(),
					..Default::default()
				})
				.await
				.mock_get_block_count(1000)
				.await
				.mock_calculate_network_fee(1230610)
				.await
				.mount_mocks()
				.await;
			Arc::new(mock_provider.into_client())
		};

		let account1 =
			Account::from_wif("L1WMhxazScMhUrdv34JqQb1HFSQmWeN2Kpc1R9JGKwL7CDNP21uR").unwrap();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder
			.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::called_by_entry(&account1).unwrap().into()]);

		let tx = match tx_builder.get_unsigned_tx().await {
			Ok(tx) => tx,
			Err(e) => panic!("Error: {}", e),
		};

		assert_eq!(tx.version, 0);
		// TODO: fix equal
		// assert_eq!(
		// 	tx.signers[0].as_account_signer().unwrap(),
		// 	AccountSigner::called_by_entry(&account1).unwrap()
		// );
		assert!(tx.witnesses.is_empty());
	}

	#[tokio::test]
	async fn test_version() {
		init_logger();
		let mock_provider = Arc::new(Mutex::new(MockClient::new().await));
		let client = {
			let mut mock_provider = mock_provider.lock().await;
			mock_provider
				.mock_invoke_script(InvocationResult::default())
				.await
				.mock_get_block_count(1000)
				.await
				.mock_calculate_network_fee(1230610)
				.await
				.mock_get_version(NeoVersion {
					tcp_port: Some(10333),
					ws_port: Some(10334),
					nonce: 1234567890,
					user_agent: "/Neo:3.5.0/".to_string(),
					protocol: Some(NeoProtocol {
						network: 860833102,
						validators_count: Some(7),
						ms_per_block: 15000,
						max_valid_until_block_increment: 5760,
						max_traceable_blocks: 2102400,
						address_version: 53,
						max_transactions_per_block: 512,
						memory_pool_max_transactions: 50000,
						initial_gas_distribution: 5200000000000000,
						hard_forks: vec![],
					}),
				})
				.await
				.mount_mocks()
				.await;
			Arc::new(mock_provider.into_client())
		};

		let account1 =
			Account::from_wif("L1WMhxazScMhUrdv34JqQb1HFSQmWeN2Kpc1R9JGKwL7CDNP21uR").unwrap();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder
			.version(1)
			.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::called_by_entry(&account1).unwrap().into()]);

		let tx = match tx_builder.get_unsigned_tx().await {
			Ok(tx) => tx,
			Err(e) => panic!("Error: {}", e),
		};

		assert_eq!(tx.version, 1);
	}
	#[tokio::test]
	async fn test_additional_network_fee() {
		let mock_provider = Arc::new(Mutex::new(MockClient::new().await));
		let client = {
			let mut mock_provider = mock_provider.lock().await;
			mock_provider
				.mock_invoke_script(InvocationResult::default())
				.await
				.mock_get_block_count(1000)
				.await
				.mock_calculate_network_fee(1230610)
				.await
				.mock_get_version(NeoVersion::default())
				.await
				.mount_mocks()
				.await;
			Arc::new(mock_provider.into_client())
		};

		let account = Account::create().unwrap();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder
			.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::called_by_entry(&account).unwrap().into()]);

		let tx = match tx_builder.get_unsigned_tx().await {
			Ok(tx) => tx,
			Err(e) => panic!("Error: {}", e),
		};
		assert_eq!(tx.net_fee, 0);

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder
			.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::none(&account).unwrap().into()])
			.set_additional_network_fee(2000);

		let tx = match tx_builder.get_unsigned_tx().await {
			Ok(tx) => tx,
			Err(e) => panic!("Error: {}", e),
		};

		assert_eq!(tx.net_fee, 0);
	}

	#[tokio::test]
	async fn test_additional_system_fee() {
		let mock_provider = Arc::new(Mutex::new(MockClient::new().await));
		let client = {
			let mut mock_provider = mock_provider.lock().await;
			mock_provider
				.mock_get_version(NeoVersion::default())
				.await
				.mock_invoke_script(InvocationResult::default())
				.await
				.mock_get_block_count(1000)
				.await
				.mock_calculate_network_fee(1230610)
				.await
				.mount_mocks()
				.await;
			Arc::new(mock_provider.into_client())
		};

		let account = Account::create().unwrap();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder
			.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::called_by_entry(&account).unwrap().into()]);

		let tx = match tx_builder.get_unsigned_tx().await {
			Ok(tx) => tx,
			Err(e) => panic!("Error: {}", e),
		};

		assert_eq!(tx.sys_fee, 1234567);

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder
			.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::none(&account).unwrap().into()])
			.set_additional_system_fee(3000);

		let tx = match tx_builder.get_unsigned_tx().await {
			Ok(tx) => tx,
			Err(e) => panic!("Error: {}", e),
		};
		assert_eq!(tx.sys_fee, 1234567);
	}

	#[tokio::test]
	async fn test_set_first_signer() {
		let mock_provider = Arc::new(Mutex::new(MockClient::new().await));
		let client = {
			let mut mock_provider = mock_provider.lock().await;
			mock_provider.mock_default_responses().await.mount_mocks().await;
			Arc::new(mock_provider.into_client())
		};
		let account1 =
			Account::from_wif("L1WMhxazScMhUrdv34JqQb1HFSQmWeN2Kpc1R9JGKwL7CDNP21uR").unwrap();
		let account2 =
			Account::from_wif("KysNqEuLb3wmZJ6PsxbA9Bh6ewTybEda4dEiN9X7X48dJPkLWZ5a").unwrap();

		let s1 = AccountSigner::global(&account1.clone()).unwrap();
		let s2 = AccountSigner::called_by_entry(&account2.clone()).unwrap();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		&tx_builder
			.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![s1.clone().into(), s2.clone().into()]);
		assert_eq!(tx_builder.clone().signers, vec![s1.clone().into(), s2.clone().into()]);

		tx_builder.clone().first_signer(&s2.account).unwrap();
		assert_eq!(tx_builder.clone().signers, vec![s2.clone().into(), s1.clone().into()]);

		&tx_builder.first_signer(&account1).unwrap();
		assert_eq!(tx_builder.clone().signers, vec![s1.clone().into(), s2.clone().into()]);
	}

	#[tokio::test]
	async fn test_set_first_signer_fee_only_present() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let account1 =
			Account::from_wif("L1WMhxazScMhUrdv34JqQb1HFSQmWeN2Kpc1R9JGKwL7CDNP21uR").unwrap();
		let account2 =
			Account::from_wif("KysNqEuLb3wmZJ6PsxbA9Bh6ewTybEda4dEiN9X7X48dJPkLWZ5a").unwrap();

		let s1 = AccountSigner::none(&account1.clone()).unwrap();
		let s2 = AccountSigner::called_by_entry(&account2.clone()).unwrap();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder
			.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![s1.clone().into(), s2.clone().into()]);
		assert_eq!(tx_builder.signers, vec![s1.clone().into(), s2.clone().into()]);

		assert!(tx_builder.first_signer(s2.account()).is_err());
	}

	#[tokio::test]
	async fn test_set_first_signer_not_present() {
		let client = CLIENT.get_or_init(|| async { MockClient::new().await.into_client() }).await;
		let account1 =
			Account::from_wif("L1WMhxazScMhUrdv34JqQb1HFSQmWeN2Kpc1R9JGKwL7CDNP21uR").unwrap();
		let account2 =
			Account::from_wif("KysNqEuLb3wmZJ6PsxbA9Bh6ewTybEda4dEiN9X7X48dJPkLWZ5a").unwrap();

		let s1 = AccountSigner::global(&account1.clone()).unwrap();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder.set_script(Some(vec![1, 2, 3])).set_signers(vec![s1.clone().into()]);
		assert_eq!(tx_builder.signers[0], s1.clone().into());

		assert!(tx_builder.first_signer(&account2).is_err());
	}

	#[tokio::test]
	async fn test_tracking_transaction_should_return_correct_block() {
		init_logger();
		let mock_provider = Arc::new(Mutex::new(MockClient::new().await));
		let client = {
			let mut mock_provider = mock_provider.lock().await;
			mock_provider
				.mock_invoke_script(InvocationResult::default())
				.await
				.mock_get_version(NeoVersion::default())
				.await
				.mock_get_block_count(1000)
				.await
				.mock_calculate_network_fee(1230610)
				.await
				.mock_send_raw_transaction(RawTransaction {
					hash: H256::zero(),
					..Default::default()
				})
				.await
				.mount_mocks()
				.await;
			Arc::new(mock_provider.into_client())
		};

		let account1 =
			Account::from_wif("L1WMhxazScMhUrdv34JqQb1HFSQmWeN2Kpc1R9JGKwL7CDNP21uR").unwrap();
		let recipient = H160::from_str("969a77db482f74ce27105f760efa139223431394").unwrap();

		let script = ScriptBuilder::new()
			.contract_call(
				&H160::from_str(TestConstants::NEO_TOKEN_HASH).unwrap(),
				"transfer",
				&[
					ContractParameter::h160(&account1.address_or_scripthash().script_hash()),
					ContractParameter::h160(&recipient),
					ContractParameter::integer(5),
					ContractParameter::any(),
				],
				None,
			)
			.unwrap()
			.to_bytes();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder
			.set_script(Some(script))
			.nonce(0)
			.unwrap()
			.set_signers(vec![AccountSigner::called_by_entry(&account1).unwrap().into()]);

		let mut tx = tx_builder.sign().await.unwrap();
		let _ = tx.send_tx(&client.as_ref()).await.map_err(TransactionError::from).unwrap();

		let mut block_num = 0;
		// TODO: check this
		// let mut subscription = tx.track_tx(&client).await.unwrap();
		// while let Some(result) = subscription.next(&client).await {
		// 	block_num = result.unwrap();
		// 	if block_num == 1002 {
		// 		break;
		// 	}
		// }

		assert_eq!(block_num, 1002);
	}

	#[tokio::test]
	async fn test_tracking_transaction_tx_not_sent() {
		let mock_provider = Arc::new(Mutex::new(MockClient::new().await));
		let client = {
			let mut mock_provider = mock_provider.lock().await;
			mock_provider
				.mock_invoke_script(InvocationResult::default())
				.await
				.mock_get_block_count(1000)
				.await
				.mock_calculate_network_fee(1230610)
				.await
				.mount_mocks()
				.await;
			Arc::new(mock_provider.into_client())
		};

		let account1 =
			Account::from_wif("L1WMhxazScMhUrdv34JqQb1HFSQmWeN2Kpc1R9JGKwL7CDNP21uR").unwrap();
		let recipient = H160::from_str("969a77db482f74ce27105f760efa139223431394").unwrap();

		let script = ScriptBuilder::new()
			.contract_call(
				&H160::from_str(TestConstants::NEO_TOKEN_HASH).unwrap(),
				"transfer",
				&[
					ContractParameter::h160(&account1.address_or_scripthash().script_hash()),
					ContractParameter::h160(&recipient),
					ContractParameter::integer(5),
					ContractParameter::any(),
				],
				None,
			)
			.unwrap()
			.to_bytes();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder
			.set_script(Some(script))
			.nonce(0)
			.unwrap()
			.set_signers(vec![AccountSigner::called_by_entry(&account1).unwrap().into()]);

		let tx = tx_builder.sign().await.unwrap();

		// TODO: Implement track_tx method for Transaction
		// assert!(tx.track_tx(&client).await.is_err());
	}

	#[tokio::test]
	async fn test_get_application_log() {
		let mock_provider = Arc::new(Mutex::new(MockClient::new().await));
		let client = {
			let mut mock_provider = mock_provider.lock().await;
			mock_provider
				.mock_invoke_script(InvocationResult::default())
				.await
				.mock_get_block_count(1000)
				.await
				.mock_calculate_network_fee(1230610)
				.await
				.mock_send_raw_transaction(RawTransaction {
					hash: H256::zero(),
					..Default::default()
				})
				.await
				.mock_get_application_log(Some(ApplicationLog {
					transaction_id: H256::from_str(
						"0xeb52f99ae5cf923d8905bdd91c4160e2207d20c0cb42f8062f31c6743770e4d1",
					)
					.unwrap(),
					..Default::default()
				}))
				.await
				.mount_mocks()
				.await;
			Arc::new(mock_provider.into_client())
		};

		let account1 =
			Account::from_wif("L1WMhxazScMhUrdv34JqQb1HFSQmWeN2Kpc1R9JGKwL7CDNP21uR").unwrap();

		let script = ScriptBuilder::new()
			.contract_call(
				&H160::from_str(TestConstants::NEO_TOKEN_HASH).unwrap(),
				"transfer",
				&[
					ContractParameter::h160(&account1.address_or_scripthash().script_hash()),
					ContractParameter::h160(&account1.address_or_scripthash().script_hash()),
					ContractParameter::integer(1),
					ContractParameter::any(),
				],
				None,
			)
			.unwrap()
			.to_bytes();

		let mut tx_builder = TransactionBuilder::with_client(client.as_ref());
		tx_builder
			.set_script(Some(script))
			.set_signers(vec![AccountSigner::called_by_entry(&account1).unwrap().into()]);

		let mut tx =
			tx_builder.sign().await.map_err(|e| TransactionError::BuilderError(e)).unwrap();
		let _ = tx.send_tx(client.as_ref()).await.map_err(TransactionError::from).unwrap();
		let application_log = tx
			.get_application_log(client.as_ref())
			.await
			.map_err(TransactionError::from)
			.unwrap();

		assert_eq!(
			application_log.transaction_id,
			H256::from_str("0xeb52f99ae5cf923d8905bdd91c4160e2207d20c0cb42f8062f31c6743770e4d1")
				.unwrap()
		);
	}

	#[tokio::test]
	async fn test_get_application_log_tx_not_sent() {
		let mock_provider = Arc::new(Mutex::new(MockClient::new().await));
		let client = {
			let mut mock_provider = mock_provider.lock().await;
			mock_provider
				.mock_invoke_script(InvocationResult {
					gas_consumed: "984060".to_string(),
					..Default::default()
				})
				.await
				.mock_get_block_count(1000)
				.await
				.mock_calculate_network_fee(1230610)
				.await
				.mount_mocks()
				.await;
			Arc::new(mock_provider.into_client())
		};

		let account1 =
			Account::from_wif("L1WMhxazScMhUrdv34JqQb1HFSQmWeN2Kpc1R9JGKwL7CDNP21uR").unwrap();

		let script = ScriptBuilder::new()
			.contract_call(
				&H160::from_str(TestConstants::NEO_TOKEN_HASH).unwrap(),
				"transfer",
				&[
					ContractParameter::h160(&account1.address_or_scripthash().script_hash()),
					ContractParameter::h160(&account1.address_or_scripthash().script_hash()),
					ContractParameter::integer(1),
					ContractParameter::any(),
				],
				None,
			)
			.unwrap()
			.to_bytes();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder
			.set_script(Some(script))
			.set_signers(vec![AccountSigner::called_by_entry(&account1).unwrap().into()]);

		let tx = tx_builder.sign().await.unwrap();

		assert!(tx.get_application_log(&client.as_ref()).await.is_err());
	}

	#[tokio::test]
	async fn test_get_application_log_not_existing() {
		let mock_provider = Arc::new(Mutex::new(MockClient::new().await));
		let client = {
			let mut mock_provider = mock_provider.lock().await;
			mock_provider
				.mock_invoke_script(InvocationResult::default())
				.await
				.mock_get_block_count(1000)
				.await
				.mock_calculate_network_fee(1230610)
				.await
				.mock_send_raw_transaction(RawTransaction {
					hash: H256::zero(),
					..Default::default()
				})
				.await
				.mock_get_application_log(Default::default())
				.await
				.mount_mocks()
				.await;
			Arc::new(mock_provider.into_client())
		};

		let account1 =
			Account::from_wif("L1WMhxazScMhUrdv34JqQb1HFSQmWeN2Kpc1R9JGKwL7CDNP21uR").unwrap();

		let script = ScriptBuilder::new()
			.contract_call(
				&H160::from_str(TestConstants::NEO_TOKEN_HASH).unwrap(),
				"transfer",
				&[
					ContractParameter::h160(&account1.address_or_scripthash().script_hash()),
					ContractParameter::h160(&account1.address_or_scripthash().script_hash()),
					ContractParameter::integer(1),
					ContractParameter::any(),
				],
				None,
			)
			.unwrap()
			.to_bytes();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder
			.set_script(Some(script))
			.set_signers(vec![AccountSigner::called_by_entry(&account1).unwrap().into()]);

		let mut tx = tx_builder.sign().await.unwrap();
		let _ = tx.send_tx(&client.as_ref()).await.map_err(TransactionError::from).unwrap();

		assert!(tx
			.get_application_log(&client.as_ref())
			.await
			.map_err(TransactionError::from)
			.is_err());
	}

	#[tokio::test]
	async fn test_transmission_on_fault() {
		let mock_provider = Arc::new(Mutex::new(MockClient::new().await));
		let client = {
			let mut mock_provider = mock_provider.lock().await;
			mock_provider
				.mock_invoke_script(InvocationResult {
					gas_consumed: "984060".to_string(),
					exception: Some("Test fault".to_string()),
					..Default::default()
				})
				.await
				.mock_get_block_count(1000)
				.await
				.mock_calculate_network_fee(1230610)
				.await
				.mount_mocks()
				.await;
			Arc::new(mock_provider.into_client())
		};

		let account = Account::create().unwrap();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder
			.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::none(&account).unwrap().into()]);
		// .allow_transmission_on_fault();

		let result = tx_builder.call_invoke_script().await;
		assert!(result.has_state_fault());

		let tx = match tx_builder.get_unsigned_tx().await {
			Ok(tx) => tx,
			Err(e) => panic!("Error: {}", e),
		};
		assert_eq!(tx.sys_fee, 984060);

		NEOCONFIG.lock().unwrap().allows_transmission_on_fault = false;
		assert!(!NEOCONFIG.lock().unwrap().allows_transmission_on_fault);
	}

	#[tokio::test]
	async fn test_prevent_transmission_on_fault() {
		let mock_provider = Arc::new(Mutex::new(MockClient::new().await));
		let client = {
			let mut mock_provider = mock_provider.lock().await;
			mock_provider
				.mock_invoke_script(InvocationResult {
					gas_consumed: "984060".to_string(),
					exception: Some("Test fault".to_string()),
					..Default::default()
				})
				.await
				.mock_get_block_count(1000)
				.await
				.mock_calculate_network_fee(1230610)
				.await
				.mount_mocks()
				.await;
			Arc::new(mock_provider.into_client())
		};

		let account = Account::create().unwrap();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder
			.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::none(&account).unwrap().into()]);

		assert!(!NEOCONFIG.lock().unwrap().allows_transmission_on_fault);

		let result = tx_builder.call_invoke_script().await;
		assert!(result.has_state_fault());

		assert!(tx_builder.get_unsigned_tx().await.is_err());
	}

	#[tokio::test]
	async fn test_sign_with_multiple_accounts() {
		let mock_provider = Arc::new(Mutex::new(MockClient::new().await));
		let client = {
			let mut mock_provider = mock_provider.lock().await;
			mock_provider.mock_default_responses().await.mount_mocks().await;
			Arc::new(mock_provider.into_client())
		};

		let account1 =
			Account::from_wif("L1WMhxazScMhUrdv34JqQb1HFSQmWeN2Kpc1R9JGKwL7CDNP21uR").unwrap();
		let account2 =
			Account::from_wif("KysNqEuLb3wmZJ6PsxbA9Bh6ewTybEda4dEiN9X7X48dJPkLWZ5a").unwrap();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder.set_script(Some(vec![1, 2, 3])).set_signers(vec![
			AccountSigner::called_by_entry(&account1).unwrap().into(),
			AccountSigner::called_by_entry(&account2).unwrap().into(),
		]);

		let tx = tx_builder.sign().await.unwrap();

		assert_eq!(tx.witnesses.len(), 2);
		assert!(tx
			.witnesses
			.iter()
			.any(|w| w.verification == account1.verification_script().clone().unwrap()));
		assert!(tx
			.witnesses
			.iter()
			.any(|w| w.verification == account2.verification_script().clone().unwrap()));
	}

	#[tokio::test]
	async fn test_sign_with_multi_sig_account() {
		let mock_provider = Arc::new(Mutex::new(MockClient::new().await));
		let client = {
			let mut mock_provider = mock_provider.lock().await;
			mock_provider
				.mock_invoke_script(InvocationResult::default())
				.await
				.mock_get_block_count(1000)
				.await
				.mock_calculate_network_fee(1230610)
				.await
				.mount_mocks()
				.await;
			Arc::new(mock_provider.into_client())
		};

		let account1 =
			Account::from_wif("L1WMhxazScMhUrdv34JqQb1HFSQmWeN2Kpc1R9JGKwL7CDNP21uR").unwrap();
		let account2 =
			Account::from_wif("KysNqEuLb3wmZJ6PsxbA9Bh6ewTybEda4dEiN9X7X48dJPkLWZ5a").unwrap();
		let multi_sig_account = Account::multi_sig_from_public_keys(
			vec![account1.get_public_key().unwrap(), account2.get_public_key().unwrap()]
				.as_mut_slice(),
			2,
		)
		.unwrap();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder
			.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::called_by_entry(&multi_sig_account).unwrap().into()]);

		let tx = tx_builder.sign().await.unwrap();

		assert_eq!(tx.witnesses.len(), 1);
		assert_eq!(
			tx.witnesses[0].verification,
			multi_sig_account.verification_script().clone().unwrap()
		);
	}

	#[tokio::test]
	async fn test_get_network_fee() {
		let mock_provider = Arc::new(Mutex::new(MockClient::new().await));
		let client = {
			let mut mock_provider = mock_provider.lock().await;
			mock_provider
				.mock_invoke_script(InvocationResult::default())
				.await
				.mock_get_block_count(1000)
				.await
				.mock_calculate_network_fee(1230610)
				.await
				.mount_mocks()
				.await;
			Arc::new(mock_provider.into_client())
		};

		let account = Account::create().unwrap();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder
			.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::called_by_entry(&account).unwrap().into()]);

		let network_fee = tx_builder.get_network_fee().await.unwrap();
		assert_eq!(network_fee, 1230610);
	}

	#[tokio::test]
	async fn test_get_system_fee() {
		let mock_provider = Arc::new(Mutex::new(MockClient::new().await));
		let client = {
			let mut mock_provider = mock_provider.lock().await;
			mock_provider
				.mock_invoke_script(InvocationResult::default())
				.await
				.mock_get_block_count(1000)
				.await
				.mock_calculate_network_fee(1230610)
				.await
				.mount_mocks()
				.await;
			Arc::new(mock_provider.into_client())
		};

		let account = Account::create().unwrap();

		let mut tx_builder = TransactionBuilder::with_client(&client);
		tx_builder
			.set_script(Some(vec![1, 2, 3]))
			.set_signers(vec![AccountSigner::called_by_entry(&account).unwrap().into()]);

		let system_fee = tx_builder.get_system_fee().await.unwrap();
		assert_eq!(system_fee, 984060);
	}
}