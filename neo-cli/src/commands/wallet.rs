use clap::{Args, Subcommand};
use neo3::prelude::*;
use neo3::providers::Http;
use crate::utils::error::{CliError, CliResult};
use crate::utils::{print_success, print_error, print_info, prompt_password};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct WalletArgs {
    #[command(subcommand)]
    pub command: WalletCommands,
}

#[derive(Subcommand, Debug)]
pub enum WalletCommands {
    /// Create a new wallet
    Create {
        /// Path to save the wallet
        #[arg(short, long)]
        path: PathBuf,
    },
    
    /// Open an existing wallet
    Open {
        /// Path to the wallet file
        #[arg(short, long)]
        path: PathBuf,
    },
    
    /// Close the current wallet
    Close,
    
    /// List addresses in the wallet
    ListAddress,
    
    /// List assets in the wallet
    ListAsset,
    
    /// Create a new address in the wallet
    CreateAddress {
        /// Number of addresses to create
        #[arg(short, long, default_value = "1")]
        count: u16,
    },
    
    /// Import a private key
    ImportKey {
        /// WIF string or path to a file containing WIF keys
        #[arg(short, long)]
        wif_or_file: String,
    },
    
    /// Export private keys
    ExportKey {
        /// Path to save the exported keys
        #[arg(short, long)]
        path: Option<PathBuf>,
        
        /// Address to export (if not specified, exports all)
        #[arg(short, long)]
        address: Option<String>,
    },
    
    /// Show unclaimed GAS
    ShowGas,
    
    /// Change wallet password
    ChangePassword,
    
    /// Transfer assets to another address
    Transfer {
        /// Asset ID (NEO, GAS, or script hash)
        #[arg(short, long)]
        asset: String,
        
        /// Recipient address
        #[arg(short, long)]
        to: String,
        
        /// Amount to transfer
        #[arg(short, long)]
        amount: String,
        
        /// Sender address (if not specified, uses the first account)
        #[arg(short, long)]
        from: Option<String>,
    },
    
    /// Show wallet balance
    Balance {
        /// Address to show balance for (if not provided, shows all addresses)
        #[arg(short, long)]
        address: Option<String>,
        
        /// Only show this token (NEO, GAS, or script hash)
        #[arg(short, long)]
        token: Option<String>,
    },
}

/// CLI state to track the current wallet and other session information
pub struct CliState {
    pub wallet: Option<Wallet>,
    pub rpc_client: Option<RpcClient<Http>>,
}

impl Default for CliState {
    fn default() -> Self {
        Self {
            wallet: None,
            rpc_client: None,
        }
    }
}

pub async fn handle_wallet_command(args: WalletArgs, state: &mut CliState) -> CliResult<()> {
    match args.command {
        WalletCommands::Create { path } => create_wallet(path, state).await,
        WalletCommands::Open { path } => open_wallet(path, state).await,
        WalletCommands::Close => close_wallet(state).await,
        WalletCommands::ListAddress => list_addresses(state).await,
        WalletCommands::ListAsset => list_assets(state).await,
        WalletCommands::CreateAddress { count } => create_addresses(count, state).await,
        WalletCommands::ImportKey { wif_or_file } => import_key(wif_or_file, state).await,
        WalletCommands::ExportKey { path, address } => export_key(path, address, state).await,
        WalletCommands::ShowGas => show_gas(state).await,
        WalletCommands::ChangePassword => change_password(state).await,
        WalletCommands::Transfer { asset, to, amount, from } => transfer_assets(asset, to, amount, from, state).await,
        WalletCommands::Balance { address, token } => balance(address, token, state).await,
    }
}

async fn create_wallet(path: PathBuf, state: &mut CliState) -> CliResult<()> {
    print_info("Creating new wallet...");
    let password = prompt_password("Enter password for new wallet")?;
    let confirm_password = prompt_password("Confirm password")?;
    
    if password != confirm_password {
        print_error("Passwords do not match");
        return Err(CliError::Input("Passwords do not match".to_string()));
    }
    
    // Create wallet using NeoRust SDK
    let wallet = Wallet::new(&path.to_string_lossy()).map_err(|e| CliError::Wallet(e.to_string()))?;
    
    // Save wallet to disk
    wallet.save_to_file(&path).map_err(|e| CliError::Wallet(e.to_string()))?;
    
    state.wallet = Some(wallet);
    
    print_success(&format!("Wallet created at: {:?}", path));
    Ok(())
}

async fn open_wallet(path: PathBuf, state: &mut CliState) -> CliResult<()> {
    if !path.exists() {
        print_error(&format!("Wallet file not found: {:?}", path));
        return Err(CliError::Input(format!("Wallet file not found: {:?}", path)));
    }
    
    print_info(&format!("Opening wallet: {:?}", path));
    let password = prompt_password("Enter wallet password")?;
    
    // Open wallet using NeoRust SDK
    let wallet = Wallet::from_file(&path).map_err(|e| CliError::Wallet(e.to_string()))?;
    
    // Verify password
    if !wallet.verify_password(&password) {
        print_error("Incorrect password");
        return Err(CliError::Wallet("Incorrect password".to_string()));
    }
    
    state.wallet = Some(wallet);
    
    print_success("Wallet opened successfully");
    Ok(())
}

async fn close_wallet(state: &mut CliState) -> CliResult<()> {
    if state.wallet.is_none() {
        print_error("No wallet is currently open");
        return Err(CliError::Wallet("No wallet is currently open".to_string()));
    }
    
    state.wallet = None;
    print_success("Wallet closed");
    Ok(())
}

async fn list_addresses(state: &mut CliState) -> CliResult<()> {
    if state.wallet.is_none() {
        print_error("No wallet is currently open");
        return Err(CliError::Wallet("No wallet is currently open".to_string()));
    }
    
    print_info("Wallet addresses:");
    
    // Get accounts from wallet
    let wallet = state.wallet.as_ref().unwrap();
    let accounts = wallet.get_accounts();
    
    if accounts.is_empty() {
        println!("  No addresses in wallet");
    } else {
        for (i, account) in accounts.iter().enumerate() {
            println!("  {}. Address: {}", i + 1, account.address());
            println!("     ScriptHash: {}", account.script_hash());
            println!();
        }
    }
    
    Ok(())
}

async fn list_assets(state: &mut CliState) -> CliResult<()> {
    if state.wallet.is_none() {
        print_error("No wallet is currently open");
        return Err(CliError::Wallet("No wallet is currently open".to_string()));
    }
    
    if state.rpc_client.is_none() {
        print_error("No RPC client is connected. Please connect to a node first.");
        return Err(CliError::Network("No RPC client is connected".to_string()));
    }
    
    print_info("Wallet assets:");
    
    let wallet = state.wallet.as_ref().unwrap();
    let rpc_client = state.rpc_client.as_ref().unwrap();
    
    // NEO and GAS script hashes
    let neo_hash = H160::from_hex_str("0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5").map_err(|e| CliError::Sdk(e.to_string()))?;
    let gas_hash = H160::from_hex_str("0xd2a4cff31913016155e38e474a2c06d08be276cf").map_err(|e| CliError::Sdk(e.to_string()))?;
    
    for account in wallet.get_accounts() {
        println!("  Address: {}", account.address());
        
        // Get NEO balance
        match rpc_client.get_nep17_balance(&account.script_hash(), &neo_hash).await {
            Ok(balance) => println!("  NEO: {}", balance),
            Err(_) => println!("  NEO: Error retrieving balance")
        }
        
        // Get GAS balance
        match rpc_client.get_nep17_balance(&account.script_hash(), &gas_hash).await {
            Ok(balance) => println!("  GAS: {}", balance),
            Err(_) => println!("  GAS: Error retrieving balance")
        }
        
        println!();
    }
    
    Ok(())
}

async fn create_addresses(count: u16, state: &mut CliState) -> CliResult<()> {
    if state.wallet.is_none() {
        print_error("No wallet is currently open");
        return Err(CliError::Wallet("No wallet is currently open".to_string()));
    }
    
    print_info(&format!("Creating {} new address(es)...", count));
    
    let wallet = state.wallet.as_mut().unwrap();
    let mut created = 0;
    
    for _ in 0..count {
        // Create new account
        let account = Account::create_random().map_err(|e| CliError::Wallet(e.to_string()))?;
        wallet.add_account(account.clone());
        
        println!("  Created address: {}", account.address());
        created += 1;
    }
    
    // Save wallet to update with new accounts
    wallet.save_to_file(&PathBuf::from(wallet.path())).map_err(|e| CliError::Wallet(e.to_string()))?;
    
    print_success(&format!("Created {} new address(es)", created));
    Ok(())
}

async fn import_key(wif_or_file: String, state: &mut CliState) -> CliResult<()> {
    if state.wallet.is_none() {
        print_error("No wallet is currently open");
        return Err(CliError::Wallet("No wallet is currently open".to_string()));
    }
    
    print_info("Importing private key(s)...");
    
    let wallet = state.wallet.as_mut().unwrap();
    let path = PathBuf::from(&wif_or_file);
    
    if path.exists() {
        // Import from file
        let keys = std::fs::read_to_string(&wif_or_file)
            .map_err(|e| CliError::Io(e))?;
        
        let mut imported = 0;
        for key in keys.lines() {
            if !key.trim().is_empty() {
                // Create account from WIF
                match Account::from_wif(key.trim()) {
                    Ok(account) => {
                        wallet.add_account(account.clone());
                        println!("  Imported address: {}", account.address());
                        imported += 1;
                    },
                    Err(e) => {
                        print_error(&format!("Failed to import key: {}", key.trim()));
                        print_error(&format!("  Error: {}", e));
                    }
                }
            }
        }
        
        if imported > 0 {
            // Save wallet to update with new accounts
            wallet.save_to_file(&PathBuf::from(wallet.path())).map_err(|e| CliError::Wallet(e.to_string()))?;
            print_success(&format!("Imported {} private key(s) successfully", imported));
        } else {
            print_error("No valid private keys were imported");
            return Err(CliError::Input("No valid private keys were imported".to_string()));
        }
    } else {
        // Import single key
        match Account::from_wif(&wif_or_file) {
            Ok(account) => {
                wallet.add_account(account.clone());
                
                // Save wallet to update with new account
                wallet.save_to_file(&PathBuf::from(wallet.path())).map_err(|e| CliError::Wallet(e.to_string()))?;
                
                println!("  Imported address: {}", account.address());
                print_success("Private key imported successfully");
            },
            Err(e) => {
                print_error(&format!("Failed to import private key: {}", e));
                return Err(CliError::Input(format!("Invalid private key: {}", e)));
            }
        }
    }
    
    Ok(())
}

async fn export_key(path: Option<PathBuf>, address: Option<String>, state: &mut CliState) -> CliResult<()> {
    if state.wallet.is_none() {
        print_error("No wallet is currently open");
        return Err(CliError::Wallet("No wallet is currently open".to_string()));
    }
    
    let password = prompt_password("Enter wallet password")?;
    
    // Verify password
    let wallet = state.wallet.as_ref().unwrap();
    if !wallet.verify_password(&password) {
        print_error("Incorrect password");
        return Err(CliError::Wallet("Incorrect password".to_string()));
    }
    
    // Export keys
    let keys = if let Some(addr) = address {
        // Export specific address
        let account = wallet.get_accounts().iter()
            .find(|a| a.address() == addr)
            .ok_or_else(|| CliError::Wallet(format!("Address not found: {}", addr)))?;
        
        if !account.has_key() {
            print_error("Account does not have a private key");
            return Err(CliError::Wallet("Account does not have a private key".to_string()));
        }
        
        vec![account.export_to_wif().map_err(|e| CliError::Wallet(e.to_string()))?]
    } else {
        // Export all addresses with private keys
        wallet.get_accounts()
            .iter()
            .filter(|a| a.has_key())
            .map(|a| a.export_to_wif())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| CliError::Wallet(e.to_string()))?
    };
    
    if keys.is_empty() {
        print_error("No private keys to export");
        return Err(CliError::Wallet("No private keys to export".to_string()));
    }
    
    if let Some(p) = path {
        std::fs::write(&p, keys.join("\n")).map_err(|e| CliError::Io(e))?;
        print_success(&format!("Exported {} key(s) to: {:?}", keys.len(), p));
    } else {
        println!("Private keys:");
        for key in keys {
            println!("  {}", key);
        }
    }
    
    print_success("Keys exported successfully");
    Ok(())
}

async fn show_gas(state: &mut CliState) -> CliResult<()> {
    if state.wallet.is_none() {
        print_error("No wallet is currently open");
        return Err(CliError::Wallet("No wallet is currently open".to_string()));
    }
    
    if state.rpc_client.is_none() {
        print_error("No RPC client is connected. Please connect to a node first.");
        return Err(CliError::Network("No RPC client is connected".to_string()));
    }
    
    print_info("Unclaimed GAS:");
    
    let wallet = state.wallet.as_ref().unwrap();
    let rpc_client = state.rpc_client.as_ref().unwrap();
    
    let mut total_unclaimed: f64 = 0.0;
    
    for account in wallet.get_accounts() {
        // Get unclaimed GAS for each account
        let script_hash = account.script_hash();
        match rpc_client.get_unclaimed_gas(&script_hash).await {
            Ok(unclaimed) => {
                println!("  Address: {}", account.address());
                println!("  Unclaimed GAS: {}", unclaimed);
                total_unclaimed += unclaimed.parse::<f64>().unwrap_or(0.0);
                println!();
            },
            Err(e) => {
                println!("  Address: {}", account.address());
                println!("  Error retrieving unclaimed GAS: {}", e);
                println!();
            }
        }
    }
    
    println!("Total unclaimed GAS across all accounts: {}", total_unclaimed);
    
    Ok(())
}

async fn change_password(state: &mut CliState) -> CliResult<()> {
    if state.wallet.is_none() {
        print_error("No wallet is currently open");
        return Err(CliError::Wallet("No wallet is currently open".to_string()));
    }
    
    let current_password = prompt_password("Enter current password")?;
    
    // Verify current password
    let wallet = state.wallet.as_mut().unwrap();
    if !wallet.verify_password(&current_password) {
        print_error("Incorrect password");
        return Err(CliError::Wallet("Incorrect password".to_string()));
    }
    
    let new_password = prompt_password("Enter new password")?;
    let confirm_password = prompt_password("Confirm new password")?;
    
    if new_password != confirm_password {
        print_error("Passwords do not match");
        return Err(CliError::Input("Passwords do not match".to_string()));
    }
    
    // Change password for each account in the wallet
    for account in wallet.get_accounts_mut() {
        if account.has_key() {
            // If the account is encrypted, decrypt it with old password and encrypt with new password
            if account.is_encrypted() {
                // First decrypt with old password
                account.decrypt_private_key(&current_password).map_err(|e| CliError::Wallet(e.to_string()))?;
                
                // Then encrypt with new password
                account.encrypt_private_key(&new_password).map_err(|e| CliError::Wallet(e.to_string()))?;
            }
        }
    }
    
    // Save wallet to disk
    wallet.save_to_file(&PathBuf::from(wallet.path())).map_err(|e| CliError::Wallet(e.to_string()))?;
    
    print_success("Password changed successfully");
    Ok(())
}

async fn transfer_assets(asset: String, to: String, amount: String, from: Option<String>, state: &mut CliState) -> CliResult<()> {
    if state.wallet.is_none() {
        print_error("No wallet is currently open");
        return Err(CliError::Wallet("No wallet is currently open".to_string()));
    }
    
    if state.rpc_client.is_none() {
        print_error("No RPC client is connected. Please connect to a node first.");
        return Err(CliError::Network("No RPC client is connected".to_string()));
    }
    
    print_info("Transferring assets...");
    
    let wallet = state.wallet.as_ref().unwrap();
    let rpc_client = state.rpc_client.as_ref().unwrap();
    
    let asset_hash = if asset.starts_with("0x") {
        H160::from_hex_str(&asset)?
    } else {
        H160::from_hex_str(&format!("0x{}", asset))?
    };
    
    let amount = amount.parse::<f64>()?;
    
    let from_address = if let Some(addr) = from {
        addr
    } else {
        wallet.get_accounts().first().unwrap().address().clone()
    };
    
    let from_script_hash = H160::from_hex_str(&from_address)?;
    
    let to_script_hash = H160::from_hex_str(&to)?;
    
    let mut total_transferred: f64 = 0.0;
    
    for account in wallet.get_accounts() {
        if account.address() == from_address {
            // Get balance of the asset
            match rpc_client.get_nep17_balance(&asset_hash, &account.script_hash()).await {
                Ok(balance) => {
                    if balance < amount {
                        print_error(&format!("Insufficient balance for transfer: {} < {}", balance, amount));
                        return Err(CliError::Wallet(format!("Insufficient balance for transfer: {} < {}", balance, amount)));
                    }
                    
                    // Transfer the asset
                    match rpc_client.transfer_nep17(&asset_hash, &from_script_hash, &to_script_hash, amount).await {
                        Ok(tx_hash) => {
                            println!("Transferred {} {} to {}", amount, asset, to);
                            println!("Transaction hash: {}", tx_hash);
                            total_transferred += amount;
                        },
                        Err(e) => {
                            print_error(&format!("Failed to transfer assets: {}", e));
                            return Err(CliError::Wallet(e.to_string()));
                        }
                    }
                },
                Err(e) => {
                    print_error(&format!("Failed to retrieve balance: {}", e));
                    return Err(CliError::Wallet(e.to_string()));
                }
            }
        }
    }
    
    println!("Total assets transferred: {}", total_transferred);
    
    Ok(())
}

async fn balance(address: Option<String>, token: Option<String>, state: &mut CliState) -> CliResult<()> {
    if state.wallet.is_none() {
        print_error("No wallet is currently open");
        return Err(CliError::Wallet("No wallet is currently open".to_string()));
    }
    
    if state.rpc_client.is_none() {
        print_error("No RPC client is connected. Please connect to a node first.");
        return Err(CliError::Network("No RPC client is connected".to_string()));
    }
    
    let wallet = state.wallet.as_ref().unwrap();
    let rpc_client = state.rpc_client.as_ref().unwrap();
    
    // Get accounts to check
    let accounts = match &address {
        Some(addr) => {
            let account = wallet.get_accounts().iter()
                .find(|a| a.address() == addr)
                .ok_or_else(|| CliError::Wallet(format!("Account not found: {}", addr)))?;
            vec![account.clone()]
        },
        None => wallet.get_accounts()
    };
    
    if accounts.is_empty() {
        print_info("No accounts in wallet");
        return Ok(());
    }
    
    // Default tokens to check
    let mut tokens = Vec::new();
    if let Some(t) = token {
        tokens.push(t);
    } else {
        // Always check NEO and GAS
        tokens.push("NEO".to_string());
        tokens.push("GAS".to_string());
    }
    
    for account in &accounts {
        print_info(&format!("Balance for {}", account.address()));
        
        for token_name in &tokens {
            let token_hash = match token_name.to_uppercase().as_str() {
                "NEO" => H160::from_str("0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5").unwrap(),
                "GAS" => H160::from_str("0xd2a4cff31913016155e38e474a2c06d08be276cf").unwrap(),
                _ => {
                    // Try to parse as script hash
                    match H160::from_str(token_name) {
                        Ok(hash) => hash,
                        Err(_) => {
                            print_error(&format!("Invalid token: {}", token_name));
                            continue;
                        }
                    }
                }
            };
            
            // Get token symbol if it's not NEO or GAS
            let display_name = if !["NEO", "GAS"].contains(&token_name.to_uppercase().as_str()) {
                match rpc_client.invoke_function(&token_hash, "symbol", vec![], None).await {
                    Ok(result) => {
                        if let Some(item) = result.stack.first() {
                            match item {
                                StackItem::ByteString(bytes) => {
                                    String::from_utf8_lossy(bytes).to_string()
                                },
                                _ => token_name.clone()
                            }
                        } else {
                            token_name.clone()
                        }
                    },
                    Err(_) => token_name.clone()
                }
            } else {
                token_name.clone()
            };
            
            // Get token balance
            match rpc_client.invoke_function(
                &token_hash,
                "balanceOf",
                vec![account.script_hash().into()],
                None
            ).await {
                Ok(result) => {
                    if let Some(item) = result.stack.first() {
                        if let StackItem::Integer(value) = item {
                            let raw_balance = value.to_i64().unwrap_or(0);
                            
                            // Get decimals for proper display
                            let decimals = if token_name.to_uppercase() == "NEO" {
                                0
                            } else if token_name.to_uppercase() == "GAS" {
                                8
                            } else {
                                match rpc_client.invoke_function(&token_hash, "decimals", vec![], None).await {
                                    Ok(result) => {
                                        if let Some(item) = result.stack.first() {
                                            if let StackItem::Integer(value) = item {
                                                value.to_u8().unwrap_or(8)
                                            } else {
                                                8
                                            }
                                        } else {
                                            8
                                        }
                                    },
                                    Err(_) => 8
                                }
                            };
                            
                            // Format balance with correct decimal places
                            let divisor = 10_f64.powi(decimals as i32);
                            let formatted_balance = (raw_balance as f64) / divisor;
                            
                            println!("  {}: {}", display_name, formatted_balance);
                        }
                    }
                },
                Err(e) => {
                    print_error(&format!("Failed to get balance for {}: {}", display_name, e));
                }
            }
        }
        println!();
    }
    
    Ok(())
}
