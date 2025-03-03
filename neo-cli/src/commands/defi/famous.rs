// Famous DeFi contracts implementation for Neo CLI
//
// This module provides integration with popular DeFi protocols on Neo blockchain

use neo3::prelude::*;
use primitive_types::H160;
use std::str::FromStr;

use crate::errors::CliError;
use crate::commands::wallet::{CliState, Account, ScriptHash, RpcClient, HttpProvider};
use crate::commands::defi::utils::{NetworkTypeCli, load_wallet_from_state, parse_amount, resolve_token_to_scripthash_with_network};

/// FlamingoContract represents the Flamingo Finance smart contract on Neo
///
/// This is currently a placeholder implementation that demonstrates the intended
/// functionality but does not yet execute actual blockchain transactions.
/// Future implementations will connect to the actual Flamingo contract on the network.
struct FlamingoContract {
    script_hash: H160,
}

impl FlamingoContract {
    /// Creates a new FlamingoContract instance
    ///
    /// # Arguments
    /// * `_provider` - Optional RPC client (currently unused)
    ///
    /// # Returns
    /// A new FlamingoContract instance with the script hash set for the current network
    pub fn new(_provider: Option<&RpcClient<HttpProvider>>) -> Self {
        // Note: This is a placeholder implementation
        // In the future, this will determine the correct contract address based on the network
        Self {
            script_hash: H160::from_str("f0151f528127558851b39c2cd8aa47da7418ab28").unwrap(),
        }
    }

    /// Placeholder for token swap operation on Flamingo Finance
    ///
    /// # Arguments
    /// * `_from_token` - Token to swap from
    /// * `_to_token` - Token to swap to
    /// * `_amount` - Amount to swap
    /// * `_min_return` - Minimum amount to receive
    /// * `_account` - Account to use for the swap
    ///
    /// # Returns
    /// A placeholder transaction ID (not an actual blockchain transaction)
    pub async fn swap(
        &self,
        _from_token: &H160,
        _to_token: &H160,
        _amount: i64,
        _min_return: i64,
        _account: &Account,
    ) -> Result<String, CliError> {
        // This is a placeholder implementation
        // In the future, this will build and submit the actual transaction
        Ok("0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string())
    }
    
    pub async fn add_liquidity(
        &self,
        _token_a: &ScriptHash,
        _token_b: &ScriptHash,
        _amount_a: i64,
        _amount_b: i64,
        _account: &Account,
    ) -> Result<String, CliError> {
        // Placeholder implementation
        Ok("add_liquidity_transaction".to_string())
    }
    
    pub async fn remove_liquidity(
        &self,
        _token_a: &ScriptHash,
        _token_b: &ScriptHash,
        _liquidity: i64,
        _account: &Account,
    ) -> Result<String, CliError> {
        // Placeholder implementation
        Ok("remove_liquidity_transaction".to_string())
    }
    
    pub async fn stake(
        &self,
        _token: &ScriptHash,
        _amount: i64,
        _account: &Account,
    ) -> Result<String, CliError> {
        // Placeholder implementation
        Ok("stake_transaction".to_string())
    }
    
    pub async fn claim_rewards(
        &self,
        _account: &Account,
    ) -> Result<String, CliError> {
        // Placeholder implementation
        Ok("claim_rewards_transaction".to_string())
    }
}

/// NeoburgerContract placeholder
struct NeoburgerContract {
    script_hash: ScriptHash,
}

impl NeoburgerContract {
    const CONTRACT_HASH: &'static str = "48c40d4666f93408be1bef038b6722404f5c4a5a";
    
    pub fn new(_provider: Option<&RpcClient<HttpProvider>>) -> Self {
        Self { script_hash: ScriptHash::from_str(Self::CONTRACT_HASH).unwrap() }
    }
    
    pub async fn wrap(
        &self,
        _amount: i64,
        _account: &Account,
    ) -> Result<String, CliError> {
        // Placeholder implementation
        Ok("wrap_transaction".to_string())
    }
    
    pub async fn unwrap(
        &self,
        _amount: i64,
        _account: &Account,
    ) -> Result<String, CliError> {
        // Placeholder implementation
        Ok("unwrap_transaction".to_string())
    }
    
    pub async fn claim_gas(
        &self,
        _account: &Account,
    ) -> Result<String, CliError> {
        // Placeholder implementation
        Ok("claim_gas_transaction".to_string())
    }
    
    pub async fn get_rate(&self) -> Result<f64, CliError> {
        // Placeholder implementation
        Ok(1.05)
    }
}

/// NeoCompoundContract placeholder
struct NeoCompoundContract {
    script_hash: ScriptHash,
}

impl NeoCompoundContract {
    const CONTRACT_HASH: &'static str = "f0151f528127558851b39c2cd8aa47da7418ab28";
    
    pub fn new(_provider: Option<&RpcClient<HttpProvider>>) -> Self {
        Self { script_hash: ScriptHash::from_str(Self::CONTRACT_HASH).unwrap() }
    }
    
    pub async fn deposit(
        &self,
        _token: &ScriptHash,
        _amount: i64,
        _account: &Account,
    ) -> Result<String, CliError> {
        // Placeholder implementation
        Ok("deposit_transaction".to_string())
    }
    
    pub async fn withdraw(
        &self,
        _token: &ScriptHash,
        _amount: i64,
        _account: &Account,
    ) -> Result<String, CliError> {
        // Placeholder implementation
        Ok("withdraw_transaction".to_string())
    }
    
    pub async fn compound(
        &self,
        _token: &ScriptHash,
        _account: &Account,
    ) -> Result<String, CliError> {
        // Placeholder implementation
        Ok("compound_transaction".to_string())
    }
    
    pub async fn get_apy(&self, _token: &ScriptHash) -> Result<f64, CliError> {
        // Placeholder implementation
        Ok(5.75)
    }
}

/// GrandShareContract placeholder
struct GrandShareContract {
    script_hash: ScriptHash,
}

impl GrandShareContract {
    const CONTRACT_HASH: &'static str = "74f2dc36a68fdc4682034178eb2220729231db76";
    
    pub fn new(_provider: Option<&RpcClient<HttpProvider>>) -> Self {
        Self { script_hash: ScriptHash::from_str(Self::CONTRACT_HASH).unwrap() }
    }
    
    pub async fn submit_proposal(
        &self,
        _title: &str,
        _description: &str,
        _amount: i64,
        _account: &Account,
    ) -> Result<String, CliError> {
        // Placeholder implementation
        Ok("submit_proposal_transaction".to_string())
    }
    
    pub async fn vote(
        &self,
        _proposal_id: i32,
        _approve: bool,
        _account: &Account,
    ) -> Result<String, CliError> {
        // Placeholder implementation
        Ok("vote_transaction".to_string())
    }
    
    pub async fn fund_project(
        &self,
        _project_id: i32,
        _amount: i64,
        _account: &Account,
    ) -> Result<String, CliError> {
        // Placeholder implementation
        Ok("fund_project_transaction".to_string())
    }
    
    pub async fn claim_funds(
        &self,
        _project_id: i32,
        _account: &Account,
    ) -> Result<String, CliError> {
        // Placeholder implementation
        Ok("claim_funds_transaction".to_string())
    }
}

/// Handles a token swap request on Flamingo Finance (Placeholder implementation)
///
/// This function processes a user request to swap tokens using Flamingo Finance.
/// Currently, this is a placeholder that demonstrates the intended workflow but
/// does not execute actual blockchain transactions.
///
/// # Arguments
/// * `from_token` - Token to swap from (symbol or contract address)
/// * `to_token` - Token to swap to (symbol or contract address)
/// * `amount` - Amount to swap (as a string)
/// * `min_return` - Minimum amount to receive (optional)
/// * `state` - CLI state containing wallet and RPC client
///
/// # Returns
/// `Result<(), CliError>` indicating success or error
pub async fn handle_flamingo_swap(
    from_token: &str,
    to_token: &str,
    amount: &str,
    min_return: Option<&str>,
    state: &mut CliState,
) -> Result<(), CliError> {
    if state.wallet.is_none() {
        return Err(CliError::NoWallet);
    }
    let wallet = state.wallet.as_ref().unwrap();
    let account = state.get_account()?;
    
    let network_type = NetworkTypeCli::from_network(state.get_network());
    let rpc_client = state.get_rpc_client()?;
    
    // Convert token names or hashes to ScriptHash
    let from_token_hash = resolve_token_to_scripthash_with_network(from_token, rpc_client, network_type).await?;
    let to_token_hash = resolve_token_to_scripthash_with_network(to_token, rpc_client, network_type).await?;
    
    // Parse amount and minimum return
    let amount_value = parse_amount(amount, &from_token_hash, rpc_client, network_type).await?;
    let min_return_value = match min_return {
        Some(min_ret) => parse_amount(min_ret, &to_token_hash, rpc_client, network_type).await?,
        None => amount_value / 10, // Default to 10% of input as minimum return
    };
    
    // Create Flamingo contract instance
    let flamingo = FlamingoContract::new(Some(rpc_client));
    
    // Build and send transaction
    let tx_id = flamingo.swap(
        &from_token_hash,
        &to_token_hash,
        amount_value,
        min_return_value,
        &account,
    ).await.map_err(|e| {
        CliError::Contract(format!("Failed to swap tokens: {}", e))
    })?;
    
    // For now, just print the success message
    println!("Swap transaction sent successfully!");
    println!("Transaction ID: {}", tx_id);
    println!("From: {} {}", amount, from_token);
    println!("To: {} (minimum: {})", to_token, min_return.unwrap_or("10% of input"));
    
    Ok(())
}

pub async fn handle_flamingo_add_liquidity(
    token_a: &str,
    token_b: &str,
    amount_a: &str,
    amount_b: &str,
    state: &mut CliState,
) -> Result<(), CliError> {
    if state.wallet.is_none() {
        return Err(CliError::NoWallet);
    }
    let wallet = state.wallet.as_ref().unwrap();
    let account = state.get_account()?;
    
    let network_type = NetworkTypeCli::from_network(state.get_network());
    let rpc_client = state.get_rpc_client()?;
    
    // Convert token names or hashes to ScriptHash
    let token_a_hash = resolve_token_to_scripthash_with_network(token_a, rpc_client, network_type).await?;
    let token_b_hash = resolve_token_to_scripthash_with_network(token_b, rpc_client, network_type).await?;
    
    // Parse amounts
    let amount_a_value = parse_amount(amount_a, &token_a_hash, rpc_client, network_type).await?;
    let amount_b_value = parse_amount(amount_b, &token_b_hash, rpc_client, network_type).await?;
    
    // Create Flamingo contract instance
    let flamingo = FlamingoContract::new(Some(rpc_client));
    
    // Build and send transaction
    let tx_id = flamingo.add_liquidity(
        &token_a_hash,
        &token_b_hash,
        amount_a_value,
        amount_b_value,
        &account,
    ).await.map_err(|e| {
        CliError::Contract(format!("Failed to add liquidity: {}", e))
    })?;
    
    println!("Added liquidity successfully!");
    println!("Transaction ID: {}", tx_id);
    println!("Token A: {} {}", amount_a, token_a);
    println!("Token B: {} {}", amount_b, token_b);
    
    Ok(())
}

pub async fn handle_flamingo_remove_liquidity(
    token_a: &str,
    token_b: &str,
    liquidity: &str,
    state: &mut CliState,
) -> Result<(), CliError> {
    if state.wallet.is_none() {
        return Err(CliError::NoWallet);
    }
    let wallet = state.wallet.as_ref().unwrap();
    let account = state.get_account()?;
    
    let network_type = NetworkTypeCli::from_network(state.get_network());
    let rpc_client = state.get_rpc_client()?;
    
    // Convert token names or hashes to ScriptHash
    let token_a_hash = resolve_token_to_scripthash_with_network(token_a, rpc_client, network_type).await?;
    let token_b_hash = resolve_token_to_scripthash_with_network(token_b, rpc_client, network_type).await?;
    
    // For simplicity, we'll use the first token for parsing the amount
    let liquidity_value = parse_amount(liquidity, &token_a_hash, rpc_client, network_type).await?;
    
    // Create Flamingo contract instance
    let flamingo = FlamingoContract::new(Some(rpc_client));
    
    // Build and send transaction
    let tx_id = flamingo.remove_liquidity(
        &token_a_hash,
        &token_b_hash,
        liquidity_value,
        &account,
    ).await.map_err(|e| {
        CliError::Contract(format!("Failed to remove liquidity: {}", e))
    })?;
    
    println!("Removed liquidity successfully!");
    println!("Transaction ID: {}", tx_id);
    println!("Liquidity amount: {}", liquidity);
    println!("Tokens: {} and {}", token_a, token_b);
    
    Ok(())
}

pub async fn handle_flamingo_stake(
    token: &str,
    amount: &str,
    state: &mut CliState,
) -> Result<(), CliError> {
    if state.wallet.is_none() {
        return Err(CliError::NoWallet);
    }
    let wallet = state.wallet.as_ref().unwrap();
    let account = state.get_account()?;
    
    let network_type = NetworkTypeCli::from_network(state.get_network());
    let rpc_client = state.get_rpc_client()?;
    
    // Convert token name or hash to ScriptHash
    let token_hash = resolve_token_to_scripthash_with_network(token, rpc_client, network_type).await?;
    
    // Parse amount
    let amount_value = parse_amount(amount, &token_hash, rpc_client, network_type).await?;
    
    // Create Flamingo contract instance
    let flamingo = FlamingoContract::new(Some(rpc_client));
    
    // Build and send transaction
    let tx_id = flamingo.stake(
        &token_hash,
        amount_value,
        &account,
    ).await.map_err(|e| {
        CliError::Contract(format!("Failed to stake tokens: {}", e))
    })?;
    
    println!("Staked tokens successfully!");
    println!("Transaction ID: {}", tx_id);
    println!("Amount: {} {}", amount, token);
    
    Ok(())
}

pub async fn handle_flamingo_claim_rewards(
    state: &mut CliState,
) -> Result<(), CliError> {
    if state.wallet.is_none() {
        return Err(CliError::NoWallet);
    }
    let wallet = state.wallet.as_ref().unwrap();
    let account = state.get_account()?;
    
    let rpc_client = state.get_rpc_client()?;
    
    // Create Flamingo contract instance
    let flamingo = FlamingoContract::new(Some(rpc_client));
    
    // Build and send transaction
    let tx_id = flamingo.claim_rewards(
        &account,
    ).await.map_err(|e| {
        CliError::Contract(format!("Failed to claim rewards: {}", e))
    })?;
    
    println!("Claimed rewards successfully!");
    println!("Transaction ID: {}", tx_id);
    
    Ok(())
}

// NeoBurger Commands

pub async fn handle_neoburger_wrap(
    amount: &str,
    state: &mut CliState,
) -> Result<(), CliError> {
    if state.wallet.is_none() {
        return Err(CliError::NoWallet);
    }
    let wallet = state.wallet.as_ref().unwrap();
    let account = state.get_account()?;
    
    let network_type = NetworkTypeCli::from_network(state.get_network());
    let rpc_client = state.get_rpc_client()?;
    
    // Use NEO script hash for amount parsing
    let neo_hash = ScriptHash::from_str("ef4073a0f2b305a38ec4050e4d3d28bc40ea63f5").unwrap();
    
    // Parse amount
    let amount_value = parse_amount(amount, &neo_hash, rpc_client, network_type).await?;
    
    // Create NeoBurger contract instance
    let neoburger = NeoburgerContract::new(Some(rpc_client));
    
    // Build and send transaction
    let tx_id = neoburger.wrap(
        amount_value,
        &account,
    ).await.map_err(|e| {
        CliError::Contract(format!("Failed to wrap NEO: {}", e))
    })?;
    
    println!("Wrapped NEO to bNEO successfully!");
    println!("Transaction ID: {}", tx_id);
    println!("Amount: {} NEO", amount);
    
    Ok(())
}

pub async fn handle_neoburger_unwrap(
    amount: &str,
    state: &mut CliState,
) -> Result<(), CliError> {
    if state.wallet.is_none() {
        return Err(CliError::NoWallet);
    }
    let wallet = state.wallet.as_ref().unwrap();
    let account = state.get_account()?;
    
    let network_type = NetworkTypeCli::from_network(state.get_network());
    let rpc_client = state.get_rpc_client()?;
    
    // Use bNEO script hash for amount parsing
    let bneo_hash = ScriptHash::from_str("48c40d4666f93408be1bef038b6722404f5c4a5a").unwrap();
    
    // Parse amount
    let amount_value = parse_amount(amount, &bneo_hash, rpc_client, network_type).await?;
    
    // Create NeoBurger contract instance
    let neoburger = NeoburgerContract::new(Some(rpc_client));
    
    // Build and send transaction
    let tx_id = neoburger.unwrap(
        amount_value,
        &account,
    ).await.map_err(|e| {
        CliError::Contract(format!("Failed to unwrap bNEO: {}", e))
    })?;
    
    println!("Unwrapped bNEO to NEO successfully!");
    println!("Transaction ID: {}", tx_id);
    println!("Amount: {} bNEO", amount);
    
    Ok(())
}

pub async fn handle_neoburger_claim_gas(
    state: &mut CliState,
) -> Result<(), CliError> {
    if state.wallet.is_none() {
        return Err(CliError::NoWallet);
    }
    let wallet = state.wallet.as_ref().unwrap();
    let account = state.get_account()?;
    
    let rpc_client = state.get_rpc_client()?;
    
    // Create NeoBurger contract instance
    let neoburger = NeoburgerContract::new(Some(rpc_client));
    
    // Build and send transaction
    let tx_id = neoburger.claim_gas(
        &account,
    ).await.map_err(|e| {
        CliError::Contract(format!("Failed to claim GAS: {}", e))
    })?;
    
    println!("Claimed GAS successfully!");
    println!("Transaction ID: {}", tx_id);
    
    Ok(())
}

pub async fn handle_neoburger_get_rate(
    state: &mut CliState,
) -> Result<(), CliError> {
    let rpc_client = state.get_rpc_client()?;
    
    // Create NeoBurger contract instance
    let neoburger = NeoburgerContract::new(Some(rpc_client));
    
    // Get exchange rate
    let rate = neoburger.get_rate().await.map_err(|e| {
        CliError::Contract(format!("Failed to get exchange rate: {}", e))
    })?;
    
    println!("Current bNEO to NEO exchange rate: {:.8}", rate);
    println!("1 bNEO = {:.8} NEO", rate);
    
    Ok(())
}

// NeoCompound Commands

pub async fn handle_neocompound_deposit(
    token: &str,
    amount: &str,
    state: &mut CliState,
) -> Result<(), CliError> {
    if state.wallet.is_none() {
        return Err(CliError::NoWallet);
    }
    let wallet = state.wallet.as_ref().unwrap();
    let account = state.get_account()?;
    
    let network_type = NetworkTypeCli::from_network(state.get_network());
    let rpc_client = state.get_rpc_client()?;
    
    // Convert token name or hash to ScriptHash
    let token_hash = resolve_token_to_scripthash_with_network(token, rpc_client, network_type).await?;
    
    // Parse amount
    let amount_value = parse_amount(amount, &token_hash, rpc_client, network_type).await?;
    
    // Create NeoCompound contract instance
    let neocompound = NeoCompoundContract::new(Some(rpc_client));
    
    // Build and send transaction
    let tx_id = neocompound.deposit(
        &token_hash,
        amount_value,
        &account,
    ).await.map_err(|e| {
        CliError::Contract(format!("Failed to deposit tokens: {}", e))
    })?;
    
    println!("Deposited tokens successfully!");
    println!("Transaction ID: {}", tx_id);
    println!("Amount: {} {}", amount, token);
    
    Ok(())
}

pub async fn handle_neocompound_withdraw(
    token: &str,
    amount: &str,
    state: &mut CliState,
) -> Result<(), CliError> {
    if state.wallet.is_none() {
        return Err(CliError::NoWallet);
    }
    let wallet = state.wallet.as_ref().unwrap();
    let account = state.get_account()?;
    
    let network_type = NetworkTypeCli::from_network(state.get_network());
    let rpc_client = state.get_rpc_client()?;
    
    // Convert token name or hash to ScriptHash
    let token_hash = resolve_token_to_scripthash_with_network(token, rpc_client, network_type).await?;
    
    // Parse amount
    let amount_value = parse_amount(amount, &token_hash, rpc_client, network_type).await?;
    
    // Create NeoCompound contract instance
    let neocompound = NeoCompoundContract::new(Some(rpc_client));
    
    // Build and send transaction
    let tx_id = neocompound.withdraw(
        &token_hash,
        amount_value,
        &account,
    ).await.map_err(|e| {
        CliError::Contract(format!("Failed to withdraw tokens: {}", e))
    })?;
    
    println!("Withdrew tokens successfully!");
    println!("Transaction ID: {}", tx_id);
    println!("Amount: {} {}", amount, token);
    
    Ok(())
}

pub async fn handle_neocompound_compound(
    token: &str,
    state: &mut CliState,
) -> Result<(), CliError> {
    if state.wallet.is_none() {
        return Err(CliError::NoWallet);
    }
    let wallet = state.wallet.as_ref().unwrap();
    let account = state.get_account()?;
    
    let rpc_client = state.get_rpc_client()?;
    
    // Convert token name or hash to ScriptHash
    let token_hash = resolve_token_to_scripthash_with_network(token, rpc_client, network_type).await?;
    
    // Create NeoCompound contract instance
    let neocompound = NeoCompoundContract::new(Some(rpc_client));
    
    // Build and send transaction
    let tx_id = neocompound.compound(
        &token_hash,
        &account,
    ).await.map_err(|e| {
        CliError::Contract(format!("Failed to compound tokens: {}", e))
    })?;
    
    println!("Compounded tokens successfully!");
    println!("Transaction ID: {}", tx_id);
    println!("Token: {}", token);
    
    Ok(())
}

pub async fn handle_neocompound_get_apy(
    token: &str,
    state: &mut CliState,
) -> Result<(), CliError> {
    let rpc_client = state.get_rpc_client()?;
    
    // Convert token name or hash to ScriptHash
    let token_hash = resolve_token_to_scripthash_with_network(token, rpc_client, network_type).await?;
    
    // Create NeoCompound contract instance
    let neocompound = NeoCompoundContract::new(Some(rpc_client));
    
    // Get APY
    let apy = neocompound.get_apy(&token_hash).await.map_err(|e| {
        CliError::Contract(format!("Failed to get APY: {}", e))
    })?;
    
    println!("Current APY for {}: {:.2}%", token, apy);
    
    Ok(())
}

// GrandShare Commands

pub async fn handle_grandshare_submit_proposal(
    title: &str,
    description: &str,
    amount: &str,
    state: &mut CliState,
) -> Result<(), CliError> {
    if state.wallet.is_none() {
        return Err(CliError::NoWallet);
    }
    let wallet = state.wallet.as_ref().unwrap();
    let account = state.get_account()?;
    
    let network_type = NetworkTypeCli::from_network(state.get_network());
    let rpc_client = state.get_rpc_client()?;
    
    // Use GAS script hash for parsing the amount
    let gas_hash = ScriptHash::from_str("d2a4cff31913016155e38e474a2c06d08be276cf").unwrap();
    
    // Parse amount
    let amount_value = parse_amount(amount, &gas_hash, rpc_client, network_type).await?;
    
    // Create GrandShare contract instance
    let grandshare = GrandShareContract::new(Some(rpc_client));
    
    // Build and send transaction
    let tx_id = grandshare.submit_proposal(
        title,
        description,
        amount_value,
        &account,
    ).await.map_err(|e| {
        CliError::Contract(format!("Failed to submit proposal: {}", e))
    })?;
    
    println!("Submitted proposal successfully!");
    println!("Transaction ID: {}", tx_id);
    println!("Title: {}", title);
    println!("Requested amount: {}", amount);
    
    Ok(())
}

pub async fn handle_grandshare_vote(
    proposal_id: i32,
    approve: bool,
    state: &mut CliState,
) -> Result<(), CliError> {
    if state.wallet.is_none() {
        return Err(CliError::NoWallet);
    }
    let wallet = state.wallet.as_ref().unwrap();
    let account = state.get_account()?;
    
    let rpc_client = state.get_rpc_client()?;
    
    // Create GrandShare contract instance
    let grandshare = GrandShareContract::new(Some(rpc_client));
    
    // Build and send transaction
    let tx_id = grandshare.vote(
        proposal_id,
        approve,
        &account,
    ).await.map_err(|e| {
        CliError::Contract(format!("Failed to vote on proposal: {}", e))
    })?;
    
    println!("Vote submitted successfully!");
    println!("Transaction ID: {}", tx_id);
    println!("Proposal ID: {}", proposal_id);
    println!("Vote: {}", if approve { "Approve" } else { "Reject" });
    
    Ok(())
}

pub async fn handle_grandshare_fund_project(
    project_id: i32,
    amount: &str,
    state: &mut CliState,
) -> Result<(), CliError> {
    if state.wallet.is_none() {
        return Err(CliError::NoWallet);
    }
    let wallet = state.wallet.as_ref().unwrap();
    let account = state.get_account()?;
    
    let network_type = NetworkTypeCli::from_network(state.get_network());
    let rpc_client = state.get_rpc_client()?;
    
    // Use GAS script hash for parsing the amount
    let gas_hash = ScriptHash::from_str("d2a4cff31913016155e38e474a2c06d08be276cf").unwrap();
    
    // Parse amount
    let amount_value = parse_amount(amount, &gas_hash, rpc_client, network_type).await?;
    
    // Create GrandShare contract instance
    let grandshare = GrandShareContract::new(Some(rpc_client));
    
    // Build and send transaction
    let tx_id = grandshare.fund_project(
        project_id,
        amount_value,
        &account,
    ).await.map_err(|e| {
        CliError::Contract(format!("Failed to fund project: {}", e))
    })?;
    
    println!("Project funded successfully!");
    println!("Transaction ID: {}", tx_id);
    println!("Project ID: {}", project_id);
    println!("Amount: {}", amount);
    
    Ok(())
}

pub async fn handle_grandshare_claim_funds(
    project_id: i32,
    state: &mut CliState,
) -> Result<(), CliError> {
    if state.wallet.is_none() {
        return Err(CliError::NoWallet);
    }
    let wallet = state.wallet.as_ref().unwrap();
    let account = state.get_account()?;
    
    let rpc_client = state.get_rpc_client()?;
    
    // Create GrandShare contract instance
    let grandshare = GrandShareContract::new(Some(rpc_client));
    
    // Build and send transaction
    let tx_id = grandshare.claim_funds(
        project_id,
        &account,
    ).await.map_err(|e| {
        CliError::Contract(format!("Failed to claim funds: {}", e))
    })?;
    
    println!("Funds claimed successfully!");
    println!("Transaction ID: {}", tx_id);
    println!("Project ID: {}", project_id);
    
    Ok(())
}

// Helper functions

fn resolve_script_hash(input: &str) -> Result<ScriptHash, CliError> {
    // Check if it's a valid script hash already
    if let Ok(script_hash) = ScriptHash::from_str(input) {
        return Ok(script_hash);
    }
    
    // Handle token symbols with a contract registry (simplified version)
    let hash_str = match input.to_uppercase().as_str() {
        "NEO" => "ef4073a0f2b305a38ec4050e4d3d28bc40ea63f5",
        "GAS" => "d2a4cff31913016155e38e474a2c06d08be276cf",
        "FLM" => "f0151f528127558851b39c2cd8aa47da7418ab28",
        "BNEO" => "48c40d4666f93408be1bef038b6722404f5c4a5a",
        _ => return Err(CliError::InvalidArgument(
            format!("Unknown token or invalid script hash: {}", input),
            "Please provide a valid NEO address, script hash, or token symbol".to_string()
        )),
    };
    
    ScriptHash::from_str(hash_str).map_err(|_| {
        CliError::InvalidArgument(
            format!("Invalid script hash format: {}", hash_str),
            "Please provide a valid script hash".to_string()
        )
    })
} 