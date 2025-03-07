// Utility functions for DeFi operations
//
// This module provides common utility functions used across the DeFi modules,
// including network type detection, formatting, and common user interactions.

use std::str::FromStr;
use rpassword::read_password;
use neo3::{
    neo_types::{
        h160::H160,
        script_hash::ScriptHash,
    },
    neo_protocol::{
        account::Account,
        address::Address,
    },
    neo_providers::{
        http::HttpProvider,
        RpcClient,
    },
};

use crate::{
    cli::CliState,
    commands::defi::network_validator::NetworkTypeCli,
    error::CliError,
};

/// Print a success message in green
pub fn print_success(message: &str) {
    println!("\x1b[32m{}\x1b[0m", message);
}

/// Print an info message in blue
pub fn print_info(message: &str) {
    println!("\x1b[34m{}\x1b[0m", message);
}

/// Print an error message in red
pub fn print_error(message: &str) {
    eprintln!("\x1b[31m{}\x1b[0m", message);
}

/// Prompt for a password with masking
pub fn prompt_password(prompt: &str) -> Result<String, CliError> {
    print!("{}", prompt);
    std::io::Write::flush(&mut std::io::stdout())
        .map_err(|e| CliError::Io(format!("Failed to flush stdout: {}", e)))?;
    
    read_password()
        .map_err(|e| CliError::Io(format!("Failed to read password: {}", e)))
}

/// Prompt for yes/no confirmation
pub fn prompt_yes_no(prompt: &str) -> bool {
    use std::io::{stdin, stdout, Write};
    
    let mut s = String::new();
    print!("{} (y/n): ", prompt);
    stdout().flush().expect("Failed to flush stdout");
    stdin().read_line(&mut s).expect("Failed to read line");
    
    s.trim().to_lowercase() == "y" || s.trim().to_lowercase() == "yes"
}

/// Ensure account is loaded and return it
pub fn ensure_account_loaded(state: &mut CliState) -> Result<neo3::neo_protocol::Account, CliError> {
    if state.current_account.is_none() {
        if let Some(wallet) = &state.wallet {
            // Get the first account from the wallet if available
            if let Some(account) = wallet.accounts.first() {
                state.current_account = Some(account.clone());
            } else {
                return Err(CliError::Wallet("No account found in wallet".to_string()));
            }
        } else {
            return Err(CliError::Wallet("No wallet loaded".to_string()));
        }
    }
    
    state.current_account
        .clone()
        .ok_or_else(|| CliError::Wallet("No account loaded".to_string()))
}

/// Format token amount with proper decimal places
pub fn format_token_amount(amount: i64, decimals: u8) -> String {
    let divisor = 10_i64.pow(decimals as u32);
    
    // Handle the case where amount is negative
    let is_negative = amount < 0;
    let abs_amount = amount.abs();
    
    let whole_part = abs_amount / divisor;
    let fractional_part = abs_amount % divisor;
    
    // Format with correct number of decimal places
    let formatted = if decimals > 0 {
        format!(
            "{}.{:0width$}",
            whole_part,
            fractional_part,
            width = decimals as usize
        )
    } else {
        format!("{}", whole_part)
    };
    
    // Add negative sign if needed
    if is_negative {
        format!("-{}", formatted)
    } else {
        formatted
    }
}

/// Convert network type from CliState to NetworkTypeCli enum
pub fn network_type_from_state(state: &CliState) -> NetworkTypeCli {
    match &state.network_type {
        Some(network) => {
            let network_str = network.to_lowercase();
            
            if network_str.contains("neox") || network_str.contains("neo_x") {
                if network_str.contains("test") {
                    NetworkTypeCli::NeoXTestNet
                } else {
                    NetworkTypeCli::NeoXMainNet
                }
            } else {
                if network_str.contains("test") {
                    NetworkTypeCli::NeoN3TestNet
                } else {
                    NetworkTypeCli::NeoN3MainNet
                }
            }
        },
        None => NetworkTypeCli::NeoN3MainNet, // Default to Neo N3 MainNet
    }
}

/// Validate an address for the current network
pub async fn validate_address(
    address: &str,
    state: &CliState,
) -> Result<(), CliError> {
    let network_type = network_type_from_state(state);
    
    // Use the network validator to check address compatibility
    crate::commands::defi::network_validator::validate_address_for_network(
        address,
        network_type,
    )
}

/// Helper function to convert address to script hash
pub fn address_to_script_hash(address: &str) -> Result<H160, CliError> {
    Address::from_str(address)
        .map_err(|_| CliError::Wallet(format!("Invalid address format: {}", address)))?
        .address_to_script_hash()
        .map_err(|e| CliError::Wallet(format!("Failed to convert address to script hash: {}", e)))
}
