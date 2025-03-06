//! Implementation of crypto adapter traits from neo-common
//!
//! This module implements the traits defined in neo-common for the crypto types in this crate.

use neo_common::{EncodablePublicKey, PublicKey, external_to_common_public_key, common_to_external_public_key};
use crate::{Secp256r1PublicKey, CryptoError};

/// Implementation of EncodablePublicKey for Secp256r1PublicKey
impl EncodablePublicKey for Secp256r1PublicKey {
    fn get_encoded(&self, compressed: bool) -> Vec<u8> {
        self.get_encoded(compressed)
    }
}

/// Convert a Secp256r1PublicKey to a neo-common PublicKey
pub fn secp256r1_to_common_public_key(public_key: &Secp256r1PublicKey) -> PublicKey {
    external_to_common_public_key(public_key)
}

/// Convert a neo-common PublicKey to a Secp256r1PublicKey
pub fn common_to_secp256r1_public_key(public_key: &PublicKey) -> Result<Secp256r1PublicKey, CryptoError> {
    common_to_external_public_key(public_key, Secp256r1PublicKey::from_bytes)
}

#[cfg(feature = "with-common")]
mod provider_error_impl {
    use neo_common::ProviderError;
    use crate::CryptoError;

    /// Error conversion from CryptoError to ProviderError
    impl From<CryptoError> for ProviderError {
        fn from(err: CryptoError) -> Self {
            ProviderError::CryptoError(format!("{:?}", err))
        }
    }
}
