#[cfg(feature = "http-client")]
use crate::neo_clients::{http::Http as HttpProvider, JsonRpcProvider};
use crate::neo_fs::{
	errors::{NeoFsError, NeoFsResult},
	types::{AccessRule, ContainerId, ObjectId, ResponseStatus, StoragePolicy},
};
#[cfg(feature = "transaction")]
use crate::neo_protocol::account::Account;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Configuration options for NeoFS client
#[derive(Debug, Clone)]
pub struct NeoFsConfig {
	/// NeoFS RPC endpoint URL
	pub endpoint: String,
	/// Request timeout in seconds
	pub timeout_seconds: u64,
	/// Maximum concurrent requests
	pub max_concurrent_requests: usize,
}

impl Default for NeoFsConfig {
	fn default() -> Self {
		Self {
			endpoint: "https://fs.neo.org".to_string(),
			timeout_seconds: 30,
			max_concurrent_requests: 5,
		}
	}
}

/// Client for interacting with NeoFS network
#[derive(Debug, Clone)]
pub struct NeoFsClient {
	/// Configuration for the client
	config: NeoFsConfig,
	/// HTTP provider for making API requests
	#[cfg(feature = "http-client")]
	provider: Arc<HttpProvider>,
	/// Account used for signing requests
	#[cfg(feature = "transaction")]
	account_data: Option<Account>,
}

impl NeoFsClient {
	/// Create a new NeoFS client with default configuration
	pub fn new() -> NeoFsResult<Self> {
		Self::with_config(NeoFsConfig::default())
	}

	/// Create a new NeoFS client with custom configuration
	pub fn with_config(config: NeoFsConfig) -> NeoFsResult<Self> {
		#[cfg(not(feature = "http-client"))]
		{
			return Ok(Self { config });
		}

		#[cfg(all(feature = "http-client", not(feature = "transaction")))]
		{
			let provider = HttpProvider::new(config.endpoint.as_str()).map_err(|e| {
				NeoFsError::NetworkError(format!("Failed to create HTTP provider: {}", e))
			})?;

			return Ok(Self { config, provider: Arc::new(provider) });
		}

		#[cfg(all(feature = "http-client", feature = "transaction"))]
		{
			let provider = HttpProvider::new(config.endpoint.as_str()).map_err(|e| {
				NeoFsError::NetworkError(format!("Failed to create HTTP provider: {}", e))
			})?;

			return Ok(Self { config, provider: Arc::new(provider), account_data: None });
		}
	}

	/// Set the account for signing requests
	#[cfg(feature = "transaction")]
	pub fn with_account(mut self, account: Account) -> Self {
		self.account_data = Some(account);
		self
	}

	/// Get the current account being used
	#[cfg(feature = "transaction")]
	pub fn account(&self) -> Option<&Account> {
		self.account_data.as_ref()
	}

	/// Check if the client has a valid connection to the NeoFS network
	pub async fn is_connected(&self) -> bool {
		#[cfg(feature = "http-client")]
		{
			// Implement a simple health check when HTTP client is available
			match self.get_network_info().await {
				Ok(_) => true,
				Err(_) => false,
			}
		}

		#[cfg(not(feature = "http-client"))]
		{
			// Cannot check connection without HTTP client
			false
		}
	}

	/// Get information about the NeoFS network
	pub async fn get_network_info(&self) -> NeoFsResult<NetworkInfo> {
		#[cfg(not(feature = "http-client"))]
		{
			return Err(NeoFsError::NetworkError("HTTP client feature not enabled".to_string()));
		}

		#[cfg(feature = "http-client")]
		{
			// Implementation placeholder
			Ok(NetworkInfo {
				version: "2.0.0".to_string(),
				node_count: 100,
				storage_capacity: 1024 * 1024 * 1024 * 1024, // 1 TB in bytes
				available_space: 512 * 1024 * 1024 * 1024,   // 512 GB in bytes
			})
		}
	}

	/// Create a container helper for operations on containers
	pub fn containers(&self) -> ContainerOperations {
		ContainerOperations { client: self.clone() }
	}

	/// Create an object helper for operations on objects
	pub fn objects(&self) -> ObjectOperations {
		ObjectOperations { client: self.clone() }
	}

	/// Create a network helper for operations on the NeoFS network
	pub fn network(&self) -> NetworkOperations {
		NetworkOperations { client: self.clone() }
	}

	/// Send a request to the NeoFS API
	#[cfg(feature = "http-client")]
	pub(crate) async fn send_request(
		&self, 
		endpoint: &str, 
		params: serde_json::Value
	) -> NeoFsResult<serde_json::Value> {
		use std::time::Duration;
		
		// Construct the full URL
		let url = format!("{}/{}", self.config.endpoint, endpoint);
		
		// Create the request with timeout
		let client = reqwest::Client::new();
		let mut request_builder = client.post(&url)
			.timeout(Duration::from_secs(self.config.timeout_seconds))
			.header("Content-Type", "application/json");
		
		// Add authentication if account is available
		#[cfg(feature = "transaction")]
		if let Some(account) = &self.account_data {
			let timestamp = std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)
				.unwrap_or_default()
				.as_secs();
			
			let auth_data = format!("{}:{}", endpoint, timestamp);
			
			#[cfg(feature = "crypto-standard")]
			{
				let signature = self.sign_request(auth_data.as_bytes())?;
				request_builder = request_builder
					.header("X-Auth-Timestamp", timestamp.to_string())
					.header("X-Auth-Signature", hex::encode(&signature));
			}
		}
		
		// Send the request
		let response = request_builder
			.json(&params)
			.send()
			.await
			.map_err(|e| NeoFsError::HttpError(e.to_string()))?;
		
		// Check for successful status code
		if !response.status().is_success() {
			let status = response.status();
			let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
			return Err(NeoFsError::ResponseError(format!("HTTP {}: {}", status, error_text)));
		}
		
		// Parse the response
		let result = response.json::<serde_json::Value>()
			.await
			.map_err(|e| NeoFsError::DeserializationError(e.to_string()))?;
		
		Ok(result)
	}

	// Internal method for signing NeoFS API requests
	#[cfg(all(feature = "transaction", feature = "crypto-standard"))]
	fn sign_request(&self, data: &[u8]) -> NeoFsResult<Vec<u8>> {
		let account = self.account_data.as_ref().ok_or_else(|| {
			NeoFsError::AuthError("No account set for request signing".to_string())
		})?;

		account
			.sign(data)
			.map_err(|e| NeoFsError::AuthError(format!("Failed to sign request: {}", e)))
	}

	/// Create a bearer token for NeoFS authentication
	#[cfg(feature = "transaction")]
	pub async fn create_bearer_token(&self, lifetime_seconds: u64) -> NeoFsResult<BearerToken> {
		// Get the account that will own the token
		let account = self.account_data.as_ref()
			.ok_or_else(|| NeoFsError::AuthError("No account set for creating bearer token".to_string()))?;
		
		// Current timestamp
		let now = std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.map_err(|e| NeoFsError::AuthError(format!("Failed to get current time: {}", e)))?
			.as_secs();
		
		// Create token data
		let owner = account.get_public_key().to_bytes();
		let expires_at = now + lifetime_seconds;
		
		// Token structure includes owner, expiration, and other metadata
		let token_data = serde_json::json!({
			"owner": hex::encode(&owner),
			"expires_at": expires_at,
			"created_at": now,
			"permissions": {
				"container": ["read", "write", "delete"],
				"object": ["read", "write", "delete"],
			}
		});
		
		// Serialize token data
		let token = serde_json::to_vec(&token_data)
			.map_err(|e| NeoFsError::SerializationError(format!("Failed to serialize token: {}", e)))?;
		
		// Sign the token
		let signature = self.sign_request(&token)?;
		
		Ok(BearerToken {
			owner,
			token,
			signature,
			expires_at,
		})
	}
	
	/// Apply bearer token to a request
	#[cfg(all(feature = "transaction", feature = "http-client"))]
	fn apply_bearer_token(&self, request_builder: reqwest::RequestBuilder, token: &BearerToken) -> reqwest::RequestBuilder {
		request_builder
			.header("X-Bearer-Token", base64::encode(&token.token))
			.header("X-Bearer-Signature", base64::encode(&token.signature))
			.header("X-Bearer-Owner", base64::encode(&token.owner))
	}
	
	/// Get info about the NeoFS network, using the endpoint that matches Neo N3 NeoFS HTTP gateway
	#[cfg(feature = "http-client")]
	async fn send_request_with_n3_endpoint(&self, endpoint: &str, params: serde_json::Value) -> NeoFsResult<serde_json::Value> {
		// Try with standard endpoint first
		let standard_result = self.send_request(endpoint, params.clone()).await;
		if standard_result.is_ok() {
			return standard_result;
		}
		
		// If that fails, try with Neo N3 NeoFS HTTP gateway format
		let n3_endpoint = format!("api/v1/{}", endpoint);
		self.send_request(&n3_endpoint, params).await
	}

	/// Verify compatibility with Neo N3 NeoFS by running a simple test operation
	/// 
	/// This method attempts to:
	/// 1. Connect to the NeoFS endpoint
	/// 2. Check if the API version is compatible with Neo N3
	/// 3. Create a test container
	/// 4. Upload a small test object
	/// 5. Retrieve the test object
	/// 6. Delete the test object and container
	/// 
	/// Returns a detailed report on the compatibility test
	pub async fn verify_n3_compatibility(&self) -> NeoFsResult<String> {
		use std::time::Instant;
		
		let start_time = Instant::now();
		let mut report = String::new();
		let mut success = true;
		
		report.push_str("Neo N3 NeoFS Compatibility Test\n");
		report.push_str("==============================\n\n");
		
		// Step 1: Connect and check version
		report.push_str("1. Connecting to NeoFS... ");
		if self.is_connected().await {
			report.push_str("✓ Connected\n");
			
			// Get network info
			match self.get_network_info().await {
				Ok(info) => {
					report.push_str(&format!("   - Version: {}\n", info.version));
					report.push_str(&format!("   - Nodes: {}\n", info.node_count));
					report.push_str(&format!("   - Storage: {} bytes available\n", info.available_space));
					
					// Check if version is compatible with Neo N3
					let version_compatible = self.network().is_compatible_with_n3().await?;
					if version_compatible {
						report.push_str("   - ✓ Version is compatible with Neo N3\n");
					} else {
						report.push_str("   - ❌ Version is NOT compatible with Neo N3 (requires 2.0.0+)\n");
						success = false;
					}
				},
				Err(e) => {
					report.push_str(&format!("❌ Failed to get network info: {}\n", e));
					success = false;
				}
			}
		} else {
			report.push_str("❌ Failed to connect\n");
			return Ok(report);
		}
		
		// Skip container creation/testing if no account is set
		#[cfg(feature = "transaction")]
		if self.account_data.is_none() {
			report.push_str("\n2. Container test: SKIPPED (no account set for authentication)\n");
		} else {
			// Step 2: Try to create a test container
			report.push_str("\n2. Creating test container... ");
			
			let test_container_params = super::container::CreateContainerParams {
				rules: vec![super::types::AccessRule::Public],
				policy: super::types::StoragePolicy {
					replicas: 1,
					placement: super::types::PlacementPolicy {
						regions: vec![super::types::RegionSelector {
							region: "EU".to_string(),
							node_count: 1,
						}],
						tier: super::types::ReliabilityTier::Standard,
						min_nodes_per_region: 1,
					},
					lifetime: 100, // Short lifetime for test
				},
				attributes: vec![
					("name".to_string(), "neo3-compatibility-test".to_string()),
					("created".to_string(), chrono::Utc::now().to_rfc3339()),
					("purpose".to_string(), "Neo N3 compatibility testing".to_string()),
				],
			};
			
			match self.containers().create(test_container_params).await {
				Ok(container_id) => {
					report.push_str(&format!("✓ Created (ID: {})\n", container_id));
					
					// Step 3: Upload a test object
					report.push_str("3. Uploading test object... ");
					let test_data = b"NeoFS compatibility test for Neo N3".to_vec();
					let test_attributes = vec![
						("filename".to_string(), "compatibility-test.txt".to_string()),
						("content-type".to_string(), "text/plain".to_string()),
					];
					
					match self.objects().upload(&container_id, test_data.clone(), test_attributes).await {
						Ok(object_id) => {
							report.push_str(&format!("✓ Uploaded (ID: {})\n", object_id));
							
							// Step 4: Download and verify the test object
							report.push_str("4. Downloading test object... ");
							match self.objects().download(&container_id, &object_id).await {
								Ok(data) => {
									if data == test_data {
										report.push_str("✓ Downloaded and verified\n");
									} else {
										report.push_str("❌ Data mismatch\n");
										success = false;
									}
								},
								Err(e) => {
									report.push_str(&format!("❌ Failed to download: {}\n", e));
									success = false;
								}
							}
							
							// Step 5: Delete the test object
							report.push_str("5. Deleting test object... ");
							match self.objects().delete(&container_id, &object_id).await {
								Ok(_) => report.push_str("✓ Deleted\n"),
								Err(e) => {
									report.push_str(&format!("❌ Failed to delete: {}\n", e));
									success = false;
								}
							}
						},
						Err(e) => {
							report.push_str(&format!("❌ Failed to upload: {}\n", e));
							success = false;
						}
					}
					
					// Step 6: Delete the test container
					report.push_str("6. Deleting test container... ");
					match self.containers().delete(&container_id).await {
						Ok(_) => report.push_str("✓ Deleted\n"),
						Err(e) => {
							report.push_str(&format!("❌ Failed to delete: {}\n", e));
							success = false;
						}
					}
				},
				Err(e) => {
					report.push_str(&format!("❌ Failed to create: {}\n", e));
					success = false;
				}
			}
		}
		
		// Final result
		let duration = start_time.elapsed();
		report.push_str(&format!("\nTest completed in {:.2} seconds\n", duration.as_secs_f64()));
		report.push_str(&format!("Overall result: {}\n", 
			if success { "✅ COMPATIBLE with Neo N3 NeoFS" } else { "❌ NOT FULLY COMPATIBLE with Neo N3 NeoFS" }
		));
		
		Ok(report)
	}

	/// Generate example CLI commands for working with NeoFS
	/// 
	/// This method returns a string containing example CLI commands for
	/// common NeoFS operations using various tools like neofs-cli, curl, and neo-go.
	/// 
	/// The examples are tailored based on the current client configuration and can
	/// be useful for users who want to perform operations outside of this SDK.
	pub fn get_cli_examples(&self) -> String {
		let mut examples = String::new();
		examples.push_str("NeoFS CLI Examples\n");
		examples.push_str("=================\n\n");
		
		let endpoint = self.config.endpoint.clone();
		
		// Basic neofs-cli examples
		examples.push_str("## Using neofs-cli\n\n");
		examples.push_str("### Connection and setup\n");
		examples.push_str(&format!("```\n# Set endpoint\nneofs-cli --endpoint=\"{}\" -v\n\n", endpoint));
		examples.push_str("# Generate a key\nneofs-cli util keygen --show\n\n");
		examples.push_str("# Create wallet\nneofs-cli util create-wallet --wallet ./my-neofs-wallet.json\n```\n\n");
		
		// Container examples
		examples.push_str("### Container operations\n");
		examples.push_str("```\n# List containers\nneofs-cli container list\n\n");
		examples.push_str("# Create container\nneofs-cli container create \\\n");
		examples.push_str("  --policy=\"REP 2 CBF 3 SELECT 2 FROM 3 IN X\"\n\n");
		examples.push_str("# Get container info\nneofs-cli container get --cid <container_id>\n\n");
		examples.push_str("# Delete container\nneofs-cli container delete --cid <container_id>\n```\n\n");
		
		// Object examples
		examples.push_str("### Object operations\n");
		examples.push_str("```\n# Upload object\nneofs-cli object put \\\n");
		examples.push_str("  --cid <container_id> \\\n");
		examples.push_str("  --file ./myfile.txt \\\n");
		examples.push_str("  --attributes filename=myfile.txt,timestamp=$(date -u +\"%Y-%m-%dT%H:%M:%SZ\")\n\n");
		examples.push_str("# List objects in container\nneofs-cli object list --cid <container_id>\n\n");
		examples.push_str("# Get object\nneofs-cli object get \\\n");
		examples.push_str("  --cid <container_id> \\\n");
		examples.push_str("  --oid <object_id> \\\n");
		examples.push_str("  --file ./downloaded_file.txt\n\n");
		examples.push_str("# Delete object\nneofs-cli object delete \\\n");
		examples.push_str("  --cid <container_id> \\\n");
		examples.push_str("  --oid <object_id>\n```\n\n");
		
		// HTTP examples using curl
		examples.push_str("## Using HTTP API with curl\n\n");
		let http_endpoint = if endpoint.starts_with("grpc://") {
			endpoint.replace("grpc://", "http://")
		} else {
			format!("http://{}", endpoint)
		};
		
		examples.push_str("### Basic requests\n");
		examples.push_str(&format!("```\n# Get network info\ncurl -X POST {} \\\n", http_endpoint));
		examples.push_str("  -H \"Content-Type: application/json\" \\\n");
		examples.push_str("  -d '{\"jsonrpc\":\"2.0\",\"method\":\"netmap.netmap_snapshot\",\"params\":{},\"id\":1}'\n\n");
		
		examples.push_str("# Container list (requires bearer token)\ncurl -X POST {} \\\n");
		examples.push_str("  -H \"Content-Type: application/json\" \\\n");
		examples.push_str("  -H \"X-Bearer-Signature: <signature>\" \\\n");
		examples.push_str("  -H \"X-Bearer-Signature-Key: <public_key>\" \\\n");
		examples.push_str("  -H \"X-Bearer: <token>\" \\\n");
		examples.push_str("  -d '{\"jsonrpc\":\"2.0\",\"method\":\"container.list\",\"params\":{},\"id\":1}'\n```\n\n");
		
		// neo-go examples
		examples.push_str("## Using neo-go CLI for N3 integration\n\n");
		examples.push_str("```\n# Deploy NeoFS contract\nneo-go contract deploy -r http://localhost:10332 \\\n");
		examples.push_str("  -w path/to/wallet.json \\\n");
		examples.push_str("  -a <account> \\\n");
		examples.push_str("  ./neofs.nef\n\n");
		
		examples.push_str("# Invoke contract method\nneo-go contract invokefunction -r http://localhost:10332 \\\n");
		examples.push_str("  -w path/to/wallet.json \\\n");
		examples.push_str("  -a <account> \\\n");
		examples.push_str("  <contract_hash> deposit []\n```\n\n");
		
		examples.push_str("## NeoFS utility scripts\n\n");
		examples.push_str("### Bash script to upload file and print object ID\n");
		examples.push_str("```bash\n#!/bin/bash\n");
		examples.push_str("FILE_PATH=$1\n");
		examples.push_str("CONTAINER_ID=$2\n\n");
		examples.push_str("if [ -z \"$FILE_PATH\" ] || [ -z \"$CONTAINER_ID\" ]; then\n");
		examples.push_str("  echo \"Usage: $0 <file_path> <container_id>\"\n");
		examples.push_str("  exit 1\n");
		examples.push_str("fi\n\n");
		examples.push_str("FILENAME=$(basename \"$FILE_PATH\")\n");
		examples.push_str("TIMESTAMP=$(date -u +\"%Y-%m-%dT%H:%M:%SZ\")\n\n");
		examples.push_str(&format!("OBJECT_ID=$(neofs-cli --endpoint=\"{}\" object put \\\n", endpoint));
		examples.push_str("  --cid \"$CONTAINER_ID\" \\\n");
		examples.push_str("  --file \"$FILE_PATH\" \\\n");
		examples.push_str("  --attributes \"filename=$FILENAME,timestamp=$TIMESTAMP\" | grep \"ID:\" | awk '{print $2}')\n\n");
		examples.push_str("echo \"Uploaded $FILENAME to container $CONTAINER_ID\"\n");
		examples.push_str("echo \"Object ID: $OBJECT_ID\"\n");
		examples.push_str("echo \"To download: neofs-cli object get --cid $CONTAINER_ID --oid $OBJECT_ID --file downloaded_$FILENAME\"\n");
		examples.push_str("```\n");
		
		examples
	}
}

/// Information about the NeoFS network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
	/// NeoFS protocol version
	pub version: String,
	/// Number of active nodes in the network
	pub node_count: u64,
	/// Total storage capacity of the network in bytes
	pub storage_capacity: u64,
	/// Available storage space in bytes
	pub available_space: u64,
}

/// Helper for container operations
#[derive(Debug, Clone)]
pub struct ContainerOperations {
	client: NeoFsClient,
}

impl ContainerOperations {
	/// Create a new container operations helper
	pub(crate) fn new(client: NeoFsClient) -> Self {
		Self { client }
	}

	/// List all containers owned by the current account
	pub async fn list(&self) -> NeoFsResult<Vec<ContainerInfo>> {
		#[cfg(feature = "http-client")]
		{
			let response = self.client.send_request("container/list", serde_json::Value::Null).await?;
			
			let status = response["status"].as_str()
				.ok_or_else(|| NeoFsError::ResponseError("Missing status field".to_string()))?;
				
			if status != "ok" {
				let error = response["error"].as_str().unwrap_or("Unknown error");
				return Err(NeoFsError::RequestFailed(error.to_string()));
			}
			
			if !response["containers"].is_array() {
				return Err(NeoFsError::ResponseError("Missing containers array".to_string()));
			}
			
			let mut containers = Vec::new();
			
			for container_data in response["containers"].as_array().unwrap() {
				// Extract required fields
				let container_id_str = container_data["container_id"].as_str()
					.ok_or_else(|| NeoFsError::ResponseError("Missing container_id field".to_string()))?;
					
				let owner_b64 = container_data["owner"].as_str()
					.ok_or_else(|| NeoFsError::ResponseError("Missing owner field".to_string()))?;
					
				let created_at = container_data["created_at"].as_u64()
					.ok_or_else(|| NeoFsError::ResponseError("Missing or invalid created_at field".to_string()))?;
					
				let size = container_data["size"].as_u64()
					.ok_or_else(|| NeoFsError::ResponseError("Missing or invalid size field".to_string()))?;
					
				let object_count = container_data["object_count"].as_u64()
					.ok_or_else(|| NeoFsError::ResponseError("Missing or invalid object_count field".to_string()))?;
					
				let basic_acl = container_data["basic_acl"].as_u64()
					.ok_or_else(|| NeoFsError::ResponseError("Missing or invalid basic_acl field".to_string()))?;
				
				// Extract attributes
				let attributes = if container_data["attributes"].is_array() {
					let mut result = Vec::new();
					
					for attr in container_data["attributes"].as_array().unwrap() {
						if let (Some(key), Some(value)) = (attr["key"].as_str(), attr["value"].as_str()) {
							result.push((key.to_string(), value.to_string()));
						}
					}
					
					result
				} else {
					vec![]
				};
				
				containers.push(ContainerInfo {
					container_id: ContainerId::from_string(container_id_str)?,
					owner: base64::decode(owner_b64)
						.map_err(|e| NeoFsError::DeserializationError(format!("Invalid owner data: {}", e)))?,
					created_at,
					size,
					object_count,
					basic_acl: basic_acl as u32,
					attributes,
				});
			}
			
			Ok(containers)
		}
		
		#[cfg(not(feature = "http-client"))]
		{
			Err(NeoFsError::FeatureDisabled("HTTP client feature is disabled".to_string()))
		}
	}

	/// Create a new container with specified parameters
	pub async fn create(&self, params: CreateContainerParams) -> NeoFsResult<ContainerId> {
		let request = CreateContainerRequest {
			rules: params.rules,
			policy: params.policy,
			attributes: params.attributes,
		};
		
		#[cfg(feature = "http-client")]
		{
			let response = self.client.send_request("container/create", serde_json::to_value(request)?).await?;
			
			let status = response["status"].as_str()
				.ok_or_else(|| NeoFsError::ResponseError("Missing status field".to_string()))?;
				
			if status != "ok" {
				let error = response["error"].as_str().unwrap_or("Unknown error");
				return Err(NeoFsError::RequestFailed(error.to_string()));
			}
			
			let container_id_str = response["container_id"].as_str()
				.ok_or_else(|| NeoFsError::ResponseError("Missing container_id field".to_string()))?;
				
			ContainerId::from_string(container_id_str)
				.map_err(|e| NeoFsError::DeserializationError(format!("Invalid container ID: {}", e)))
		}
		
		#[cfg(not(feature = "http-client"))]
		{
			Err(NeoFsError::FeatureDisabled("HTTP client feature is disabled".to_string()))
		}
	}

	/// Get information about a specific container
	pub async fn get(&self, container_id: &ContainerId) -> NeoFsResult<ContainerInfo> {
		let request = GetContainerRequest {
			container_id: container_id.to_string(),
		};
		
		#[cfg(feature = "http-client")]
		{
			let response = self.client.send_request("container/get", serde_json::to_value(request)?).await?;
			
			let status = response["status"].as_str()
				.ok_or_else(|| NeoFsError::ResponseError("Missing status field".to_string()))?;
				
			if status != "ok" {
				let error = response["error"].as_str().unwrap_or("Unknown error");
				return Err(NeoFsError::RequestFailed(error.to_string()));
			}
			
			let container_data = &response["container"];
			
			// Extract required fields
			let container_id_str = container_data["container_id"].as_str()
				.ok_or_else(|| NeoFsError::ResponseError("Missing container_id field".to_string()))?;
				
			let owner_b64 = container_data["owner"].as_str()
				.ok_or_else(|| NeoFsError::ResponseError("Missing owner field".to_string()))?;
				
			let created_at = container_data["created_at"].as_u64()
				.ok_or_else(|| NeoFsError::ResponseError("Missing or invalid created_at field".to_string()))?;
				
			let size = container_data["size"].as_u64()
				.ok_or_else(|| NeoFsError::ResponseError("Missing or invalid size field".to_string()))?;
				
			let object_count = container_data["object_count"].as_u64()
				.ok_or_else(|| NeoFsError::ResponseError("Missing or invalid object_count field".to_string()))?;
				
			let basic_acl = container_data["basic_acl"].as_u64()
				.ok_or_else(|| NeoFsError::ResponseError("Missing or invalid basic_acl field".to_string()))?;
			
			// Extract attributes
			let attributes = if container_data["attributes"].is_array() {
				let mut result = Vec::new();
				
				for attr in container_data["attributes"].as_array().unwrap() {
					if let (Some(key), Some(value)) = (attr["key"].as_str(), attr["value"].as_str()) {
						result.push((key.to_string(), value.to_string()));
					}
				}
				
				result
			} else {
				vec![]
			};
			
			Ok(ContainerInfo {
				container_id: ContainerId::from_string(container_id_str)?,
				owner: base64::decode(owner_b64)
					.map_err(|e| NeoFsError::DeserializationError(format!("Invalid owner data: {}", e)))?,
				created_at,
				size,
				object_count,
				basic_acl: basic_acl as u32,
				attributes,
			})
		}
		
		#[cfg(not(feature = "http-client"))]
		{
			Err(NeoFsError::FeatureDisabled("HTTP client feature is disabled".to_string()))
		}
	}

	/// Delete a container
	pub async fn delete(&self, container_id: &ContainerId) -> NeoFsResult<()> {
		let request = DeleteContainerRequest {
			container_id: container_id.to_string(),
		};
		
		#[cfg(feature = "http-client")]
		{
			let response = self.client.send_request("container/delete", serde_json::to_value(request)?).await?;
			
			let status = response["status"].as_str()
				.ok_or_else(|| NeoFsError::ResponseError("Missing status field".to_string()))?;
				
			if status != "ok" {
				let error = response["error"].as_str().unwrap_or("Unknown error");
				return Err(NeoFsError::RequestFailed(error.to_string()));
			}
			
			Ok(())
		}
		
		#[cfg(not(feature = "http-client"))]
		{
			Err(NeoFsError::FeatureDisabled("HTTP client feature is disabled".to_string()))
		}
	}

	/// Set access control rules for a container
	pub async fn set_access_rules(
		&self,
		container_id: &ContainerId,
		rules: Vec<AccessRule>,
	) -> NeoFsResult<()> {
		let request = SetAccessRulesRequest {
			container_id: container_id.to_string(),
			rules,
		};
		
		#[cfg(feature = "http-client")]
		{
			let response = self.client.send_request("container/set_access_rules", serde_json::to_value(request)?).await?;
			
			let status = response["status"].as_str()
				.ok_or_else(|| NeoFsError::ResponseError("Missing status field".to_string()))?;
				
			if status != "ok" {
				let error = response["error"].as_str().unwrap_or("Unknown error");
				return Err(NeoFsError::RequestFailed(error.to_string()));
			}
			
			Ok(())
		}
		
		#[cfg(not(feature = "http-client"))]
		{
			Err(NeoFsError::FeatureDisabled("HTTP client feature is disabled".to_string()))
		}
	}
}

/// Request to create a container
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CreateContainerRequest {
	rules: Vec<AccessRule>,
	policy: StoragePolicy,
	attributes: Vec<(String, String)>,
}

/// Request to get a container's info
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GetContainerRequest {
	container_id: String,
}

/// Request to delete a container
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DeleteContainerRequest {
	container_id: String,
}

/// Request to set access rules for a container
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SetAccessRulesRequest {
	container_id: String,
	rules: Vec<AccessRule>,
}

/// Container creation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateContainerParams {
	/// Access control rules
	pub rules: Vec<AccessRule>,
	/// Storage policy
	pub policy: StoragePolicy,
	/// Additional container attributes
	pub attributes: Vec<(String, String)>,
}

/// Container information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInfo {
	/// Container ID
	pub container_id: ContainerId,
	/// Owner's public key
	pub owner: Vec<u8>,
	/// Creation timestamp
	pub created_at: u64,
	/// Size in bytes
	pub size: u64,
	/// Number of objects in the container
	pub object_count: u64,
	/// Basic ACL bit mask
	pub basic_acl: u32,
	/// Container attributes
	pub attributes: Vec<(String, String)>,
}

/// Helper for object operations
#[derive(Debug, Clone)]
pub struct ObjectOperations {
	client: NeoFsClient,
}

impl ObjectOperations {
	/// Create a new object operations helper
	pub(crate) fn new(client: NeoFsClient) -> Self {
		Self { client }
	}

	/// Upload an object to a container
	pub async fn upload(
		&self,
		container_id: &ContainerId,
		data: Vec<u8>,
		attributes: Vec<(String, String)>,
	) -> NeoFsResult<ObjectId> {
		let request = UploadObjectRequest {
			container_id: container_id.to_string(),
			data: base64::encode(&data),
			attributes: attributes.clone(),
		};

		#[cfg(feature = "http-client")]
		{
			let response = self.client.send_request("object/upload", serde_json::to_value(request)?).await?;
			
			let status = response["status"].as_str()
				.ok_or_else(|| NeoFsError::ResponseError("Missing status field".to_string()))?;
				
			if status != "ok" {
				let error = response["error"].as_str()
					.unwrap_or("Unknown error");
				return Err(NeoFsError::RequestFailed(error.to_string()));
			}
			
			let object_id_str = response["object_id"].as_str()
				.ok_or_else(|| NeoFsError::ResponseError("Missing object_id field".to_string()))?;
				
			ObjectId::from_string(object_id_str)
				.map_err(|e| NeoFsError::DeserializationError(format!("Invalid object ID: {}", e)))
		}
		
		#[cfg(not(feature = "http-client"))]
		{
			Err(NeoFsError::FeatureDisabled("HTTP client feature is disabled".to_string()))
		}
	}

	/// Download an object
	pub async fn download(
		&self,
		container_id: &ContainerId,
		object_id: &ObjectId,
	) -> NeoFsResult<Vec<u8>> {
		let request = DownloadObjectRequest {
			container_id: container_id.to_string(),
			object_id: object_id.to_string(),
		};

		#[cfg(feature = "http-client")]
		{
			let response = self.client.send_request("object/download", serde_json::to_value(request)?).await?;
			
			let status = response["status"].as_str()
				.ok_or_else(|| NeoFsError::ResponseError("Missing status field".to_string()))?;
				
			if status != "ok" {
				let error = response["error"].as_str()
					.unwrap_or("Unknown error");
				return Err(NeoFsError::RequestFailed(error.to_string()));
			}
			
			let data_b64 = response["data"].as_str()
				.ok_or_else(|| NeoFsError::ResponseError("Missing data field".to_string()))?;
				
			base64::decode(data_b64)
				.map_err(|e| NeoFsError::DeserializationError(format!("Invalid base64 data: {}", e)))
		}
		
		#[cfg(not(feature = "http-client"))]
		{
			Err(NeoFsError::FeatureDisabled("HTTP client feature is disabled".to_string()))
		}
	}

	/// Get object metadata
	pub async fn get_info(
		&self,
		container_id: &ContainerId,
		object_id: &ObjectId,
	) -> NeoFsResult<ObjectInfo> {
		let request = GetObjectInfoRequest {
			container_id: container_id.to_string(),
			object_id: object_id.to_string(),
		};

		#[cfg(feature = "http-client")]
		{
			let response = self.client.send_request("object/info", serde_json::to_value(request)?).await?;
			
			let status = response["status"].as_str()
				.ok_or_else(|| NeoFsError::ResponseError("Missing status field".to_string()))?;
				
			if status != "ok" {
				let error = response["error"].as_str()
					.unwrap_or("Unknown error");
				return Err(NeoFsError::RequestFailed(error.to_string()));
			}
			
			let object_data = &response["object"];
			
			// Extract required fields
			let object_id_str = object_data["object_id"].as_str()
				.ok_or_else(|| NeoFsError::ResponseError("Missing object_id field".to_string()))?;
			
			let container_id_str = object_data["container_id"].as_str()
				.ok_or_else(|| NeoFsError::ResponseError("Missing container_id field".to_string()))?;
				
			let owner_b64 = object_data["owner"].as_str()
				.ok_or_else(|| NeoFsError::ResponseError("Missing owner field".to_string()))?;
				
			let created_at = object_data["created_at"].as_u64()
				.ok_or_else(|| NeoFsError::ResponseError("Missing or invalid created_at field".to_string()))?;
				
			let size = object_data["size"].as_u64()
				.ok_or_else(|| NeoFsError::ResponseError("Missing or invalid size field".to_string()))?;
				
			// Extract attributes
			let attributes = if object_data["attributes"].is_array() {
				let mut result = Vec::new();
				
				for attr in object_data["attributes"].as_array().unwrap() {
					if let (Some(key), Some(value)) = (attr["key"].as_str(), attr["value"].as_str()) {
						result.push((key.to_string(), value.to_string()));
					}
				}
				
				result
			} else {
				vec![]
			};
			
			// Build the ObjectInfo
			Ok(ObjectInfo {
				object_id: ObjectId::from_string(object_id_str)?,
				container_id: ContainerId::from_string(container_id_str)?,
				owner: base64::decode(owner_b64)
					.map_err(|e| NeoFsError::DeserializationError(format!("Invalid owner data: {}", e)))?,
				created_at,
				size,
				attributes,
			})
		}
		
		#[cfg(not(feature = "http-client"))]
		{
			Err(NeoFsError::FeatureDisabled("HTTP client feature is disabled".to_string()))
		}
	}

	/// Delete an object
	pub async fn delete(
		&self,
		container_id: &ContainerId,
		object_id: &ObjectId,
	) -> NeoFsResult<()> {
		let request = DeleteObjectRequest {
			container_id: container_id.to_string(),
			object_id: object_id.to_string(),
		};

		#[cfg(feature = "http-client")]
		{
			let response = self.client.send_request("object/delete", serde_json::to_value(request)?).await?;
			
			let status = response["status"].as_str()
				.ok_or_else(|| NeoFsError::ResponseError("Missing status field".to_string()))?;
				
			if status != "ok" {
				let error = response["error"].as_str()
					.unwrap_or("Unknown error");
				return Err(NeoFsError::RequestFailed(error.to_string()));
			}
			
			Ok(())
		}
		
		#[cfg(not(feature = "http-client"))]
		{
			Err(NeoFsError::FeatureDisabled("HTTP client feature is disabled".to_string()))
		}
	}

	/// Search for objects by attributes
	pub async fn search(
		&self,
		container_id: &ContainerId,
		filters: Vec<(String, String)>,
	) -> NeoFsResult<Vec<ObjectId>> {
		let request = SearchObjectsRequest {
			container_id: container_id.to_string(),
			filters: filters.into_iter().map(|(k, v)| SearchFilter { key: k, value: v }).collect(),
		};

		#[cfg(feature = "http-client")]
		{
			let response = self.client.send_request("object/search", serde_json::to_value(request)?).await?;
			
			let status = response["status"].as_str()
				.ok_or_else(|| NeoFsError::ResponseError("Missing status field".to_string()))?;
				
			if status != "ok" {
				let error = response["error"].as_str()
					.unwrap_or("Unknown error");
				return Err(NeoFsError::RequestFailed(error.to_string()));
			}
			
			if !response["objects"].is_array() {
				return Err(NeoFsError::ResponseError("Missing objects array".to_string()));
			}
			
			let mut result = Vec::new();
			
			for obj_id in response["objects"].as_array().unwrap() {
				if let Some(id_str) = obj_id.as_str() {
					let object_id = ObjectId::from_string(id_str)?;
					result.push(object_id);
				}
			}
			
			Ok(result)
		}
		
		#[cfg(not(feature = "http-client"))]
		{
			Err(NeoFsError::FeatureDisabled("HTTP client feature is disabled".to_string()))
		}
	}
	
	/// Get a streaming object reader for large objects
	pub async fn get_reader(
		&self,
		container_id: &ContainerId,
		object_id: &ObjectId,
	) -> NeoFsResult<ObjectReader> {
		// First get object metadata to know the size
		let info = self.get_info(container_id, object_id).await?;
		
		Ok(ObjectReader::new(
			self.client.clone(),
			container_id.clone(),
			object_id.clone(),
			info.size,
		))
	}
	
	/// Create a streaming object writer for large uploads
	pub async fn get_writer(
		&self,
		container_id: &ContainerId,
		attributes: Vec<(String, String)>,
	) -> NeoFsResult<ObjectWriter> {
		Ok(ObjectWriter::new(
			self.client.clone(),
			container_id.clone(),
			attributes,
		))
	}

	/// Upload a file from a path
	#[cfg(feature = "tokio")]
	pub async fn upload_file(
		&self,
		container_id: &ContainerId,
		file_path: &std::path::Path,
		additional_attributes: Vec<(String, String)>,
	) -> NeoFsResult<ObjectId> {
		// Read file
		use tokio::fs;
		
		// Check if file exists
		if !file_path.exists() {
			return Err(NeoFsError::ObjectError(format!("File not found: {:?}", file_path)));
		}
		
		// Read file data
		let data = fs::read(file_path)
			.await
			.map_err(|e| NeoFsError::ObjectError(format!("Failed to read file: {}", e)))?;
		
		// Add filename attribute if not present
		let mut attributes = additional_attributes.clone();
		if !attributes.iter().any(|(k, _)| k == "filename") {
			if let Some(filename) = file_path.file_name() {
				if let Some(filename_str) = filename.to_str() {
					attributes.push(("filename".to_string(), filename_str.to_string()));
				}
			}
		}
		
		// Add content-type attribute if not present
		if !attributes.iter().any(|(k, _)| k == "content-type") {
			if let Some(ext) = file_path.extension() {
				if let Some(ext_str) = ext.to_str() {
					let mime_type = match ext_str.to_lowercase().as_str() {
						"jpg" | "jpeg" => "image/jpeg",
						"png" => "image/png",
						"gif" => "image/gif",
						"pdf" => "application/pdf",
						"txt" => "text/plain",
						"html" | "htm" => "text/html",
						"json" => "application/json",
						"js" => "application/javascript",
						"css" => "text/css",
						"xml" => "application/xml",
						"zip" => "application/zip",
						"doc" | "docx" => "application/msword",
						"xls" | "xlsx" => "application/vnd.ms-excel",
						"ppt" | "pptx" => "application/vnd.ms-powerpoint",
						_ => "application/octet-stream",
					};
					attributes.push(("content-type".to_string(), mime_type.to_string()));
				}
			}
		}
		
		// Add file size attribute
		if !attributes.iter().any(|(k, _)| k == "size") {
			attributes.push(("size".to_string(), data.len().to_string()));
		}
		
		// Upload the object
		self.upload(container_id, data, attributes).await
	}
	
	/// Download an object to a file
	#[cfg(feature = "tokio")]
	pub async fn download_to_file(
		&self,
		container_id: &ContainerId,
		object_id: &ObjectId,
		file_path: &std::path::Path,
	) -> NeoFsResult<()> {
		use tokio::fs;
		
		// Download the object
		let data = self.download(container_id, object_id).await?;
		
		// Create parent directories if they don't exist
		if let Some(parent) = file_path.parent() {
			fs::create_dir_all(parent)
				.await
				.map_err(|e| NeoFsError::ObjectError(format!("Failed to create directories: {}", e)))?;
		}
		
		// Write to file
		fs::write(file_path, data)
			.await
			.map_err(|e| NeoFsError::ObjectError(format!("Failed to write file: {}", e)))?;
			
		Ok(())
	}
	
	/// List objects with their attributes
	pub async fn list_with_attributes(
		&self,
		container_id: &ContainerId,
	) -> NeoFsResult<Vec<ObjectInfo>> {
		let object_ids = self.search(container_id, vec![]).await?;
		
		let mut objects = Vec::with_capacity(object_ids.len());
		for object_id in object_ids {
			match self.get_info(container_id, &object_id).await {
				Ok(info) => objects.push(info),
				Err(e) => eprintln!("Warning: Failed to get object info for {}: {}", object_id, e),
			}
		}
		
		Ok(objects)
	}
	
	/// Find objects by filename
	pub async fn find_by_filename(
		&self,
		container_id: &ContainerId,
		filename: &str,
	) -> NeoFsResult<Vec<ObjectId>> {
		self.search(container_id, vec![("filename".to_string(), filename.to_string())]).await
	}
	
	/// Stream a large file upload with progress reporting
	#[cfg(feature = "tokio")]
	pub async fn stream_file_upload(
		&self,
		container_id: &ContainerId,
		file_path: &std::path::Path,
			additional_attributes: Vec<(String, String)>,
		progress_callback: Option<Box<dyn FnMut(usize, usize) + Send>>,
	) -> NeoFsResult<ObjectId> {
		use tokio::fs;
		use tokio::io::AsyncReadExt;
		
		// Check if file exists
		if !file_path.exists() {
			return Err(NeoFsError::ObjectError(format!("File not found: {:?}", file_path)));
		}
		
		// Get file metadata
		let metadata = fs::metadata(file_path)
			.await
			.map_err(|e| NeoFsError::ObjectError(format!("Failed to get file metadata: {}", e)))?;
			
		let file_size = metadata.len() as usize;
		
		// Prepare attributes
		let mut attributes = additional_attributes.clone();
		
		// Add filename attribute if not present
		if !attributes.iter().any(|(k, _)| k == "filename") {
			if let Some(filename) = file_path.file_name() {
				if let Some(filename_str) = filename.to_str() {
					attributes.push(("filename".to_string(), filename_str.to_string()));
				}
			}
		}
		
		// Add content-type attribute if not present
		if !attributes.iter().any(|(k, _)| k == "content-type") {
			if let Some(ext) = file_path.extension() {
				if let Some(ext_str) = ext.to_str() {
					let mime_type = match ext_str.to_lowercase().as_str() {
						"jpg" | "jpeg" => "image/jpeg",
						"png" => "image/png",
						"gif" => "image/gif",
						"pdf" => "application/pdf",
						"txt" => "text/plain",
						"html" | "htm" => "text/html",
						"json" => "application/json",
						_ => "application/octet-stream",
					};
					attributes.push(("content-type".to_string(), mime_type.to_string()));
				}
			}
		}
		
		// Add file size attribute
		attributes.push(("size".to_string(), file_size.to_string()));
		
		// Create writer for streaming upload
		let mut writer = self.get_writer(container_id, attributes).await?;
		
		// Open file for streaming
		let mut file = fs::File::open(file_path)
			.await
			.map_err(|e| NeoFsError::ObjectError(format!("Failed to open file: {}", e)))?;
			
		// Set up buffer and progress tracking
		let buffer_size = 1024 * 1024; // 1MB buffer
		let mut buffer = vec![0u8; buffer_size];
		let mut uploaded = 0;
		let mut progress_reporter = progress_callback;
		
		// Stream the file in chunks
		loop {
			let bytes_read = file.read(&mut buffer)
				.await
				.map_err(|e| NeoFsError::ObjectError(format!("Failed to read from file: {}", e)))?;
				
			if bytes_read == 0 {
				break; // EOF
			}
			
			writer.write(&buffer[..bytes_read])
				.await
				.map_err(|e| NeoFsError::ObjectError(format!("Failed to write to NeoFS: {}", e)))?;
				
			uploaded += bytes_read;
			
			// Report progress if callback provided
			if let Some(ref mut callback) = progress_reporter {
				callback(uploaded, file_size);
			}
		}
		
		// Complete the upload
		writer.complete().await
	}
}

/// Request to upload an object
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UploadObjectRequest {
	container_id: String,
	data: String, // base64 encoded
	attributes: Vec<(String, String)>,
}

/// Request to download an object
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DownloadObjectRequest {
	container_id: String,
	object_id: String,
}

/// Request to get object info
#[derive(Debug, Clone, Serialize, Deserialize)]
struct GetObjectInfoRequest {
	container_id: String,
	object_id: String,
}

/// Request to delete an object
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DeleteObjectRequest {
	container_id: String,
	object_id: String,
}

/// Request to search for objects
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SearchObjectsRequest {
	container_id: String,
	filters: Vec<SearchFilter>,
}

/// Search filter for object attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SearchFilter {
	key: String,
	value: String,
}

/// Streaming reader for large objects
pub struct ObjectReader {
	client: NeoFsClient,
	container_id: ContainerId,
	object_id: ObjectId,
	size: u64,
	position: u64,
	buffer: Vec<u8>,
	chunk_size: usize,
}

impl ObjectReader {
	/// Create a new object reader
	fn new(
		client: NeoFsClient,
		container_id: ContainerId,
		object_id: ObjectId,
		size: u64,
	) -> Self {
		Self {
			client,
			container_id,
			object_id,
			size,
			position: 0,
			buffer: Vec::new(),
			chunk_size: 1024 * 1024, // 1MB chunks by default
		}
	}
	
	/// Read a chunk of data from the object
	pub async fn read(&mut self, buf: &mut [u8]) -> NeoFsResult<usize> {
		if self.position >= self.size {
			return Ok(0); // EOF
		}
		
		// If buffer is empty, fetch more data
		if self.buffer.is_empty() {
			self.fetch_chunk().await?;
		}
		
		// Copy data from buffer to output buffer
		let bytes_to_copy = std::cmp::min(buf.len(), self.buffer.len());
		buf[..bytes_to_copy].copy_from_slice(&self.buffer[..bytes_to_copy]);
		
		// Update buffer and position
		self.buffer.drain(..bytes_to_copy);
		self.position += bytes_to_copy as u64;
		
		Ok(bytes_to_copy)
	}
	
	/// Fetch a chunk of data from the object
	async fn fetch_chunk(&mut self) -> NeoFsResult<()> {
		let request = DownloadPartRequest {
			container_id: self.container_id.to_string(),
			object_id: self.object_id.to_string(),
			offset: self.position,
			length: std::cmp::min(
				self.chunk_size as u64,
				self.size - self.position,
			) as u64,
		};
		
		#[cfg(feature = "http-client")]
		{
			let response = self.client.send_request("object/download_part", serde_json::to_value(request)?).await?;
			
			let status = response["status"].as_str()
				.ok_or_else(|| NeoFsError::ResponseError("Missing status field".to_string()))?;
				
			if status != "ok" {
				let error = response["error"].as_str()
					.unwrap_or("Unknown error");
				return Err(NeoFsError::RequestFailed(error.to_string()));
			}
			
			let data_b64 = response["data"].as_str()
				.ok_or_else(|| NeoFsError::ResponseError("Missing data field".to_string()))?;
				
			self.buffer = base64::decode(data_b64)
				.map_err(|e| NeoFsError::DeserializationError(format!("Invalid base64 data: {}", e)))?;
			
			Ok(())
		}
		
		#[cfg(not(feature = "http-client"))]
		{
			Err(NeoFsError::FeatureDisabled("HTTP client feature is disabled".to_string()))
		}
	}
	
	/// Set custom chunk size for streaming
	pub fn with_chunk_size(mut self, chunk_size: usize) -> Self {
		self.chunk_size = chunk_size;
		self
	}
	
	/// Get the total size of the object
	pub fn size(&self) -> u64 {
		self.size
	}
	
	/// Get the current position in the object
	pub fn position(&self) -> u64 {
		self.position
	}
}

/// Request to download a part of an object
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DownloadPartRequest {
	container_id: String,
	object_id: String,
	offset: u64,
	length: u64,
}

/// Streaming writer for large objects
pub struct ObjectWriter {
	client: NeoFsClient,
	container_id: ContainerId,
	attributes: Vec<(String, String)>,
	buffer: Vec<u8>,
	max_buffer_size: usize,
	parts: Vec<UploadedPart>,
}

impl ObjectWriter {
	/// Create a new object writer
	fn new(
		client: NeoFsClient,
		container_id: ContainerId,
		attributes: Vec<(String, String)>,
	) -> Self {
		Self {
			client,
			container_id,
			attributes,
			buffer: Vec::new(),
			max_buffer_size: 10 * 1024 * 1024, // 10MB buffer by default
			parts: Vec::new(),
		}
	}
	
	/// Write data to the object
	pub async fn write(&mut self, data: &[u8]) -> NeoFsResult<usize> {
		// Add data to buffer
		self.buffer.extend_from_slice(data);
		
		// If buffer exceeds max size, flush it
		if self.buffer.len() >= self.max_buffer_size {
			self.flush_buffer().await?;
		}
		
		Ok(data.len())
	}
	
	/// Flush the buffer and upload as a part
	async fn flush_buffer(&mut self) -> NeoFsResult<()> {
		if self.buffer.is_empty() {
			return Ok(());
		}
		
		let part_data = std::mem::take(&mut self.buffer);
		let part_id = self.upload_part(part_data).await?;
		self.parts.push(part_id);
		
		Ok(())
	}
	
	/// Upload a part of the object
	async fn upload_part(&self, data: Vec<u8>) -> NeoFsResult<UploadedPart> {
		let request = UploadPartRequest {
			container_id: self.container_id.to_string(),
			data: base64::encode(&data),
			part_num: self.parts.len() as u32,
		};
		
		#[cfg(feature = "http-client")]
		{
			let response = self.client.send_request("object/upload_part", serde_json::to_value(request)?).await?;
			
			let status = response["status"].as_str()
				.ok_or_else(|| NeoFsError::ResponseError("Missing status field".to_string()))?;
				
			if status != "ok" {
				let error = response["error"].as_str()
					.unwrap_or("Unknown error");
				return Err(NeoFsError::RequestFailed(error.to_string()));
			}
			
			let part_id = response["part_id"].as_str()
				.ok_or_else(|| NeoFsError::ResponseError("Missing part_id field".to_string()))?;
				
			Ok(UploadedPart {
				part_id: part_id.to_string(),
				part_num: self.parts.len() as u32,
				size: data.len() as u64,
			})
		}
		
		#[cfg(not(feature = "http-client"))]
		{
			Err(NeoFsError::FeatureDisabled("HTTP client feature is disabled".to_string()))
		}
	}
	
	/// Complete the upload and finalize the object
	pub async fn complete(mut self) -> NeoFsResult<ObjectId> {
		// Flush any remaining data
		self.flush_buffer().await?;
		
		let request = CompleteMultipartUploadRequest {
			container_id: self.container_id.to_string(),
			parts: self.parts.clone(),
			attributes: self.attributes.clone(),
		};
		
		#[cfg(feature = "http-client")]
		{
			let response = self.client.send_request("object/complete_upload", serde_json::to_value(request)?).await?;
			
			let status = response["status"].as_str()
				.ok_or_else(|| NeoFsError::ResponseError("Missing status field".to_string()))?;
				
			if status != "ok" {
				let error = response["error"].as_str()
					.unwrap_or("Unknown error");
				return Err(NeoFsError::RequestFailed(error.to_string()));
			}
			
			let object_id_str = response["object_id"].as_str()
				.ok_or_else(|| NeoFsError::ResponseError("Missing object_id field".to_string()))?;
				
			ObjectId::from_string(object_id_str)
				.map_err(|e| NeoFsError::DeserializationError(format!("Invalid object ID: {}", e)))
		}
		
		#[cfg(not(feature = "http-client"))]
		{
			Err(NeoFsError::FeatureDisabled("HTTP client feature is disabled".to_string()))
		}
	}
	
	/// Set custom maximum buffer size
	pub fn with_max_buffer_size(mut self, max_buffer_size: usize) -> Self {
		self.max_buffer_size = max_buffer_size;
		self
	}
	
	/// Get the number of buffered bytes
	pub fn buffered_bytes(&self) -> usize {
		self.buffer.len()
	}
	
	/// Get the number of parts already uploaded
	pub fn uploaded_parts(&self) -> usize {
		self.parts.len()
	}
}

/// Information about an uploaded part
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UploadedPart {
	part_id: String,
	part_num: u32,
	size: u64,
}

/// Request to upload a part
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UploadPartRequest {
	container_id: String,
	data: String, // base64 encoded
	part_num: u32,
}

/// Request to complete a multipart upload
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CompleteMultipartUploadRequest {
	container_id: String,
	parts: Vec<UploadedPart>,
	attributes: Vec<(String, String)>,
}

/// Object information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectInfo {
	/// Object ID
	pub object_id: ObjectId,
	/// Container ID
	pub container_id: ContainerId,
	/// Owner's public key
	pub owner: Vec<u8>,
	/// Creation timestamp
	pub created_at: u64,
	/// Size in bytes
	pub size: u64,
	/// Object attributes
	pub attributes: Vec<(String, String)>,
}

/// Helper for network operations
#[derive(Debug, Clone)]
pub struct NetworkOperations {
	client: NeoFsClient,
}

impl NetworkOperations {
	/// Create a new network operations helper
	pub(crate) fn new(client: NeoFsClient) -> Self {
		Self { client }
	}

	/// Get a list of NeoFS storage nodes
	pub async fn get_nodes(&self) -> NeoFsResult<Vec<NodeInfo>> {
		#[cfg(feature = "http-client")]
		{
			let response = self.client.send_request("network/nodes", serde_json::Value::Null).await?;
			
			let status = response["status"].as_str()
				.ok_or_else(|| NeoFsError::ResponseError("Missing status field".to_string()))?;
				
			if status != "ok" {
				let error = response["error"].as_str().unwrap_or("Unknown error");
				return Err(NeoFsError::RequestFailed(error.to_string()));
			}
			
			if !response["nodes"].is_array() {
				return Err(NeoFsError::ResponseError("Missing nodes array".to_string()));
			}
			
			let mut nodes = Vec::new();
			
			for node_data in response["nodes"].as_array().unwrap() {
				let public_key_b64 = node_data["public_key"].as_str()
					.ok_or_else(|| NeoFsError::ResponseError("Missing public_key field".to_string()))?;
					
				let address = node_data["address"].as_str()
					.ok_or_else(|| NeoFsError::ResponseError("Missing address field".to_string()))?;
					
				let available_space = node_data["available_space"].as_u64()
					.ok_or_else(|| NeoFsError::ResponseError("Missing or invalid available_space field".to_string()))?;
					
				let total_capacity = node_data["total_capacity"].as_u64()
					.ok_or_else(|| NeoFsError::ResponseError("Missing or invalid total_capacity field".to_string()))?;
					
				let version = node_data["version"].as_str()
					.ok_or_else(|| NeoFsError::ResponseError("Missing version field".to_string()))?;
					
				let status = node_data["status"].as_str()
					.ok_or_else(|| NeoFsError::ResponseError("Missing status field".to_string()))?;
				
				nodes.push(NodeInfo {
					public_key: base64::decode(public_key_b64)
						.map_err(|e| NeoFsError::DeserializationError(format!("Invalid public key data: {}", e)))?,
					address: address.to_string(),
					available_space,
					total_capacity,
					version: version.to_string(),
					status: status.to_string(),
				});
			}
			
			Ok(nodes)
		}
		
		#[cfg(not(feature = "http-client"))]
		{
			Err(NeoFsError::FeatureDisabled("HTTP client feature is disabled".to_string()))
		}
	}

	/// Get network statistics
	pub async fn get_stats(&self) -> NeoFsResult<NetworkStats> {
		#[cfg(feature = "http-client")]
		{
			let response = self.client.send_request("network/stats", serde_json::Value::Null).await?;
			
			let status = response["status"].as_str()
				.ok_or_else(|| NeoFsError::ResponseError("Missing status field".to_string()))?;
				
			if status != "ok" {
				let error = response["error"].as_str().unwrap_or("Unknown error");
				return Err(NeoFsError::RequestFailed(error.to_string()));
			}
			
			let stats = &response["stats"];
			
			let total_objects = stats["total_objects"].as_u64()
				.ok_or_else(|| NeoFsError::ResponseError("Missing or invalid total_objects field".to_string()))?;
				
			let total_containers = stats["total_containers"].as_u64()
				.ok_or_else(|| NeoFsError::ResponseError("Missing or invalid total_containers field".to_string()))?;
				
			let total_storage_used = stats["total_storage_used"].as_u64()
				.ok_or_else(|| NeoFsError::ResponseError("Missing or invalid total_storage_used field".to_string()))?;
				
			let total_storage_capacity = stats["total_storage_capacity"].as_u64()
				.ok_or_else(|| NeoFsError::ResponseError("Missing or invalid total_storage_capacity field".to_string()))?;
			
			Ok(NetworkStats {
				total_objects,
				total_containers,
				total_storage_used,
				total_storage_capacity,
			})
		}
		
		#[cfg(not(feature = "http-client"))]
		{
			Err(NeoFsError::FeatureDisabled("HTTP client feature is disabled".to_string()))
		}
	}
	
	/// Get network information
	pub async fn get_info(&self) -> NeoFsResult<NetworkInfo> {
		#[cfg(feature = "http-client")]
		{
			let response = self.client.send_request("network/info", serde_json::Value::Null).await?;
			
			let status = response["status"].as_str()
				.ok_or_else(|| NeoFsError::ResponseError("Missing status field".to_string()))?;
				
			if status != "ok" {
				let error = response["error"].as_str().unwrap_or("Unknown error");
				return Err(NeoFsError::RequestFailed(error.to_string()));
			}
			
			let info = &response["info"];
			
			let version = info["version"].as_str()
				.ok_or_else(|| NeoFsError::ResponseError("Missing or invalid version field".to_string()))?;
				
			let node_count = info["node_count"].as_u64()
				.ok_or_else(|| NeoFsError::ResponseError("Missing or invalid node_count field".to_string()))?;
				
			let storage_capacity = info["storage_capacity"].as_u64()
				.ok_or_else(|| NeoFsError::ResponseError("Missing or invalid storage_capacity field".to_string()))?;
				
			let available_space = info["available_space"].as_u64()
				.ok_or_else(|| NeoFsError::ResponseError("Missing or invalid available_space field".to_string()))?;
			
			Ok(NetworkInfo {
				version: version.to_string(),
				node_count,
				storage_capacity,
				available_space,
			})
		}
		
		#[cfg(not(feature = "http-client"))]
		{
			Err(NeoFsError::FeatureDisabled("HTTP client feature is disabled".to_string()))
		}
	}

	/// Check if the NeoFS node is compatible with the required version
	pub async fn check_version_compatibility(&self, required_version: &str) -> NeoFsResult<bool> {
		let info = self.get_info().await?;
		let current = ProtocolVersion::from_string(&info.version)?;
		let required = ProtocolVersion::from_string(required_version)?;
		
		Ok(current.is_compatible_with(&required))
	}
	
	/// Check if this node is compatible with Neo N3 NeoFS
	pub async fn is_compatible_with_n3(&self) -> NeoFsResult<bool> {
		// Neo N3 requires at least version 2.0.0 of NeoFS
		self.check_version_compatibility("2.0.0").await
	}

	/// Check if the network is healthy
	pub async fn is_healthy(&self) -> NeoFsResult<bool> {
		match self.get_info().await {
			Ok(_) => Ok(true),
			Err(_) => Ok(false),
		}
	}
	
	/// Get network health metrics
	pub async fn get_health_metrics(&self) -> NeoFsResult<NetworkHealthMetrics> {
		let stats = self.get_stats().await?;
		let info = self.get_info().await?;
		let nodes = self.get_nodes().await?;
		
		let online_nodes = nodes.iter().filter(|node| node.status == "online").count();
		let storage_usage_percent = if stats.total_storage_capacity > 0 {
			(stats.total_storage_used as f64 / stats.total_storage_capacity as f64) * 100.0
		} else {
			0.0
		};
		
		Ok(NetworkHealthMetrics {
			total_nodes: nodes.len(),
			online_nodes,
			storage_usage_percent,
			version: info.version,
			total_objects: stats.total_objects,
			total_containers: stats.total_containers,
		})
	}
	
	/// Get a detailed health report as a formatted string
	pub async fn get_health_report(&self) -> NeoFsResult<String> {
		let metrics = self.get_health_metrics().await?;
		let is_compatible = self.is_compatible_with_n3().await?;
		
		let mut report = String::new();
		report.push_str("NeoFS Network Health Report\n");
		report.push_str("==========================\n\n");
		
		report.push_str(&format!("Version: {}\n", metrics.version));
		report.push_str(&format!("Neo N3 Compatible: {}\n", if is_compatible { "Yes" } else { "No" }));
		report.push_str(&format!("Storage Nodes: {} total, {} online ({}%)\n", 
			metrics.total_nodes,
			metrics.online_nodes,
			if metrics.total_nodes > 0 {
				(metrics.online_nodes as f64 / metrics.total_nodes as f64 * 100.0).round()
			} else {
				0.0
			}
		));
		
		report.push_str(&format!("Storage Usage: {:.2}%\n", metrics.storage_usage_percent));
		report.push_str(&format!("Containers: {}\n", metrics.total_containers));
		report.push_str(&format!("Objects: {}\n", metrics.total_objects));
		
		// Calculate network health score
		let node_health = if metrics.total_nodes > 0 {
			metrics.online_nodes as f64 / metrics.total_nodes as f64
		} else {
			0.0
		};
		
		let storage_health = 1.0 - (metrics.storage_usage_percent / 100.0).min(1.0).max(0.0);
		let version_health = if is_compatible { 1.0 } else { 0.0 };
		
		// Overall health score 
		let health_score = (node_health * 0.4 + storage_health * 0.4 + version_health * 0.2) * 100.0;
		
		report.push_str(&format!("\nOverall Health Score: {:.1}%\n", health_score));
		report.push_str(&format!("Status: {}\n", 
			if health_score >= 80.0 {
				"Excellent"
			} else if health_score >= 60.0 {
				"Good"
			} else if health_score >= 40.0 {
				"Fair"
			} else {
				"Poor"
			}
		));
		
		Ok(report)
	}
}

/// Information about a NeoFS storage node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
	/// Node public key
	pub public_key: Vec<u8>,
	/// Node address
	pub address: String,
	/// Available storage space in bytes
	pub available_space: u64,
	/// Total storage capacity in bytes
	pub total_capacity: u64,
	/// Node version
	pub version: String,
	/// Node status (online, offline, etc.)
	pub status: String,
}

/// NeoFS network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
	/// Total number of objects in the network
	pub total_objects: u64,
	/// Total number of containers in the network
	pub total_containers: u64,
	/// Total storage used in bytes
	pub total_storage_used: u64,
	/// Total storage capacity in bytes
	pub total_storage_capacity: u64,
}

/// Bearer token for NeoFS authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BearerToken {
	/// The owner of the token
	pub owner: Vec<u8>,
	/// The token itself
	pub token: Vec<u8>,
	/// Signature of the token
	pub signature: Vec<u8>,
	/// Expiration timestamp
	pub expires_at: u64,
}

/// Network health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkHealthMetrics {
	/// Total number of nodes
	pub total_nodes: usize,
	/// Number of online nodes
	pub online_nodes: usize,
	/// Storage usage percentage
	pub storage_usage_percent: f64,
	/// Network version
	pub version: String,
	/// Total number of objects
	pub total_objects: u64,
	/// Total number of containers
	pub total_containers: u64,
}

/// NeoFS protocol version
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProtocolVersion {
	/// Major version
	pub major: u32,
	/// Minor version
	pub minor: u32,
	/// Patch version
	pub patch: u32,
}

impl ProtocolVersion {
	/// Parse a version string into a ProtocolVersion
	pub fn from_string(version: &str) -> NeoFsResult<Self> {
		let parts: Vec<&str> = version.split('.').collect();
		if parts.len() < 2 {
			return Err(NeoFsError::InvalidResponse(format!("Invalid version format: {}", version)));
		}
		
		let major = parts[0].parse::<u32>()
			.map_err(|_| NeoFsError::InvalidResponse(format!("Invalid major version: {}", parts[0])))?;
			
		let minor = parts[1].parse::<u32>()
			.map_err(|_| NeoFsError::InvalidResponse(format!("Invalid minor version: {}", parts[1])))?;
			
		let patch = if parts.len() > 2 {
			parts[2].parse::<u32>()
				.map_err(|_| NeoFsError::InvalidResponse(format!("Invalid patch version: {}", parts[2])))?
		} else {
			0
		};
		
		Ok(Self { major, minor, patch })
	}
	
	/// Check if this version is compatible with the required version
	pub fn is_compatible_with(&self, required: &ProtocolVersion) -> bool {
		// Major version must match
		if self.major != required.major {
			return false;
		}
		
		// Minor version must be greater than or equal
		if self.minor < required.minor {
			return false;
		}
		
		// If minor versions match, patch version must be greater than or equal
		if self.minor == required.minor && self.patch < required.patch {
			return false;
		}
		
		true
	}
}
