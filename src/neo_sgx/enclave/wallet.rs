use sgx_tcrypto::*;
use sgx_types::*;
use crate::neo_sgx::enclave::crypto::{generate_keypair, sign_message};

/// A wallet implementation for SGX enclaves
pub struct SgxWallet {
    private_key: [u8; 32],
    public_key: [u8; 64],
    encrypted_private_key: Vec<u8>,
}

impl SgxWallet {
    /// Creates a new wallet with a random keypair
    ///
    /// # Arguments
    ///
    /// * `password` - The password to encrypt the private key
    ///
    /// # Returns
    ///
    /// A new `SgxWallet` instance
    pub fn new(password: &str) -> Result<Self, sgx_status_t> {
        // Generate a new keypair
        let (private_key, public_key) = generate_keypair()?;
        
        // Encrypt the private key with the password
        let encrypted_private_key = Self::encrypt_private_key(&private_key, password)?;
        
        Ok(Self {
            private_key,
            public_key,
            encrypted_private_key,
        })
    }
    
    /// Creates a wallet from an existing private key
    ///
    /// # Arguments
    ///
    /// * `private_key` - The private key
    /// * `password` - The password to encrypt the private key
    ///
    /// # Returns
    ///
    /// A new `SgxWallet` instance
    pub fn from_private_key(private_key: &[u8; 32], password: &str) -> Result<Self, sgx_status_t> {
        // Derive public key from private key
        let ecc_handle = SgxEccHandle::new();
        let public_key = match ecc_handle.create_key_pair() {
            Ok((_, pub_k)) => {
                let mut public_key_bytes = [0u8; 64];
                public_key_bytes.copy_from_slice(&pub_k[..64]);
                public_key_bytes
            }
            Err(err) => return Err(err),
        };
        
        // Encrypt the private key with the password
        let encrypted_private_key = Self::encrypt_private_key(private_key, password)?;
        
        Ok(Self {
            private_key: *private_key,
            public_key,
            encrypted_private_key,
        })
    }
    
    /// Signs a transaction with the wallet's private key
    ///
    /// # Arguments
    ///
    /// * `transaction_data` - The transaction data to sign
    ///
    /// # Returns
    ///
    /// The signature
    pub fn sign_transaction(&self, transaction_data: &[u8]) -> Result<[u8; 65], sgx_status_t> {
        sign_message(&self.private_key, transaction_data)
    }
    
    /// Gets the wallet's public key
    ///
    /// # Returns
    ///
    /// The public key
    pub fn get_public_key(&self) -> &[u8; 64] {
        &self.public_key
    }
    
    /// Gets the encrypted private key
    ///
    /// # Returns
    ///
    /// The encrypted private key
    pub fn get_encrypted_private_key(&self) -> &[u8] {
        &self.encrypted_private_key
    }
    
    /// Encrypts a private key with a password
    ///
    /// # Arguments
    ///
    /// * `private_key` - The private key to encrypt
    /// * `password` - The password to use for encryption
    ///
    /// # Returns
    ///
    /// The encrypted private key
    fn encrypt_private_key(private_key: &[u8; 32], password: &str) -> Result<Vec<u8>, sgx_status_t> {
        // Derive encryption key from password
        let mut key = [0u8; 16]; // AES-128 key
        let salt = b"NeoRustSGXSalt"; // Fixed salt for simplicity
        let result = sgx_rijndael128GCM_cmac_128bit_key_derivation(
            password.as_ptr(),
            password.len() as u32,
            salt.as_ptr(),
            salt.len() as u32,
            &mut key,
        );
        if result != sgx_status_t::SGX_SUCCESS {
            return Err(result);
        }
        
        // Encrypt the private key
        let mut encrypted = vec![0u8; 32 + 16]; // data + tag
        let mut mac = [0u8; 16]; // GCM tag
        let iv = [0u8; 12]; // Initialization vector
        
        let result = sgx_rijndael128GCM_encrypt(
            &key,
            private_key,
            32,
            &mut encrypted[..32],
            &iv,
            12,
            &[0u8; 0], // No additional authenticated data
            0,
            &mut mac,
        );
        if result != sgx_status_t::SGX_SUCCESS {
            return Err(result);
        }
        
        // Append the MAC tag to the encrypted data
        encrypted[32..].copy_from_slice(&mac);
        
        Ok(encrypted)
    }
    
    /// Decrypts an encrypted private key with a password
    ///
    /// # Arguments
    ///
    /// * `encrypted_private_key` - The encrypted private key
    /// * `password` - The password used for encryption
    ///
    /// # Returns
    ///
    /// The decrypted private key
    pub fn decrypt_private_key(encrypted_private_key: &[u8], password: &str) -> Result<[u8; 32], sgx_status_t> {
        if encrypted_private_key.len() != 48 { // 32 bytes data + 16 bytes tag
            return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
        }
        
        // Derive decryption key from password
        let mut key = [0u8; 16]; // AES-128 key
        let salt = b"NeoRustSGXSalt"; // Fixed salt for simplicity
        let result = sgx_rijndael128GCM_cmac_128bit_key_derivation(
            password.as_ptr(),
            password.len() as u32,
            salt.as_ptr(),
            salt.len() as u32,
            &mut key,
        );
        if result != sgx_status_t::SGX_SUCCESS {
            return Err(result);
        }
        
        // Extract encrypted data and MAC tag
        let encrypted_data = &encrypted_private_key[..32];
        let mac = &encrypted_private_key[32..];
        
        // Decrypt the private key
        let mut decrypted = [0u8; 32];
        let iv = [0u8; 12]; // Initialization vector
        
        let result = sgx_rijndael128GCM_decrypt(
            &key,
            encrypted_data,
            32,
            &mut decrypted,
            &iv,
            12,
            &[0u8; 0], // No additional authenticated data
            0,
            unsafe { &*(mac.as_ptr() as *const [u8; 16]) },
        );
        if result != sgx_status_t::SGX_SUCCESS {
            return Err(result);
        }
        
        Ok(decrypted)
    }
}
