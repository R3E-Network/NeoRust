use crate::neo_types::{Block, Transaction, ApplicationLog, ContractState};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use url::Url;

/// Errors that can occur when using the REST client
#[derive(Error, Debug)]
pub enum RestClientError {
    #[error("HTTP error: {0}")]
    HttpError(String),
    
    #[error("Failed to parse URL: {0}")]
    UrlParseError(#[from] url::ParseError),
    
    #[error("Request error: {0}")]
    RequestError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    
    #[error("API error: {0}")]
    ApiError(String),
}

/// Client for Neo N3 RESTful API
pub struct RestClient {
    /// The base URL for the REST API
    base_url: Url,
    /// HTTP client for making requests
    client: reqwest::Client,
}

impl RestClient {
    /// Create a new REST client
    pub fn new(base_url: &str) -> Result<Self, RestClientError> {
        let base_url = Url::parse(base_url)?;
        let client = reqwest::Client::new();
        
        Ok(Self {
            base_url,
            client,
        })
    }
    
    /// Make a GET request to the REST API
    async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str, query: Option<&[(&str, &str)]>) -> Result<T, RestClientError> {
        let url = self.base_url.join(path)?;
        
        let mut req = self.client.get(url);
        if let Some(query) = query {
            req = req.query(query);
        }
        
        let response = req.send().await.map_err(|e| RestClientError::HttpError(e.to_string()))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_else(|_| "No error message".to_string());
            return Err(RestClientError::ApiError(format!("Status {}: {}", status, text)));
        }
        
        let data = response.json().await.map_err(|e| RestClientError::SerializationError(e.into()))?;
        Ok(data)
    }
    
    /// Make a POST request to the REST API
    async fn post<T: for<'de> Deserialize<'de>, B: Serialize>(&self, path: &str, body: B) -> Result<T, RestClientError> {
        let url = self.base_url.join(path)?;
        
        let response = self.client.post(url)
            .json(&body)
            .send()
            .await
            .map_err(|e| RestClientError::HttpError(e.to_string()))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_else(|_| "No error message".to_string());
            return Err(RestClientError::ApiError(format!("Status {}: {}", status, text)));
        }
        
        let data = response.json().await.map_err(|e| RestClientError::SerializationError(e.into()))?;
        Ok(data)
    }
    
    /// Get the current blockchain height
    pub async fn get_height(&self) -> Result<u32, RestClientError> {
        #[derive(Deserialize)]
        struct HeightResponse {
            height: u32,
        }
        
        let response: HeightResponse = self.get("/height", None).await?;
        Ok(response.height)
    }
    
    /// Get a block by index
    pub async fn get_block_by_index(&self, index: u32) -> Result<Block, RestClientError> {
        self.get(&format!("/block/{}", index), None).await
    }
    
    /// Get a block by hash
    pub async fn get_block_by_hash(&self, hash: &str) -> Result<Block, RestClientError> {
        self.get(&format!("/block/hash/{}", hash), None).await
    }
    
    /// Get a transaction by hash
    pub async fn get_transaction(&self, hash: &str) -> Result<Transaction, RestClientError> {
        self.get(&format!("/transaction/{}", hash), None).await
    }
    
    /// Get application logs for a transaction
    pub async fn get_application_log(&self, tx_hash: &str) -> Result<ApplicationLog, RestClientError> {
        self.get(&format!("/log/tx/{}", tx_hash), None).await
    }
    
    /// Get a contract state by contract hash
    pub async fn get_contract_state(&self, contract_hash: &str) -> Result<ContractState, RestClientError> {
        self.get(&format!("/contract/{}", contract_hash), None).await
    }
    
    /// Invoke a contract method (read-only)
    pub async fn invoke_function(&self, contract_hash: &str, method: &str, params: Vec<Value>) -> Result<InvocationResult, RestClientError> {
        #[derive(Serialize)]
        struct InvocationRequest {
            contract: String,
            method: String,
            params: Vec<Value>,
        }
        
        let request = InvocationRequest {
            contract: contract_hash.to_string(),
            method: method.to_string(),
            params,
        };
        
        self.post("/contract/invoke", request).await
    }
    
    /// Get token balances for an address
    pub async fn get_balances(&self, address: &str) -> Result<TokenBalances, RestClientError> {
        self.get(&format!("/address/{}/balance", address), None).await
    }
}

/// Result of a contract invocation
#[derive(Debug, Deserialize)]
pub struct InvocationResult {
    /// Script executed for the invocation
    pub script: String,
    /// State after execution
    pub state: String,
    /// Amount of gas consumed
    pub gas_consumed: String,
    /// Stack items returned
    pub stack: Vec<Value>,
    /// Notifications generated during execution
    pub notifications: Vec<Value>,
}

/// Token balances for an address
#[derive(Debug, Deserialize)]
pub struct TokenBalances {
    /// Address of the account
    pub address: String,
    /// List of token balances
    pub balances: Vec<TokenBalance>,
}

/// Balance of a specific token
#[derive(Debug, Deserialize)]
pub struct TokenBalance {
    /// Asset hash (contract hash)
    pub asset_hash: String,
    /// Name of the token
    pub name: String,
    /// Symbol of the token
    pub symbol: String,
    /// Decimals of the token
    pub decimals: u8,
    /// Balance amount as a string
    pub amount: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_get_height() {
        let client = RestClient::new("https://testnet1.neo.org:443/api").unwrap();
        let height = client.get_height().await;
        assert!(height.is_ok());
        println!("Current height: {}", height.unwrap());
    }
    
    #[tokio::test]
    async fn test_get_block() {
        let client = RestClient::new("https://testnet1.neo.org:443/api").unwrap();
        let height = client.get_height().await.unwrap();
        let block = client.get_block_by_index(height - 1).await;
        assert!(block.is_ok());
        println!("Block: {:?}", block.unwrap());
    }
} 