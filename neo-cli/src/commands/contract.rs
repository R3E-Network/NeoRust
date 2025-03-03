use std::path::PathBuf;
use std::str::FromStr;
use clap::{Args, Subcommand};
use futures::TryFutureExt;
use neo3::prelude::*;
use primitive_types::H160;
use neo3::builder::{AccountSigner, ScriptBuilder, Signer, TransactionBuilder, WitnessScope};
use neo3::neo_clients::APITrait;
use neo3::neo_types::{ContractManifest, NefFile};
use crate::errors::CliError;
use crate::commands::wallet::CliState;
use crate::utils::{print_success, print_error, print_info, prompt_password, prompt_yes_no};
use crate::commands::defi::create_h160_param;

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

pub async fn handle_contract_command(args: ContractArgs, state: &mut crate::commands::wallet::CliState) -> Result<(), CliError> {
    match args.command {
        ContractCommands::Deploy { nef, manifest, account } => deploy_contract(nef, manifest, account, state).await,
        ContractCommands::Update { script_hash, nef, manifest, account } => update_contract(script_hash, nef, manifest, account, state).await,
        ContractCommands::Invoke { script_hash, method, params, account, test_invoke } => invoke_contract(script_hash, method, params, account, test_invoke, state).await,
        ContractCommands::ListNativeContracts => list_native_contracts(state).await,
    }
}

async fn deploy_contract(nef_path: PathBuf, manifest_path: PathBuf, account: Option<String>, state: &mut crate::commands::wallet::CliState) -> Result<(), CliError> {
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
            accounts[0].get_address().to_string()
        }
    };
    
    // Find account in wallet
    let account_obj = wallet.get_accounts().iter()
        .find(|a| a.get_address() == account_address)
        .ok_or_else(|| CliError::Wallet(format!("Account not found: {}", account_address)))?
        .clone();
    
    // Get password for signing
    let password = prompt_password("Enter wallet password")?;
    
    // Create and sign deployment transaction
    let rpc_client = state.rpc_client.as_ref().unwrap();
    
    // Get system fee
    let params = vec![
        ContractParameter::byte_array(nef_bytes),
        ContractParameter::string(manifest_json)
    ];
    
    let invocation_result = rpc_client.invoke_function(
        &H160::from_hex_str("0xfffdc93764dbaddd97c48f252a53ea4643faa3fd").unwrap(), // Management contract
        "deploy",
        params,
        Some(vec![account_obj.get_script_hash()]),
    ).await.map_err(|e| CliError::Network(format!("Failed to test invoke deploy: {}", e)))?;
    
    let system_fee = invocation_result.gas_consumed;
    print_info(&format!("Estimated system fee: {} GAS", system_fee));
    
    // Get current block count and calculate validUntilBlock
    let block_count = rpc_client.get_block_count().await
        .map_err(|e| CliError::Network(format!("Failed to get block count: {}", e)))?;
    let valid_until_block = block_count + 100; // Valid for ~16 minutes (assuming 10s blocks)
    
    // Build transaction
    let signer = AccountSigner::called_by_entry(account_obj).map_err(|e| CliError::TransactionBuilder(e.to_string()))?;
    let signers = vec![Signer::AccountSigner(signer)];
    
    let mut tx_builder = TransactionBuilder::new();
    
    tx_builder = tx_builder
        .version(0)
        .nonce((rand::random::<u32>() % 1000000) as u32)
        .map_err(|e| CliError::TransactionBuilder(e.to_string()))?
        .valid_until_block(valid_until_block)
        .map_err(|e| CliError::TransactionBuilder(e.to_string()))?
        .signers(signers)
        .map_err(|e| CliError::TransactionBuilder(e.to_string()))?;
    
    // Add deploy script
    tx_builder = tx_builder.script(nef.get_script())
        .map_err(|e| CliError::TransactionBuilder(e.to_string()))?;
    
    // Calculate network fee
    let network_fee = rpc_client.calculate_network_fee(&tx_builder).await
        .map_err(|e| CliError::Network(format!("Failed to calculate network fee: {}", e)))?;
    
    tx_builder = tx_builder.network_fee(network_fee);
    
    // Sign transaction
    let signed_tx = sign_transaction(account_obj, tx_builder, &password).await?;
    
    // Create a JSON structure directly that matches the expected format
    let transaction_json = serde_json::json!({
        "script": base64::encode(signed_tx.script().clone()),
        "signers": signed_tx.signers().iter().map(|signer| {
            serde_json::json!({
                "account": signer.account().to_string(),
                "scopes": signer.scope().to_string()
            })
        }).collect::<Vec<_>>(),
        "attributes": [],
        "witnesses": signed_tx.witnesses().iter().map(|witness| {
            serde_json::json!({
                "invocation": base64::encode(witness.invocation_script().clone()),
                "verification": base64::encode(witness.verification_script().clone())
            })
        }).collect::<Vec<_>>()
    });
    
    // Convert the JSON to a string for sending
    let tx_json = serde_json::to_string(&transaction_json)
        .map_err(|e| CliError::Network(format!("Failed to serialize transaction: {}", e)))?;
    
    // Send transaction
    let result = rpc_client.send_raw_transaction(tx_json).await
        .map_err(|e| CliError::Network(format!("Failed to send transaction: {}", e)))?;
    
    print_success("Contract deployment transaction sent successfully");
    println!("Transaction hash: {}", result.hash);
    println!("Contract hash: {}", crate::utils::calculate_contract_hash(
        &account_obj.get_script_hash(), 
        nef.get_checksum(), 
        manifest.name.as_ref().map(|s| s.as_bytes()).unwrap_or_default()
    ));
    
    Ok(())
}

async fn update_contract(script_hash: String, nef_path: PathBuf, manifest_path: PathBuf, account: Option<String>, state: &mut crate::commands::wallet::CliState) -> Result<(), CliError> {
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
            accounts[0].get_address().to_string()
        }
    };
    
    // Find account in wallet
    let account_obj = wallet.get_accounts().iter()
        .find(|a| a.get_address() == account_address)
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
        ContractParameter::h160(contract_hash.clone()),
        ContractParameter::byte_array(nef_bytes),
        ContractParameter::string(manifest_json)
    ];
    
    let invocation_result = rpc_client.invoke_function(
        &H160::from_hex_str("0xfffdc93764dbaddd97c48f252a53ea4643faa3fd").unwrap(), // Management contract
        "update",
        params,
        Some(vec![account_obj.get_script_hash()]),
    ).await.map_err(|e| CliError::Network(format!("Failed to test invoke update: {}", e)))?;
    
    let system_fee = invocation_result.gas_consumed;
    print_info(&format!("Estimated system fee: {} GAS", system_fee));
    
    // Get current block count and calculate validUntilBlock
    let block_count = rpc_client.get_block_count().await
        .map_err(|e| CliError::Network(format!("Failed to get block count: {}", e)))?;
    let valid_until_block = block_count + 100; // Valid for ~16 minutes (assuming 10s blocks)
    
    // Build transaction
    let signer = AccountSigner::called_by_entry(account_obj).map_err(|e| CliError::TransactionBuilder(e.to_string()))?;
    let signers = vec![Signer::AccountSigner(signer)];

    let mut tx_builder = TransactionBuilder::new();
    
    tx_builder = tx_builder
        .version(0)
        .nonce((rand::random::<u32>() % 1000000) as u32)
        .map_err(|e| CliError::TransactionBuilder(e.to_string()))?
        .valid_until_block(valid_until_block)
        .map_err(|e| CliError::TransactionBuilder(e.to_string()))?
        .script_hash(account_obj.get_script_hash())
        .map_err(|e| CliError::TransactionBuilder(e.to_string()))?
        .witness_scope(WitnessScope::CalledByEntry)
        .map_err(|e| CliError::TransactionBuilder(e.to_string()))?;
    
    // Add update script
    let script = ScriptBuilder::new()
        .contract_call(
            H160::from_hex_str("0xfffdc93764dbaddd97c48f252a53ea4643faa3fd").unwrap(),
            "update",
            &[
                ContractParameter::h160(contract_hash),
                ContractParameter::byte_array(nef_bytes),
                ContractParameter::string(manifest_json)
            ],
            None
        )
        .map_err(|e| CliError::Builder(e.to_string()))?
        .build();
    
    tx_builder = tx_builder.script(script)
        .map_err(|e| CliError::TransactionBuilder(e.to_string()))?;
    
    // Calculate network fee
    let network_fee = rpc_client.calculate_network_fee(&tx_builder).await
        .map_err(|e| CliError::Network(format!("Failed to calculate network fee: {}", e)))?;
    
    tx_builder = tx_builder.network_fee(network_fee);
    
    // Sign transaction
    let signed_tx = sign_transaction(account_obj, tx_builder, &password).await?;
    
    // Create a JSON structure directly that matches the expected format
    let transaction_json = serde_json::json!({
        "script": base64::encode(signed_tx.script().clone()),
        "signers": signed_tx.signers().iter().map(|signer| {
            serde_json::json!({
                "account": signer.account().to_string(),
                "scopes": signer.scope().to_string()
            })
        }).collect::<Vec<_>>(),
        "attributes": [],
        "witnesses": signed_tx.witnesses().iter().map(|witness| {
            serde_json::json!({
                "invocation": base64::encode(witness.invocation_script().clone()),
                "verification": base64::encode(witness.verification_script().clone())
            })
        }).collect::<Vec<_>>()
    });
    
    // Convert the JSON to a string for sending
    let tx_json = serde_json::to_string(&transaction_json)
        .map_err(|e| CliError::Network(format!("Failed to serialize transaction: {}", e)))?;
    
    // Send transaction
    let result = rpc_client.send_raw_transaction(tx_json).await
        .map_err(|e| CliError::Network(format!("Failed to send transaction: {}", e)))?;
    
    print_success("Contract updated successfully");
    println!("Transaction hash: {}", result.hash);
    
    Ok(())
}

async fn invoke_contract(script_hash: String, method: String, params: Option<String>, account: Option<String>, test_invoke: bool, state: &mut crate::commands::wallet::CliState) -> Result<(), CliError> {
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
        println!("  State: {:?}", result.state);
        println!("  Gas consumed: {}", result.gas_consumed);
        println!("  Stack:");
        for (i, item) in result.stack.iter().enumerate() {
            println!("    {}: {:?}", i, item);
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
                accounts[0].get_address().to_string()
            }
        };
        
        // Find account in wallet
        let account_obj = wallet.get_accounts().iter()
            .find(|a| a.get_address() == account_address)
            .ok_or_else(|| CliError::Wallet(format!("Account not found: {}", account_address)))?
            .clone();
        
        // Get password for signing
        let password = prompt_password("Enter wallet password")?;
        
        // Get system fee
        let invocation_result = rpc_client.invoke_function(
            &contract_hash,
            &method,
            parameters.clone(),
            Some(vec![account_obj.get_script_hash()]),
        ).await.map_err(|e| CliError::Network(format!("Failed to test invoke: {}", e)))?;
        
        let system_fee = invocation_result.gas_consumed;
        print_info(&format!("Estimated system fee: {} GAS", system_fee));
        
        // Get current block count and calculate validUntilBlock
        let block_count = rpc_client.get_block_count().await
            .map_err(|e| CliError::Network(format!("Failed to get block count: {}", e)))?;
        let valid_until_block = block_count + 100; // Valid for ~16 minutes (assuming 10s blocks)
        
        // Build transaction
        let signer = AccountSigner::called_by_entry(account_obj).map_err(|e| CliError::TransactionBuilder(e.to_string()))?;
        let signers = vec![Signer::AccountSigner(signer)];

        let mut tx_builder = TransactionBuilder::new();
        
        tx_builder = tx_builder
            .version(0)
            .nonce((rand::random::<u32>() % 1000000) as u32)
            .map_err(|e| CliError::TransactionBuilder(e.to_string()))?
            .valid_until_block(valid_until_block)
            .map_err(|e| CliError::TransactionBuilder(e.to_string()))?
            .script_hash(account_obj.get_script_hash())
            .map_err(|e| CliError::TransactionBuilder(e.to_string()))?
            .witness_scope(WitnessScope::CalledByEntry)
            .map_err(|e| CliError::TransactionBuilder(e.to_string()))?;
        
        // Create a script to execute the invoke
        let script = ScriptBuilder::new()
            .contract_call(
                contract_hash,
                &method,
                &parameters,
                None
            )
            .map_err(|e| CliError::Builder(e.to_string()))?
            .build();
        
        tx_builder = tx_builder.script(script)
            .map_err(|e| CliError::TransactionBuilder(e.to_string()))?;
        
        // Calculate network fee
        let network_fee = rpc_client.calculate_network_fee(&tx_builder).await
            .map_err(|e| CliError::Network(format!("Failed to calculate network fee: {}", e)))?;
        
        tx_builder = tx_builder.network_fee(network_fee);
        
        // Build and sign the transaction
        let tx = tx_builder.build()
            .await
            .map_err(|e| CliError::Transaction(format!("Failed to build transaction: {}", e)))?;
        
        let signed_tx = sign_transaction(account_obj, tx_builder, &password).await?;
        
        // Create a JSON structure directly that matches the expected format
        let transaction_json = serde_json::json!({
            "script": base64::encode(signed_tx.script().clone()),
            "signers": signed_tx.signers().iter().map(|signer| {
                serde_json::json!({
                    "account": signer.account().to_string(),
                    "scopes": signer.scope().to_string()
                })
            }).collect::<Vec<_>>(),
            "attributes": [],
            "witnesses": signed_tx.witnesses().iter().map(|witness| {
                serde_json::json!({
                    "invocation": base64::encode(witness.invocation_script().clone()),
                    "verification": base64::encode(witness.verification_script().clone())
                })
            }).collect::<Vec<_>>()
        });
        
        // Convert the JSON to a string for sending
        let tx_json = serde_json::to_string(&transaction_json)
            .map_err(|e| CliError::Network(format!("Failed to serialize transaction: {}", e)))?;
        
        // Send transaction
        let result = rpc_client.send_raw_transaction(tx_json).await
            .map_err(|e| CliError::Network(format!("Failed to send transaction: {}", e)))?;
        
        print_success("Contract method invoked successfully");
        println!("Transaction hash: {}", result.hash);
    }
    
    Ok(())
}

async fn list_native_contracts(state: &mut crate::commands::wallet::CliState) -> Result<(), CliError> {
    if state.rpc_client.is_none() {
        print_error("No RPC client is connected. Please connect to a node first.");
        return Err(CliError::Network("No RPC client is connected".to_string()));
    }
    
    print_info("Native contracts:");
    
    // List native contracts
    let rpc_client = state.rpc_client.as_ref().unwrap();
    let native_contracts = rpc_client.get_native_contracts().await
        .map_err(|e| CliError::Network(format!("Failed to get native contracts: {}", e)))?;
    
    for (i, contract) in native_contracts.iter().enumerate() {
        println!("{}. {} ({})", i + 1, contract.manifest().name.as_ref().unwrap_or(&"Unknown".to_string()), contract.hash());
        println!("  Supported Standards: {:?}", contract.manifest().supported_standards);
        println!();
    }
    
    print_success("Native contracts retrieved successfully");
    Ok(())
}

// Helper to convert JSON to ContractParameter
fn contract_parameter_from_json(value: serde_json::Value) -> Result<ContractParameter, CliError> {
    match value {
        serde_json::Value::Null => Ok(ContractParameter::any()),
        serde_json::Value::Bool(b) => Ok(ContractParameter::bool(b)),
        serde_json::Value::Number(n) => {
            if n.is_i64() {
                Ok(ContractParameter::integer(n.as_i64().unwrap().into()))
            } else if n.is_f64() {
                Ok(ContractParameter::string(n.to_string()))
            } else {
                Err(CliError::Input("Invalid number type".to_string()))
            }
        },
        serde_json::Value::String(s) => {
            // Check if it's a hex string (for ByteArray)
            if s.starts_with("0x") {
                let hex_str = &s[2..];
                match hex::decode(hex_str) {
                    Ok(bytes) => Ok(ContractParameter::byte_array(bytes)),
                    Err(_) => Ok(ContractParameter::string(s))
                }
            } else if s.starts_with("@") { // Special format for Hash160
                let hash_str = &s[1..];
                match H160::from_str(hash_str) {
                    Ok(hash) => create_h160_param(&format!("{:x}", hash)),
                    Err(_) => Ok(ContractParameter::string(s))
                }
            } else {
                Ok(ContractParameter::string(s))
            }
        },
        serde_json::Value::Array(arr) => {
            let mut params = Vec::new();
            for item in arr {
                params.push(contract_parameter_from_json(item)?);
            }
            Ok(ContractParameter::array(params))
        },
        serde_json::Value::Object(_) => {
            Err(CliError::Input("Object parameters not supported".to_string()))
        }
    }
}

// Add this helper function at the end of the file
async fn sign_transaction<'a>(account: &'a Account, mut tx_builder: TransactionBuilder<'a>, password: &'a str) -> Result<Transaction<'a>, CliError> {
    // Generate transaction with a witness scope of CalledByEntry
    
    // Build and sign the transaction
    let tx = tx_builder.build()
        .await
        .map_err(|e| CliError::Transaction(format!("Failed to build transaction: {}", e)))?;
    
    let signed_tx = tx.sign(&[account], password)
        .map_err(|e| CliError::Transaction(format!("Failed to sign transaction: {}", e)))?;
    
    Ok(signed_tx)
}
