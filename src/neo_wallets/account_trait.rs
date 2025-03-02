use primitive_types::H160;
use std::str::FromStr;

use crate::neo_types::Address;
use crate::neo_wallets::wallet::wallet_error::WalletError;

/// Represents the core functionalities required by a cryptocurrency account.
///
/// This trait defines the essential operations an account should support,
/// including retrieving its address and label, checking its status (such as
/// whether it's locked or the default), and manipulating its associated key.
///
/// # Required Methods
///
/// - `get_script_hash`, `get_address`, `get_label`: Read access to account identifiers.
/// - `is_default`, `is_locked`: Status checks on the account.
/// - `get_key`, `get_encrypted_key`: Key management operations.
/// - Other utility and account management functions.
///
/// # Example
///
/// Implementing the `AccountTrait` for a simple account structure:
///
/// ```ignore
/// struct SimpleAccount {
///     script_hash: H160,
///     address: String,
///     label: String,
///     is_default: bool,
///     lock: bool,
///     key: Option<String>,
/// }
///
/// impl AccountTrait for SimpleAccount {
///     fn get_script_hash(&self) -> &H160 {
///         &self.script_hash
///     }
///
///     // Implementations for other methods follow...
/// }
/// ```
pub trait AccountTrait {
    /// Creates a new account with randomly generated keys
    fn create() -> Result<Self, WalletError> where Self: Sized;
    
    /// Gets the script hash of the account
    fn get_script_hash(&self) -> H160;
    
    /// Gets the address of the account
    fn get_address(&self) -> String;
    
    /// Gets the label of the account
    fn get_label(&self) -> Option<String>;
    
    /// Checks if this is the default account
    fn is_default(&self) -> bool;
    
    /// Checks if the account is locked
    fn is_locked(&self) -> bool;
    
    /// Gets the private key of the account, if available
    fn get_private_key(&self) -> Option<String>;
    
    /// Gets the encrypted private key of the account, if available
    fn get_encrypted_key(&self) -> Option<String>;
    
    /// Updates the encrypted key for the account
    fn update_encrypted_key(&mut self, encrypted_key: String);
    
    /// Sets the account as the default
    fn set_default(&mut self, is_default: bool);
    
    /// Sets the locked status of the account
    fn set_locked(&mut self, locked: bool);
    
    /// Sets the label for the account
    fn set_label(&mut self, label: String);
    
    /// Decrypts the account's encrypted key with the provided password
    fn decrypt(&self, password: &str) -> Result<String, WalletError>;
}

/// Implementation of AccountTrait for the standard Account struct
impl AccountTrait for crate::neo_wallets::wallet::wallet::Account {
    fn create() -> Result<Self, WalletError> {
        // Generate a new key pair
        let key_pair = crate::neo_crypto::key_pair::KeyPair::new_random();
        
        // Get the private key as WIF - we'll just use a dummy string for now
        let private_key = "dummy_key".to_string();
        
        // Use address from key pair - we'll just use a dummy address for now
        let address = "NeoXDummyAddress".to_string();
        
        // Create the account
        Ok(Self {
            address,
            label: "Account".to_string(),
            is_default: false,
            lock: false,
            key: Some(private_key),
            contract: None,
            extra: None,
        })
    }
    
    fn get_script_hash(&self) -> H160 {
        // Just return zero for now
        H160::zero()
    }
    
    fn get_address(&self) -> String {
        self.address.clone()
    }
    
    fn get_label(&self) -> Option<String> {
        Some(self.label.clone())
    }
    
    fn is_default(&self) -> bool {
        self.is_default
    }
    
    fn is_locked(&self) -> bool {
        self.lock
    }
    
    fn get_private_key(&self) -> Option<String> {
        self.key.clone()
    }
    
    fn get_encrypted_key(&self) -> Option<String> {
        self.key.clone()
    }
    
    fn update_encrypted_key(&mut self, encrypted_key: String) {
        self.key = Some(encrypted_key);
    }
    
    fn set_default(&mut self, is_default: bool) {
        self.is_default = is_default;
    }
    
    fn set_locked(&mut self, locked: bool) {
        self.lock = locked;
    }
    
    fn set_label(&mut self, label: String) {
        self.label = label;
    }
    
    fn decrypt(&self, password: &str) -> Result<String, WalletError> {
        // Get the encrypted key
        let encrypted_key = self.get_encrypted_key()
            .ok_or_else(|| WalletError::from("No encrypted key available"))?;
        
        // Decrypt the key
        #[cfg(feature = "wallet-secure")]
        {
            crate::neo_crypto::nep2::decrypt(&encrypted_key, password)
                .map_err(|e| WalletError::CryptoError(e))
        }
        
        #[cfg(not(feature = "wallet-secure"))]
        {
            Err(WalletError::from("Decryption not available without wallet-secure feature"))
        }
    }
} 