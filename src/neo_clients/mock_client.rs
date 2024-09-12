use neo::prelude::*;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use url::Url;
use wiremock::{
	matchers::{body_json, body_partial_json, method, path},
	Mock, MockServer, ResponseTemplate,
};

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

	pub async fn mock_default_responses(&mut self) -> &mut Self {
		self.mock_response_ignore_param(
			"invokescript",
			json!(Ok::<InvocationResult, ()>(InvocationResult::default())),
		)
		.await;
		self.mock_invoke_function(InvocationResult::default()).await;
		self.mock_response_ignore_param("getblockcount", json!(Ok::<i32, ()>(1000)))
			.await;
		self.mock_response_ignore_param("calculatenetworkfee", json!(Ok::<i32, ()>(1000000)))
			.await;
		self
	}

	pub async fn mock_invoke_script(&mut self, result: InvocationResult) -> &mut Self {
		self.mock_response_ignore_param("invokescript", json!(Ok::<InvocationResult, ()>(result)))
			.await;
		self
	}

	pub async fn mock_get_block_count(&mut self, result: i32) -> &mut Self {
		self.mock_response_ignore_param("getblockcount", json!(Ok::<i32, ()>(result)))
			.await;
		self
	}

	pub async fn mock_calculate_network_fee(&mut self, result: i32) -> &mut Self {
		self.mock_response_ignore_param("calculatenetworkfee", json!(Ok::<i32, ()>(result)))
			.await;
		self
	}

	pub async fn mock_send_raw_transaction(&mut self, result: RawTransaction) -> &mut Self {
		self.mock_response_ignore_param(
			"sendrawtransaction",
			json!(Ok::<RawTransaction, ()>(result)),
		)
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
