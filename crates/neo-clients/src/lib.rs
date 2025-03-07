//! # Neo Clients
//!
//! Client implementations for interacting with Neo N3 blockchain nodes.
//!
//! This crate provides various client implementations for connecting to and interacting with Neo N3 blockchain nodes, including:
//!
//! - JSON-RPC client for communicating with Neo nodes
//! - WebSocket client for real-time event subscriptions
//! - HTTP client for standard API requests
//! - Provider abstractions for different connection types
//! - Middleware support for request/response processing
//!
//! ## Usage
//!
//! ```rust,ignore
//! use neo_clients::{JsonRpcProvider, RpcClient};
//! use std::str::FromStr;
//!
//! // Create a JSON-RPC provider
//! let provider = JsonRpcProvider::new("https://mainnet.neoline.io:443");
//!
//! // Get the latest block height
//! let block_height = provider.get_block_count().await?;
//!
//! // Create a WebSocket client for subscriptions
//! let ws_client = RpcClient::new_websocket("wss://mainnet.neoline.io:4443/ws");
//!
//! // Subscribe to new blocks
//! let subscription = ws_client.subscribe_to_new_blocks().await?;
//! ```

#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms, unreachable_pub)]
#![deny(rustdoc::broken_intra_doc_links)]

mod api_trait;
mod errors;
mod error_adapter;
mod ext;
mod mock_blocks;
mod mock_client;
mod models;
mod nep17_provider_impl;
mod rpc;
mod rx;
mod utils;

// Re-export all public items
pub use api_trait::*;
pub use errors::*;
pub use error_adapter::*;
pub use ext::*;
pub use mock_blocks::*;
pub use mock_client::*;
pub use models::*;
pub use rpc::*;
pub use rx::*;
pub use utils::*;

use std::str::FromStr;
use primitive_types::{H256, H160};
use log;

/// Helper function to parse a string into a TransactionSigner object.
/// 
/// # Format
/// The expected format is: `"<script_hash>:<scope>"` where:
/// - `script_hash` is the hex representation of the account's script hash (with or without 0x prefix)
/// - `scope` is a numeric value representing the witness scope (e.g., 1 for CalledByEntry)
/// 
/// Alternative formats supported:
/// - `"<script_hash>"` - Defaults to CalledByEntry scope (1)
/// - `"<script_hash>:entry"` - Translates to CalledByEntry scope
/// - `"<script_hash>:global"` - Translates to Global scope
/// 
/// # Returns
/// Returns `Some(TransactionSigner)` if parsing is successful, or `None` if the input format is invalid.
/// 
/// # Compatibility
/// This function supports both Neo N3 and Neo X signer formats and address conventions.
fn parse_signer(signer_str: &str) -> Option<neo_common::transaction_signer::TransactionSigner> {
    // Format expected: "<script_hash>:<scope>"
    let parts: Vec<&str> = signer_str.split(':').collect();
    if parts.is_empty() {
        log::warn!("Empty signer string provided");
        return None;
    }
    
    // Extract script_hash and handle 0x prefix for both Neo N3 and Neo X addresses
    let hash_str = parts[0].trim_start_matches("0x");
    
    // Parse hash
    let script_hash = match H160::from_str(hash_str) {
        Ok(hash) => hash,
        Err(e) => {
            log::warn!("Failed to parse signer script hash '{}': {}", hash_str, e);
            return None;
        }
    };
    
    // Extract and parse scope
    let scope = if parts.len() >= 2 {
        let scope_str = parts[1].to_lowercase();
        match scope_str.as_str() {
            // Text-based scope values for better usability
            "entry" | "calledbyentry" => 1, // CALLED_BY_ENTRY
            "global" => 128, // GLOBAL
            "none" => 0, // NONE
            "custom" | "customcontracts" => 16, // CUSTOM_CONTRACTS
            
            // Numeric scope values
            _ => match u8::from_str(scope_str.as_str()) {
                Ok(s) => s,
                Err(e) => {
                    log::warn!("Failed to parse signer scope '{}': {}, defaulting to CalledByEntry", scope_str, e);
                    1 // Default to CalledByEntry
                }
            },
        }
    } else {
        // Default to CalledByEntry if no scope is provided
        log::debug!("No scope provided for signer, defaulting to CalledByEntry");
        1 // CalledByEntry
    };
    
    log::debug!("Parsed signer with script hash: {} and scope: {}", script_hash, scope);
    
    // Convert scope to WitnessScope
    let witness_scope = match scope {
        0 => vec![],
        1 => vec![neo_common::WitnessScope::CalledByEntry],
        16 => vec![neo_common::WitnessScope::CustomContracts],
        128 => vec![neo_common::WitnessScope::Global],
        _ => {
            log::warn!("Unrecognized scope value: {}, defaulting to CalledByEntry", scope);
            vec![neo_common::WitnessScope::CalledByEntry]
        }
    };
    
    // Create TransactionSigner
    Some(neo_common::transaction_signer::TransactionSigner::new(
        script_hash,
        witness_scope,
        false, // allow_only_fee
        vec![], // rules
    ))
}

/// Helper function to parse a string into a ContractParameter object.
/// 
/// # Format
/// The expected format is: `"<type>:<value>"` where:
/// - `type` is one of: string, integer/int, boolean/bool, hash160, hash256, bytearray/bytes
/// - `value` is the parameter value in an appropriate format for the type
/// 
/// # Examples
/// - `"string:hello"` -> String parameter with value "hello"
/// - `"integer:42"` -> Integer parameter with value 42
/// - `"boolean:true"` -> Boolean parameter with value true
/// - `"hash160:0x6725cfc0a0b96731b8f454d0a6b30f6f6a36e13f"` -> Hash160 parameter
/// 
/// # Returns
/// Returns `Some(ContractParameter)` if parsing is successful, or `None` if the input format is invalid.
/// 
/// # Compatibility
/// This function supports parameter formats for both Neo N3 and Neo X contract invocations.
fn parse_contract_parameter(param_str: &str) -> Option<neo_types::ContractParameter> {
    // Format expected: "<type>:<value>"
    // Examples: "string:hello", "integer:42", "boolean:true", "hash160:0x..."
    let parts: Vec<&str> = param_str.split(':').collect();
    if parts.len() >= 2 {
        let param_type = parts[0].to_lowercase();
        let param_value = parts[1];
        
        log::debug!("Parsing parameter of type '{}' with value '{}'", param_type, param_value);
        
        match param_type.as_str() {
            "string" => Some(neo_types::ContractParameter::string(param_value.to_string())),
            "integer" | "int" => {
                match param_value.parse::<i64>() {
                    Ok(val) => Some(neo_types::ContractParameter::integer(val.into())),
                    Err(e) => {
                        log::warn!("Failed to parse integer parameter '{}': {}", param_value, e);
                        None
                    }
                }
            },
            "boolean" | "bool" => {
                match param_value.to_lowercase().as_str() {
                    "true" | "1" | "yes" => Some(neo_types::ContractParameter::bool(true)),
                    "false" | "0" | "no" => Some(neo_types::ContractParameter::bool(false)),
                    _ => {
                        log::warn!("Failed to parse boolean parameter: {}", param_value);
                        None
                    }
                }
            },
            "hash160" => {
                // Handle both Neo N3 and Neo X address formats
                let clean_value = param_value.trim_start_matches("0x");
                match H160::from_str(clean_value) {
                    Ok(hash) => Some(neo_types::ContractParameter::h160(&hash)),
                    Err(e) => {
                        log::warn!("Failed to parse Hash160 parameter '{}': {}", clean_value, e);
                        None
                    }
                }
            },
            "hash256" => {
                let clean_value = param_value.trim_start_matches("0x");
                match H256::from_str(clean_value) {
                    Ok(hash) => Some(neo_types::ContractParameter::h256(&hash)),
                    Err(e) => {
                        log::warn!("Failed to parse Hash256 parameter '{}': {}", clean_value, e);
                        None
                    }
                }
            },
            "bytearray" | "bytes" => {
                // Assuming the value is hex encoded
                match hex::decode(param_value.trim_start_matches("0x")) {
                    Ok(bytes) => Some(neo_types::ContractParameter::byte_array(bytes)),
                    Err(e) => {
                        log::warn!("Failed to parse ByteArray parameter '{}': {}", param_value, e);
                        None
                    }
                }
            },
            // Neo X specific parameter types
            "address" => {
                // Handle Neo X address format and convert to Hash160
                log::debug!("Processing Neo X address parameter: {}", param_value);
                let clean_value = param_value.trim_start_matches("0x");
                match H160::from_str(clean_value) {
                    Ok(hash) => Some(neo_types::ContractParameter::h160(&hash)),
                    Err(e) => {
                        log::warn!("Failed to parse Neo X address parameter '{}': {}", clean_value, e);
                        None
                    }
                }
            },
            // Additional parameter types can be added as needed
            _ => {
                log::warn!("Unsupported parameter type: {}", param_type);
                None
            }
        }
    } else {
        // If no type is specified, try to infer the type
        // Default to String if we can't infer
        log::debug!("No parameter type specified, defaulting to String: {}", param_str);
        Some(neo_types::ContractParameter::string(param_str.to_string()))
    }
}

/// Implementation of the `RpcClient` trait for JSON-RPC clients.
/// 
/// This implementation provides compatibility with both Neo N3 and Neo X networks,
/// handling the differences in API responses and parameter formats between the
/// two network types. It correctly manages network identification, transaction signing,
/// and parameter serialization for both network versions.
impl neo_common::RpcClient for crate::rpc::RpcClient<crate::rpc::Http> {
    fn max_valid_until_block_increment(&self) -> u32 {
        2048 // Default value
    }
    
    /// Invokes a raw NEO VM script with the given signers.
    ///
    /// This method is particularly useful for advanced contract interactions, including
    /// DeFi operations and cross-chain bridges that might require complex script execution.
    ///
    /// # Parameters
    /// - `script`: The hex-encoded NEO VM script to execute
    /// - `signers`: Vector of signers in string format using the "script_hash:scope" syntax
    ///
    /// # Returns
    /// - `Ok(String)`: JSON string representation of the invocation result
    /// - `Err(ProviderError)`: If there was an error during script execution
    ///
    /// # Network Compatibility
    /// This method supports script execution on both Neo N3 and Neo X networks and can be used for:
    /// - DeFi contract interactions that require complex parameter passing
    /// - Cross-chain bridge operations between Neo N3 and Neo X
    /// - Token transfers and swaps that involve multiple contracts
    fn invoke_script<'a>(&'a self, script: String, signers: Vec<String>) 
        -> Box<dyn std::future::Future<Output = Result<String, neo_common::provider_error::ProviderError>> + Send + 'a> {
        Box::new(async move {
            log::debug!("Invoking script with {} signers", signers.len());
            
            // Verify script format
            if script.is_empty() {
                log::error!("Empty script provided");
                return Err(neo_common::provider_error::ProviderError::RpcError(
                    "Empty script provided".to_string()
                ));
            }
            
            // Convert string signers to proper Signer objects
            let mut signer_objects = Vec::new();
            for (i, s) in signers.iter().enumerate() {
                match parse_signer(s) {
                    Some(signer) => signer_objects.push(signer),
                    None => {
                        log::warn!("Could not parse signer at position {}: {}", i, s);
                        // Continue with the signers we could parse
                    }
                }
            }
            
            log::debug!("Successfully parsed {} out of {} signers", signer_objects.len(), signers.len());
            
            // Forward to the actual implementation
            match api_trait::APITrait::invoke_script(self, script.clone(), signer_objects).await {
                Ok(result) => {
                    // Check if execution was successful
                    let state_str = format!("{:?}", result.state);
                    if state_str != "HALT" {
                        log::warn!("Script execution completed with non-HALT state: {:?}", result.state);
                    } else {
                        log::debug!("Script execution completed successfully with HALT state");
                    }
                    
                    // Convert InvocationResult to String
                    match serde_json::to_string(&result) {
                        Ok(json) => {
                            log::debug!("Script invocation successful");
                            Ok(json)
                        },
                        Err(e) => {
                            log::error!("Failed to serialize invocation result: {}", e);
                            Err(neo_common::provider_error::ProviderError::RpcError(
                                format!("Failed to serialize invocation result: {}", e)
                            ))
                        }
                    }
                },
                Err(e) => {
                    log::error!("Script invocation failed: {}", e);
                    Err(neo_common::provider_error::ProviderError::RpcError(
                        format!("Script invocation failed: {}", e)
                    ))
                }
            }
        })
    }
    
    fn calculate_network_fee<'a>(&'a self, tx_hex: String) 
        -> Box<dyn std::future::Future<Output = Result<u64, neo_common::provider_error::ProviderError>> + Send + 'a> {
        Box::new(async move {
            // Forward to the actual implementation
            let result = api_trait::APITrait::calculate_network_fee(self, tx_hex)
                .await
                .map_err(|e| neo_common::provider_error::ProviderError::RpcError(e.to_string()))?;
            
            // Extract fee value from NeoNetworkFee
            Ok(result.network_fee.parse::<u64>().unwrap_or(0))
        })
    }
    
    fn get_block_count<'a>(&'a self) 
        -> Box<dyn std::future::Future<Output = Result<u32, neo_common::provider_error::ProviderError>> + Send + 'a> {
        Box::new(async move {
            // Forward to the actual implementation
            api_trait::APITrait::get_block_count(self)
                .await
                .map_err(|e| neo_common::provider_error::ProviderError::RpcError(e.to_string()))
        })
    }
    
    /// Invokes a contract function with the given parameters and signers.
    ///
    /// This method provides a standardized way to invoke smart contract methods across
    /// both Neo N3 and Neo X networks. It handles the conversion of parameters and signers
    /// into the appropriate format for the underlying network.
    ///
    /// # Parameters
    /// - `script_hash`: The contract script hash in string format (e.g., "0x6725cfc0a0b96731b8f454d0a6b30f6f6a36e13f")
    /// - `operation`: The name of the operation/method to invoke
    /// - `params`: Vector of contract parameters in string format using the "type:value" syntax
    /// - `signers`: Vector of signers in string format using the "script_hash:scope" syntax
    ///
    /// # Returns
    /// - `Ok(String)`: JSON string representation of the invocation result
    /// - `Err(ProviderError)`: If there was an error during invocation
    ///
    /// # Network Compatibility
    /// This method works with both Neo N3 and Neo X contracts, automatically handling
    /// any differences in parameter formats or invocation requirements between the networks.
    fn invoke_function<'a>(&'a self, script_hash: String, operation: String, params: Vec<String>, signers: Vec<String>) 
        -> Box<dyn std::future::Future<Output = Result<String, neo_common::provider_error::ProviderError>> + Send + 'a> {
        Box::new(async move {
            // Convert script_hash from String to H160
            let script_hash_h160 = match H160::from_str(&script_hash) {
                Ok(h) => h,
                Err(e) => {
                    log::error!("Invalid contract script hash {}: {}", script_hash, e);
                    return Err(neo_common::provider_error::ProviderError::InvalidAddress);
                }
            };
            
            log::debug!("Invoking {}::{} on contract {}", script_hash, operation, script_hash_h160);
            
            // Convert params from Vec<String> to Vec<ContractParameter>
            let mut contract_params = Vec::new();
            for (i, p) in params.iter().enumerate() {
                match parse_contract_parameter(p) {
                    Some(param) => contract_params.push(param),
                    None => {
                        log::warn!("Could not parse parameter at position {}: {}", i, p);
                        // Continue with the parameters we could parse
                    }
                }
            }
            
            log::debug!("Parsed {} parameters for contract invocation", contract_params.len());
            
            // Convert signers from Vec<String> to Vec<Signer>
            let mut signer_objects = Vec::new();
            for (i, s) in signers.iter().enumerate() {
                match parse_signer(s) {
                    Some(signer) => signer_objects.push(signer),
                    None => {
                        log::warn!("Could not parse signer at position {}: {}", i, s);
                        // Continue with the signers we could parse
                    }
                }
            }
            
            let signers_option = if signer_objects.is_empty() {
                log::debug!("No valid signers provided, proceeding without signers");
                None
            } else {
                log::debug!("Using {} signers for contract invocation", signer_objects.len());
                Some(signer_objects)
            };
            
            // Forward to the actual implementation
            match api_trait::APITrait::invoke_function(
                self, 
                &script_hash_h160, 
                operation.clone(), 
                contract_params,
                signers_option
            ).await {
                Ok(result) => {
                    // Check if execution was successful
                    let state_str = format!("{:?}", result.state);
                    if state_str != "HALT" {
                        log::warn!("Function execution completed with non-HALT state: {:?}", result.state);
                    } else {
                        log::debug!("Function execution completed successfully with HALT state");
                    }
                    
                    // Convert InvocationResult to String
                    match serde_json::to_string(&result) {
                        Ok(json) => {
                            log::debug!("Successfully invoked {}::{}", script_hash, operation);
                            Ok(json)
                        },
                        Err(e) => {
                            log::error!("Failed to serialize invocation result: {}", e);
                            Err(neo_common::provider_error::ProviderError::RpcError(
                                format!("Failed to serialize invocation result: {}", e)
                            ))
                        }
                    }
                },
                Err(e) => {
                    log::error!("Failed to invoke {}::{}: {}", script_hash, operation, e);
                    Err(neo_common::provider_error::ProviderError::RpcError(e.to_string()))
                }
            }
        })
    }
    
    fn get_committee<'a>(&'a self) 
        -> Box<dyn std::future::Future<Output = Result<Vec<String>, neo_common::provider_error::ProviderError>> + Send + 'a> {
        Box::new(async move {
            // Forward to the actual implementation
            api_trait::APITrait::get_committee(self)
                .await
                .map_err(|e| neo_common::provider_error::ProviderError::RpcError(e.to_string()))
        })
    }
    
    /// Returns the network magic number/protocol ID of the connected Neo network.
    ///
    /// This method is crucial for identifying whether the node is connected to:
    /// - Neo N3 Mainnet
    /// - Neo N3 Testnet
    /// - Neo X Mainnet
    /// - Neo X Testnet
    ///
    /// The network ID is essential for constructing valid transactions and properly
    /// interacting with the correct network-specific contracts and features.
    /// 
    /// # Returns
    /// - `Ok(u32)`: The network magic number as a u32 value
    /// - `Err(ProviderError)`: If there was an error getting the network information
    fn network<'a>(&'a self)
        -> Box<dyn std::future::Future<Output = Result<u32, neo_common::provider_error::ProviderError>> + Send + 'a> {
        Box::new(async move {
            // Forward to the actual implementation
            match api_trait::APITrait::get_version(self).await {
                Ok(version) => {
                    // Check if protocol is available
                    if let Some(protocol) = version.protocol {
                        Ok(protocol.network)
                    } else {
                        log::warn!("Protocol information missing in version response. Defaulting to 0.");
                        Ok(0)
                    }
                },
                Err(e) => {
                    log::error!("Failed to get network version: {}", e);
                    Err(neo_common::provider_error::ProviderError::RpcError(
                        format!("Failed to determine network type: {}", e)
                    ))
                }
            }
        })
    }
        
    fn get_block_hash<'a>(&'a self, block_index: u32)
        -> Box<dyn std::future::Future<Output = Result<String, neo_common::provider_error::ProviderError>> + Send + 'a> {
        Box::new(async move {
            // Forward to the actual implementation
            api_trait::APITrait::get_block_hash(self, block_index)
                .await
                .map(|hash| hash.to_string())
                .map_err(|e| neo_common::provider_error::ProviderError::RpcError(e.to_string()))
        })
    }
        
    fn get_block<'a>(&'a self, block_hash: String, full_transactions: bool)
        -> Box<dyn std::future::Future<Output = Result<String, neo_common::provider_error::ProviderError>> + Send + 'a> {
        Box::new(async move {
            // Convert block hash to H256
            let block_hash_h256 = match primitive_types::H256::from_str(&block_hash) {
                Ok(h) => h,
                Err(_) => return Err(neo_common::provider_error::ProviderError::RpcError("Invalid block hash".to_string())),
            };
            
            // Forward to the actual implementation
            let result = api_trait::APITrait::get_block(self, block_hash_h256, full_transactions)
                .await
                .map_err(|e| neo_common::provider_error::ProviderError::RpcError(e.to_string()))?;
                
            // Convert Block to String
            Ok(serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string()))
        })
    }
        
    /// Sends a serialized raw transaction to the network.
    ///
    /// This method is crucial for broadcasting signed transactions to the blockchain,
    /// including token transfers, DeFi operations, and cross-chain transactions.
    ///
    /// # Parameters
    /// - `hex`: The hex-encoded serialized transaction
    ///
    /// # Returns
    /// - `Ok(String)`: JSON string containing the transaction hash and validation information
    /// - `Err(ProviderError)`: If there was an error during transaction submission
    ///
    /// # Network Compatibility
    /// This method supports transactions on both Neo N3 and Neo X networks and is essential for:
    /// - Token transfers between wallets
    /// - DeFi contract interactions (swaps, liquidity provision, staking)
    /// - Interacting with bridge contracts for cross-chain asset transfers
    /// - Any operation that requires modifying blockchain state
    fn send_raw_transaction<'a>(&'a self, hex: String)
        -> Box<dyn std::future::Future<Output = Result<String, neo_common::provider_error::ProviderError>> + Send + 'a> {
        Box::new(async move {
            // Validate transaction hex format
            if hex.is_empty() {
                log::error!("Empty transaction hex provided");
                return Err(neo_common::provider_error::ProviderError::RpcError(
                    "Empty transaction hex provided".to_string()
                ));
            }
            
            // Remove 0x prefix if present (for compatibility with different standards)
            let clean_hex = hex.trim_start_matches("0x").to_string();
            
            // Basic validation of the hex string
            if clean_hex.len() % 2 != 0 {
                log::error!("Invalid transaction hex length - must be even");
                return Err(neo_common::provider_error::ProviderError::RpcError(
                    "Invalid transaction hex length - must be even".to_string()
                ));
            }
            
            // Check if the hex string is valid
            if !clean_hex.chars().all(|c| c.is_digit(16)) {
                log::error!("Invalid transaction hex format - contains non-hex characters");
                return Err(neo_common::provider_error::ProviderError::RpcError(
                    "Invalid transaction hex format - contains non-hex characters".to_string()
                ));
            }
            
            log::debug!("Sending raw transaction with hex length: {}", clean_hex.len());
            
            // Forward to the actual implementation
            match api_trait::APITrait::send_raw_transaction(self, clean_hex).await {
                Ok(tx_result) => {
                    // Convert result to JSON string
                    match serde_json::to_string(&tx_result) {
                        Ok(json) => {
                            log::info!("Transaction successfully sent");
                            Ok(json)
                        },
                        Err(e) => {
                            log::error!("Failed to serialize transaction result: {}", e);
                            Err(neo_common::provider_error::ProviderError::RpcError(
                                format!("Failed to serialize transaction result: {}", e)
                            ))
                        }
                    }
                },
                Err(e) => {
                    log::error!("Failed to send transaction: {}", e);
                    Err(neo_common::provider_error::ProviderError::RpcError(e.to_string()))
                }
            }
        })
    }
        
    /// Retrieves the application execution log for a transaction.
    ///
    /// This method is essential for monitoring smart contract events, analyzing DeFi operations,
    /// and debugging transaction execution. It provides detailed information about the execution
    /// of smart contracts triggered by a transaction.
    ///
    /// # Parameters
    /// - `tx_hash`: The transaction hash as a hex string
    ///
    /// # Returns
    /// - `Ok(String)`: JSON string containing the application logs, including notifications and execution results
    /// - `Err(ProviderError)`: If there was an error retrieving the logs
    ///
    /// # Network Compatibility
    /// This method works identically on both Neo N3 and Neo X networks and is particularly useful for:
    /// - Tracking DeFi operation results (swaps, liquidity changes, staking rewards)
    /// - Monitoring bridge contract events for cross-chain transfers
    /// - Debugging failed transactions by examining execution traces
    /// - Analyzing token transfer notifications from NEP-17 and NEP-11 contracts
    fn get_application_log<'a>(&'a self, tx_hash: String)
        -> Box<dyn std::future::Future<Output = Result<String, neo_common::provider_error::ProviderError>> + Send + 'a> {
        Box::new(async move {
            // Convert tx_hash to H256
            let tx_hash_h256 = match primitive_types::H256::from_str(&tx_hash) {
                Ok(h) => h,
                Err(_) => return Err(neo_common::provider_error::ProviderError::RpcError("Invalid transaction hash".to_string())),
            };
            
            // Forward to the actual implementation
            let result = api_trait::APITrait::get_application_log(self, tx_hash_h256)
                .await
                .map_err(|e| neo_common::provider_error::ProviderError::RpcError(e.to_string()))?;
                
            // Convert ApplicationLog to String
            Ok(serde_json::to_string(&result).unwrap_or_else(|_| "{}".to_string()))
        })
    }
}
