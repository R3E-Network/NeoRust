pub mod error;
pub mod extensions;

use colored::*;
use dialoguer::{Input, Password};
use error::{CliError, CliResult};
use neo3::prelude::*;

pub fn print_success(message: &str) {
    println!("{}", message.green());
}

pub fn print_info(message: &str) {
    println!("{}", message.blue());
}

pub fn print_warning(message: &str) {
    println!("{}", message.yellow());
}

pub fn print_error(message: &str) {
    eprintln!("{}", message.red());
}

pub fn prompt_input<T>(prompt: &str) -> CliResult<T>
where
    T: std::str::FromStr + std::clone::Clone + std::fmt::Display,
    T::Err: std::fmt::Display,
{
    Input::new()
        .with_prompt(prompt)
        .interact()
        .map_err(|e| CliError::Input(e.to_string()))
}

pub fn prompt_password(prompt: &str) -> CliResult<String> {
    Password::new()
        .with_prompt(prompt)
        .interact()
        .map_err(|e| CliError::Input(e.to_string()))
}

pub fn prompt_yes_no(prompt: &str) -> CliResult<bool> {
    let input: String = Input::new()
        .with_prompt(format!("{} (y/n)", prompt))
        .interact()
        .map_err(|e| CliError::Input(e.to_string()))?;
    
    Ok(input.to_lowercase().starts_with('y'))
}

/// Calculate the contract hash based on sender, NEF checksum and contract name
pub fn calculate_contract_hash(sender: &H160, checksum: u32, name_bytes: &[u8]) -> H160 {
    use neo3::crypto::*;
    
    // Concatenate sender bytes, checksum bytes and name bytes
    let mut data = Vec::new();
    data.extend_from_slice(&sender.to_array());
    data.extend_from_slice(&checksum.to_le_bytes());
    data.extend_from_slice(name_bytes);
    
    // Calculate the hash
    let hash = Sha256::hash(&data);
    let hash = Ripemd160::hash(&hash);
    
    H160::from_slice(&hash)
}

/// Format GAS amount for display (convert from smallest unit to display unit)
pub fn format_gas(amount: &str) -> String {
    match amount.parse::<f64>() {
        Ok(value) => {
            let gas_value = value / 100_000_000.0; // Convert from fraction to whole GAS
            format!("{:.8} GAS", gas_value)
        },
        Err(_) => amount.to_string(),
    }
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
