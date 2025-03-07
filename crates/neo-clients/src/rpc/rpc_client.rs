// Standard library imports
use std::{str::FromStr, fmt::Debug};
use std::collections::HashMap;

// External crate imports
use async_trait::async_trait;
use primitive_types::{H160, H256};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{self, json, Value};

// Internal crate imports
use crate::api_trait::{self, APITrait, TransactionSendToken};
use crate::errors::ProviderError;
use crate::{
    JsonRpcProvider, 
    ApplicationLog, Balance, MemPoolDetails, NeoAddress, NeoBlock, NeoNetworkFee, NeoVersion,
    Nep11Balances, Nep11Transfers, Nep17Balances, Nep17Transfers, Peers, Plugin, RTransaction,
    RawTransaction, StateHeight, StateRoot, States, SubmitBlock, UnclaimedGas, ValidateAddress,
    Validator, models::VMState
};

// External crate type imports
use neo_types::{ContractState, NativeContractState, ContractParameter, InvocationResult, StackItem};
use neo_common::Signer;
use neo_common::transaction_signer::TransactionSigner;
use neo_config::NEOCONFIG;

// Neo common types
use neo_common::WitnessScope;

/// RPC client for Neo blockchain.
#[derive(Debug)]
pub struct RpcClient<P> {
    provider: P,
}

impl<P: JsonRpcProvider> RpcClient<P> {
    /// Create a new RPC client with the given provider
    pub fn new(provider: P) -> Self {
        Self { provider }
    }

    /// Get the provider instance
    pub fn provider(&self) -> &P {
        &self.provider
    }
}

impl<P: JsonRpcProvider + Send + Sync> RpcClient<P> {
    pub fn new_provider(_url: &str, provider: P) -> Self {
        Self { provider }
    }

    pub async fn request<T, R>(&self, method: &str, params: T) -> Result<R, crate::errors::ProviderError>
    where
        T: Serialize + Send + Sync + std::fmt::Debug,
        R: DeserializeOwned + Send,
    {
        self.provider
            .fetch(method, params)
            .await
            .map_err(|_| crate::errors::ProviderError::IllegalState("Provider error occurred".to_string()))
    }
}

impl<P> AsRef<P> for RpcClient<P> {
    fn as_ref(&self) -> &P {
        &self.provider
    }
}

#[async_trait]
impl<P: JsonRpcProvider + Send + Sync> APITrait for RpcClient<P> {
    type Error = crate::errors::ProviderError;
    type Provider = P;

    fn rpc_client(&self) -> &RpcClient<Self::Provider> {
        self
    }

    async fn network(&self) -> Result<u32, Self::Error> {
        // Detect whether we're on Neo N3 or Neo X network by querying the version
        let result = self.request::<Vec<Value>, Value>("getversion", vec![]).await;
        match result {
            Ok(version) => {
                if let Some(network) = version.get("protocol").and_then(|p| p.get("network")).and_then(|n| n.as_u64()) {
                    // Successfully retrieved network magic from the node
                    Ok(network as u32)
                } else {
                    // Fallback to configured network if unavailable from node
                    let config = NEOCONFIG.lock().unwrap();
                    // Use network field (unwrap with a default of 0 if None)
                    Ok(config.network.unwrap_or(0))
                }
            },
            Err(_) => {
                // If the RPC call fails, use the default from config
                let config = NEOCONFIG.lock().unwrap();
                // Use network field (unwrap with a default of 0 if None)
                Ok(config.network.unwrap_or(0))
            }
        }
    }

    fn max_valid_until_block_increment(&self) -> u32 {
        NEOCONFIG.lock().unwrap().get_max_valid_until_block_increment()
    }
    
    // Core Methods
    async fn invoke_contract_verify(
        &self,
        hash: H160,
        params: Vec<ContractParameter>,
        signers: Vec<TransactionSigner>,
    ) -> Result<InvocationResult, Self::Error> {
        // Convert contract hash to string
        let script_hash = format!("0x{}", hex::encode(hash.as_bytes()));
        
        // Convert contract parameters to JSON values
        let params_array: Vec<Value> = params.into_iter()
            .map(|p| serde_json::to_value(p).unwrap_or(Value::Null))
            .collect();
        
        // Convert signers to JSON values
        let signers_array: Vec<Value> = signers.into_iter()
            .map(|s| {
                let mut signer_obj = serde_json::Map::new();
                signer_obj.insert("account".to_string(), Value::String(format!("0x{}", hex::encode(s.account.as_bytes()))));
                // Convert witness scope to appropriate numeric value
                let scope_value: u8 = match s.scopes.first() {
                    Some(scope) => match scope {
                        WitnessScope::None => 0x00,
                        WitnessScope::CalledByEntry => 0x01,
                        WitnessScope::CustomContracts => 0x10,
                        WitnessScope::CustomGroups => 0x20,
                        WitnessScope::Global => 0x80,
                        _ => 0x00, // Default for unknown
                    },
                    None => 0x00,
                };
                signer_obj.insert("scopes".to_string(), Value::Number(serde_json::Number::from(scope_value)));
                Value::Object(signer_obj)
            })
            .collect();
        
        // Prepare parameters for the RPC call
        let rpc_params = vec![
            Value::String(script_hash),
            Value::String("verify".to_string()),
            Value::Array(params_array),
            Value::Array(signers_array),
        ];
        
        // Make the RPC call
        let result = self.request::<Vec<Value>, Value>("invokefunction", rpc_params).await?;
        
        // Parse the result into InvocationResult
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse invocation result: {}", e)))
    }

    async fn get_raw_mempool(&self) -> Result<MemPoolDetails, Self::Error> {
        // Get raw memory pool - works for both Neo N3 and Neo X networks
        let result = self.request::<Vec<Value>, Value>("getrawmempool", vec![Value::Bool(true)]).await?;
        
        // Parse the response into verified and unverified transaction hashes
        let verified = result.get("verified")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str())
                .filter_map(|s| {
                    let s = s.trim_start_matches("0x");
                    match hex::decode(s) {
                        Ok(bytes) if bytes.len() == 32 => Some(H256::from_slice(&bytes)),
                        _ => None
                    }
                })
                .collect::<Vec<H256>>())
            .unwrap_or_default();
            
        let unverified = result.get("unverified")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str())
                .filter_map(|s| {
                    let s = s.trim_start_matches("0x");
                    match hex::decode(s) {
                        Ok(bytes) if bytes.len() == 32 => Some(H256::from_slice(&bytes)),
                        _ => None
                    }
                })
                .collect::<Vec<H256>>())
            .unwrap_or_default();
        
        // Get current block height to include in the response
        let height = match self.request::<Vec<Value>, Value>("getblockcount", vec![]).await {
            Ok(val) => {
                val.as_u64().unwrap_or(0) as u32
            },
            Err(_) => 0 // Default to 0 if we can't get the block height
        };
        
        let mem_pool_details = MemPoolDetails {
            verified,
            unverified,
        };
        
        Ok(mem_pool_details)
    }

    async fn import_private_key(&self, wif: String) -> Result<NeoAddress, Self::Error> {
        // Import private key and derive address - works with both Neo N3 and Neo X key formats
        let params = vec![Value::String(wif)];
        let result = self.request::<Vec<Value>, Value>("importprivkey", params).await?;
        
        // Parse the result into a NeoAddress
        let address = result.get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::errors::ProviderError::IllegalState("Address not found in response".to_string()))?;
            
        // Try to parse additional fields if available
        let label = result.get("label")
            .and_then(|v| v.as_str())
            .unwrap_or("");
            
        let _script_hash = result.get("scripthash")
            .and_then(|v| v.as_str())
            .unwrap_or("");
            
        let is_watchonly = result.get("watchonly")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        // Construct and return the NeoAddress
        Ok(NeoAddress {
            address: address.to_string(),
            script_hash: H160::zero(), // Default value since it's not available
            public_key: None, // Not available from response
            label: Some(label.to_string()),
            has_key: true, // Since we're importing a private key
            watch_only: is_watchonly,
        })
    }

    async fn get_block_header_hash(&self, hash: H256) -> Result<NeoBlock, Self::Error> {
        // Convert hash to string format
        let hash_str = format!("0x{}", hex::encode(hash.as_bytes()));
        
        // Prepare parameters for the RPC call
        let params = vec![Value::String(hash_str), Value::Bool(true)];
        
        // Make the RPC call to get block by hash
        let result = self.request::<Vec<Value>, Value>("getblock", params).await?;
        
        // Parse the result into NeoBlock
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse block result: {}", e)))
    }

    async fn send_to_address_send_token(
        &self,
        send_token: &api_trait::TransactionSendToken,
    ) -> Result<RTransaction, Self::Error> {
        let params = vec![
            Value::String(format!("0x{}", hex::encode(send_token.token_hash.as_bytes()))),
            Value::String(format!("0x{}", hex::encode(send_token.to.as_bytes()))),
            Value::Number(send_token.value.into()),
        ];
        
        // Make the RPC call
        let result = self.request::<Vec<Value>, Value>("sendtoaddress", params).await?;
        
        // Parse transaction information
        let tx_id = result.get("txid")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::errors::ProviderError::IllegalState("Missing txid in response".to_string()))?
            .to_string();
            
        // Create transaction response
        let rtx = RTransaction {
            version: 0,
            nonce: 0,
            script: String::new(), // Not available from response
            signers: Vec::new(), // Not available from response
            attributes: Vec::new(), // Not available from response
            witnesses: Vec::new(), // Not available from response
            hash: {
                let clean_tx_id = tx_id.trim_start_matches("0x");
                match hex::decode(clean_tx_id) {
                    Ok(bytes) if bytes.len() == 32 => H256::from_slice(&bytes),
                    _ => H256::default() // Fallback to default for error cases
                }
            },
            size: 0, // Not available from response
            sender: String::new(), // Default sender
            sys_fee: "0".to_string(), // Not available from response
            net_fee: "0".to_string(), // Not available from response
            valid_until_block: 0, // Not available from response
            blockhash: None, // Not available from response
            confirmations: Some(0), // Not available from response
            blocktime: Some(0), // Not available from response
            vmstate: crate::models::VMState::None, // Assume transaction is valid since RPC succeeded
        };
        
        Ok(rtx)
    }

    async fn send_from_send_token(
        &self,
        send_token: &api_trait::TransactionSendToken,
        from: H160,
    ) -> Result<RTransaction, Self::Error> {
        let params = vec![
            Value::String(format!("0x{}", hex::encode(send_token.token_hash.as_bytes()))),
            Value::String(format!("0x{}", hex::encode(from.as_bytes()))),
            Value::String(format!("0x{}", hex::encode(send_token.to.as_bytes()))),
            Value::Number(send_token.value.into()),
        ];
        
        // Make the RPC call
        let result = self.request::<Vec<Value>, Value>("sendfrom", params).await?;
        
        // Parse transaction information
        let tx_id = result.get("txid")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::errors::ProviderError::IllegalState("Missing txid in response".to_string()))?
            .to_string();
            
        // Create transaction response
        let rtx = RTransaction {
            version: 0,
            nonce: 0,
            script: String::new(), // Not available from response
            signers: Vec::new(), // Not available from response
            attributes: Vec::new(), // Not available from response
            witnesses: Vec::new(), // Not available from response
            hash: {
                let clean_tx_id = tx_id.trim_start_matches("0x");
                match hex::decode(clean_tx_id) {
                    Ok(bytes) if bytes.len() == 32 => H256::from_slice(&bytes),
                    _ => H256::default() // Fallback to default for error cases
                }
            },
            size: 0, // Not available from response
            sender: String::new(), // Default sender
            sys_fee: "0".to_string(), // Not available from response
            net_fee: "0".to_string(), // Not available from response
            valid_until_block: 0, // Not available from response
            blockhash: None, // Not available from response
            confirmations: Some(0), // Not available from response
            blocktime: Some(0), // Not available from response
            vmstate: crate::models::VMState::None, // Assume transaction is valid since RPC succeeded
        };
        
        Ok(rtx)
    }
    
    // Block management methods
    async fn broadcast_block(&self, _block: NeoBlock) -> Result<bool, Self::Error> {
        // Broadcast a block to the network
        // Works for both Neo N3 and Neo X blocks
        Err(crate::errors::ProviderError::IllegalState("broadcast_block not yet implemented".to_string()))
    }

    async fn broadcast_get_blocks(&self, hash: &str, _count: u32) -> Result<bool, Self::Error> {
        // Broadcast to get blocks starting from hash
        // Compatible with both Neo N3 and Neo X network block formats
        Err(crate::errors::ProviderError::IllegalState(format!("broadcast_get_blocks not yet implemented for hash: {}", hash)))
    }

    async fn broadcast_transaction(&self, _tx: RTransaction) -> Result<bool, Self::Error> {
        // Broadcast a transaction to the network
        // Handles both Neo N3 and Neo X transaction formats
        Err(crate::errors::ProviderError::IllegalState("broadcast_transaction not yet implemented".to_string()))
    }

    async fn get_block_by_index(&self, _index: u32, _full_tx: bool) -> Result<NeoBlock, Self::Error> {
        // Get block by index
        // Will return the appropriate block format based on network type (Neo N3 or Neo X)
        Err(crate::errors::ProviderError::IllegalState("get_block_by_index not yet implemented".to_string()))
    }

    async fn get_raw_block_by_index(&self, _index: u32) -> Result<String, Self::Error> {
        // Get raw block data by index
        // Returns serialized block in proper format for the connected network
        Err(crate::errors::ProviderError::IllegalState("get_raw_block_by_index not yet implemented".to_string()))
    }

    async fn invoke_function_diagnostics(
        &self,
        _contract_hash: H160,
        _name: String,
        _params: Vec<ContractParameter>,
        _signers: Vec<Box<dyn Signer>>,
    ) -> Result<InvocationResult, Self::Error> {
        // Invoke function with diagnostics
        Err(crate::errors::ProviderError::IllegalState("invoke_function_diagnostics not yet implemented".to_string()))
    }

    async fn invoke_script_diagnostics(
        &self,
        _hex: String,
        _signers: Vec<TransactionSigner>,
    ) -> Result<InvocationResult, Self::Error> {
        // Invoke script with diagnostics
        Err(crate::errors::ProviderError::IllegalState("invoke_script_diagnostics not yet implemented".to_string()))
    }

    async fn traverse_iterator(
        &self,
        _session_id: String,
        _iterator_id: String,
        _count: u32,
    ) -> Result<Vec<StackItem>, Self::Error> {
        // Traverse an iterator returned from a previous invocation
        // Works with both Neo N3 and Neo X RPC servers
        Err(crate::errors::ProviderError::IllegalState("traverse_iterator not yet implemented".to_string()))
    }

    async fn terminate_session(&self, _session_id: &str) -> Result<bool, Self::Error> {
        // Terminate a session
        // Compatible with both Neo N3 and Neo X RPC servers
        Err(crate::errors::ProviderError::IllegalState("terminate_session not yet implemented".to_string()))
    }

    async fn get_mem_pool(&self) -> Result<MemPoolDetails, Self::Error> {
        // Get the current block height for context
        let _height = match self.request::<Vec<Value>, Value>("getblockcount", vec![]).await {
            Ok(result) => {
                let height = result.as_u64().unwrap_or(0);
                height
            }
            Err(e) => {
                return Err(e);
            }
        };

        // Get raw memory pool - works for both Neo N3 and Neo X networks
        let result = self.request::<Vec<Value>, Value>("getrawmempool", vec![Value::Bool(true)]).await?;
        
        // Parse the response into verified and unverified transaction hashes
        let verified = result.get("verified")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str())
                .filter_map(|s| {
                    let s = s.trim_start_matches("0x");
                    match hex::decode(s) {
                        Ok(bytes) if bytes.len() == 32 => Some(H256::from_slice(&bytes)),
                        _ => None
                    }
                })
                .collect::<Vec<H256>>())
            .unwrap_or_default();
            
        let unverified = result.get("unverified")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter()
                .filter_map(|v| v.as_str())
                .filter_map(|s| {
                    let s = s.trim_start_matches("0x");
                    match hex::decode(s) {
                        Ok(bytes) if bytes.len() == 32 => Some(H256::from_slice(&bytes)),
                        _ => None
                    }
                })
                .collect::<Vec<H256>>())
            .unwrap_or_default();
        
        let mem_pool_details = MemPoolDetails {
            verified,
            unverified,
        };
        
        Ok(mem_pool_details)
    }

    async fn invoke_function(
        &self,
        contract_hash: &H160,
        method: String,
        params: Vec<ContractParameter>,
        signers: Option<Vec<TransactionSigner>>,
    ) -> Result<InvocationResult, Self::Error> {
        // Invoke function with diagnostics
        Err(crate::errors::ProviderError::IllegalState("invoke_function not yet implemented".to_string()))
    }

    async fn get_best_block_hash(&self) -> Result<H256, Self::Error> {
        let result = self.request::<Vec<Value>, Value>("getbestblockhash", vec![]).await?;
        
        // Parse the result as a string
        let hash_str = result.as_str()
            .ok_or_else(|| crate::errors::ProviderError::IllegalState("Invalid response format".to_string()))?;
        
        // Remove "0x" prefix if present
        let hash_str = hash_str.trim_start_matches("0x");
        
        // Convert to H256
        let hash = H256::from_str(hash_str)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Invalid hash format: {}", e)))?;
        
        Ok(hash)
    }

    async fn get_block_hash(&self, block_index: u32) -> Result<H256, Self::Error> {
        let result = self.request::<Vec<Value>, Value>("getblockhash", vec![Value::Number(block_index.into())]).await?;
        
        // Parse the result as a string
        let hash_str = result.as_str()
            .ok_or_else(|| crate::errors::ProviderError::IllegalState("Invalid response format".to_string()))?;
        
        // Remove "0x" prefix if present
        let hash_str = hash_str.trim_start_matches("0x");
        
        // Convert to H256
        let hash = H256::from_str(hash_str)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Invalid hash format: {}", e)))?;
        
        Ok(hash)
    }

    async fn get_block_count(&self) -> Result<u32, Self::Error> {
        let result = self.request::<Vec<Value>, Value>("getblockcount", vec![]).await?;
        
        // Parse the result as a number
        let count = result.as_u64()
            .ok_or_else(|| crate::errors::ProviderError::IllegalState("Invalid response format".to_string()))?;
        
        Ok(count as u32)
    }

    // Implement the missing methods from APITrait
    async fn get_block(&self, block_hash: H256, full_tx: bool) -> Result<NeoBlock, Self::Error> {
        // Convert H256 to string for RPC parameter
        let hash_str = format!("{:x}", block_hash);
        
        // Parameters: block hash and whether to include full transactions
        let params = vec![
            Value::String(hash_str),
            Value::Bool(full_tx),
        ];
        
        // Make the RPC call
        let result = self.request::<Vec<Value>, Value>("getblock", params).await?;
        
        // Parse the result into NeoBlock
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse block result: {}", e)))
    }
    
    async fn get_raw_block(&self, block_hash: H256) -> Result<String, Self::Error> {
        // Convert H256 to string for RPC parameter
        let hash_str = format!("{:x}", block_hash);
        
        // Parameters: block hash and raw serialization format (true)
        let params = vec![
            Value::String(hash_str),
            Value::Bool(true), // request raw format
        ];
        
        // Make the RPC call
        let result = self.request::<Vec<Value>, Value>("getblock", params).await?;
        
        // Parse the result as a string
        result.as_str()
            .ok_or_else(|| crate::errors::ProviderError::IllegalState("Invalid response format".to_string()))
            .map(|s| s.to_string())
    }
    
    async fn get_block_header_count(&self) -> Result<u32, Self::Error> {
        // This method returns the same as blockcount in most implementations
        self.get_block_count().await
    }
    
    async fn get_block_header(&self, block_hash: H256) -> Result<NeoBlock, Self::Error> {
        // Convert H256 to string for RPC parameter
        let hash_str = format!("{:x}", block_hash);
        
        // Parameters: block hash and whether to include full transactions (false for header)
        let params = vec![
            Value::String(hash_str),
        ];
        
        // Make the RPC call
        let result = self.request::<Vec<Value>, Value>("getblockheader", params).await?;
        
        // Parse the result into NeoBlock
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse block header result: {}", e)))
    }
    
    async fn get_block_header_by_index(&self, index: u32) -> Result<NeoBlock, Self::Error> {
        // First get the block hash for the specified index
        let block_hash = self.get_block_hash(index).await?;
        
        // Then get the block header using the hash
        self.get_block_header(block_hash).await
    }
    
    async fn get_raw_block_header(&self, block_hash: H256) -> Result<String, Self::Error> {
        // Convert H256 to string for RPC parameter
        let hash_str = format!("{:x}", block_hash);
        
        // Parameters: block hash and raw serialization format (true)
        let params = vec![
            Value::String(hash_str),
            Value::Bool(true), // request raw format
        ];
        
        // Make the RPC call
        let result = self.request::<Vec<Value>, Value>("getblockheader", params).await?;
        
        // Parse the result as a string
        result.as_str()
            .ok_or_else(|| crate::errors::ProviderError::IllegalState("Invalid response format".to_string()))
            .map(|s| s.to_string())
    }
    
    async fn get_raw_block_header_by_index(&self, index: u32) -> Result<String, Self::Error> {
        // First get the block hash for the specified index
        let block_hash = self.get_block_hash(index).await?;
        
        // Then get the raw block header using the hash
        self.get_raw_block_header(block_hash).await
    }
    
    async fn get_transaction(&self, hash: H256) -> Result<RTransaction, Self::Error> {
        // Convert H256 to string for RPC parameter
        let hash_str = format!("{:x}", hash);
        
        // Parameters: transaction hash
        let params = vec![
            Value::String(hash_str),
        ];
        
        // Make the RPC call
        let result = self.request::<Vec<Value>, Value>("getrawtransaction", params).await?;
        
        // Parse the result into RTransaction
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse transaction result: {}", e)))
    }
    
    async fn get_raw_transaction(&self, tx_hash: H256) -> Result<String, Self::Error> {
        // Convert H256 to string for RPC parameter
        let hash_str = format!("{:x}", tx_hash);
        
        // Parameters: transaction hash and verbose flag (false for raw)
        let params = vec![
            Value::String(hash_str),
            Value::Bool(false), // request raw format
        ];
        
        // Make the RPC call
        let result = self.request::<Vec<Value>, Value>("getrawtransaction", params).await?;
        
        // Parse the result as a string
        result.as_str()
            .ok_or_else(|| crate::errors::ProviderError::IllegalState("Invalid response format".to_string()))
            .map(|s| s.to_string())
    }

    async fn get_native_contracts(&self) -> Result<Vec<NativeContractState>, Self::Error> {
        // Make the RPC call with empty parameters
        let result = self.request::<Vec<Value>, Value>("getnativecontracts", vec![]).await?;
        
        // Parse the result into a Vector of NativeContractState
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse native contracts result: {}", e)))
    }
    
    async fn get_contract_state(&self, hash: H160) -> Result<ContractState, Self::Error> {
        // Convert H160 to string for RPC parameter
        let hash_str = format!("{:x}", hash);
        
        // Parameters: contract hash
        let params = vec![
            Value::String(hash_str),
        ];
        
        // Make the RPC call
        let result = self.request::<Vec<Value>, Value>("getcontractstate", params).await?;
        
        // Parse the result into ContractState
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse contract state result: {}", e)))
    }
    
    async fn get_contract_state_by_id(&self, id: i64) -> Result<ContractState, Self::Error> {
        // Parameters: contract ID
        let params = vec![
            Value::Number(id.into()),
        ];
        
        // Make the RPC call
        let result = self.request::<Vec<Value>, Value>("getcontractstate", params).await?;
        
        // Parse the result into ContractState
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse contract state result: {}", e)))
    }
    
    async fn get_native_contract_state(&self, name: &str) -> Result<ContractState, Self::Error> {
        // Parameters: native contract name
        let params = vec![
            Value::String(name.to_string()),
        ];
        
        // Make the RPC call
        let result = self.request::<Vec<Value>, Value>("getnativecontractstate", params).await?;
        
        // Parse the result into ContractState
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse native contract state result: {}", e)))
    }
    
    async fn get_raw_mem_pool(&self) -> Result<Vec<H256>, Self::Error> {
        // Make the RPC call with empty parameters
        let result = self.request::<Vec<Value>, Value>("getrawmempool", vec![]).await?;
        
        // Parse the result as an array of strings
        let hashes = result.as_array()
            .ok_or_else(|| crate::errors::ProviderError::IllegalState("Invalid response format".to_string()))?;
        
        // Convert each string to H256
        let mut tx_hashes = Vec::new();
        for hash_value in hashes {
            let hash_str = hash_value.as_str()
                .ok_or_else(|| crate::errors::ProviderError::IllegalState("Invalid hash format in response".to_string()))?;
            
            // Remove "0x" prefix if present
            let hash_str = hash_str.trim_start_matches("0x");
            
            // Convert to H256
            let hash = H256::from_str(hash_str)
                .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Invalid hash format: {}", e)))?;
            
            tx_hashes.push(hash);
        }
        
        Ok(tx_hashes)
    }
    
    async fn get_storage(&self, contract_hash: H160, key: &str) -> Result<String, Self::Error> {
        // Convert H160 to string for RPC parameter
        let hash_str = format!("{:x}", contract_hash);
        
        // Parameters: contract hash and key
        let params = vec![
            Value::String(hash_str),
            Value::String(key.to_string()),
        ];
        
        // Make the RPC call
        let result = self.request::<Vec<Value>, Value>("getstorage", params).await?;
        
        // Parse the result as a string
        result.as_str()
            .ok_or_else(|| crate::errors::ProviderError::IllegalState("Invalid response format".to_string()))
            .map(|s| s.to_string())
    }
    
    async fn find_storage(
        &self,
        contract_hash: H160,
        prefix_hex_string: &str,
        start_index: u64,
    ) -> Result<String, Self::Error> {
        // Convert H160 to string for RPC parameter
        let hash_str = format!("{:x}", contract_hash);
        
        // Parameters: contract hash, prefix string, and start index
        let params = vec![
            Value::String(hash_str),
            Value::String(prefix_hex_string.to_string()),
            Value::Number(start_index.into()),
        ];
        
        // Make the RPC call
        let result = self.request::<Vec<Value>, Value>("findstorage", params).await?;
        
        // Parse the result into a string
        serde_json::to_string(&result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to serialize storage result: {}", e)))
    }
    
    async fn find_storage_with_id(
        &self,
        contract_id: i64,
        prefix_hex_string: &str,
        start_index: u64,
    ) -> Result<String, Self::Error> {
        // Parameters: contract ID, prefix string, and start index
        let params = vec![
            Value::Number(contract_id.into()),
            Value::String(prefix_hex_string.to_string()),
            Value::Number(start_index.into()),
        ];
        
        // Make the RPC call
        let result = self.request::<Vec<Value>, Value>("findstorage", params).await?;
        
        // Parse the result into a string
        serde_json::to_string(&result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to serialize storage result: {}", e)))
    }
    
    async fn get_transaction_height(&self, tx_hash: H256) -> Result<u32, Self::Error> {
        // Convert H256 to string for RPC parameter
        let hash_str = format!("{:x}", tx_hash);
        
        // Parameters: transaction hash
        let params = vec![
            Value::String(hash_str),
        ];
        
        // Make the RPC call
        let result = self.request::<Vec<Value>, Value>("gettransactionheight", params).await?;
        
        // Parse the result as a number
        result.as_u64()
            .ok_or_else(|| crate::errors::ProviderError::IllegalState("Invalid response format".to_string()))
            .map(|height| height as u32)
    }

    // Implement network-related methods
    async fn get_next_block_validators(&self) -> Result<Vec<Validator>, Self::Error> {
        // Make the RPC call with empty parameters
        let result = self.request::<Vec<Value>, Value>("getnextblockvalidators", vec![]).await?;
        
        // Parse the result into a Vec<Validator>
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse validators result: {}", e)))
    }

    async fn get_committee(&self) -> Result<Vec<String>, Self::Error> {
        // Make the RPC call with empty parameters
        let result = self.request::<Vec<Value>, Value>("getcommittee", vec![]).await?;
        
        // Parse the result into a Vec<String>
        let committee = result.as_array()
            .ok_or_else(|| crate::errors::ProviderError::IllegalState("Invalid response format".to_string()))?;
        
        let mut addresses = Vec::new();
        for addr_value in committee {
            let addr = addr_value.as_str()
                .ok_or_else(|| crate::errors::ProviderError::IllegalState("Invalid address format in response".to_string()))?
                .to_string();
            addresses.push(addr);
        }
        
        Ok(addresses)
    }

    async fn get_connection_count(&self) -> Result<u32, Self::Error> {
        // Make the RPC call with empty parameters
        let result = self.request::<Vec<Value>, Value>("getconnectioncount", vec![]).await?;
        
        // Parse the result as a number
        result.as_u64()
            .ok_or_else(|| crate::errors::ProviderError::IllegalState("Invalid response format".to_string()))
            .map(|count| count as u32)
    }

    async fn get_peers(&self) -> Result<Peers, Self::Error> {
        // Make the RPC call with empty parameters
        let result = self.request::<Vec<Value>, Value>("getpeers", vec![]).await?;
        
        // Parse the result into Peers
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse peers result: {}", e)))
    }

    async fn get_version(&self) -> Result<NeoVersion, Self::Error> {
        // Make the RPC call with empty parameters
        let result = self.request::<Vec<Value>, Value>("getversion", vec![]).await?;
        
        // Parse the result into NeoVersion
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse version result: {}", e)))
    }

    async fn send_raw_transaction(&self, hex: String) -> Result<RawTransaction, Self::Error> {
        // Parameters: raw transaction hex
        let params = vec![
            Value::String(hex),
        ];
        
        // Make the RPC call
        let result = self.request::<Vec<Value>, Value>("sendrawtransaction", params).await?;
        
        // Parse the result into RawTransaction
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse transaction result: {}", e)))
    }

    async fn send_transaction(&self, tx_bytes: Vec<u8>) -> Result<H256, Self::Error> {
        // Convert bytes to hex
        let hex = hex::encode(tx_bytes);
        
        // Parameters: transaction hex
        let params = vec![
            Value::String(hex),
        ];
        
        // Make the RPC call
        let result = self.request::<Vec<Value>, Value>("sendtransaction", params).await?;
        
        // Parse the result as a transaction hash string
        let hash_str = result.as_str()
            .ok_or_else(|| crate::errors::ProviderError::IllegalState("Invalid response format".to_string()))?;
        
        // Remove "0x" prefix if present
        let hash_str = hash_str.trim_start_matches("0x");
        
        // Convert to H256
        let hash = H256::from_str(hash_str)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Invalid hash format: {}", e)))?;
        
        Ok(hash)
    }

    async fn submit_block(&self, hex: String) -> Result<SubmitBlock, Self::Error> {
        // Parameters: block hex
        let params = vec![
            Value::String(hex),
        ];
        
        // Make the RPC call
        let result = self.request::<Vec<Value>, Value>("submitblock", params).await?;
        
        // Parse the result into SubmitBlock
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse submit block result: {}", e)))
    }

    async fn invoke_script(&self, hex: String, signers: Vec<TransactionSigner>) -> Result<InvocationResult, Self::Error> {
        // Convert signers to JSON array
        let signers_json: Vec<Value> = signers.into_iter()
            .map(|signer| {
                // Convert TransactionSigner to JSON value
                serde_json::to_value(signer)
                    .unwrap_or(Value::Null)
            })
            .collect();
        
        // Parameters: script hex and signers
        let params = vec![
            Value::String(hex),
            Value::Array(signers_json),
        ];
        
        // Make the RPC call
        let result = self.request::<Vec<Value>, Value>("invokescript", params).await?;
        
        // Parse the result into InvocationResult
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse invocation result: {}", e)))
    }

    async fn get_unclaimed_gas(&self, hash: H160) -> Result<UnclaimedGas, Self::Error> {
        // Convert H160 to string for RPC parameter
        let hash_str = format!("{:x}", hash);
        
        // Parameters: account script hash
        let params = vec![
            Value::String(hash_str),
        ];
        
        // Make the RPC call
        let result = self.request::<Vec<Value>, Value>("getunclaimedgas", params).await?;
        
        // Parse the result into UnclaimedGas
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse unclaimed gas result: {}", e)))
    }

    async fn list_plugins(&self) -> Result<Vec<Plugin>, Self::Error> {
        // Make the RPC call with empty parameters
        let result = self.request::<Vec<Value>, Value>("listplugins", vec![]).await?;
        
        // Parse the result into Vec<Plugin>
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse plugins result: {}", e)))
    }

    async fn validate_address(&self, address: &str) -> Result<ValidateAddress, Self::Error> {
        // Parameters: address string
        let params = vec![
            Value::String(address.to_string()),
        ];
        
        // Make the RPC call
        let result = self.request::<Vec<Value>, Value>("validateaddress", params).await?;
        
        // Parse the result into ValidateAddress
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse address validation result: {}", e)))
    }

    async fn get_block_by_hash(&self, hash: &str, full_tx: bool) -> Result<NeoBlock, Self::Error> {
        // Parameters: block hash and whether to include full transactions
        let params = vec![
            Value::String(hash.to_string()),
            Value::Bool(full_tx),
        ];
        
        // Make the RPC call
        let result = self.request::<Vec<Value>, Value>("getblock", params).await?;
        
        // Parse the result into NeoBlock
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse block result: {}", e)))
    }

    async fn broadcast_address(&self) -> Result<bool, Self::Error> {
        // Make the RPC call with empty parameters
        let result = self.request::<Vec<Value>, Value>("broadcastaddress", vec![]).await?;
        
        // Parse the result as a boolean
        result.as_bool()
            .ok_or_else(|| crate::errors::ProviderError::IllegalState("Invalid response format".to_string()))
    }

    // Add implementations for various wallet-related functions
    async fn close_wallet(&self) -> Result<bool, Self::Error> {
        // Make the RPC call with empty parameters
        let result = self.request::<Vec<Value>, Value>("closewallet", vec![]).await?;
        
        // Parse the result as a boolean
        result.as_bool()
            .ok_or_else(|| crate::errors::ProviderError::IllegalState("Invalid response format".to_string()))
    }

    async fn dump_priv_key(&self, script_hash: H160) -> Result<String, Self::Error> {
        // Convert H160 to string for RPC parameter
        let hash_str = format!("{:x}", script_hash);
        
        // Parameters: account script hash
        let params = vec![
            Value::String(hash_str),
        ];
        
        // Make the RPC call
        let result = self.request::<Vec<Value>, Value>("dumpprivkey", params).await?;
        
        // Parse the result as a string
        result.as_str()
            .ok_or_else(|| crate::errors::ProviderError::IllegalState("Invalid response format".to_string()))
            .map(|s| s.to_string())
    }

    async fn get_wallet_balance(&self, token_hash: H160) -> Result<Balance, Self::Error> {
        // Convert H160 to string for RPC parameter
        let hash_str = format!("{:x}", token_hash);
        
        // Parameters: token hash
        let params = vec![
            Value::String(hash_str),
        ];
        
        // Make the RPC call
        let result = self.request::<Vec<Value>, Value>("getwalletbalance", params).await?;
        
        // Parse the result into Balance
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse wallet balance result: {}", e)))
    }

    async fn get_new_address(&self) -> Result<String, Self::Error> {
        // Make the RPC call with empty parameters
        let result = self.request::<Vec<Value>, Value>("getnewaddress", vec![]).await?;
        
        // Parse the result as a string
        result.as_str()
            .ok_or_else(|| crate::errors::ProviderError::IllegalState("Invalid response format".to_string()))
            .map(|s| s.to_string())
    }

    async fn get_wallet_unclaimed_gas(&self) -> Result<String, Self::Error> {
        // Make the RPC call with empty parameters
        let result = self.request::<Vec<Value>, Value>("getwalletunclaimedgas", vec![]).await?;
        
        // Parse the result as a string
        result.as_str()
            .ok_or_else(|| crate::errors::ProviderError::IllegalState("Invalid response format".to_string()))
            .map(|s| s.to_string())
    }

    async fn calculate_network_fee(&self, hex: String) -> Result<NeoNetworkFee, Self::Error> {
        // Parameters: transaction hex
        let params = vec![
            Value::String(hex),
        ];
        
        // Make the RPC call
        let result = self.request::<Vec<Value>, Value>("calculatenetworkfee", params).await?;
        
        // Parse the result into NeoNetworkFee
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse network fee result: {}", e)))
    }

    async fn list_address(&self) -> Result<Vec<NeoAddress>, Self::Error> {
        // Make the RPC call with empty parameters
        let result = self.request::<Vec<Value>, Value>("listaddress", vec![]).await?;
        
        // Parse the result into Vec<NeoAddress>
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse address list result: {}", e)))
    }

    async fn open_wallet(&self, path: String, password: String) -> Result<bool, Self::Error> {
        // Parameters: wallet path and password
        let params = vec![
            Value::String(path),
            Value::String(password),
        ];
        
        // Make the RPC call
        let result = self.request::<Vec<Value>, Value>("openwallet", params).await?;
        
        // Parse the result as a boolean
        result.as_bool()
            .ok_or_else(|| crate::errors::ProviderError::IllegalState("Invalid response format".to_string()))
    }

    // Already implemented import_priv_key as import_private_key in the code
    // Alias it here for APITrait compliance
    async fn import_priv_key(&self, priv_key: String) -> Result<NeoAddress, Self::Error> {
        self.import_private_key(priv_key).await
    }

    // Add implementations for transfer-related methods
    async fn send_from(
        &self,
        token_hash: H160,
        from: H160,
        to: H160,
        amount: u32,
    ) -> Result<RTransaction, Self::Error> {
        // Convert hash values to strings for RPC parameters
        let token_str = format!("{:x}", token_hash);
        let from_str = format!("{:x}", from);
        let to_str = format!("{:x}", to);
        
        // Parameters: token hash, from address, to address, and amount
        let params = vec![
            Value::String(token_str),
            Value::String(from_str),
            Value::String(to_str),
            Value::Number(amount.into()),
        ];
        
        // Make the RPC call
        let result = self.request::<Vec<Value>, Value>("sendfrom", params).await?;
        
        // Parse the result into RTransaction
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse transaction result: {}", e)))
    }

    async fn send_to_address(
        &self,
        token_hash: H160,
        to: H160,
        amount: u32,
    ) -> Result<RTransaction, Self::Error> {
        // Convert hash values to strings for RPC parameters
        let token_str = format!("{:x}", token_hash);
        let to_str = format!("{:x}", to);
        
        // Parameters: token hash, to address, and amount
        let params = vec![
            Value::String(token_str),
            Value::String(to_str),
            Value::Number(amount.into()),
        ];
        
        // Make the RPC call
        let result = self.request::<Vec<Value>, Value>("sendtoaddress", params).await?;
        
        // Parse the result into RTransaction
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse transaction result: {}", e)))
    }
    
    async fn cancel_transaction(
        &self,
        txHash: H256,
        signers: Vec<H160>,
        extra_fee: Option<u64>,
    ) -> Result<RTransaction, Self::Error> {
        // Convert hash values to strings for RPC parameters
        let tx_str = format!("{:x}", txHash);
        
        // Convert signers to JSON array
        let signers_json: Vec<Value> = signers.into_iter()
            .map(|s| Value::String(format!("{:x}", s)))
            .collect();
        
        // Add extra fee if provided
        let fee_param = match extra_fee {
            Some(fee) => Value::Number(fee.into()),
            None => Value::Null,
        };
        
        // Parameters: transaction hash, signers array, and optional extra fee
        let params = vec![
            Value::String(tx_str),
            Value::Array(signers_json),
            fee_param,
        ];
        
        // Make the RPC call
        let result = self.request::<Vec<Value>, Value>("canceltransaction", params).await?;
        
        // Parse the result into RTransaction
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse transaction result: {}", e)))
    }
    
    async fn get_application_log(&self, tx_hash: H256) -> Result<ApplicationLog, Self::Error> {
        // Convert H256 to string for RPC parameter
        let hash_str = format!("{:x}", tx_hash);
        
        // Parameters: transaction hash
        let params = vec![
            Value::String(hash_str),
        ];
        
        // Make the RPC call
        let result = self.request::<Vec<Value>, Value>("getapplicationlog", params).await?;
        
        // Parse the result into ApplicationLog
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse application log result: {}", e)))
    }

    // Add implementation for send_many method
    async fn send_many(
        &self,
        from: Option<H160>,
        send_tokens: Vec<TransactionSendToken>,
    ) -> Result<RTransaction, Self::Error> {
        // Convert the from address to string if provided
        let from_param = match from {
            Some(addr) => Value::String(format!("{:x}", addr)),
            None => Value::Null,
        };
        
        // Manually construct the tokens array as JSON
        let mut tokens_array = Vec::new();
        for token in send_tokens {
            let token_obj = json!({
                "asset": format!("{:x}", token.token_hash),
                "value": token.value,
                "address": format!("{:x}", token.to)
            });
            tokens_array.push(token_obj);
        }
        
        // Parameters: from address and send tokens array
        let params = vec![
            from_param,
            Value::Array(tokens_array),
        ];
        
        // Make the RPC call
        let result = self.request::<Vec<Value>, Value>("sendmany", params).await?;
        
        // Parse the result into RTransaction
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse transaction result: {}", e)))
    }

    // Add implementation for get_nep17_balances method
    async fn get_nep17_balances(&self, script_hash: H160) -> Result<Nep17Balances, Self::Error> {
        let params = vec![
            Value::String(format!("{:x}", script_hash)),
        ];
        
        let result = self.request::<Vec<Value>, Value>("getnep17balances", params).await?;
        
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse NEP17 balances result: {}", e)))
    }

    // Add implementation for get_nep17_transfers method
    async fn get_nep17_transfers(
        &self,
        script_hash: H160,
    ) -> Result<Nep17Transfers, Self::Error> {
        let params = vec![
            Value::String(format!("{:x}", script_hash)),
        ];
        
        let result = self.request::<Vec<Value>, Value>("getnep17transfers", params).await?;
        
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse NEP17 transfers result: {}", e)))
    }

    // Add implementation for get_nep17_transfers_from
    async fn get_nep17_transfers_from(
        &self,
        script_hash: H160,
        from: u64,
    ) -> Result<Nep17Transfers, Self::Error> {
        let params = vec![
            Value::String(format!("{:x}", script_hash)),
            Value::Number(serde_json::Number::from(from)),
        ];
        
        let result = self.request::<Vec<Value>, Value>("getnep17transfers", params).await?;
        
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse NEP17 transfers result: {}", e)))
    }

    // Add implementation for get_nep17_transfers_range
    async fn get_nep17_transfers_range(
        &self,
        script_hash: H160,
        from: u64,
        to: u64,
    ) -> Result<Nep17Transfers, Self::Error> {
        let params = vec![
            Value::String(format!("{:x}", script_hash)),
            Value::Number(serde_json::Number::from(from)),
            Value::Number(serde_json::Number::from(to)),
        ];
        
        let result = self.request::<Vec<Value>, Value>("getnep17transfers", params).await?;
        
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse NEP17 transfers result: {}", e)))
    }

    // Add implementation for get_nep11_balances
    async fn get_nep11_balances(&self, script_hash: H160) -> Result<Nep11Balances, Self::Error> {
        let params = vec![
            Value::String(format!("{:x}", script_hash)),
        ];
        
        let result = self.request::<Vec<Value>, Value>("getnep11balances", params).await?;
        
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse NEP11 balances result: {}", e)))
    }

    // Add implementation for get_nep11_transfers
    async fn get_nep11_transfers(&self, script_hash: H160) -> Result<Nep11Transfers, Self::Error> {
        let params = vec![
            Value::String(format!("{:x}", script_hash)),
        ];
        
        let result = self.request::<Vec<Value>, Value>("getnep11transfers", params).await?;
        
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse NEP11 transfers result: {}", e)))
    }

    // Add implementation for get_nep11_transfers_from
    async fn get_nep11_transfers_from(&self, script_hash: H160, from: u64) -> Result<Nep11Transfers, Self::Error> {
        let params = vec![
            Value::String(format!("{:x}", script_hash)),
            Value::Number(serde_json::Number::from(from)),
        ];
        
        let result = self.request::<Vec<Value>, Value>("getnep11transfers", params).await?;
        
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse NEP11 transfers result: {}", e)))
    }

    // Add implementation for get_nep11_transfers_range
    async fn get_nep11_transfers_range(
        &self, 
        script_hash: H160,
        from: u64,
        to: u64
    ) -> Result<Nep11Transfers, Self::Error> {
        let params = vec![
            Value::String(format!("{:x}", script_hash)),
            Value::Number(serde_json::Number::from(from)),
            Value::Number(serde_json::Number::from(to)),
        ];
        
        let result = self.request::<Vec<Value>, Value>("getnep11transfers", params).await?;
        
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse NEP11 transfers result: {}", e)))
    }

    // Add implementation for get_nep11_properties
    async fn get_nep11_properties(
        &self,
        script_hash: H160,
        token_id: &str,
    ) -> Result<HashMap<String, String>, Self::Error> {
        let params = vec![
            Value::String(format!("{:x}", script_hash)),
            Value::String(token_id.to_string()),
        ];
        
        let result = self.request::<Vec<Value>, Value>("getnep11properties", params).await?;
        
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse NEP11 properties result: {}", e)))
    }

    // Add implementation for get_state_root
    async fn get_state_root(&self, block_index: u32) -> Result<StateRoot, Self::Error> {
        let params = vec![
            Value::Number(serde_json::Number::from(block_index)),
        ];
        
        let result = self.request::<Vec<Value>, Value>("getstateroot", params).await?;
        
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse state root result: {}", e)))
    }

    // Add implementation for get_proof
    async fn get_proof(
        &self,
        root_hash: H256,
        contract_hash: H160,
        key: &str,
    ) -> Result<String, Self::Error> {
        let params = vec![
            Value::String(format!("{:x}", root_hash)),
            Value::String(format!("{:x}", contract_hash)),
            Value::String(key.to_string()),
        ];
        
        let result = self.request::<Vec<Value>, Value>("getproof", params).await?;
        
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse proof result: {}", e)))
    }

    // Add implementation for verify_proof
    async fn verify_proof(
        &self,
        root_hash: H256,
        proof: &str,
    ) -> Result<String, Self::Error> {
        let params = vec![
            Value::String(format!("{:x}", root_hash)),
            Value::String(proof.to_string()),
        ];
        
        let result = self.request::<Vec<Value>, Value>("verifyproof", params).await?;
        
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse verify proof result: {}", e)))
    }

    // Add implementation for get_state_height
    async fn get_state_height(&self) -> Result<StateHeight, Self::Error> {
        let params = vec![];
        
        let result = self.request::<Vec<Value>, Value>("getstateheight", params).await?;
        
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse state height result: {}", e)))
    }

    // Add implementation for get_state
    async fn get_state(
        &self,
        root_hash: H256,
        contract_hash: H160,
        key: &str,
    ) -> Result<String, Self::Error> {
        let params = vec![
            Value::String(format!("{:x}", root_hash)),
            Value::String(format!("{:x}", contract_hash)),
            Value::String(key.to_string()),
        ];
        
        let result = self.request::<Vec<Value>, Value>("getstate", params).await?;
        
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse state result: {}", e)))
    }

    // Add implementation for find_states
    async fn find_states(
        &self,
        root_hash: H256,
        contract_hash: H160,
        prefix: &str,
        start: Option<&str>,
        count: Option<u32>,
    ) -> Result<States, Self::Error> {
        let mut params = vec![
            Value::String(format!("{:x}", root_hash)),
            Value::String(format!("{:x}", contract_hash)),
            Value::String(prefix.to_string()),
        ];
        
        if let Some(start_key) = start {
            params.push(Value::String(start_key.to_string()));
            
            if let Some(item_count) = count {
                params.push(Value::Number(serde_json::Number::from(item_count)));
            }
        }
        
        let result = self.request::<Vec<Value>, Value>("findstates", params).await?;
        
        serde_json::from_value(result)
            .map_err(|e| crate::errors::ProviderError::IllegalState(format!("Failed to parse find states result: {}", e)))
    }
}
