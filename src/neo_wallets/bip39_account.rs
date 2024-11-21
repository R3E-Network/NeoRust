use bip39::{Mnemonic, Language};
use sha2::{Sha256, Digest};
use crate::prelude::{Account, AccountTrait, KeyPair};

/// A BIP-39 compatible NEO account.
#[derive(Debug)]
pub struct Bip39Account {
    /// The underlying NEO account
    account: Account,
    
    /// Generated BIP-39 mnemonic for the account
    mnemonic: String,
}

impl Bip39Account {
    /// Creates a new BIP-39 compatible NEO account.
    /// 
    /// The private key for the wallet is calculated using:
    /// `Key = SHA-256(BIP_39_SEED(mnemonic, password))`
    /// 
    /// The password is only used as passphrase for BIP-39 seed (i.e., used to recover the account).
    ///
    /// # Arguments
    /// * `password` - The passphrase to encrypt the private key
    ///
    /// # Returns
    /// A BIP-39 compatible Neo account
    pub fn create(password: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut rng = bip39::rand::thread_rng();
        let mnemonic = Mnemonic::generate_in_with(&mut rng, Language::English, 24).unwrap();
        let seed = mnemonic.to_seed(password);
        
        let mut hasher = Sha256::new();
        hasher.update(&seed);
        let private_key = hasher.finalize();

        let key_pair = KeyPair::from_private_key(private_key.as_ref()).unwrap();
        let account = Account::from_key_pair(key_pair.clone(), None, None).unwrap();
        
        Ok(Self {
            account,
            mnemonic: mnemonic.to_string(),
        })
    }

    /// Recovers a key pair based on BIP-39 mnemonic and password.
    ///
    /// # Arguments
    /// * `password` - The passphrase given when the BIP-39 account was generated
    /// * `mnemonic` - The generated mnemonic with the given passphrase
    ///
    /// # Returns
    /// A recovered Bip39Account
    pub fn from_bip39_mnemonic(password: &str, mnemonic: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mnemonic = Mnemonic::parse_in(Language::English, mnemonic)?;
        let seed = mnemonic.to_seed(password);
        
        let mut hasher = Sha256::new();
        hasher.update(&seed);
        let private_key = hasher.finalize();

        let key_pair = KeyPair::from_private_key(private_key.as_ref()).unwrap();
        let account = Account::from_key_pair(key_pair.clone(), None, None).unwrap();
        
        Ok(Self {
            account,
            mnemonic: mnemonic.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_bip39_account() {
        let password = "test_password";
        let account = Bip39Account::create(password).unwrap();
        
        // Check that mnemonic is 24 words
        assert_eq!(account.mnemonic.split_whitespace().count(), 24);
        
        // Verify account was created with valid key pair
        assert!(account.account.key_pair().is_some());
    }

    #[test]
    fn test_recover_from_mnemonic() {
        let password = "test_password";
        let original = Bip39Account::create(password).unwrap();
        let mnemonic = original.mnemonic.clone();
        
        // Recover account using mnemonic
        let recovered = Bip39Account::from_bip39_mnemonic(password, &mnemonic).unwrap();
        
        // Verify recovered account matches original
        assert_eq!(
            original.account.get_script_hash(),
            recovered.account.get_script_hash()
        );
        assert_eq!(original.mnemonic, recovered.mnemonic);
    }

    #[test]
    fn test_invalid_mnemonic() {
        let result = Bip39Account::from_bip39_mnemonic(
            "password",
            "invalid mnemonic phrase"
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_different_passwords_different_accounts() {
        let account1 = Bip39Account::create("password1").unwrap();
        let account2 = Bip39Account::create("password2").unwrap();
        
        assert_ne!(
            account1.account.get_script_hash(),
            account2.account.get_script_hash()
        );
    }

    #[test]
    fn test_generate_and_recover_bip39_account() {
        let password = "Insecure Pa55w0rd";
        let account1 = Bip39Account::create(password).unwrap();
        let account2 = Bip39Account::from_bip39_mnemonic(password, &account1.mnemonic).unwrap();

        assert_eq!(account1.account.get_address(), account2.account.get_address());
        assert!(account1.account.key_pair().is_some());
        assert_eq!(account1.account.key_pair(), account2.account.key_pair());
        assert_eq!(account1.mnemonic, account2.mnemonic);
        assert!(!account1.mnemonic.is_empty());
    }
}
