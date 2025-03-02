//! Key pair module for Neo blockchain
//!
//! This module contains the key pair type and related functionality.

use std::fmt;

use crate::neo_crypto::{
    error::CryptoError,
    keys::{Secp256r1PrivateKey, Secp256r1PublicKey, Secp256r1Signature},
};

use crate::neo_types::ScriptHash;

#[cfg(feature = "utils")]
use crate::neo_types::script_hash_extension::ScriptHashExtension;

/// Key pair for Neo blockchain
#[derive(Clone)]
pub struct KeyPair {
    /// Private key
    private_key: Secp256r1PrivateKey,
    /// Public key
    public_key: Secp256r1PublicKey,
}

impl KeyPair {
    /// Create a new key pair from a private key
    pub fn new(private_key: Secp256r1PrivateKey) -> Self {
        let public_key = private_key.public_key();
        Self { private_key, public_key }
    }
    
    /// Create a new random key pair
    #[cfg(feature = "crypto-standard")]
    pub fn random() -> Self {
        let private_key = Secp256r1PrivateKey::random();
        Self::new(private_key)
    }
    
    /// Get the private key
    pub fn private_key(&self) -> &Secp256r1PrivateKey {
        &self.private_key
    }
    
    /// Get the public key
    pub fn public_key(&self) -> &Secp256r1PublicKey {
        &self.public_key
    }
    
    /// Sign a message
    pub fn sign_message(&self, message: &[u8]) -> Result<Secp256r1Signature, CryptoError> {
        self.private_key.sign_message(message)
    }
    
    /// Verify a signature
    pub fn verify_signature(&self, message: &[u8], signature: &Secp256r1Signature) -> Result<bool, CryptoError> {
        self.public_key.verify_signature(message, signature)
    }
    
    /// Get the script hash
    #[cfg(feature = "utils")]
    pub fn get_script_hash(&self) -> Result<ScriptHash, CryptoError> {
        let public_key_bytes = self.public_key.get_encoded();
        
        match ScriptHash::from_public_key(&public_key_bytes) {
            Ok(script_hash) => Ok(script_hash),
            Err(_) => Err(CryptoError::InvalidPublicKey("Failed to derive script hash".to_string())),
        }
    }
    
    /// Create a key pair from WIF
    pub fn from_wif(_wif: &str) -> Result<Self, CryptoError> {
        // Implementation requires the utils feature
        #[cfg(feature = "utils")]
        {
            // TODO: Implement WIF decoding
        }
        
        Err(CryptoError::InvalidFormat("WIF decoding not implemented".to_string()))
    }
    
    /// Export the key pair to WIF
    pub fn export_wif(&self) -> Result<String, CryptoError> {
        // Implementation requires the utils feature
        #[cfg(feature = "utils")]
        {
            // TODO: Implement WIF encoding
        }
        
        Err(CryptoError::InvalidFormat("WIF encoding not implemented".to_string()))
    }
}

impl fmt::Debug for KeyPair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeyPair")
            .field("public_key", &self.public_key)
            .finish()
    }
}
