pub mod config;
pub mod error;
pub mod extensions;
pub mod neofs;

use crate::errors::CliError;
use colored::*;
use dialoguer::{Input, Password};

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

pub fn prompt_input<T>(prompt: &str) -> Result<T, CliError>
where
	T: std::str::FromStr + std::clone::Clone + std::fmt::Display,
	T::Err: std::fmt::Display,
{
	Input::new()
		.with_prompt(prompt)
		.interact()
		.map_err(|e| CliError::Input(e.to_string()))
}

pub fn prompt_password(prompt: &str) -> Result<String, CliError> {
	Password::new()
		.with_prompt(prompt)
		.interact()
		.map_err(|e| CliError::Input(e.to_string()))
}

pub fn prompt_yes_no(prompt: &str) -> Result<bool, CliError> {
	let input = prompt_input::<String>(&format!("{} (y/n)", prompt))?;
	let input = input.to_lowercase();

	Ok(input == "y" || input == "yes")
}
