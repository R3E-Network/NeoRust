use lazy_static::lazy_static;
use neo::prelude::*;
use primitive_types::{H160, H256};
use regex::Regex;
use serde_json::{json, Value};
use std::{collections::HashMap, fs, path::PathBuf, str::FromStr, sync::Arc};
use tokio::sync::Mutex;
use url::Url;
use wiremock::{
	matchers::{body_json, body_partial_json, method, path},
	Match, Mock, MockServer, ResponseTemplate,
};

lazy_static! {
	pub static ref ACCOUNT1: Account = Account::from_key_pair(
		KeyPair::from_secret_key(
			&Secp256r1PrivateKey::from_bytes(
				&hex::decode("e6e919577dd7b8e97805151c05ae07ff4f752654d6d8797597aca989c02c4cb3")
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
				&hex::decode("b4b2b579cac270125259f08a5f414e9235817e7637b9a66cfeb3b77d90c8e7f9")
					.unwrap()
			)
			.unwrap()
		),
		None,
		None
	)
	.expect("Failed to create ACCOUNT2");
}

pub struct MockClient {
	server: MockServer,
	mocks: Vec<Mock>,
}

impl MockClient {
	pub async fn new() -> Self {
		let server = MockServer::start().await;
		Self { server, mocks: Vec::new() }
	}

	pub async fn mock_response(
		&mut self,
		method_name: &str,
		params: serde_json::Value,
		result: serde_json::Value,
	) {
		let mock = Mock::given(method("POST"))
			.and(path("/"))
			.and(body_json(json!({
				"jsonrpc": "2.0",
				"method": method_name,
				"params": params,
				"id": 1
			})))
			.respond_with(ResponseTemplate::new(200).set_body_json(json!({
				"jsonrpc": "2.0",
				"id": 1,
				"result": result
			})));
		self.mocks.push(mock);
	}

	pub async fn mock_response_error(&mut self, error: serde_json::Value) {
		let mock = Mock::given(method("POST")).and(path("/")).respond_with(
			ResponseTemplate::new(200).set_body_json(json!({
				"jsonrpc": "2.0",
				"id": 1,
				"error": error
			})),
		);
		self.mocks.push(mock);
	}

	pub async fn mock_response_ignore_param(
		&mut self,
		method_name: &str,
		result: serde_json::Value,
	) -> &mut Self {
		let mock = Mock::given(method("POST"))
			.and(path("/"))
			.and(body_partial_json(json!({
				"jsonrpc": "2.0",
				"method": method_name,
			})))
			.respond_with(ResponseTemplate::new(200).set_body_json(json!({
				"jsonrpc": "2.0",
				"id": 1,
				"result": result
			})));
		self.mocks.push(mock);
		self
	}

	pub async fn mock_response_with_file(
		&mut self,
		method_name: &str,
		response_file: &str,
		params: serde_json::Value,
	) -> &mut Self {
		// Construct the path to the response file relative to the project root
		let mut response_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
		response_file_path.push("test_resources");
		response_file_path.push("responses");
		response_file_path.push(response_file);

		// Load the response body from the specified file
		let response_body = tokio::fs::read_to_string(response_file_path)
			.await
			.expect("Failed to read response file");

		let mock = Mock::given(method("POST"))
			.and(path("/"))
			.and(body_partial_json(json!({
				"jsonrpc": "2.0",
				"method": method_name,
				"params": params,
			})))
			.respond_with(ResponseTemplate::new(200).set_body_string(response_body));
		self.mocks.push(mock);
		self
	}

	pub async fn mock_response_with_file_ignore_param(
		&mut self,
		method_name: &str,
		response_file: &str,
	) -> &mut Self {
		// Construct the path to the response file relative to the project root
		let mut response_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
		response_file_path.push("test_resources");
		response_file_path.push("responses");
		response_file_path.push(response_file);

		// Load the response body from the specified file
		let response_body = tokio::fs::read_to_string(response_file_path)
			.await
			.expect("Failed to read response file");
		// // Load the response body from the specified file
		// let response_body = tokio::fs::read_to_string(format!("/responses/{}", response_file))
		// .await
		// .expect("Failed to read response file");

		let mock = Mock::given(method("POST"))
			.and(path("/"))
			.and(body_partial_json(json!({
				"jsonrpc": "2.0",
				"method": method_name,
			})))
			.respond_with(ResponseTemplate::new(200).set_body_string(response_body));
		self.mocks.push(mock);
		self
	}

	pub async fn mock_response_for_balance_of(
		&mut self,
		contract_hash: &str,
		account_script_hash: &str,
		response_file: &str,
	) -> &mut Self {
		// Construct the path to the response file relative to the project root
		let mut response_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
		response_file_path.push("test_resources");
		response_file_path.push("responses");
		response_file_path.push(response_file);

		// Load the response body from the specified file
		let response_body = tokio::fs::read_to_string(response_file_path)
			.await
			.expect("Failed to read response file");

		let mock = Mock::given(method("POST"))
			.and(path("/"))
			.and(body_partial_json(json!({
				"jsonrpc": "2.0",
				"method": "invokefunction",
				"params": [
					contract_hash,
					"balanceOf",
					[
						{
							"type": "Hash160",
							"value": account_script_hash,
						}
					]
				],
			})))
			.respond_with(ResponseTemplate::new(200).set_body_string(response_body));

		self.mocks.push(mock);
		self
	}

	pub async fn mock_default_responses(&mut self) -> &mut Self {
		self.mock_response_with_file_ignore_param(
			"invokescript",
			"invokescript_necessary_mock.json",
		)
		.await;
		self.mock_response_with_file(
			"invokefunction",
			"invokefunction_transfer_neo.json",
			json!([
				TestConstants::NEO_TOKEN_HASH,
				"transfer",
				vec![
					ContractParameter::from(ACCOUNT1.address_or_scripthash().script_hash()),
					ContractParameter::from(
						H160::from_str("969a77db482f74ce27105f760efa139223431394").unwrap(),
					),
					ContractParameter::from(5),
					ContractParameter::any(),
				],
			]),
		)
		.await;
		self.mock_response_with_file_ignore_param("getblockcount", "getblockcount_1000.json")
			.await;
		self.mock_response_with_file_ignore_param(
			"calculatenetworkfee",
			"calculatenetworkfee.json",
		)
		.await;
		self
	}

	pub async fn mock_invoke_script(&mut self, result: InvocationResult) -> &mut Self {
		self.mock_response_ignore_param("invokescript", json!(Ok::<InvocationResult, ()>(result)))
			.await;
		self
	}

	// pub async fn mock_get_block_count(&mut self, result: i32) -> &mut Self {
	// 	self.mock_response_ignore_param("getblockcount", json!(Ok::<i32, ()>(result)))
	// 		.await;
	// 	self
	// }

	pub async fn mock_get_block_count(&mut self, block_count: u32) -> &mut Self {
		// Construct the path to the response file relative to the project root
		let mut response_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
		response_file_path.push("test_resources");
		response_file_path.push("responses");
		response_file_path.push(format!("getblockcount_{}.json", block_count));

		// Load the response body from the specified file
		let response_body = tokio::fs::read_to_string(response_file_path)
			.await
			.expect("Failed to read response file");
		// // Load the response body from the specified file
		// let response_body = tokio::fs::read_to_string(format!("/responses/{}", response_file))
		// .await
		// .expect("Failed to read response file");

		let mock = Mock::given(method("POST"))
			.and(path("/"))
			.and(body_partial_json(json!({
				"jsonrpc": "2.0",
				"method": "getblockcount",
			})))
			.respond_with(ResponseTemplate::new(200).set_body_string(response_body));
		self.mocks.push(mock);
		self
	}

	// pub async fn mock_calculate_network_fee(&mut self, result: i32) -> &mut Self {
	// 	self.mock_response_ignore_param("calculatenetworkfee", json!(Ok::<i32, ()>(result)))
	// 		.await;
	// 	self
	// }

	pub async fn mock_send_raw_transaction(&mut self, result: RawTransaction) -> &mut Self {
		self.mock_response_with_file_ignore_param("sendrawtransaction", "sendrawtransaction.json")
			.await;
		self
	}

	pub async fn mock_get_version(&mut self, result: NeoVersion) -> &mut Self {
		self.mock_response_ignore_param("getversion", json!(Ok::<NeoVersion, ()>(result)))
			.await;
		self
	}

	pub async fn mock_invoke_function(&mut self, result: InvocationResult) -> &mut Self {
		self.mock_response_ignore_param(
			"invokefunction",
			json!(Ok::<InvocationResult, ()>(result)),
		)
		.await;
		self
	}

	pub async fn mock_get_application_log(&mut self, result: Option<ApplicationLog>) -> &mut Self {
		self.mock_response_ignore_param("getapplicationlog", json!(result)).await;
		self
	}

	pub async fn mount_mocks(&mut self) -> &mut Self {
		for mock in self.mocks.drain(..) {
			mock.mount(&self.server).await;
		}
		self
	}

	pub fn url(&self) -> Url {
		Url::parse(&self.server.uri()).expect("Invalid mock server URL")
	}

	pub fn into_client(&self) -> RpcClient<HttpProvider> {
		let http_provider = HttpProvider::new(self.url()).expect("Failed to create HTTP provider");
		RpcClient::new(http_provider)
	}

	pub fn server(&self) -> &MockServer {
		&self.server
	}
}

/// Completely offline mock client that doesn't use any real network connections
/// Use this implementation for tests when running with a VPN or without network access
pub struct OfflineMockClient {
	/// Map of method name to predefined responses
	responses: HashMap<String, serde_json::Value>,
	/// Current response ID
	id: u64,
}

impl OfflineMockClient {
	/// Create a new offline mock client
	pub fn new() -> Self {
		Self { responses: HashMap::new(), id: 1 }
	}

	/// Mock a response for a specific method name ignoring parameters
	pub fn mock_response_ignore_param(
		&mut self,
		method_name: &str,
		result: serde_json::Value,
	) -> &mut Self {
		self.responses.insert(method_name.to_string(), result);
		self
	}

	/// Mock a response for a method using a JSON file
	pub fn mock_response_with_file_ignore_param(
		&mut self,
		method_name: &str,
		response_file: &str,
	) -> &mut Self {
		// Load from the test resources directory
		let path = PathBuf::from("test_resources").join(response_file);
		let content = fs::read_to_string(&path)
			.unwrap_or_else(|_| panic!("Failed to read test resource: {:?}", path));

		let result: serde_json::Value = serde_json::from_str(&content)
			.unwrap_or_else(|_| panic!("Invalid JSON in test resource: {:?}", path));

		self.responses.insert(method_name.to_string(), result);
		self
	}

	/// Add common mock responses for Neo RPC calls
	pub fn mock_default_responses(&mut self) -> &mut Self {
		// Block count
		self.mock_response_ignore_param("getblockcount", json!(1000));

		// Version
		self.mock_response_ignore_param(
			"getversion",
			json!({
				"protocol": {
					"network": 860833102,
					"validatorscount": 7,
					"msperblock": 15000
				},
				"tcpport": 10333,
				"wsport": 10334,
				"nonce": 1571394188,
				"useragent": "/NEO:3.0.3/"
			}),
		);

		// Mock invocation result
		self.mock_response_ignore_param(
			"invokescript",
			json!({
				"script": "DAABECcMFDiu8tS4X2XbLccGJGsHKQJ2qE1nFMAfDAR0ZXN0Bm9iamVjdFNldEYMFBfK8A==",
				"state": "HALT",
				"gasconsumed": "1007390",
				"stack": [
					{
						"type": "Boolean",
						"value": true
					}
				],
				"notifications": []
			}),
		);

		// Mock network fee
		self.mock_response_ignore_param(
			"calculatenetworkfee",
			json!({
				"networkfee": "1234567"
			}),
		);

		self
	}

	/// Convert this offline mock to a Neo RPC client
	pub fn into_client(&self) -> RpcClient<OfflineHttpProvider> {
		// Create an offline HTTP provider with our mocked responses
		let provider = OfflineHttpProvider::new(self.responses.clone());
		RpcClient::new(provider)
	}
}

/// Offline implementation of the HttpProvider that doesn't make actual HTTP requests
pub struct OfflineHttpProvider {
	responses: HashMap<String, serde_json::Value>,
	id: std::sync::atomic::AtomicU64,
}

impl OfflineHttpProvider {
	pub fn new(responses: HashMap<String, serde_json::Value>) -> Self {
		Self { responses, id: std::sync::atomic::AtomicU64::new(1) }
	}
}

impl JsonRpcProvider for OfflineHttpProvider {
	async fn request<T: for<'de> serde::Deserialize<'de>>(
		&self,
		method: &str,
		params: serde_json::Value,
	) -> Result<T, ProviderError> {
		// Get the mocked response or return an error
		let result = self.responses.get(method).cloned().unwrap_or_else(|| {
			json!({
				"error": {
					"code": -32601,
					"message": format!("Method '{}' not found or mocked", method)
				}
			})
		});

		// Build a JSON-RPC response
		let id = self.id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
		let response = json!({
			"jsonrpc": "2.0",
			"id": id,
			"result": result,
		});

		// Try to deserialize the response
		serde_json::from_value(response["result"].clone())
			.map_err(|e| ProviderError::JsonRpcError(e.to_string()))
	}
}
