// Neo X Bridge module for Neo CLI
//
// This module provides commands for interacting with the Neo X Bridge contract.
// It enables token transfers between Neo N3 and Neo X networks.

use crate::{
    commands::wallet::CliState,
    errors::CliError,
    utils::print_info,
    utils::print_success,
};
use clap::{Args, Subcommand};
use neo3::{
    neo_clients::{HttpProvider, RpcClient},
    neo_protocol::Account,
    neo_types::ScriptHash,
};
use neo_x::{
    bridge::bridge_contract::NeoXBridgeContract,
    evm::provider::NeoXProvider,
};
use std::{path::PathBuf, str::FromStr};
use super::{network_validator, utils::{NetworkTypeCli, get_token_address_for_network, parse_amount}};

/// Arguments for Neo X bridge operations
#[derive(Args, Debug, Clone)]
pub struct BridgeArgs {
    /// Path to wallet file
    #[arg(short, long)]
    pub wallet: Option<PathBuf>,

    /// Wallet password
    #[arg(short, long)]
    pub password: Option<String>,

    #[clap(subcommand)]
    pub command: BridgeCommands,
}

#[derive(Subcommand, Debug, Clone)]
pub enum BridgeCommands {
    /// Deposit tokens from Neo N3 to Neo X
    Deposit {
        /// Token symbol or contract address to deposit (currently only NEO and GAS are supported)
        #[arg(long)]
        token: String,

        /// Amount to deposit
        #[arg(long)]
        amount: String,

        /// Destination address on Neo X
        #[arg(long)]
        destination: String,
    },

    /// Withdraw tokens from Neo X to Neo N3 (note: this operation must be initiated from Neo X)
    Withdraw {
        /// Token symbol or contract address to withdraw (currently only NEO and GAS are supported)
        #[arg(long)]
        token: String,

        /// Amount to withdraw
        #[arg(long)]
        amount: String,

        /// Destination address on Neo N3
        #[arg(long)]
        destination: String,
    },

    /// Get fee information for the bridge
    Fee {
        /// Token symbol or contract address
        #[arg(long)]
        token: String,
    },

    /// Get bridge capacity information
    Cap {
        /// Token symbol or contract address
        #[arg(long)]
        token: String,
    },
}

/// Handle Neo X bridge command processing
pub async fn handle_bridge_command(
    args: BridgeArgs,
    state: &mut CliState,
) -> Result<(), CliError> {
    // Load wallet from CLI state
    let wallet = super::load_wallet(state)?;
    let account = wallet.get_accounts().get(0).ok_or_else(|| {
        CliError::Wallet("No account found in wallet. Please create or import an account.".to_string())
    })?.clone();

    // Get RPC client
    let rpc_client = state.get_rpc_client()?;
    
    // Determine network type
    let network_type_str = state.get_network_type_string();
    let network_type = NetworkTypeCli::from_network_string(&network_type_str);
    let is_testnet = network_type.is_testnet();

    // Bridge must be accessed from Neo N3
    if network_type.is_neox() {
        return Err(CliError::InvalidArgument(
            "Bridge commands should be executed on a Neo N3 network".to_string(),
            "Please switch to a Neo N3 network first using 'network connect'".to_string(),
        ));
    }

    // Get the bridge contract hash using our utility function
    let bridge_contract_hash = super::utils::get_bridge_contract_hash(network_type)?;
    
    // Create the bridge contract instance with the correct hash
    let bridge_contract = NeoXBridgeContract::with_script_hash(bridge_contract_hash, Some(rpc_client));
    
    // Get bridge limits for logging and validation
    let bridge_limits = match args.command {
        BridgeCommands::Deposit { ref token, .. } | BridgeCommands::Fee { ref token } | BridgeCommands::Cap { ref token } => {
            // Only fetch limits for specific token commands
            if let Some(token_hash) = get_token_address_for_network(token, network_type) {
                Some(network_validator::get_bridge_limits_for_token(&token_hash, rpc_client, is_testnet).await?)
            } else {
                None
            }
        },
        _ => None,
    };

    match args.command {
        BridgeCommands::Deposit { token, amount, destination } => {
            handle_deposit(token, amount, destination, account, rpc_client, bridge_contract, network_type).await
        },
        BridgeCommands::Withdraw { token, amount, destination } => {
            handle_withdraw(token, amount, destination, account, rpc_client, bridge_contract, network_type).await
        },
        BridgeCommands::Fee { token } => {
            handle_fee(token, rpc_client, bridge_contract, network_type).await
        },
        BridgeCommands::Cap { token } => {
            handle_cap(token, rpc_client, bridge_contract, network_type).await
        },
    }
}

async fn handle_deposit(
    token: String,
    amount: String,
    destination: String,
    account: Account,
    rpc_client: &RpcClient<HttpProvider>,
    bridge_contract: NeoXBridgeContract<'_, HttpProvider>,
    network_type: NetworkTypeCli,
) -> Result<(), CliError> {
    // Resolve token to script hash
    let token_hash = super::resolve_token_to_scripthash_with_network(&token, rpc_client, network_type).await?;
    
    // Validate destination address is a valid Neo X address
    super::network_validator::validate_bridge_destination(&destination, network_type)?;
    
    // Parse amount using token decimals
    let raw_amount = parse_amount(&amount, &token_hash, rpc_client, network_type).await?;

    print_info(&format!(
        "Depositing {} {} from Neo N3 to Neo X address: {}",
        amount, token, destination
    ));

    // Execute deposit
    let builder = bridge_contract.deposit(&token_hash, raw_amount, &destination, &account).await?;
    let tx = builder.build().await?;
    let tx_id = tx.hash.to_string();
    
    print_success(&format!(
        "Successfully deposited {} {} to Neo X. Transaction ID: {}",
        amount, token, tx_id
    ));

    Ok(())
}

async fn handle_withdraw(
    token: String,
    amount: String,
    destination: String,
    account: Account,
    rpc_client: &RpcClient<HttpProvider>,
    bridge_contract: NeoXBridgeContract<'_, HttpProvider>,
    network_type: NetworkTypeCli,
) -> Result<(), CliError> {
    // Withdraw operations must be initiated from Neo X, so provide instructions
    print_info("Withdraw operations must be initiated from Neo X, not Neo N3.");
    
    // Validate destination address is a valid Neo N3 address
    // For withdraw, the destination network is the opposite of the current network
    if let Err(e) = super::network_validator::validate_bridge_destination(&destination, NetworkTypeCli::NeoX) {
        print_warning(&format!("Note: The provided destination '{}' may not be a valid Neo N3 address: {}", destination, e));
    } else {
        print_info(&format!("The provided destination address '{}' appears to be a valid Neo N3 address", destination));
    }
    
    print_info("Please use a Neo X compatible wallet to initiate the withdrawal.");
    print_info("Instructions:");
    print_info("1. Connect to the Neo X network");
    print_info("2. Use the bridge contract to initiate a withdrawal");
    print_info("3. Specify the destination Neo N3 address");
    
    Ok(())
}

async fn handle_fee(
    token: String,
    rpc_client: &RpcClient<HttpProvider>,
    bridge_contract: NeoXBridgeContract<'_, HttpProvider>,
    network_type: NetworkTypeCli,
) -> Result<(), CliError> {
    // Resolve token to script hash
    let token_hash = super::resolve_token_to_scripthash_with_network(&token, rpc_client, network_type).await?;
    
    // Get token decimals
    let token_decimals = super::get_token_decimals(&token_hash, rpc_client, network_type).await?;
    
    // Get fee
    let fee = bridge_contract.get_fee(&token_hash).await?;
    
    // Format the fee amount
    let formatted_fee = super::format_token_amount(fee as i64, token_decimals);
    
    print_info(&format!("Bridge fee for {}: {} {}", token, formatted_fee, token));
    
    Ok(())
}

async fn handle_cap(
    token: String,
    rpc_client: &RpcClient<HttpProvider>,
    bridge_contract: NeoXBridgeContract<'_, HttpProvider>,
    network_type: NetworkTypeCli,
) -> Result<(), CliError> {
    // Resolve token to script hash
    let token_hash = super::resolve_token_to_scripthash_with_network(&token, rpc_client, network_type).await?;
    
    // Get token decimals
    let token_decimals = super::get_token_decimals(&token_hash, rpc_client, network_type).await?;
    
    // Get cap
    let cap = bridge_contract.get_cap(&token_hash).await?;
    
    // Format the cap amount
    let formatted_cap = super::format_token_amount(cap as i64, token_decimals);
    
    print_info(&format!("Bridge capacity for {}: {} {}", token, formatted_cap, token));
    
    Ok(())
}
