use clap::{Args, Subcommand};
use neo3::prelude::*;
use crate::utils::error::{CliError, CliResult};
use crate::utils::{print_success, print_error, print_info, prompt_password};
use std::path::PathBuf;
use std::fs;
use std::str::FromStr;
use neo3::types::{ContractManifest, NefFile, Signer, SignerScope};
use neo3::transaction::TransactionBuilder;
use neo3::script::ScriptBuilder;

#[derive(Args, Debug)]
pub struct ContractArgs {
    #[command(subcommand)]
    pub command: ContractCommands,
}

#[derive(Subcommand, Debug)]
pub enum ContractCommands {
    /// Deploy a smart contract
    Deploy {
        /// Path to the contract file (.nef)
        #[arg(short, long)]
        nef: PathBuf,
        
        /// Path to the contract manifest file (.json)
        #[arg(short, long)]
        manifest: PathBuf,
        
        /// Account to pay for deployment
        #[arg(short, long)]
        account: Option<String>,
    },
    
    /// Update an existing contract
    Update {
        /// Contract script hash
        #[arg(short, long)]
        script_hash: String,
        
        /// Path to the new contract file (.nef)
        #[arg(short, long)]
        nef: PathBuf,
        
        /// Path to the new contract manifest file (.json)
        #[arg(short, long)]
        manifest: PathBuf,
        
        /// Account to pay for update
        #[arg(short, long)]
        account: Option<String>,
    },
    
    /// Invoke a contract method
    Invoke {
        /// Contract script hash
        #[arg(short, long)]
        script_hash: String,
        
        /// Method name
        #[arg(short, long)]
        method: String,
        
        /// Method parameters as JSON array
        #[arg(short, long)]
        params: Option<String>,
        
        /// Account to pay for invocation
        #[arg(short, long)]
        account: Option<String>,
        
        /// Whether to just test the invocation without submitting to the blockchain
        #[arg(short, long, default_value = "false")]
        test_invoke: bool,
    },
    
    /// List native contracts
    ListNativeContracts,
}

/// CLI state is defined in wallet.rs

pub async fn handle_contract_command(args: ContractArgs, state: &mut crate::commands::wallet::CliState) -> CliResult<()> {
    match args.command {
        ContractCommands::Deploy { nef, manifest, account } => deploy_contract(nef, manifest, account, state).await,
        ContractCommands::Update { script_hash, nef, manifest, account } => update_contract(script_hash, nef, manifest, account, state).await,
        ContractCommands::Invoke { script_hash, method, params, account, test_invoke } => invoke_contract(script_hash, method, params, account, test_invoke, state).await,
        ContractCommands::ListNativeContracts => list_native_contracts(state).await,
    }
}

async fn deploy_contract(nef_path: PathBuf, manifest_path: PathBuf, account: Option<String>, state: &mut crate::commands::wallet::CliState) -> CliResult<()> {
    if state.wallet.is_none() {
        print_error("No wallet is currently open");
        return Err(CliError::Wallet("No wallet is currently open".to_string()));
    }
    
    if state.rpc_client.is_none() {
        print_error("No RPC client is connected. Please connect to a node first.");
        return Err(CliError::Network("No RPC client is connected".to_string()));
    }
    
    // Check if files exist
    if !nef_path.exists() {
        print_error(&format!("NEF file not found: {:?}", nef_path));
        return Err(CliError::Input(format!("NEF file not found: {:?}", nef_path)));
    }
    
    if !manifest_path.exists() {
        print_error(&format!("Manifest file not found: {:?}", manifest_path));
        return Err(CliError::Input(format!("Manifest file not found: {:?}", manifest_path)));
    }
    
    print_info("Deploying smart contract...");
    
    // Read NEF and manifest files
    let nef_bytes = std::fs::read(&nef_path)
        .map_err(|e| CliError::Io(e))?;
    let manifest_json = std::fs::read_to_string(&manifest_path)
        .map_err(|e| CliError::Io(e))?;
    
    // Parse NEF and manifest
    let nef = NefFile::deserialize(&nef_bytes)
        .map_err(|e| CliError::Input(format!("Failed to parse NEF file: {}", e)))?;
    let manifest: ContractManifest = serde_json::from_str(&manifest_json)
        .map_err(|e| CliError::Input(format!("Failed to parse manifest file: {}", e)))?;
    
    // Get account to pay for deployment
    let wallet = state.wallet.as_ref().unwrap();
    let account_address = match account {
        Some(addr) => addr,
        None => {
            // If no account specified, use the first account in the wallet
            let accounts = wallet.get_accounts();
            if accounts.is_empty() {
                print_error("No accounts in wallet");
                return Err(CliError::Wallet("No accounts in wallet".to_string()));
            }
            accounts[0].address().to_string()
        }
    };
    
    // Find account in wallet
    let account_obj = wallet.get_accounts().iter()
        .find(|a| a.address() == account_address)
        .ok_or_else(|| CliError::Wallet(format!("Account not found: {}", account_address)))?
        .clone();
    
    // Get password for signing
    let password = prompt_password("Enter wallet password")?;
    
    // Create and sign deployment transaction
    let rpc_client = state.rpc_client.as_ref().unwrap();
    
    // Get system fee
    let params = vec![
        neo3::types::ContractParameter::ByteArray(nef_bytes),
        neo3::types::ContractParameter::String(manifest_json)
    ];
    
    let invocation_result = rpc_client.invoke_function(
        &H160::from_hex_str("0xfffdc93764dbaddd97c48f252a53ea4643faa3fd").unwrap(), // Management contract
        "deploy",
        params,
        Some(vec![account_obj.script_hash()]),
    ).await.map_err(|e| CliError::Network(format!("Failed to test invoke deploy: {}", e)))?;
    
    let system_fee = invocation_result.gas_consumed;
    print_info(&format!("Estimated system fee: {} GAS", system_fee));
    
    // Get current block count and calculate validUntilBlock
    let block_count = rpc_client.get_block_count().await
        .map_err(|e| CliError::Network(format!("Failed to get block count: {}", e)))?;
    let valid_until_block = block_count + 100; // Valid for ~16 minutes (assuming 10s blocks)
    
    // Build transaction
    let signers = vec![Signer {
        account: account_obj.script_hash(),
        scopes: SignerScope::CalledByEntry,
        allowed_contracts: vec![],
        allowed_groups: vec![],
    }];
    
    let mut tx_builder = TransactionBuilder::new()
        .version(0)
        .nonce((rand::random::<u32>() % 1000000) as u32)
        .valid_until_block(valid_until_block)
        .signers(signers)
        .system_fee(system_fee);
    
    // Add deploy script
    tx_builder = tx_builder.script(nef.to_bytes())?;
    
    // Calculate network fee
    let network_fee = rpc_client.calculate_network_fee(&tx_builder).await
        .map_err(|e| CliError::Network(format!("Failed to calculate network fee: {}", e)))?;
    
    tx_builder = tx_builder.network_fee(network_fee);
    
    // Sign transaction
    let signed_tx = account_obj.sign_tx(tx_builder, &password)
        .map_err(|e| CliError::Wallet(format!("Failed to sign transaction: {}", e)))?;
    
    // Send transaction
    let result = rpc_client.send_raw_transaction(&signed_tx).await
        .map_err(|e| CliError::Network(format!("Failed to send transaction: {}", e)))?;
    
    // Calculate contract hash
    let contract_hash = crate::utils::calculate_contract_hash(&account_obj.script_hash(), nef.checksum, manifest.name.as_bytes());
    
    print_success("Contract deployment transaction sent successfully");
    println!("Transaction hash: {}", result.hash);
    println!("Contract hash: {}", contract_hash);
    
    Ok(())
}

async fn update_contract(script_hash: String, nef_path: PathBuf, manifest_path: PathBuf, account: Option<String>, state: &mut crate::commands::wallet::CliState) -> CliResult<()> {
    if state.wallet.is_none() {
        print_error("No wallet is currently open");
        return Err(CliError::Wallet("No wallet is currently open".to_string()));
    }
    
    if state.rpc_client.is_none() {
        print_error("No RPC client is connected. Please connect to a node first.");
        return Err(CliError::Network("No RPC client is connected".to_string()));
    }
    
    // Check if files exist
    if !nef_path.exists() {
        print_error(&format!("NEF file not found: {:?}", nef_path));
        return Err(CliError::Input(format!("NEF file not found: {:?}", nef_path)));
    }
    
    if !manifest_path.exists() {
        print_error(&format!("Manifest file not found: {:?}", manifest_path));
        return Err(CliError::Input(format!("Manifest file not found: {:?}", manifest_path)));
    }
    
    print_info(&format!("Updating contract: {}", script_hash));
    
    // Read NEF and manifest files
    let nef_bytes = std::fs::read(&nef_path)
        .map_err(|e| CliError::Io(e))?;
    let manifest_json = std::fs::read_to_string(&manifest_path)
        .map_err(|e| CliError::Io(e))?;
    
    // Parse NEF and manifest
    let nef = NefFile::deserialize(&nef_bytes)
        .map_err(|e| CliError::Input(format!("Failed to parse NEF file: {}", e)))?;
    let manifest: ContractManifest = serde_json::from_str(&manifest_json)
        .map_err(|e| CliError::Input(format!("Failed to parse manifest file: {}", e)))?;
    
    // Get account to pay for update
    let wallet = state.wallet.as_ref().unwrap();
    let account_address = match account {
        Some(addr) => addr,
        None => {
            // If no account specified, use the first account in the wallet
            let accounts = wallet.get_accounts();
            if accounts.is_empty() {
                print_error("No accounts in wallet");
                return Err(CliError::Wallet("No accounts in wallet".to_string()));
            }
            accounts[0].address().to_string()
        }
    };
    
    // Find account in wallet
    let account_obj = wallet.get_accounts().iter()
        .find(|a| a.address() == account_address)
        .ok_or_else(|| CliError::Wallet(format!("Account not found: {}", account_address)))?
        .clone();
    
    // Get password for signing
    let password = prompt_password("Enter wallet password")?;
    
    // Parse contract hash
    let contract_hash = H160::from_str(&script_hash)
        .map_err(|_| CliError::Input("Invalid script hash format".to_string()))?;
    
    // Create and sign update transaction
    let rpc_client = state.rpc_client.as_ref().unwrap();
    
    // Get system fee
    let params = vec![
        neo3::types::ContractParameter::Hash160(contract_hash.clone()),
        neo3::types::ContractParameter::ByteArray(nef_bytes),
        neo3::types::ContractParameter::String(manifest_json)
    ];
    
    let invocation_result = rpc_client.invoke_function(
        &H160::from_hex_str("0xfffdc93764dbaddd97c48f252a53ea4643faa3fd").unwrap(), // Management contract
        "update",
        params,
        Some(vec![account_obj.script_hash()]),
    ).await.map_err(|e| CliError::Network(format!("Failed to test invoke update: {}", e)))?;
    
    let system_fee = invocation_result.gas_consumed;
    print_info(&format!("Estimated system fee: {} GAS", system_fee));
    
    // Get current block count and calculate validUntilBlock
    let block_count = rpc_client.get_block_count().await
        .map_err(|e| CliError::Network(format!("Failed to get block count: {}", e)))?;
    let valid_until_block = block_count + 100; // Valid for ~16 minutes (assuming 10s blocks)
    
    // Build transaction
    let signers = vec![Signer {
        account: account_obj.script_hash(),
        scopes: SignerScope::CalledByEntry,
        allowed_contracts: vec![],
        allowed_groups: vec![],
    }];
    
    let mut tx_builder = TransactionBuilder::new()
        .version(0)
        .nonce((rand::random::<u32>() % 1000000) as u32)
        .valid_until_block(valid_until_block)
        .signers(signers)
        .system_fee(system_fee);
    
    // Add update script
    let script = ScriptBuilder::new()
        .contract_call(
            H160::from_hex_str("0xfffdc93764dbaddd97c48f252a53ea4643faa3fd").unwrap(),
            "update",
            vec![
                neo3::types::ContractParameter::Hash160(contract_hash),
                neo3::types::ContractParameter::ByteArray(nef_bytes),
                neo3::types::ContractParameter::String(manifest_json)
            ]
        )
        .to_array();
    
    tx_builder = tx_builder.script(script);
    
    // Calculate network fee
    let network_fee = rpc_client.calculate_network_fee(&tx_builder).await
        .map_err(|e| CliError::Network(format!("Failed to calculate network fee: {}", e)))?;
    
    tx_builder = tx_builder.network_fee(network_fee);
    
    // Sign transaction
    let signed_tx = account_obj.sign_tx(tx_builder, &password)
        .map_err(|e| CliError::Wallet(format!("Failed to sign transaction: {}", e)))?;
    
    // Send transaction
    let result = rpc_client.send_raw_transaction(&signed_tx).await
        .map_err(|e| CliError::Network(format!("Failed to send transaction: {}", e)))?;
    
    print_success("Contract updated successfully");
    println!("Transaction hash: {}", result.hash);
    
    Ok(())
}

async fn invoke_contract(script_hash: String, method: String, params: Option<String>, account: Option<String>, test_invoke: bool, state: &mut crate::commands::wallet::CliState) -> CliResult<()> {
    if state.rpc_client.is_none() {
        print_error("No RPC client is connected. Please connect to a node first.");
        return Err(CliError::Network("No RPC client is connected".to_string()));
    }
    
    // Parse parameters if provided
    let parameters = match params {
        Some(p) => {
            let params_json: Vec<serde_json::Value> = serde_json::from_str(&p)
                .map_err(|e| CliError::Input(format!("Invalid JSON parameters: {}", e)))?;
            
            // Convert JSON parameters to ContractParameter
            params_json.into_iter()
                .map(|v| contract_parameter_from_json(v))
                .collect::<Result<Vec<_>, _>>()?
        },
        None => Vec::new(),
    };
    
    // Convert script hash
    let contract_hash = H160::from_str(&script_hash)
        .map_err(|_| CliError::Input("Invalid script hash format".to_string()))?;
    
    let rpc_client = state.rpc_client.as_ref().unwrap();
    
    if test_invoke {
        print_info(&format!("Test invoking method '{}' on contract {}", method, script_hash));
        
        // Test invoke
        let result = rpc_client.invoke_function(
            &contract_hash,
            &method,
            parameters,
            None
        ).await.map_err(|e| CliError::Network(format!("Failed to invoke function: {}", e)))?;
        
        // Display result
        println!("Invocation result:");
        println!("  State: {}", result.state);
        println!("  Gas consumed: {}", result.gas_consumed);
        println!("  Stack:");
        for (i, item) in result.stack.iter().enumerate() {
            println!("    {}: {}", i, item);
        }
    } else {
        // Real invocation
        if state.wallet.is_none() {
            print_error("No wallet is currently open");
            return Err(CliError::Wallet("No wallet is currently open".to_string()));
        }
        
        print_info(&format!("Invoking method '{}' on contract {}", method, script_hash));
        
        // Get account to pay for invocation
        let wallet = state.wallet.as_ref().unwrap();
        let account_address = match account {
            Some(addr) => addr,
            None => {
                // If no account specified, use the first account in the wallet
                let accounts = wallet.get_accounts();
                if accounts.is_empty() {
                    print_error("No accounts in wallet");
                    return Err(CliError::Wallet("No accounts in wallet".to_string()));
                }
                accounts[0].address().to_string()
            }
        };
        
        // Find account in wallet
        let account_obj = wallet.get_accounts().iter()
            .find(|a| a.address() == account_address)
            .ok_or_else(|| CliError::Wallet(format!("Account not found: {}", account_address)))?
            .clone();
        
        // Get password for signing
        let password = prompt_password("Enter wallet password")?;
        
        // Get system fee
        let invocation_result = rpc_client.invoke_function(
            &contract_hash,
            &method,
            parameters.clone(),
            Some(vec![account_obj.script_hash()]),
        ).await.map_err(|e| CliError::Network(format!("Failed to test invoke: {}", e)))?;
        
        let system_fee = invocation_result.gas_consumed;
        print_info(&format!("Estimated system fee: {} GAS", system_fee));
        
        // Get current block count and calculate validUntilBlock
        let block_count = rpc_client.get_block_count().await
            .map_err(|e| CliError::Network(format!("Failed to get block count: {}", e)))?;
        let valid_until_block = block_count + 100; // Valid for ~16 minutes (assuming 10s blocks)
        
        // Build transaction
        let signers = vec![Signer {
            account: account_obj.script_hash(),
            scopes: SignerScope::CalledByEntry,
            allowed_contracts: vec![],
            allowed_groups: vec![],
        }];
        
        let mut tx_builder = TransactionBuilder::new()
            .version(0)
            .nonce((rand::random::<u32>() % 1000000) as u32)
            .valid_until_block(valid_until_block)
            .signers(signers)
            .system_fee(system_fee);
        
        // Build script
        let script = ScriptBuilder::new()
            .contract_call(contract_hash, &method, parameters)
            .to_array();
        
        tx_builder = tx_builder.script(script);
        
        // Calculate network fee
        let network_fee = rpc_client.calculate_network_fee(&tx_builder).await
            .map_err(|e| CliError::Network(format!("Failed to calculate network fee: {}", e)))?;
        
        tx_builder = tx_builder.network_fee(network_fee);
        
        // Sign transaction
        let signed_tx = account_obj.sign_tx(tx_builder, &password)
            .map_err(|e| CliError::Wallet(format!("Failed to sign transaction: {}", e)))?;
        
        // Send transaction
        let result = rpc_client.send_raw_transaction(&signed_tx).await
            .map_err(|e| CliError::Network(format!("Failed to send transaction: {}", e)))?;
        
        print_success("Contract method invoked successfully");
        println!("Transaction hash: {}", result.hash);
    }
    
    Ok(())
}

// Helper to convert JSON to ContractParameter
fn contract_parameter_from_json(value: serde_json::Value) -> Result<neo3::types::ContractParameter, CliError> {
    match value {
        serde_json::Value::Null => Ok(neo3::types::ContractParameter::Any),
        serde_json::Value::Bool(b) => Ok(neo3::types::ContractParameter::Boolean(b)),
        serde_json::Value::Number(n) => {
            if n.is_i64() {
                Ok(neo3::types::ContractParameter::Integer(n.as_i64().unwrap().into()))
            } else if n.is_f64() {
                Ok(neo3::types::ContractParameter::String(n.to_string()))
            } else {
                Err(CliError::Input("Invalid number type".to_string()))
            }
        },
        serde_json::Value::String(s) => {
            // Check if it's a hex string (for ByteArray)
            if s.starts_with("0x") {
                let hex_str = &s[2..];
                match hex::decode(hex_str) {
                    Ok(bytes) => Ok(neo3::types::ContractParameter::ByteArray(bytes)),
                    Err(_) => Ok(neo3::types::ContractParameter::String(s))
                }
            } else if s.starts_with("@") { // Special format for Hash160
                let hash_str = &s[1..];
                match H160::from_str(hash_str) {
                    Ok(hash) => Ok(neo3::types::ContractParameter::Hash160(hash)),
                    Err(_) => Ok(neo3::types::ContractParameter::String(s))
                }
            } else {
                Ok(neo3::types::ContractParameter::String(s))
            }
        },
        serde_json::Value::Array(arr) => {
            let mut params = Vec::new();
            for item in arr {
                params.push(contract_parameter_from_json(item)?);
            }
            Ok(neo3::types::ContractParameter::Array(params))
        },
        serde_json::Value::Object(_) => {
            Err(CliError::Input("Object parameters not supported".to_string()))
        }
    }
}

async fn list_native_contracts(state: &mut crate::commands::wallet::CliState) -> CliResult<()> {
    if state.rpc_client.is_none() {
        print_error("No RPC client is connected. Please connect to a node first.");
        return Err(CliError::Network("No RPC client is connected".to_string()));
    }
    
    print_info("Native contracts:");
    
    // List native contracts
    let rpc_client = state.rpc_client.as_ref().unwrap();
    let native_contracts = rpc_client.get_native_contracts().await
        .map_err(|e| CliError::Network(format!("Failed to get native contracts: {}", e)))?;
    
    for contract in native_contracts {
        println!("Contract Hash: {}", contract.hash);
        println!("  Name: {}", contract.manifest.name);
        println!("  ID: {}", contract.id);
        println!("  Supported Standards: {:?}", contract.manifest.supported_standards);
        println!();
    }
    
    print_success("Native contracts retrieved successfully");
    Ok(())
}
