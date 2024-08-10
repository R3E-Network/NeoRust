use neo::prelude::*;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::Mutex;
use url::Url;
use wiremock::{
	matchers::{body_json, body_partial_json, method, path},
	Mock, MockServer, ResponseTemplate,
};

/// A mock provider for testing Neo RPC interactions.
///
/// This provider wraps a `MockServer` to simulate RPC responses,
/// allowing for controlled testing of Neo RPC client code.
pub struct MockClient {
	server: MockServer,
}

impl MockClient {
	/// Creates a new instance of `MockProvider`.
	///
	/// This method starts a new `MockServer` and returns a `MockProvider`
	/// that wraps it.
	///
	/// # Returns
	///
	/// A new `MockProvider` instance.
	pub async fn new() -> Self {
		let server = MockServer::start().await;
		Self { server }
	}

	/// Mocks a response for a specific RPC method and parameters.
	///
	/// This method sets up the mock server to respond to a particular
	/// RPC method call with the specified parameters and result.
	///
	/// # Arguments
	///
	/// * `method` - The RPC method name.
	/// * `params` - The parameters for the RPC method call.
	/// * `result` - The result to be returned by the mock server.
	///
	/// # Example
	///
	/// ```
	/// # use serde_json::json;
	/// # use neo_rs::prelude::*;
	/// # #[tokio::main]
	/// # async fn main() {
	/// let mock_provider = MockProvider::new().await;
	/// mock_provider.mock_response(
	///     "getbestblockhash",
	///     json!([]),
	///     json!("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef")
	/// ).await;
	/// # }
	/// ```
	pub async fn mock_response(
		&self,
		method: &str,
		params: serde_json::Value,
		result: serde_json::Value,
	) {
		Mock::given(method("POST"))
			.and(path("/"))
			.and(body_json(json!({
				"jsonrpc": "2.0",
				"method": method,
				"params": params,
				"id": 1
			})))
			.respond_with(ResponseTemplate::new(200).set_body_json(json!({
				"jsonrpc": "2.0",
				"id": 1,
				"result": result
			})))
			.mount(&self.server)
			.await;
	}

	/// Mocks a response for a specific RPC method, ignoring the parameters.
	///
	/// This method sets up the mock server to respond to a particular
	/// RPC method call with the specified result, regardless of the parameters.
	///
	/// # Arguments
	///
	/// * `method` - The RPC method name.
	/// * `result` - The result to be returned by the mock server.
	///
	/// # Example
	///
	/// ```
	/// # use serde_json::json;
	/// # use neo_rs::prelude::*;
	/// # #[tokio::main]
	/// # async fn main() {
	/// let mock_provider = MockProvider::new().await;
	/// mock_provider.mock_response(
	///     "getbestblockhash",
	///     json!("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef")
	/// ).await;
	/// # }
	/// ```
	pub async fn mock_response_ignore_param(&self, method: &str, result: serde_json::Value) {
		Mock::given(method("POST"))
			.and(path("/"))
			.and(body_partial_json(json!({
				"jsonrpc": "2.0",
				"method": method,
			})))
			.respond_with(ResponseTemplate::new(200).set_body_json(json!({
				"jsonrpc": "2.0",
				"id": 1,
				"result": result
			})))
			.mount(&self.server)
			.await;
	}

	/// Returns the URL of the mock server.
	///
	/// This method can be used to get the URL of the underlying mock server,
	/// which can be useful for manual testing or debugging.
	///
	/// # Returns
	///
	/// A `Url` representing the address of the mock server.
	pub fn url(&self) -> Url {
		Url::parse(&self.server.uri()).expect("Invalid mock server URL")
	}

	/// Converts this `MockProvider` into a `Provider<HttpProvider>`.
	///
	/// This method creates a new `HttpProvider` using the URL of the mock server,
	/// and then wraps it in a `Provider`. This allows the mock to be used
	/// in place of a real Neo RPC provider in tests.
	///
	/// # Returns
	///
	/// A `Provider<HttpProvider>` that uses this mock server.
	pub fn into_client(&self) -> NeoClient<HttpProvider> {
		let http_provider = HttpProvider::new(self.url());
		NeoClient::new(http_provider)
	}

	/// Returns a reference to the internal `MockServer`.
	///
	/// This method provides access to the underlying `MockServer` instance,
	/// allowing for more advanced operations and verifications in tests.
	///
	/// # Returns
	///
	/// A reference to the `MockServer`.
	pub fn server(&self) -> &MockServer {
		&self.server
	}
}
