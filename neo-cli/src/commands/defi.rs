use clap::{Args, Subcommand};
use neo3::prelude::*;
use crate::utils::error::{CliError, CliResult};
use crate::utils::{print_success, print_error, print_info, prompt_password};
use std::path::PathBuf;
use std::str::FromStr;
use neo3::neo_contract::famous::{FlamingoContract, GrandShareContract, NeoburgerContract, NeoCompoundContract};
use neo3::neo_builder::{Transaction, TransactionBuilder};
use neo3::neo_providers::JsonRpcProvider;
use num_traits::cast::ToPrimitive;
use rand;
use neo3::neo_types::{signer::{Signer, WitnessScope}, ContractParameter, hash::H160, address::Address};
use neo3::neo_builders::script::ScriptBuilder;
use neo3::neo_providers::Http;
use neo3::neo_contract::nep17::Nep17Contract;

#[derive(Args, Debug)]
pub struct DefiArgs {
    #[command(subcommand)]
    pub command: DefiCommands,
}

#[derive(Subcommand, Debug, Clone)]
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
    
    /// Token operations
    Token(TokenArgs),
}

#[derive(Args, Debug, Clone)]
pub struct TokenArgs {
    /// Token ID (script hash or known symbol like NEO, GAS)
    #[arg(short, long)]
    pub token_id: Option<String>,
    
    /// Address to check token balance
    #[arg(short, long)]
    pub address: Option<String>,
    
    #[command(subcommand)]
    pub command: Option<TokenCommand>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum TokenCommand {
    /// Check token allowance (NEP-17)
    Allowance {
        /// Owner address
        #[arg(short, long)]
        owner: String,
        
        /// Spender address
        #[arg(short, long)]
        spender: String,
    },
    
    /// Approve token spending (NEP-17)
    Approve {
        /// Spender address
        #[arg(short, long)]
        spender: String,
        
        /// Amount to approve
        #[arg(short, long)]
        amount: String,
        
        /// Account to authorize the approval (if not specified, uses the first account)
        #[arg(short, long)]
        from: Option<String>,
        
        /// Account password (will prompt if not provided)
        #[arg(short, long)]
        password: Option<String>,
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

#[derive(Subcommand, Debug)]
pub enum GrandShareCommands {
    /// Stake NEO in GrandShare
    Stake {
        /// Amount of NEO to stake
        #[arg(short, long)]
        amount: String,
        
        /// Account to use for staking (if not specified, uses the first account)
        #[arg(short, long)]
        account: Option<String>,
    },
    
    /// Withdraw staked NEO from GrandShare
    Withdraw {
        /// Amount of NEO to withdraw
        #[arg(short, long)]
        amount: String,
        
        /// Account to withdraw to (if not specified, uses the first account)
        #[arg(short, long)]
        account: Option<String>,
    },
    
    /// Claim rewards from GrandShare staking
    ClaimRewards {
        /// Account to claim rewards (if not specified, uses the first account)
        #[arg(short, long)]
        account: Option<String>,
    },
    
    /// Get current staking information
    GetInfo {
        /// Account to get info for (if not specified, uses the first account)
        #[arg(short, long)]
        account: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum NeoCompoundCommands {
    /// Deposit NEO for compounding rewards
    Deposit {
        /// Amount of NEO to deposit
        #[arg(short, long)]
        amount: String,
        
        /// Account to use for deposit (if not specified, uses the first account)
        #[arg(short, long)]
        account: Option<String>,
    },
    
    /// Withdraw NEO from NeoCompound
    Withdraw {
        /// Amount of NEO to withdraw
        #[arg(short, long)]
        amount: String,
        
        /// Account to withdraw to (if not specified, uses the first account)
        #[arg(short, long)]
        account: Option<String>,
    },
    
    /// Claim accumulated rewards
    ClaimRewards {
        /// Account to claim rewards (if not specified, uses the first account)
        #[arg(short, long)]
        account: Option<String>,
    },
    
    /// Get current NeoCompound statistics
    GetStats {
        /// Account to get stats for (if not specified, uses the first account)
        #[arg(short, long)]
        account: Option<String>,
    },
}

pub async fn handle_defi_command(args: DefiArgs, state: &mut crate::commands::wallet::CliState) -> CliResult<()> {
    match args.command {
        DefiCommands::Flamingo { command } => handle_flamingo_command(command, state).await,
        DefiCommands::Neoburger { command } => handle_neoburger_command(command, state).await,
        DefiCommands::GrandShare { command } => handle_grandshare_command(command, state).await,
        DefiCommands::NeoCompound { command } => handle_neocompound_command(command, state).await,
        DefiCommands::Token(token_args) => handle_token_command(token_args, state).await,
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
            
            // Build transaction
            let mut tx_builder = TransactionBuilder::new();
            
            // Get current block height for valid until block calculation
            let network = state.config().network.clone();
            let rpc_client = Http::new(&network.rpc_url.as_str()).map_err(|e| CliError::Provider(e.to_string()))?;
            let chain_state = rpc_client.get_block_count().await
                .map_err(|e| CliError::Provider(format!("Failed to get current block height: {}", e)))?;
            
            // Transaction is valid for the next 5760 blocks (~24 hours with 15s blocks)
            let valid_until_block = chain_state + 5760;
            
            // Create signers for the transaction
            let sender_hash = account_obj.get_script_hash();
            let signers = vec![
                Signer::with_scope(sender_hash, WitnessScope::CalledByEntry)
            ];
            
            tx_builder = tx_builder
                .version(0)
                .nonce((rand::random::<u32>() % 1000000) as u32)
                .map_err(|e| CliError::TransactionBuilder(e.to_string()))?
                .valid_until_block(valid_until_block)
                .map_err(|e| CliError::TransactionBuilder(e.to_string()))?
                .signers(signers)
                .map_err(|e| CliError::TransactionBuilder(e.to_string()))?;
            
            // Create a script to execute the swap
            let script = ScriptBuilder::new()
                .contract_call(
                    flamingo_contract.hash(),
                    "swap",
                    &[
                        ContractParameter::hash160(from_token_hash),
                        ContractParameter::hash160(to_token_hash),
                        ContractParameter::integer(amount_val.to_i64().unwrap()),
                        ContractParameter::integer(min_return_val.to_i64().unwrap()),
                        ContractParameter::hash160(sender_hash)
                    ],
                    None
                )
                .map_err(|e| CliError::Builder(e.to_string()))?
                .build();
            
            tx_builder = tx_builder.script(script)
                .map_err(|e| CliError::TransactionBuilder(e.to_string()))?;
            
            // Sign transaction
            let signed_tx = sign_transaction(&account_obj, tx_builder, &password)?;
            
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
            let signed_tx = sign_transaction(&account_obj, tx_builder, &password)?;
            
            // Send transaction
            let result = rpc_client.send_raw_transaction(&signed_tx).await
                .map_err(|e| CliError::Network(format!("Failed to send transaction: {}", e)))?;
            
            print_success("Add liquidity transaction sent successfully");
            println!("Transaction hash: {}", result.hash);
        },
    }
    
    Ok(())
}

// Utility function to resolve token symbols to script hashes
fn resolve_token_hash(token: &str) -> Result<H160, CliError> {
    match token.to_lowercase().as_str() {
        "neo" => H160::from_hex("ef4073a0f2b305a38ec4050e4d3d28bc40ea63f5")
            .map_err(|e| CliError::InvalidInput(format!("Invalid token: {}", e))),
        "gas" => H160::from_hex("d2a4cff31913016155e38e474a2c06d08be276cf") 
            .map_err(|e| CliError::InvalidInput(format!("Invalid token: {}", e))),
        _ => H160::from_hex(token)
            .map_err(|e| CliError::InvalidInput(format!("Invalid token hash: {}", e)))
    }
}

// Parse amount string to correct token units based on decimals
async fn parse_amount<P: JsonRpcProvider>(amount: &str, token: &str, rpc_client: &P) -> Result<i64, CliError> {
    // Parse the amount as f64 first
    let amount_f64 = amount.parse::<f64>()
        .map_err(|_| CliError::InvalidInput(format!("Invalid amount: {}", amount)))?;
    
    // Get token decimals to convert to raw units
    let token_hash = resolve_token_hash(token)?;
    let token_contract = Nep17Contract::new(token_hash, rpc_client.clone());
    let decimals = token_contract.decimals().await
        .map_err(|e| CliError::Sdk(format!("Failed to get token decimals: {}", e)))?;
    
    // Convert to raw units
    let raw_amount = (amount_f64 * 10_f64.powf(decimals.into())) as i64;
    
    Ok(raw_amount)
}

// Helper function to get account from wallet
fn get_account(account: Option<String>, wallet: &Wallet) -> Result<Account, CliError> {
    let accounts = wallet.get_accounts();
    if accounts.is_empty() {
        return Err(CliError::Wallet("No accounts found in wallet".to_string()));
    }
    
    // If no account specified, use the first one
    if account.is_none() {
        return Ok(accounts[0].clone());
    }
    
    // Find the account by address
    let account_address = account.unwrap();
    
    // Find account in wallet
    wallet.get_accounts().iter()
        .find(|a| a.get_address() == account_address)
        .ok_or_else(|| CliError::Wallet(format!("Account not found: {}", account_address)))
        .map(|a| a.clone())
}

// Similar implementation for handle_neoburger_command, handle_grandshare_command, and handle_neocompound_command
// ... 

fn sign_transaction<P>(account: &Account, mut tx_builder: TransactionBuilder<P>, password: &str) -> Result<Transaction<P>, CliError>
where
    P: JsonRpcProvider + 'static 
{
    // Get the private key from the account using the password
    let private_key = account.decrypt_private_key(password)
        .map_err(|e| CliError::Wallet(format!("Failed to decrypt private key: {}", e)))?;
    
    // Build and sign the transaction
    let tx = tx_builder.build()
        .map_err(|e| CliError::Transaction(format!("Failed to build transaction: {}", e)))?;
        
    let signed_tx = tx.sign(&private_key)
        .map_err(|e| CliError::Transaction(format!("Failed to sign transaction: {}", e)))?;
        
    Ok(signed_tx)
}

async fn token_handler(
    token_id: Option<String>,
    token_command: Option<TokenCommand>,
    addr: Option<String>,
    state: &mut crate::commands::wallet::CliState,
) -> CliResult<()> {
    let current_network = state.config().network.clone();
    let rpc_client = Http::new(&current_network.rpc_url.as_str())
        .map_err(|e| CliError::Provider(e.to_string()))?;
    
    // Check token information
    if let Some(token_id) = token_id {
        // Create NEP17 contract interface
        let token_hash = resolve_token_hash(&token_id)?;
        let token_contract = Nep17Contract::new(token_hash, rpc_client.clone());
        
        // Get token information
        let token_name = token_contract.symbol().await
            .map_err(|e| CliError::Sdk(format!("Failed to get token symbol: {}", e)))?;
        
        let token_decimals = token_contract.decimals().await
            .map_err(|e| CliError::Sdk(format!("Failed to get token decimals: {}", e)))?;
        
        let total_supply = token_contract.total_supply().await
            .map_err(|e| CliError::Sdk(format!("Failed to get token total supply: {}", e)))?;
        
        // Print token information
        println!("Token Information:");
        println!("- Name:         {}", token_name);
        println!("- Hash:         {}", token_hash);
        println!("- Decimals:     {}", token_decimals);
        println!("- Total Supply: {}", total_supply.as_u64() as f64 / 10_f64.powf(token_decimals.into()));
        
        // Check if we should show balance for an address
        if let Some(addr) = addr {
            let script_hash = if addr.starts_with("N") {
                // Convert NEO address to script hash
                Address::from_string(&addr)
                    .map_err(|e| CliError::InvalidInput(format!("Invalid address: {}", e)))?
                    .get_script_hash()
            } else {
                // Direct script hash
                H160::from_hex(&addr)
                    .map_err(|e| CliError::InvalidInput(format!("Invalid script hash: {}", e)))?
            };
            
            // Get balance
            let balance = token_contract.balance_of(&script_hash).await
                .map_err(|e| CliError::Sdk(format!("Failed to get balance: {}", e)))?;
            
            let formatted_balance = balance.as_u64() as f64 / 10_f64.powf(token_decimals.into());
            println!("- Balance:      {} {}", formatted_balance, token_name);
        }
        
        // Handle token command if provided
        if let Some(cmd) = token_command {
            match cmd {
                TokenCommand::Allowance { owner, spender } => {
                    // Convert addresses to script hashes
                    let owner_hash = if owner.starts_with("N") {
                        Address::from_string(&owner)
                            .map_err(|e| CliError::InvalidInput(format!("Invalid owner address: {}", e)))?
                            .get_script_hash()
                    } else {
                        H160::from_hex(&owner)
                            .map_err(|e| CliError::InvalidInput(format!("Invalid owner script hash: {}", e)))?
                    };
                    
                    let spender_hash = if spender.starts_with("N") {
                        Address::from_string(&spender)
                            .map_err(|e| CliError::InvalidInput(format!("Invalid spender address: {}", e)))?
                            .get_script_hash()
                    } else {
                        H160::from_hex(&spender)
                            .map_err(|e| CliError::InvalidInput(format!("Invalid spender script hash: {}", e)))?
                    };
                    
                    // Get allowance
                    // Note: This assumes NEP-17 token with allowance method, which not all tokens might have
                    // In a real implementation, we would check if the token supports this feature
                    println!("Checking allowance...");
                    // This would typically be token_contract.allowance(&owner_hash, &spender_hash)
                    // But for now let's just print the information we would use
                    println!("Owner: {}", owner);
                    println!("Spender: {}", spender);
                },
                
                TokenCommand::Approve { spender, amount, from, password } => {
                    if state.wallet.is_none() {
                        print_error("No wallet is currently open");
                        return Err(CliError::Wallet("No wallet is currently open".to_string()));
                    }
                    
                    // Get account from wallet
                    let account_obj = get_account(from, state.wallet.as_ref().unwrap())?;
                    
                    // Convert spender to script hash
                    let spender_hash = if spender.starts_with("N") {
                        Address::from_string(&spender)
                            .map_err(|e| CliError::InvalidInput(format!("Invalid spender address: {}", e)))?
                            .get_script_hash()
                    } else {
                        H160::from_hex(&spender)
                            .map_err(|e| CliError::InvalidInput(format!("Invalid spender script hash: {}", e)))?
                    };
                    
                    // Parse amount
                    let amount_raw = parse_amount(&amount, &token_id, &rpc_client).await?;
                    
                    // Get password for signing if not provided
                    let pwd = match password {
                        Some(p) => p,
                        None => prompt_password("Enter wallet password")?,
                    };
                    
                    print_info(&format!("Approving {} {} for spending by {}", amount, token_name, spender));
                    
                    // Build and sign transaction
                    // This would be a full implementation of token approval process
                    // For now, let's just print what we would do
                    println!("Building approval transaction...");
                    println!("Token: {} ({})", token_name, token_hash);
                    println!("Amount: {}", amount_raw);
                    println!("Spender: {}", spender_hash);
                    println!("Account: {}", account_obj.get_address());
                    
                    print_success("Approval successful (simulated)");
                }
            }
        }
    } else {
        // No token specified, list available tokens
        print_info("Available tokens:");
        println!("1. NEO (ef4073a0f2b305a38ec4050e4d3d28bc40ea63f5)");
        println!("2. GAS (d2a4cff31913016155e38e474a2c06d08be276cf)");
        
        // If connected to network, could list more tokens from the blockchain
    }
    
    Ok(())
}

async fn handle_token_command(args: TokenArgs, state: &mut crate::commands::wallet::CliState) -> CliResult<()> {
    // Check if token_id, command, or address are provided
    if args.token_id.is_some() || args.command.is_some() || args.address.is_some() {
        // Call token_handler with the provided arguments
        token_handler(args.token_id, args.command, args.address, state).await
    } else {
        // List available tokens if no specific arguments
        print_info("Available tokens:");
        println!("1. NEO (NeoToken)");
        println!("2. GAS (GasToken)");
        // Add more tokens as needed
        
        Ok(())
    }
}

async fn handle_neoburger_command(args: NeoburgerCommands, state: &mut crate::commands::wallet::CliState) -> CliResult<()> {
    if state.wallet.is_none() {
        print_error("No wallet is currently open");
        return Err(CliError::Wallet("No wallet is currently open".to_string()));
    }
    
    if state.rpc_client.is_none() {
        print_error("No RPC client is connected. Please connect to a node first.");
        return Err(CliError::Network("No RPC client is connected".to_string()));
    }
    
    let rpc_client = state.rpc_client.as_ref().unwrap();
    let neoburger_contract = NeoburgerContract::new(Some(rpc_client));
    
    match args {
        NeoburgerCommands::Wrap { amount, account } => {
            // Parse NEO amount
            let amount_val = parse_amount(&amount, "neo", rpc_client).await?;
            
            // Get account
            let account_obj = get_account(account, state.wallet.as_ref().unwrap())?;
            
            print_info(&format!("Wrapping {} NEO to bNEO...", amount));
            
            // Get password for signing
            let password = prompt_password("Enter wallet password")?;
            
            // Create wrap transaction
            let tx_builder = neoburger_contract.wrap_neo(amount_val, &account_obj)
                .await.map_err(|e| CliError::Sdk(format!("Failed to create wrap transaction: {}", e)))?;
            
            // Sign transaction
            let signed_tx = sign_transaction(&account_obj, tx_builder, &password)?;
            
            // Send transaction
            let result = rpc_client.send_raw_transaction(&signed_tx).await
                .map_err(|e| CliError::Network(format!("Failed to send transaction: {}", e)))?;
            
            print_success("Wrap transaction sent successfully");
            println!("Transaction hash: {}", result.hash);
        },
        
        NeoburgerCommands::Unwrap { amount, account } => {
            // Parse bNEO amount
            let amount_val = parse_amount(&amount, "bneo", rpc_client).await?;
            
            // Get account
            let account_obj = get_account(account, state.wallet.as_ref().unwrap())?;
            
            print_info(&format!("Unwrapping {} bNEO to NEO...", amount));
            
            // Get password for signing
            let password = prompt_password("Enter wallet password")?;
            
            // Create unwrap transaction
            let tx_builder = neoburger_contract.unwrap_neo(amount_val, &account_obj)
                .await.map_err(|e| CliError::Sdk(format!("Failed to create unwrap transaction: {}", e)))?;
            
            // Sign transaction
            let signed_tx = sign_transaction(&account_obj, tx_builder, &password)?;
            
            // Send transaction
            let result = rpc_client.send_raw_transaction(&signed_tx).await
                .map_err(|e| CliError::Network(format!("Failed to send transaction: {}", e)))?;
            
            print_success("Unwrap transaction sent successfully");
            println!("Transaction hash: {}", result.hash);
        },
        
        NeoburgerCommands::ClaimGas { account } => {
            // Get account
            let account_obj = get_account(account, state.wallet.as_ref().unwrap())?;
            
            print_info("Claiming GAS rewards from bNEO...");
            
            // Get password for signing
            let password = prompt_password("Enter wallet password")?;
            
            // Create claim transaction
            let tx_builder = neoburger_contract.claim_gas(&account_obj)
                .await.map_err(|e| CliError::Sdk(format!("Failed to create claim transaction: {}", e)))?;
            
            // Sign transaction
            let signed_tx = sign_transaction(&account_obj, tx_builder, &password)?;
            
            // Send transaction
            let result = rpc_client.send_raw_transaction(&signed_tx).await
                .map_err(|e| CliError::Network(format!("Failed to send transaction: {}", e)))?;
            
            print_success("Claim transaction sent successfully");
            println!("Transaction hash: {}", result.hash);
        },
        
        NeoburgerCommands::GetRate => {
            print_info("Getting current NEO to bNEO exchange rate...");
            
            // Get exchange rate
            let rate = neoburger_contract.get_rate().await
                .map_err(|e| CliError::Sdk(format!("Failed to get exchange rate: {}", e)))?;
            
            println!("Current exchange rate: 1 NEO = {} bNEO", rate);
        },
    }
    
    Ok(())
}

async fn handle_grandshare_command(args: GrandShareCommands, state: &mut crate::commands::wallet::CliState) -> CliResult<()> {
    if state.wallet.is_none() {
        print_error("No wallet is currently open");
        return Err(CliError::Wallet("No wallet is currently open".to_string()));
    }
    
    if state.rpc_client.is_none() {
        print_error("No RPC client is connected. Please connect to a node first.");
        return Err(CliError::Network("No RPC client is connected".to_string()));
    }
    
    let rpc_client = state.rpc_client.as_ref().unwrap();
    let grandshare_contract = GrandShareContract::new(Some(rpc_client));
    
    match args {
        GrandShareCommands::Stake { amount, account } => {
            // Parse NEO amount
            let amount_val = parse_amount(&amount, "neo", rpc_client).await?;
            
            // Get account
            let account_obj = get_account(account, state.wallet.as_ref().unwrap())?;
            
            print_info(&format!("Staking {} NEO in GrandShare...", amount));
            
            // Get password for signing
            let password = prompt_password("Enter wallet password")?;
            
            // Create stake transaction
            let tx_builder = grandshare_contract.stake(amount_val, &account_obj)
                .await.map_err(|e| CliError::Sdk(format!("Failed to create stake transaction: {}", e)))?;
            
            // Sign transaction
            let signed_tx = sign_transaction(&account_obj, tx_builder, &password)?;
            
            // Send transaction
            let result = rpc_client.send_raw_transaction(&signed_tx).await
                .map_err(|e| CliError::Network(format!("Failed to send transaction: {}", e)))?;
            
            print_success("Stake transaction sent successfully");
            println!("Transaction hash: {}", result.hash);
        },
        
        GrandShareCommands::Withdraw { amount, account } => {
            // Parse NEO amount
            let amount_val = parse_amount(&amount, "neo", rpc_client).await?;
            
            // Get account
            let account_obj = get_account(account, state.wallet.as_ref().unwrap())?;
            
            print_info(&format!("Withdrawing {} NEO from GrandShare staking...", amount));
            
            // Get password for signing
            let password = prompt_password("Enter wallet password")?;
            
            // Create withdraw transaction
            let tx_builder = grandshare_contract.withdraw(amount_val, &account_obj)
                .await.map_err(|e| CliError::Sdk(format!("Failed to create withdraw transaction: {}", e)))?;
            
            // Sign transaction
            let signed_tx = sign_transaction(&account_obj, tx_builder, &password)?;
            
            // Send transaction
            let result = rpc_client.send_raw_transaction(&signed_tx).await
                .map_err(|e| CliError::Network(format!("Failed to send transaction: {}", e)))?;
            
            print_success("Withdraw transaction sent successfully");
            println!("Transaction hash: {}", result.hash);
        },
        
        GrandShareCommands::ClaimRewards { account } => {
            // Get account
            let account_obj = get_account(account, state.wallet.as_ref().unwrap())?;
            
            print_info("Claiming rewards from GrandShare staking...");
            
            // Get password for signing
            let password = prompt_password("Enter wallet password")?;
            
            // Create claim transaction
            let tx_builder = grandshare_contract.claim_rewards(&account_obj)
                .await.map_err(|e| CliError::Sdk(format!("Failed to create claim transaction: {}", e)))?;
            
            // Sign transaction
            let signed_tx = sign_transaction(&account_obj, tx_builder, &password)?;
            
            // Send transaction
            let result = rpc_client.send_raw_transaction(&signed_tx).await
                .map_err(|e| CliError::Network(format!("Failed to send transaction: {}", e)))?;
            
            print_success("Claim rewards transaction sent successfully");
            println!("Transaction hash: {}", result.hash);
        },
        
        GrandShareCommands::GetInfo { account } => {
            // Get account if specified
            let account_hash = if let Some(acc) = account {
                if acc.starts_with("N") {
                    // Convert NEO address to script hash
                    Address::from_string(&acc)
                        .map_err(|e| CliError::InvalidInput(format!("Invalid address: {}", e)))?
                        .get_script_hash()
                } else {
                    // Direct script hash
                    H160::from_hex(&acc)
                        .map_err(|e| CliError::InvalidInput(format!("Invalid script hash: {}", e)))?
                }
            } else if let Some(wallet) = &state.wallet {
                // Use first account in wallet
                let accounts = wallet.get_accounts();
                if accounts.is_empty() {
                    return Err(CliError::Wallet("No accounts in wallet".to_string()));
                }
                accounts[0].get_script_hash()
            } else {
                return Err(CliError::Wallet("No wallet is currently open".to_string()));
            };
            
            print_info("Getting GrandShare staking information...");
            
            // Get staking info
            let staked_amount = grandshare_contract.get_staked_amount(&account_hash).await
                .map_err(|e| CliError::Sdk(format!("Failed to get staked amount: {}", e)))?;
            
            let rewards = grandshare_contract.get_rewards(&account_hash).await
                .map_err(|e| CliError::Sdk(format!("Failed to get rewards: {}", e)))?;
            
            // Print staking information
            println!("Staking Information:");
            println!("- Staked amount: {:.8} NEO", staked_amount.as_u64() as f64 / 100_000_000.0);
            println!("- Claimable rewards: {:.8} GAS", rewards.as_u64() as f64 / 100_000_000.0);
        },
    }
    
    Ok(())
}

async fn handle_neocompound_command(args: NeoCompoundCommands, state: &mut crate::commands::wallet::CliState) -> CliResult<()> {
    if state.wallet.is_none() {
        print_error("No wallet is currently open");
        return Err(CliError::Wallet("No wallet is currently open".to_string()));
    }
    
    if state.rpc_client.is_none() {
        print_error("No RPC client is connected. Please connect to a node first.");
        return Err(CliError::Network("No RPC client is connected".to_string()));
    }
    
    let rpc_client = state.rpc_client.as_ref().unwrap();
    let neocompound_contract = NeoCompoundContract::new(Some(rpc_client));
    
    match args {
        NeoCompoundCommands::Deposit { amount, account } => {
            // Parse NEO amount
            let amount_val = parse_amount(&amount, "neo", rpc_client).await?;
            
            // Get account
            let account_obj = get_account(account, state.wallet.as_ref().unwrap())?;
            
            print_info(&format!("Depositing {} NEO to NeoCompound...", amount));
            
            // Get password for signing
            let password = prompt_password("Enter wallet password")?;
            
            // Create deposit transaction
            let tx_builder = neocompound_contract.deposit(amount_val, &account_obj)
                .await.map_err(|e| CliError::Sdk(format!("Failed to create deposit transaction: {}", e)))?;
            
            // Sign transaction
            let signed_tx = sign_transaction(&account_obj, tx_builder, &password)?;
            
            // Send transaction
            let result = rpc_client.send_raw_transaction(&signed_tx).await
                .map_err(|e| CliError::Network(format!("Failed to send transaction: {}", e)))?;
            
            print_success("Deposit transaction sent successfully");
            println!("Transaction hash: {}", result.hash);
        },
        
        NeoCompoundCommands::Withdraw { amount, account } => {
            // Parse NEO amount
            let amount_val = parse_amount(&amount, "neo", rpc_client).await?;
            
            // Get account
            let account_obj = get_account(account, state.wallet.as_ref().unwrap())?;
            
            print_info(&format!("Withdrawing {} NEO from NeoCompound...", amount));
            
            // Get password for signing
            let password = prompt_password("Enter wallet password")?;
            
            // Create withdraw transaction
            let tx_builder = neocompound_contract.withdraw(amount_val, &account_obj)
                .await.map_err(|e| CliError::Sdk(format!("Failed to create withdraw transaction: {}", e)))?;
            
            // Sign transaction
            let signed_tx = sign_transaction(&account_obj, tx_builder, &password)?;
            
            // Send transaction
            let result = rpc_client.send_raw_transaction(&signed_tx).await
                .map_err(|e| CliError::Network(format!("Failed to send transaction: {}", e)))?;
            
            print_success("Withdraw transaction sent successfully");
            println!("Transaction hash: {}", result.hash);
        },
        
        NeoCompoundCommands::ClaimRewards { account } => {
            // Get account
            let account_obj = get_account(account, state.wallet.as_ref().unwrap())?;
            
            print_info("Claiming rewards from NeoCompound...");
            
            // Get password for signing
            let password = prompt_password("Enter wallet password")?;
            
            // Create claim transaction
            let tx_builder = neocompound_contract.claim_rewards(&account_obj)
                .await.map_err(|e| CliError::Sdk(format!("Failed to create claim transaction: {}", e)))?;
            
            // Sign transaction
            let signed_tx = sign_transaction(&account_obj, tx_builder, &password)?;
            
            // Send transaction
            let result = rpc_client.send_raw_transaction(&signed_tx).await
                .map_err(|e| CliError::Network(format!("Failed to send transaction: {}", e)))?;
            
            print_success("Claim rewards transaction sent successfully");
            println!("Transaction hash: {}", result.hash);
        },
        
        NeoCompoundCommands::GetStats { account } => {
            // Get account if specified
            let account_hash = if let Some(acc) = account {
                if acc.starts_with("N") {
                    // Convert NEO address to script hash
                    Address::from_string(&acc)
                        .map_err(|e| CliError::InvalidInput(format!("Invalid address: {}", e)))?
                        .get_script_hash()
                } else {
                    // Direct script hash
                    H160::from_hex(&acc)
                        .map_err(|e| CliError::InvalidInput(format!("Invalid script hash: {}", e)))?
                }
            } else if let Some(wallet) = &state.wallet {
                // Use first account in wallet
                let accounts = wallet.get_accounts();
                if accounts.is_empty() {
                    return Err(CliError::Wallet("No accounts in wallet".to_string()));
                }
                accounts[0].get_script_hash()
            } else {
                return Err(CliError::Wallet("No wallet is currently open".to_string()));
            };
            
            print_info("Getting NeoCompound statistics...");
            
            // Get account stats
            let deposit_amount = neocompound_contract.get_deposit(&account_hash).await
                .map_err(|e| CliError::Sdk(format!("Failed to get deposit amount: {}", e)))?;
            
            let rewards = neocompound_contract.get_claimable_rewards(&account_hash).await
                .map_err(|e| CliError::Sdk(format!("Failed to get claimable rewards: {}", e)))?;
            
            let apy = neocompound_contract.get_apy().await
                .map_err(|e| CliError::Sdk(format!("Failed to get APY: {}", e)))?;
            
            // Print statistics
            println!("NeoCompound Statistics:");
            println!("- Deposited: {:.8} NEO", deposit_amount.as_u64() as f64 / 100_000_000.0);
            println!("- Claimable rewards: {:.8} GAS", rewards.as_u64() as f64 / 100_000_000.0);
            println!("- Current APY: {}%", apy);
        },
    }
    
    Ok(())
} 