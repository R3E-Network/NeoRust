//! Helpers for creating wallets for YubiHSM2
use elliptic_curve::sec1::{FromEncodedPoint, ToEncodedPoint};
use p256::{NistP256, PublicKey};
use signature::Verifier;
use yubihsm::{
	asymmetric::Algorithm::EcP256, ecdsa::Signer as YubiSigner, object, object::Label, Capability,
	Client, Connector, Credentials, Domain,
};

use neo::{
	neo_providers::public_key_to_address,
	prelude::{Secp256r1PublicKey, WalletSigner},
};

use crate::{crypto::HashableForVec, neo_types::Address};

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
			.put_asymmetric_key(id, label, domain, Capability::SIGN_ECDSA, EcP256, key)
			.unwrap();
		let signer = YubiSigner::create(client, id).unwrap();
		signer.into()
	}

	// /// Verifies a given message and signature.
	// /// The message will be hashed using the `Sha256` algorithm before being verified.
	// /// If the signature is valid, the method will return `Ok(())`, otherwise it will return a `WalletError`.
	// /// # Arguments
	// /// * `message` - The message to be verified.
	// /// * `signature` - The signature to be verified.
	// /// # Returns
	// /// A `Result` containing `Ok(())` if the signature is valid, or a `WalletError` on failure.
	// pub async fn verify_message(
	// 	&self,
	// 	message: &[u8],
	// 	signature: &Signature<NistP256>,
	// ) -> Result<(), WalletError> {
	// 	let hash = message.hash256();
	// 	// let hash = H256::from_slice(hash.as_slice());
	// 	let verify_key = p256::ecdsa::VerifyingKey::from_encoded_point(self.signer.public_key()).unwrap();
	// 	match verify_key.verify(hash, &signature) {
	// 		Ok(_) => Ok(()),
	// 		Err(_) => Err(WalletError::VerifyError),
	// 	}
	// 	// let signature: ecdsa::Signature<NistP256> = signer.sign(TEST_MESSAGE);
	// }
}

impl From<YubiSigner<NistP256>> for WalletSigner<YubiSigner<NistP256>> {
	fn from(signer: YubiSigner<NistP256>) -> Self {
		// this will never fail
		let public_key = PublicKey::from_encoded_point(signer.public_key()).unwrap();
		let public_key = public_key.to_encoded_point(true);
		let public_key = public_key.as_bytes();
		debug_assert_eq!(public_key[0], 0x02);
		let address = public_key_to_address(&Secp256r1PublicKey::from_bytes(&public_key).unwrap());

		Self { signer, address, network: None }
	}
}

#[cfg(test)]
pub mod tests {
	use std::str::FromStr;

	use super::*;

	#[tokio::test]
	async fn from_key() {
		let key = hex::decode("2d8c44dc2dd2f0bea410e342885379192381e82d855b1b112f9b55544f1e0900")
			.unwrap();

		let connector = yubihsm::Connector::mockhsm();
		let wallet = WalletSigner::from_key(
			connector,
			Credentials::default(),
			0,
			Label::from_bytes(&[]).unwrap(),
			Domain::at(1).unwrap(),
			key,
		);

		let msg = "Some data";
		let sig = wallet.sign_message(msg.as_bytes()).await.unwrap();

		let verify_key =
			p256::ecdsa::VerifyingKey::from_encoded_point(wallet.signer.public_key()).unwrap();
		assert!(verify_key.verify(msg.as_bytes(), &sig).is_ok());

		assert_eq!(
			wallet.address(),
			Address::from_str("NPZyWCdSCWghLM7hcxT5kgc7cC2V2RGeHZ").unwrap()
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
		// assert!(wallet.verify_message(msg.as_bytes(), &sig).await.is_ok());
		let verify_key =
			p256::ecdsa::VerifyingKey::from_encoded_point(wallet.signer.public_key()).unwrap();
		assert!(verify_key.verify(msg.as_bytes(), &sig).is_ok());
	}
}
