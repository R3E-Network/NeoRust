use std::{fs::File, io::Write, path::PathBuf};

use neo::prelude::{Wallet, WalletError};

/// Provides functionality for backing up and recovering Neo wallets.
pub struct WalletBackup;

impl WalletBackup {
	/// Backs up a wallet to the specified file path.
	///
	/// This method serializes the wallet to JSON format and saves it to the specified file.
	/// It's recommended to store backups in a secure location.
	///
	/// # Arguments
	///
	/// * `wallet` - The wallet to back up
	/// * `path` - The file path where the backup will be saved
	///
	/// # Returns
	///
	/// A `Result` indicating success or failure
	///
	/// # Example
	///
	/// ```no_run
	/// use std::path::PathBuf;
	/// use NeoRust::prelude::{Wallet, WalletBackup};
	///
	/// let wallet = Wallet::new();
	/// let backup_path = PathBuf::from("wallet_backup.json");
	/// WalletBackup::backup(&wallet, backup_path).unwrap();
	/// ```
	pub fn backup(wallet: &Wallet, path: PathBuf) -> Result<(), WalletError> {
		// Convert wallet to NEP6
		let nep6 = wallet.to_nep6()?;

		// Encode as JSON
		let json = serde_json::to_string_pretty(&nep6)
			.map_err(|e| WalletError::AccountState(format!("Serialization error: {}", e)))?;

		// Write to file at path
		let mut file = File::create(path).map_err(|e| WalletError::IoError(e))?;

		file.write_all(json.as_bytes()).map_err(|e| WalletError::IoError(e))?;

		Ok(())
	}

	/// Recovers a wallet from a backup file.
	///
	/// This method reads a wallet backup file in JSON format and deserializes it into a Wallet.
	///
	/// # Arguments
	///
	/// * `path` - The file path of the backup
	///
	/// # Returns
	///
	/// A `Result` containing the recovered wallet or an error
	///
	/// # Example
	///
	/// ```no_run
	/// use std::path::PathBuf;
	/// use NeoRust::prelude::WalletBackup;
	///
	/// let backup_path = PathBuf::from("wallet_backup.json");
	/// let recovered_wallet = WalletBackup::recover(backup_path).unwrap();
	/// ```
	pub fn recover(path: PathBuf) -> Result<Wallet, WalletError> {
		// Read file content
		let file_content = std::fs::read_to_string(path).map_err(|e| WalletError::IoError(e))?;

		// Parse JSON to NEP6Wallet
		let nep6_wallet = serde_json::from_str(&file_content)
			.map_err(|e| WalletError::AccountState(format!("Deserialization error: {}", e)))?;

		// Convert NEP6Wallet to Wallet
		Wallet::from_nep6(nep6_wallet)
	}
}

#[cfg(test)]
mod tests {
	use std::{fs, path::PathBuf};

	use neo::prelude::{Account, AccountTrait, Wallet, WalletBackup, WalletTrait};

	#[test]
	fn test_backup_and_recover() {
		// Create a wallet with an account
		let mut wallet = Wallet::new();
		let account = Account::create().unwrap();
		wallet.add_account(account);

		// Encrypt the accounts to avoid the "Account private key is available but not encrypted" error
		wallet.encrypt_accounts("test_password");

		// Create a temporary backup file
		let temp_dir = std::env::temp_dir();
		let backup_path = temp_dir.join("wallet_backup_test.json");

		// Backup the wallet
		WalletBackup::backup(&wallet, backup_path.clone()).unwrap();

		// Verify the backup file exists
		assert!(backup_path.exists());

		// Recover the wallet
		let recovered_wallet = WalletBackup::recover(backup_path.clone()).unwrap();

		// Verify the recovered wallet has the same properties
		assert_eq!(wallet.name(), recovered_wallet.name());
		assert_eq!(wallet.version(), recovered_wallet.version());
		assert_eq!(wallet.accounts().len(), recovered_wallet.accounts().len());

		// Clean up
		fs::remove_file(backup_path).unwrap();
	}
}
