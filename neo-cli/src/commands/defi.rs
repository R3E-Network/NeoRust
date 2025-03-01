use clap::{Args, Subcommand};
use neo3::prelude::*;
use crate::utils::error::{CliError, CliResult};
use crate::utils::{print_success, print_error, print_info, prompt_password};
use std::path::PathBuf;
use std::str::FromStr;
use neo3::neo_contract::famous::{FlamingoContract, GrandShareContract, NeoburgerContract, NeoCompoundContract};

#[derive(Args, Debug)]
pub struct DefiArgs {
    #[command(subcommand)]
    pub command: DefiCommands,
}

#[derive(Subcommand, Debug)]
pub enum DefiCommands {
    /// Flamingo Finance operations
    Flamingo {
        #[command(subcommand)]
        command: FlamingoCommands,
    },
    
    /// NeoburgerNeo operations
    Neoburger {
        #[command(subcommand)]
        command: NeoburgerCommands,
    },
    
    /// GrandShare operations
    GrandShare {
        #[command(subcommand)]
        command: GrandShareCommands,
    },
    
    /// NeoCompound operations
    NeoCompound {
        #[command(subcommand)]
        command: NeoCompoundCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum FlamingoCommands {
    /// Swap tokens on Flamingo
    Swap {
        /// Source token script hash or symbol (NEO, GAS)
        #[arg(short, long)]
        from_token: String,
        
        /// Destination token script hash or symbol
        #[arg(short, long)]
        to_token: String,
        
        /// Amount of source token to swap
        #[arg(short, long)]
        amount: String,
        
        /// Minimum amount of destination token to receive
        #[arg(short, long)]
        min_return: String,
        
        /// Account to pay for the swap (if not specified, uses the first account)
        #[arg(short, long)]
        account: Option<String>,
    },
    
    /// Add liquidity to a pool
    AddLiquidity {
        /// First token script hash or symbol
        #[arg(short, long)]
        token_a: String,
        
        /// Second token script hash or symbol
        #[arg(short, long)]
        token_b: String,
        
        /// Amount of first token
        #[arg(short, long)]
        amount_a: String,
        
        /// Amount of second token
        #[arg(short, long)]
        amount_b: String,
        
        /// Account to pay for adding liquidity (if not specified, uses the first account)
        #[arg(short, long)]
        account: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum NeoburgerCommands {
    /// Wrap NEO to bNEO
    Wrap {
        /// Amount of NEO to wrap
        #[arg(short, long)]
        amount: String,
        
        /// Account to pay for wrapping (if not specified, uses the first account)
        #[arg(short, long)]
        account: Option<String>,
    },
    
    /// Unwrap bNEO to NEO
    Unwrap {
        /// Amount of bNEO to unwrap
        #[arg(short, long)]
        amount: String,
        
        /// Account to pay for unwrapping (if not specified, uses the first account)
        #[arg(short, long)]
        account: Option<String>,
    },
    
    /// Claim GAS rewards from bNEO
    ClaimGas {
        /// Account to claim GAS (if not specified, uses the first account)
        #[arg(short, long)]
        account: Option<String>,
    },
    
    /// Get the current exchange rate between NEO and bNEO
    GetRate,
}

// Similar structures for GrandShareCommands and NeoCompoundCommands...

pub async fn handle_defi_command(args: DefiArgs, state: &mut crate::commands::wallet::CliState) -> CliResult<()> {
    match args.command {
        DefiCommands::Flamingo { command } => handle_flamingo_command(command, state).await,
        DefiCommands::Neoburger { command } => handle_neoburger_command(command, state).await,
        DefiCommands::GrandShare { command } => handle_grandshare_command(command, state).await,
        DefiCommands::NeoCompound { command } => handle_neocompound_command(command, state).await,
    }
}

async fn handle_flamingo_command(args: FlamingoCommands, state: &mut crate::commands::wallet::CliState) -> CliResult<()> {
    if state.wallet.is_none() {
        print_error("No wallet is currently open");
        return Err(CliError::Wallet("No wallet is currently open".to_string()));
    }
    
    if state.rpc_client.is_none() {
        print_error("No RPC client is connected. Please connect to a node first.");
        return Err(CliError::Network("No RPC client is connected".to_string()));
    }
    
    let rpc_client = state.rpc_client.as_ref().unwrap();
    let flamingo_contract = FlamingoContract::new(Some(rpc_client));
    
    match args {
        FlamingoCommands::Swap { from_token, to_token, amount, min_return, account } => {
            // Resolve token script hashes
            let from_token_hash = resolve_token_hash(&from_token)?;
            let to_token_hash = resolve_token_hash(&to_token)?;
            
            // Parse amounts
            let amount_val = parse_amount(&amount, &from_token, rpc_client).await?;
            let min_return_val = parse_amount(&min_return, &to_token, rpc_client).await?;
            
            // Get account
            let account_obj = get_account(account, state.wallet.as_ref().unwrap())?;
            
            print_info(&format!("Swapping {} {} for at least {} {} on Flamingo...", 
                     amount, from_token, min_return, to_token));
            
            // Get password for signing
            let password = prompt_password("Enter wallet password")?;
            
            // Create and sign swap transaction
            let tx_builder = flamingo_contract.swap(
                &from_token_hash, 
                &to_token_hash, 
                amount_val, 
                min_return_val, 
                &account_obj,
            ).await.map_err(|e| CliError::Sdk(format!("Failed to create swap transaction: {}", e)))?;
            
            // Sign transaction
            let signed_tx = account_obj.sign_tx(tx_builder, &password)
                .map_err(|e| CliError::Wallet(format!("Failed to sign transaction: {}", e)))?;
            
            // Send transaction
            let result = rpc_client.send_raw_transaction(&signed_tx).await
                .map_err(|e| CliError::Network(format!("Failed to send transaction: {}", e)))?;
            
            print_success("Swap transaction sent successfully");
            println!("Transaction hash: {}", result.hash);
        },
        
        FlamingoCommands::AddLiquidity { token_a, token_b, amount_a, amount_b, account } => {
            // Resolve token script hashes
            let token_a_hash = resolve_token_hash(&token_a)?;
            let token_b_hash = resolve_token_hash(&token_b)?;
            
            // Parse amounts
            let amount_a_val = parse_amount(&amount_a, &token_a, rpc_client).await?;
            let amount_b_val = parse_amount(&amount_b, &token_b, rpc_client).await?;
            
            // Get account
            let account_obj = get_account(account, state.wallet.as_ref().unwrap())?;
            
            print_info(&format!("Adding liquidity with {} {} and {} {} on Flamingo...", 
                     amount_a, token_a, amount_b, token_b));
            
            // Get password for signing
            let password = prompt_password("Enter wallet password")?;
            
            // Create and sign add liquidity transaction
            let tx_builder = flamingo_contract.add_liquidity(
                &token_a_hash, 
                &token_b_hash, 
                amount_a_val, 
                amount_b_val, 
                &account_obj,
            ).await.map_err(|e| CliError::Sdk(format!("Failed to create add liquidity transaction: {}", e)))?;
            
            // Sign transaction
            let signed_tx = account_obj.sign_tx(tx_builder, &password)
                .map_err(|e| CliError::Wallet(format!("Failed to sign transaction: {}", e)))?;
            
            // Send transaction
            let result = rpc_client.send_raw_transaction(&signed_tx).await
                .map_err(|e| CliError::Network(format!("Failed to send transaction: {}", e)))?;
            
            print_success("Add liquidity transaction sent successfully");
            println!("Transaction hash: {}", result.hash);
        },
    }
    
    Ok(())
}

// Helper function for resolving token hashes
fn resolve_token_hash(token: &str) -> Result<H160, CliError> {
    match token.to_uppercase().as_str() {
        "NEO" => {
            // Neo Governance token hash
            Ok(H160::from_str("0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5").unwrap())
        },
        "GAS" => {
            // GAS token hash
            Ok(H160::from_str("0xd2a4cff31913016155e38e474a2c06d08be276cf").unwrap())
        },
        _ => {
            // Try to parse as script hash
            match H160::from_str(token) {
                Ok(hash) => Ok(hash),
                Err(_) => Err(CliError::Input(format!("Invalid token: {}", token)))
            }
        }
    }
}

// Helper function for converting a user-friendly amount to the smallest unit value
async fn parse_amount(amount: &str, token: &str, rpc_client: &RpcClient<impl JsonRpcProvider>) -> Result<i64, CliError> {
    // Parse the amount string
    let amount_val = amount.parse::<f64>()
        .map_err(|_| CliError::Input(format!("Invalid amount: {}", amount)))?;
    
    // Get decimals for the token
    let decimals = match token.to_uppercase().as_str() {
        "NEO" => 0, // NEO is indivisible
        "GAS" => 8, // GAS has 8 decimals
        _ => {
            // Try to get decimals from the contract
            let token_hash = resolve_token_hash(token)?;
            match rpc_client.invoke_function(&token_hash, "decimals", vec![], None).await {
                Ok(result) => {
                    if let Some(item) = result.stack.first() {
                        if let StackItem::Integer(value) = item {
                            value.to_u8().unwrap_or(8)
                        } else {
                            8 // Default to 8 if not an integer
                        }
                    } else {
                        8 // Default to 8
                    }
                },
                Err(_) => 8 // Default to 8
            }
        }
    };
    
    // Convert to the smallest unit
    let factor = 10_u64.pow(decimals as u32);
    let amount_int = (amount_val * factor as f64) as i64;
    
    Ok(amount_int)
}

// Helper function to get account from wallet
fn get_account(account: Option<String>, wallet: &Wallet) -> Result<Account, CliError> {
    let account_address = match account {
        Some(addr) => addr,
        None => {
            // If no account specified, use the first account in the wallet
            let accounts = wallet.get_accounts();
            if accounts.is_empty() {
                return Err(CliError::Wallet("No accounts in wallet".to_string()));
            }
            accounts[0].address().to_string()
        }
    };
    
    // Find account in wallet
    wallet.get_accounts().iter()
        .find(|a| a.address() == account_address)
        .ok_or_else(|| CliError::Wallet(format!("Account not found: {}", account_address)))
        .map(|a| a.clone())
}

// Similar implementation for handle_neoburger_command, handle_grandshare_command, and handle_neocompound_command
// ... 