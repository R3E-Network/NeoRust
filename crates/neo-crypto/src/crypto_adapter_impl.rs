//! Implementation of crypto adapter traits
//!
//! This module implements the traits for the crypto types in this crate.

use crate::keys::Secp256r1PublicKey;
use neo_error::crypto_error::CryptoError;

/// A trait for public keys that can be encoded
pub trait EncodablePublicKey {
    /// Get the encoded representation of the public key
    fn get_encoded(&self, compressed: bool) -> Vec<u8>;
}

/// Implementation of EncodablePublicKey for Secp256r1PublicKey
impl EncodablePublicKey for Secp256r1PublicKey {
    fn get_encoded(&self, compressed: bool) -> Vec<u8> {
        self.get_encoded(compressed)
    }
}

/// A simple public key structure
#[derive(Debug, Clone, PartialEq)]
pub struct PublicKey {
    /// The raw bytes of the public key
    pub bytes: Vec<u8>,
}

/// Convert a Secp256r1PublicKey to a PublicKey
pub fn secp256r1_to_public_key(public_key: &Secp256r1PublicKey) -> PublicKey {
    PublicKey {
        bytes: public_key.get_encoded(true),
    }
}

/// Convert a PublicKey to a Secp256r1PublicKey
pub fn public_key_to_secp256r1(public_key: &PublicKey) -> Result<Secp256r1PublicKey, CryptoError> {
    Secp256r1PublicKey::from_bytes(&public_key.bytes)
}

#[cfg(feature = "with-common")]
mod provider_error_impl {
    use neo_error::provider_error::ProviderError;
    use neo_error::crypto_error::CryptoError;

    /// Error conversion from CryptoError to ProviderError
    impl From<CryptoError> for ProviderError {
        fn from(err: CryptoError) -> Self {
            ProviderError::CryptoError(format!("{:?}", err))
        }
    }
}
