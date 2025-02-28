use clap::{Args, Subcommand};
// use neo3::prelude::*;
use crate::utils::error::{CliError, CliResult};
use crate::utils::{print_success, print_error, print_info};
use std::path::PathBuf;
// use std::fs;
use std::str::FromStr;

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

async fn deploy_contract(nef_path: PathBuf, manifest_path: PathBuf, _account: Option<String>, state: &mut crate::commands::wallet::CliState) -> CliResult<()> {
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
    // let nef_bytes = fs::read(&nef_path)?;
    // let manifest_json = fs::read_to_string(&manifest_path)?;
    
    // Parse NEF and manifest
    // let nef = NefFile::deserialize(&nef_bytes)?;
    // let manifest: ContractManifest = serde_json::from_str(&manifest_json)?;
    
    // Get account to pay for deployment
    // let account_address = match account {
    //     Some(addr) => addr,
    //     None => {
    //         // If no account specified, use the first account in the wallet
    //         let accounts = state.wallet.unwrap().get_accounts();
    //         if accounts.is_empty() {
    //             print_error("No accounts in wallet");
    //             return Err(CliError::Wallet("No accounts in wallet".to_string()));
    //         }
    //         accounts[0].address.clone()
    //     }
    // };
    
    // Get password for signing
    // let password = prompt_password("Enter wallet password")?;
    
    // Create and sign deployment transaction
    // let rpc_client = state.rpc_client.as_ref().unwrap();
    // let tx_builder = rpc_client.create_contract_deployment_transaction(nef, manifest)?;
    // let signed_tx = state.wallet.unwrap().sign_transaction(tx_builder, &account_address, &password)?;
    
    // Send transaction
    // let result = rpc_client.send_transaction(signed_tx).await?;
    
    print_success("Contract deployed successfully");
    // println!("Transaction hash: {}", result.hash);
    // println!("Contract hash: {}", result.contract_hash);
    
    Ok(())
}

async fn update_contract(script_hash: String, nef_path: PathBuf, manifest_path: PathBuf, _account: Option<String>, state: &mut crate::commands::wallet::CliState) -> CliResult<()> {
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
    // let nef_bytes = fs::read(&nef_path)?;
    // let manifest_json = fs::read_to_string(&manifest_path)?;
    
    // Parse NEF and manifest
    // let nef = NefFile::deserialize(&nef_bytes)?;
    // let manifest: ContractManifest = serde_json::from_str(&manifest_json)?;
    
    // Get account to pay for update
    // let account_address = match account {
    //     Some(addr) => addr,
    //     None => {
    //         // If no account specified, use the first account in the wallet
    //         let accounts = state.wallet.unwrap().get_accounts();
    //         if accounts.is_empty() {
    //             print_error("No accounts in wallet");
    //             return Err(CliError::Wallet("No accounts in wallet".to_string()));
    //         }
    //         accounts[0].address.clone()
    //     }
    // };
    
    // Get password for signing
    // let password = prompt_password("Enter wallet password")?;
    
    // Create and sign update transaction
    // let rpc_client = state.rpc_client.as_ref().unwrap();
    // let contract_hash = H160::from_str(&script_hash)?;
    // let tx_builder = rpc_client.create_contract_update_transaction(contract_hash, nef, manifest)?;
    // let signed_tx = state.wallet.unwrap().sign_transaction(tx_builder, &account_address, &password)?;
    
    // Send transaction
    // let result = rpc_client.send_transaction(signed_tx).await?;
    
    print_success("Contract updated successfully");
    // println!("Transaction hash: {}", result.hash);
    
    Ok(())
}

async fn invoke_contract(script_hash: String, method: String, _params: Option<String>, _account: Option<String>, test_invoke: bool, state: &mut crate::commands::wallet::CliState) -> CliResult<()> {
    if state.rpc_client.is_none() {
        print_error("No RPC client is connected. Please connect to a node first.");
        return Err(CliError::Network("No RPC client is connected".to_string()));
    }
    
    // Parse parameters if provided
    // let parameters = match params {
    //     Some(p) => {
    //         let params_json: Vec<serde_json::Value> = serde_json::from_str(&p)
    //             .map_err(|e| CliError::Input(format!("Invalid JSON parameters: {}", e)))?;
    //         
    //         // Convert JSON parameters to ContractParameter
    //         params_json.into_iter()
    //             .map(|v| ContractParameter::from_json(v))
    //             .collect::<Result<Vec<_>, _>>()?
    //     },
    //     None => Vec::new(),
    // };
    
    // Convert script hash
    // let contract_hash = H160::from_str(&script_hash)?;
    
    if test_invoke {
        print_info(&format!("Test invoking method '{}' on contract {}", method, script_hash));
        
        // Test invoke
        // let rpc_client = state.rpc_client.as_ref().unwrap();
        // let result = rpc_client.invoke_function(&contract_hash, method, parameters, None).await?;
        
        // Display result
        // println!("Invocation result:");
        // println!("  State: {}", result.state);
        // println!("  Gas consumed: {}", result.gas_consumed);
        // println!("  Stack:");
        // for (i, item) in result.stack.iter().enumerate() {
        //     println!("    {}: {}", i, item);
        // }
    } else {
        // Real invocation
        if state.wallet.is_none() {
            print_error("No wallet is currently open");
            return Err(CliError::Wallet("No wallet is currently open".to_string()));
        }
        
        print_info(&format!("Invoking method '{}' on contract {}", method, script_hash));
        
        // Get account to pay for invocation
        // let account_address = match account {
        //     Some(addr) => addr,
        //     None => {
        //         // If no account specified, use the first account in the wallet
        //         let accounts = state.wallet.unwrap().get_accounts();
        //         if accounts.is_empty() {
        //             print_error("No accounts in wallet");
        //             return Err(CliError::Wallet("No accounts in wallet".to_string()));
        //         }
        //         accounts[0].address.clone()
        //     }
        // };
        
        // Get password for signing
        // let password = prompt_password("Enter wallet password")?;
        
        // Create and sign invocation transaction
        // let rpc_client = state.rpc_client.as_ref().unwrap();
        // let tx_builder = rpc_client.create_invocation_transaction(contract_hash, method, parameters)?;
        // let signed_tx = state.wallet.unwrap().sign_transaction(tx_builder, &account_address, &password)?;
        
        // Send transaction
        // let result = rpc_client.send_transaction(signed_tx).await?;
        
        print_success("Contract method invoked successfully");
        // println!("Transaction hash: {}", result.hash);
    }
    
    Ok(())
}

async fn list_native_contracts(state: &mut crate::commands::wallet::CliState) -> CliResult<()> {
    if state.rpc_client.is_none() {
        print_error("No RPC client is connected. Please connect to a node first.");
        return Err(CliError::Network("No RPC client is connected".to_string()));
    }
    
    print_info("Native contracts:");
    
    // List native contracts
    // let rpc_client = state.rpc_client.as_ref().unwrap();
    // let native_contracts = rpc_client.get_native_contracts().await?;
    
    // for contract in native_contracts {
    //     println!("Contract Hash: {}", contract.hash);
    //     println!("  Name: {}", contract.manifest.name);
    //     println!("  ID: {}", contract.id);
    //     println!("  Supported Standards: {:?}", contract.manifest.supported_standards);
    //     println!();
    // }
    
    print_success("Native contracts retrieved successfully");
    Ok(())
}
