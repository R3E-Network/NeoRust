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
}

impl MockClient {
    pub async fn new() -> Self {
        let server = MockServer::start().await;
        Self { server }
    }

    pub async fn mock_response(
        &self,
        method_name: &str,
        params: serde_json::Value,
        result: serde_json::Value,
    ) {
        Mock::given(method("POST"))
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
			})))
            .mount(&self.server)
            .await;
    }

    pub async fn mock_response_ignore_param(&self, method_name: &str, result: serde_json::Value) {
        Mock::given(method("POST"))
            .and(path("/"))
            .and(body_partial_json(json!({
				"jsonrpc": "2.0",
				"method": method_name,
			})))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
				"jsonrpc": "2.0",
				"id": 1,
				"result": result
			})))
            .mount(&self.server)
            .await;
    }

    pub fn url(&self) -> Url {
        Url::parse(&self.server.uri()).expect("Invalid mock server URL")
    }

    pub fn into_client(&self) -> RpcClient<HttpProvider> {
        let http_provider = HttpProvider::new(self.url());
        RpcClient::new(http_provider)
    }

    pub fn server(&self) -> &MockServer {
        &self.server
    }
}
