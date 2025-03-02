//! Keys module for Neo blockchain
//!
//! This module contains the key types and related functionality.

use std::fmt;

#[cfg(feature = "crypto-standard")]
use p256::{
    ecdsa::{
        signature::{Signer, Verifier},
        SigningKey, VerifyingKey,
    },
    SecretKey,
};

use crate::neo_crypto::error::CryptoError;

/// Secp256r1 private key
#[derive(Clone)]
pub struct Secp256r1PrivateKey {
    #[cfg(feature = "crypto-standard")]
    inner: SecretKey,
    #[cfg(not(feature = "crypto-standard"))]
    inner: Vec<u8>,
}

/// Secp256r1 public key
#[derive(Clone)]
pub struct Secp256r1PublicKey {
    #[cfg(feature = "crypto-standard")]
    inner: VerifyingKey,
    #[cfg(not(feature = "crypto-standard"))]
    inner: Vec<u8>,
}

/// Secp256r1 signature
#[derive(Clone)]
pub struct Secp256r1Signature {
    #[cfg(feature = "crypto-standard")]
    inner: p256::ecdsa::Signature,
    #[cfg(not(feature = "crypto-standard"))]
    inner: Vec<u8>,
}

impl Secp256r1PrivateKey {
    /// Create a new random private key
    #[cfg(feature = "crypto-standard")]
    pub fn random() -> Self {
        use rand::thread_rng;
        let secret_key = SecretKey::random(&mut thread_rng());
        Self { inner: secret_key }
    }
    
    /// Get the public key
    pub fn public_key(&self) -> Secp256r1PublicKey {
        #[cfg(feature = "crypto-standard")]
        {
            // Create a SigningKey from the SecretKey, then get the VerifyingKey
            let signing_key = SigningKey::from(self.inner.clone());
            let verifying_key = signing_key.verifying_key();
            Secp256r1PublicKey { inner: verifying_key.clone() }
        }
        
        #[cfg(not(feature = "crypto-standard"))]
        {
            Secp256r1PublicKey { inner: vec![] }
        }
    }
    
    /// Sign a message
    pub fn sign_message(&self, message: &[u8]) -> Result<Secp256r1Signature, CryptoError> {
        #[cfg(feature = "crypto-standard")]
        {
            // Create a SigningKey from the SecretKey
            let signing_key = SigningKey::from(self.inner.clone());
            let signature = signing_key.sign(message);
            Ok(Secp256r1Signature { inner: signature })
        }
        
        #[cfg(not(feature = "crypto-standard"))]
        {
            Err(CryptoError::SigningError("Signing not available without crypto-standard feature".to_string()))
        }
    }
    
    /// Create a private key from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        #[cfg(feature = "crypto-standard")]
        {
            if bytes.len() != 32 {
                return Err(CryptoError::InvalidPrivateKey("Invalid private key length".to_string()));
            }
            
            SecretKey::from_slice(bytes)
                .map(|inner| Self { inner })
                .map_err(|_| CryptoError::InvalidPrivateKey("Invalid private key".to_string()))
        }
        
        #[cfg(not(feature = "crypto-standard"))]
        {
            Err(CryptoError::InvalidPrivateKey("Private key creation not available without crypto-standard feature".to_string()))
        }
    }
    
    /// Create a private key from slice
    pub fn from_slice(slice: &[u8]) -> Result<Self, CryptoError> {
        Self::from_bytes(slice)
    }
}

impl Secp256r1PublicKey {
    /// Get the encoded public key
    pub fn get_encoded(&self) -> Vec<u8> {
        #[cfg(feature = "crypto-standard")]
        {
            self.inner.to_encoded_point(true).as_bytes().to_vec()
        }
        
        #[cfg(not(feature = "crypto-standard"))]
        {
            vec![]
        }
    }
    
    /// Verify a signature
    pub fn verify_signature(&self, message: &[u8], signature: &Secp256r1Signature) -> Result<bool, CryptoError> {
        #[cfg(feature = "crypto-standard")]
        {
            match self.inner.verify(message, &signature.inner) {
                Ok(_) => Ok(true),
                Err(_) => Ok(false),
            }
        }
        
        #[cfg(not(feature = "crypto-standard"))]
        {
            Err(CryptoError::SignatureVerificationError("Signature verification not available without crypto-standard feature".to_string()))
        }
    }
    
    /// Create a public key from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        #[cfg(feature = "crypto-standard")]
        {
            VerifyingKey::from_sec1_bytes(bytes)
                .map(|inner| Self { inner })
                .map_err(|_| CryptoError::InvalidPublicKey("Invalid public key".to_string()))
        }
        
        #[cfg(not(feature = "crypto-standard"))]
        {
            Err(CryptoError::InvalidPublicKey("Public key creation not available without crypto-standard feature".to_string()))
        }
    }
    
    /// Create a public key from slice
    pub fn from_slice(slice: &[u8]) -> Result<Self, CryptoError> {
        Self::from_bytes(slice)
    }
}

impl fmt::Debug for Secp256r1PrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Secp256r1PrivateKey")
            .field("inner", &"[redacted]")
            .finish()
    }
}

impl fmt::Debug for Secp256r1PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Secp256r1PublicKey")
            .field("encoded", &self.get_encoded())
            .finish()
    }
}

impl fmt::Debug for Secp256r1Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(feature = "crypto-standard")]
        {
            write!(f, "Secp256r1Signature({:?})", self.inner)
        }
        
        #[cfg(not(feature = "crypto-standard"))]
        {
            write!(f, "Secp256r1Signature([redacted])")
        }
    }
}
