mod nep6account;
pub use nep6account::*;
mod nep6contract;
pub use nep6contract::*;
mod nep6wallet;
pub use nep6wallet::*;
mod wallet;
pub use wallet::*;
mod wallet_error;
pub use wallet_error::*;

#[cfg(all(feature = "yubihsm", not(target_arch = "wasm32")))]
mod yubi;

use crate::Signer;
use neo_crypto::keys::PrivateKeyExtension;
use p256::ecdsa::signature::hazmat::PrehashSigner;

// /// A neo private-public key pair which can be used for signing messages.
// ///
// /// # Examples
// ///
// /// ## Signing and Verifying a message
// ///
// /// The wallet can be used to produce ECDSA [`Signature`] objects, which can be
// /// then verified. Note that this uses [`hash_message`] under the hood which will
// /// prefix the message being hashed with the `neo Signed Message` domain separator.
// ///
// /// ```
// /// use rand::thread_rng;
// /// use neo_signers::{LocalWallet, Signer};
// ///
// /// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
// /// let wallet = LocalWallet::new(&mut thread_rng());
// ///
// /// // Optionally, the wallet's network magic can be set, in order to use
// /// // replay protection with different chains
// /// let wallet = wallet.with_network_magic(1337u64);
// ///
// /// // The wallet can be used to sign messages
// /// let message = b"hello";
// /// let signature = wallet.sign_message(message).await?;
// /// assert_eq!(signature.recover(&message[..]).unwrap(), wallet.address());
// ///
// /// // LocalWallet is clonable:
// /// let wallet_clone = wallet.clone();
// /// let signature2 = wallet_clone.sign_message(message).await?;
// /// assert_eq!(signature, signature2);
// /// # Ok(())
// /// # }
// /// ```
// ///
// /// [`Signature`]: neo_types::Signature
// /// [`hash_message`]: fn@neo_core::utils::hash_message
// #[derive(Clone)]
// pub struct Wallet {
// 	/// The Wallet's private Key
// 	pub signer: Secp256r1PrivateKey,
// 	/// The wallet's address
// 	pub address: AddressOrScriptHash,
// 	/// The wallet's network magic
// 	pub network_magic: u64,
// }
//
// impl Wallet {
// 	/// Construct a new wallet with an external Signer
// 	pub fn new_with_signer(
// 		signer: Secp256r1PrivateKey,
// 		address: Address,
// 		network_magic: u64,
// 	) -> Self {
// 		Wallet { signer, address: AddressOrScriptHash::Address(address), network_magic }
// 	}
// }
//
// #[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
// #[cfg_attr(not(target_arch = "wasm32"), async_trait)]
// impl Signer for Wallet {
// 	type Error = WalletError;
//
// 	async fn sign_message<S: Send + Sync + AsRef<[u8]>>(
// 		&self,
// 		message: S,
// 	) -> Result<Secp256r1Signature, Self::Error> {
// 		let message = message.as_ref();
// 		let message_hash = hash_message(message);
// 		self.sign_hash(message_hash)
// 	}
//
// 	async fn sign_transaction(&self, tx: &Transaction) -> Result<Secp256r1Signature, Self::Error> {
// 		let mut tx_with_chain = tx.clone();
// 		if tx_with_chain.network_magic().is_none() {
// 			// in the case we don't have a network_magic, let's use the signer network magic instead
// 			tx_with_chain.set_network_magic(self.network_magic);
// 		}
// 		self.sign_transaction_sync(&tx_with_chain)
// 	}
//
// 	fn address(&self) -> Address {
// 		self.address.address()
// 	}
//
// 	/// Gets the wallet's network magic
// 	fn network_magic(&self) -> u64 {
// 		self.network_magic
// 	}
//
// 	/// Sets the wallet's network_magic, used in conjunction with EIP-155 signing
// 	fn with_network_magic<T: Into<u64>>(mut self, network_magic: T) -> Self {
// 		self.network_magic = network_magic.into();
// 		self
// 	}
// }
//
// impl Wallet {
// 	pub fn sign_transaction_sync(
// 		&self,
// 		tx: &Transaction,
// 	) -> Result<Secp256r1Signature, WalletError> {
// 		let network_magic = tx.network_magic().unwrap_or(self.network_magic);
// 		let mut tx = tx.clone();
// 		tx.set_network_magic(network_magic);
//
// 		let sighash = tx.sighash();
// 		let mut sig = self.sign_hash(sighash)?;
// 		Ok(sig)
// 	}
//
// 	/// Signs the provided hash.
// 	pub fn sign_hash(&self, hash: H256) -> Result<Secp256r1Signature, WalletError> {
// 		let p256_sig = self
// 			.signer
// 			.sign_prehash(hash.as_ref())
// 			.map_err(|_| WalletError::SignHashError)?;
//
// 		let r = U256::from_big_endian(p256_sig.r().as_ref());
// 		let s = U256::from_big_endian(p256_sig.s().as_ref());
//
// 		Ok(Secp256r1Signature::from_u256(r, s))
// 	}
//
// 	/// Gets the wallet's signer
// 	pub fn signer(&self) -> &Secp256r1PrivateKey {
// 		&self.signer
// 	}
// }
//
// // do not log the signer
// impl fmt::Debug for Wallet {
// 	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
// 		f.debug_struct("Wallet")
// 			.field("address", &self.address)
// 			.field("chain_Id", &self.network_magic)
// 			.finish()
// 	}
// }
