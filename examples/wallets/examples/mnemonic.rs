use std::path::PathBuf;

use neo::{
	neo_protocol::account::Account,
	neo_wallets::{Wallet, WalletBackup, WalletTrait},
	prelude::{NeoNetwork, ScryptParamsDef},
};
use neo3 as neo;

/// This example demonstrates how to manage wallets in the Neo N3 blockchain.
/// It covers wallet creation, account management, wallet backup and recovery.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	println!("Neo N3 Wallet Management Example");
	println!("================================");

	// Create a new wallet
	println!("\nCreating a new wallet...");
	let mut wallet = Wallet::new();
	println!("Wallet created with name: {}", wallet.name());
	println!("Wallet version: {}", wallet.version());
	println!(
		"Default account address: {}",
		wallet.default_account().address_or_scripthash().address()
	);

	// Set wallet properties
	println!("\nUpdating wallet properties...");
	wallet.set_name("MyNeoWallet".to_string());
	wallet.set_version("1.0".to_string());

	// Configure for TestNet
	let wallet = wallet.with_network(NeoNetwork::TestNet.to_magic());
	println!("Wallet configured for network: {}", wallet.network());

	// Create and add a new account
	println!("\nCreating and adding a new account...");
	let new_account = Account::create()?;
	println!("New account created with address: {}", new_account.address_or_scripthash().address());

	let mut wallet = wallet;
	wallet.add_account(new_account.clone());
	println!("Account added to wallet");
	println!("Wallet now has {} accounts", wallet.accounts().len());

	// Set the new account as default
	println!("\nSetting the new account as default...");
	wallet.set_default_account(new_account.get_script_hash());
	println!(
		"Default account is now: {}",
		wallet.default_account().address_or_scripthash().address()
	);

	// Encrypt accounts in the wallet
	println!("\nEncrypting accounts in the wallet...");
	wallet.encrypt_accounts("password123");
	println!("Accounts encrypted");

	// Backup the wallet
	println!("\nBacking up the wallet...");
	let backup_path = PathBuf::from("my_neo_wallet_backup.json");
	WalletBackup::backup(&wallet, backup_path.clone())?;
	println!("Wallet backed up to: {}", backup_path.display());

	// Recover the wallet from backup
	println!("\nRecovering wallet from backup...");
	let recovered_wallet = WalletBackup::recover(backup_path.clone())?;
	println!("Wallet recovered successfully");
	println!("Recovered wallet name: {}", recovered_wallet.name());
	println!("Recovered wallet has {} accounts", recovered_wallet.accounts().len());

	// Clean up the backup file
	std::fs::remove_file(backup_path)?;
	println!("\nBackup file removed");

	println!("\nWallet management example completed successfully!");
	Ok(())
}

