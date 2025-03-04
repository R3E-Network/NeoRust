use crate::errors::CliError;
use colored::*;

/// Prints a success message in green color
pub fn print_success(message: &str) {
	println!("{}", message.green());
}

/// Prints an info message in blue color
pub fn print_info(message: &str) {
	println!("{}", message.blue());
}

/// Prints an error message in red color
pub fn print_error(message: &str) {
	eprintln!("{}", message.red());
}

/// Prompts the user for a password
pub fn prompt_password(prompt: &str) -> Result<String, CliError> {
	use std::io::{self, Write};

	print!("{}: ", prompt);
	io::stdout().flush().map_err(|e| CliError::Io(e))?;

	let mut password = String::new();
	io::stdin().read_line(&mut password).map_err(|e| CliError::Io(e))?;

	Ok(password.trim().to_string())
}

/// Ensure an account is loaded before proceeding with operations
pub fn ensure_account_loaded(
	state: &mut crate::commands::wallet::CliState,
) -> Result<neo3::neo_protocol::Account, crate::errors::CliError> {
	state.get_account()
}

/// Prompts the user for a yes/no response
pub fn prompt_yes_no(prompt: &str) -> bool {
	use std::io::{self, Write};

	print!("{} [y/N]: ", prompt);
	io::stdout().flush().unwrap();

	let mut input = String::new();
	io::stdin().read_line(&mut input).unwrap();

	let input = input.trim().to_lowercase();
	input == "y" || input == "yes"
}
