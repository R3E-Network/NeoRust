use sgx_types::*;
use sgx_urts::SgxEnclave;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::neo_sgx::app::network::ocall_send_request;

/// RPC client for SGX-compatible Neo blockchain interactions
pub struct SgxRpcClient {
    enclave: Arc<SgxEnclave>,
    url: String,
}

impl SgxRpcClient {
    /// Creates a new SGX RPC client
    ///
    /// # Arguments
    ///
    /// * `enclave` - The SGX enclave instance
    /// * `url` - The URL of the Neo RPC node
    ///
    /// # Returns
    ///
    /// A new SGX RPC client
    pub fn new(enclave: SgxEnclave, url: String) -> Self {
        Self {
            enclave: Arc::new(enclave),
            url,
        }
    }
    
    /// Calls an RPC method on the Neo blockchain
    ///
    /// # Arguments
    ///
    /// * `method` - The RPC method to call
    /// * `params` - The parameters to pass to the method
    ///
    /// # Returns
    ///
    /// The JSON response from the RPC call
    pub fn call_method(&self, method: &str, params: Vec<Value>) -> Result<Value, sgx_status_t> {
        // Create JSON-RPC request
        let request = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });
        
        // Convert request to string
        let request_str = request.to_string();
        
        // Allocate buffer for response
        let mut response_buf = vec![0u8; 4096]; // Adjust size as needed
        let mut response_len = response_buf.len();
        
        // Call the untrusted function
        let mut retval = sgx_status_t::SGX_SUCCESS;
        let status = unsafe {
            ocall_send_request(
                &mut retval,
                self.url.as_ptr(),
                self.url.len(),
                "POST".as_ptr(),
                4,
                request_str.as_ptr(),
                request_str.len(),
                response_buf.as_mut_ptr(),
                &mut response_len,
            )
        };
        
        // Check for errors
        if status != sgx_status_t::SGX_SUCCESS {
            return Err(status);
        }
        if retval != sgx_status_t::SGX_SUCCESS {
            return Err(retval);
        }
        
        // Convert response to string
        response_buf.truncate(response_len);
        let response_str = match std::str::from_utf8(&response_buf) {
            Ok(s) => s,
            Err(_) => return Err(sgx_status_t::SGX_ERROR_UNEXPECTED),
        };
        
        // Parse JSON response
        match serde_json::from_str::<Value>(response_str) {
            Ok(value) => Ok(value),
            Err(_) => Err(sgx_status_t::SGX_ERROR_UNEXPECTED),
        }
    }
    
    /// Gets the best block height from the Neo blockchain
    ///
    /// # Returns
    ///
    /// The best block height
    pub fn get_block_count(&self) -> Result<u32, sgx_status_t> {
        let response = self.call_method("getblockcount", vec![])?;
        
        // Extract result from response
        match response.get("result") {
            Some(Value::Number(n)) => {
                match n.as_u64() {
                    Some(height) => Ok(height as u32),
                    None => Err(sgx_status_t::SGX_ERROR_UNEXPECTED),
                }
            }
            _ => Err(sgx_status_t::SGX_ERROR_UNEXPECTED),
        }
    }
    
    /// Gets a block by its hash or index
    ///
    /// # Arguments
    ///
    /// * `hash_or_index` - The hash or index of the block
    ///
    /// # Returns
    ///
    /// The block as a JSON value
    pub fn get_block(&self, hash_or_index: &str) -> Result<Value, sgx_status_t> {
        let response = self.call_method("getblock", vec![json!(hash_or_index), json!(1)])?;
        
        // Extract result from response
        match response.get("result") {
            Some(block) => Ok(block.clone()),
            None => Err(sgx_status_t::SGX_ERROR_UNEXPECTED),
        }
    }
    
    /// Gets the balance of an address
    ///
    /// # Arguments
    ///
    /// * `address` - The address to get the balance for
    /// * `asset_id` - The asset ID to get the balance for
    ///
    /// # Returns
    ///
    /// The balance as a string
    pub fn get_balance(&self, address: &str, asset_id: &str) -> Result<String, sgx_status_t> {
        let response = self.call_method(
            "getbalance",
            vec![json!(address), json!(asset_id)],
        )?;
        
        // Extract result from response
        match response.get("result") {
            Some(Value::String(balance)) => Ok(balance.clone()),
            _ => Err(sgx_status_t::SGX_ERROR_UNEXPECTED),
        }
    }
    
    /// Sends a raw transaction to the Neo blockchain
    ///
    /// # Arguments
    ///
    /// * `hex` - The raw transaction as a hex string
    ///
    /// # Returns
    ///
    /// The transaction hash
    pub fn send_raw_transaction(&self, hex: &str) -> Result<String, sgx_status_t> {
        let response = self.call_method("sendrawtransaction", vec![json!(hex)])?;
        
        // Extract result from response
        match response.get("result") {
            Some(Value::String(hash)) => Ok(hash.clone()),
            _ => Err(sgx_status_t::SGX_ERROR_UNEXPECTED),
        }
    }
}
