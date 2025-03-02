pub mod error;
pub mod extensions;
pub mod config;
pub mod ledger;

use colored::Colorize;
use dialoguer::{Input, Password};
use error::{CliError, CliResult};
use neo3::prelude::*;
use std::error::Error;
use std::io::{self, Write};
use std::fs::File;
use std::path::Path;
use std::fmt::Display;
use primitive_types::H160;
use neo3::crypto::hash::{HashableForSha256, HashableForRipemd160};

/// Print a success message
pub fn print_success(message: &str) {
    println!("{}", message.green());
}

/// Print an error message
pub fn print_error(message: &str) {
    eprintln!("{}", message.red());
}

/// Print an informational message
pub fn print_info(message: &str) {
    println!("{}", message.blue());
}

/// Print a warning message
pub fn print_warning(message: &str) {
    println!("{}", message.yellow());
}

/// Prompt for user input
pub fn prompt_input(prompt: &str) -> Result<String, error::CliError> {
    print!("{}: ", prompt);
    io::stdout().flush().map_err(|e| error::CliError::Io(e))?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| error::CliError::Io(e))?;
    
    Ok(input.trim().to_string())
}

/// Prompt for a password (without echoing characters)
pub fn prompt_password(prompt: &str) -> Result<String, error::CliError> {
    let password = dialoguer::Password::new()
        .with_prompt(prompt)
        .interact()
        .map_err(|e| error::CliError::Input(format!("Failed to read password: {}", e)))?;
        
    Ok(password)
}

/// Prompt for yes/no input
pub fn prompt_yes_no(prompt: &str) -> Result<bool, error::CliError> {
    print!("{} [y/n]: ", prompt);
    io::stdout().flush().map_err(|e| error::CliError::Io(e))?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| error::CliError::Io(e))?;
    
    match input.trim().to_lowercase().as_str() {
        "y" | "yes" => Ok(true),
        "n" | "no" => Ok(false),
        _ => {
            print_error("Please enter 'y' or 'n'");
            prompt_yes_no(prompt)
        }
    }
}

/// Formats a gas value for display (divides by 10^8)
pub fn format_gas(gas: i64) -> String {
    let amount = gas as f64 / 100_000_000.0;
    format!("{:.8} GAS", amount)
}

/// Calculate contract hash based on sender, checksum, and name bytes
pub fn calculate_contract_hash(
    sender: &primitive_types::H160,
    checksum: u32,
    name_bytes: &[u8]
) -> primitive_types::H160 {
    extensions::calculate_contract_hash(sender, checksum, name_bytes)
}

/// Get a human-readable transaction type name
pub fn get_tx_type_name(tx_type: u8) -> String {
    match tx_type {
        0x00 => "Miner".to_string(),
        0x01 => "Issue".to_string(),
        0x02 => "Claim".to_string(),
        0x03 => "Enrollment".to_string(),
        0x04 => "Register".to_string(),
        0x05 => "Contract".to_string(),
        0x06 => "State".to_string(),
        0x07 => "Publish".to_string(),
        0x08 => "Invocation".to_string(),
        _ => format!("Unknown ({})", tx_type),
    }
}
