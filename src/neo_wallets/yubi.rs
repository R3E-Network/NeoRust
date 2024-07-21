//! Helpers for creating wallets for YubiHSM2
use elliptic_curve::sec1::{FromEncodedPoint, ToEncodedPoint};
use p256::{NistP256, PublicKey};
use yubihsm::{
    asymmetric::Algorithm::{EcK256, EcP256},
    Capability,
    Client,
    Connector,
    Credentials, Domain, ecdsa::Signer as YubiSigner, object, object::Label,
};

use neo::{
    neo_providers::public_key_to_address,
    prelude::{Secp256r1PublicKey, WalletSigner},
};

use crate::neo_types::Address;

use super::Wallet;

impl WalletSigner<YubiSigner<NistP256>> {
    /// Connects to a yubi key's ECDSA account at the provided id
    pub fn connect(connector: Connector, credentials: Credentials, id: object::Id) -> Self {
        let client = Client::open(connector, credentials, true).unwrap();
        let signer = YubiSigner::create(client, id).unwrap();
        signer.into()
    }

    /// Creates a new random ECDSA keypair on the yubi at the provided id
    pub fn new(
        connector: Connector,
        credentials: Credentials,
        id: object::Id,
        label: Label,
        domain: Domain,
    ) -> Self {
        let client = Client::open(connector, credentials, true).unwrap();
        let id = client
            .generate_asymmetric_key(id, label, domain, Capability::SIGN_ECDSA, EcP256)
            .unwrap();
        let signer = YubiSigner::create(client, id).unwrap();
        signer.into()
    }

    /// Uploads the provided keypair on the yubi at the provided id
    pub fn from_key(
        connector: Connector,
        credentials: Credentials,
        id: object::Id,
        label: Label,
        domain: Domain,
        key: impl Into<Vec<u8>>,
    ) -> Self {
        let client = Client::open(connector, credentials, true).unwrap();
        let id = client
            .put_asymmetric_key(id, label, domain, Capability::SIGN_ECDSA, EcK256, key)
            .unwrap();
        let signer = YubiSigner::create(client, id).unwrap();
        signer.into()
    }
}

impl From<YubiSigner<NistP256>> for WalletSigner<YubiSigner<NistP256>> {
    fn from(signer: YubiSigner<NistP256>) -> Self {
        // this will never fail
        let public_key = PublicKey::from_encoded_point(signer.public_key()).unwrap();
        let public_key = public_key.to_encoded_point(true);
        let public_key = public_key.as_bytes();
        debug_assert_eq!(public_key[0], 0x04);
        let address = public_key_to_address(&Secp256r1PublicKey::from_bytes(&public_key).unwrap());

        Self { signer, address, network: None }
    }
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[tokio::test]
    async fn from_key() {
        let key = hex::decode("2d8c44dc2dd2f0bea410e342885379192381e82d855b1b112f9b55544f1e0900")
            .unwrap();

        let connector = yubihsm::Connector::mockhsm();
        let wallet = Wallet::from_key(
            connector,
            Credentials::default(),
            0,
            Label::from_bytes(&[]).unwrap(),
            Domain::at(1).unwrap(),
            key,
        );

        let msg = "Some data";
        let sig = wallet.sign_message(msg).await.unwrap();
        assert_eq!(sig.recover(msg).unwrap(), wallet.address());
        assert_eq!(
            wallet.address(),
            Address::from_str("2DE2C386082Cff9b28D62E60983856CE1139eC49").unwrap()
        );
    }

    #[tokio::test]
    async fn new_key() {
        let connector = yubihsm::Connector::mockhsm();
        let wallet = WalletSigner::<YubiSigner<NistP256>>::new(
            connector,
            Credentials::default(),
            0,
            Label::from_bytes(&[]).unwrap(),
            Domain::at(1).unwrap(),
        );

        let msg = "Some data";
        let sig = wallet.sign_message(msg.as_bytes()).await.unwrap();
        assert!(wallet.verify_message(msg.as_bytes(), &sig).await.is_ok());
    }
}
