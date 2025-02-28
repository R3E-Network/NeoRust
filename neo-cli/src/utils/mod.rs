pub mod error;

use colored::*;
use dialoguer::{Input, Password};
use error::{CliError, CliResult};

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
