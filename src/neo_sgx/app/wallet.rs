#[cfg(feature = "sgx_deps")]
use sgx_types::*;
use std::path::Path;
#[cfg(feature = "sgx_deps")]
use sgx_urts::SgxEnclave;

/// Wallet functionality for the untrusted app
pub struct SgxWallet {
	#[cfg(feature = "sgx_deps")]
	enclave: SgxEnclave,
	#[cfg(feature = "sgx_deps")]
	wallet_id: sgx_enclave_id_t,
	#[cfg(not(feature = "sgx_deps"))]
	_private: (),
}

#[cfg(feature = "sgx_deps")]
extern "C" {
	fn ecall_create_wallet(
		eid: sgx_enclave_id_t,
		retval: *mut sgx_status_t,
		password: *const u8,
		password_len: usize,
		wallet_id: *mut sgx_enclave_id_t,
	) -> sgx_status_t;

	fn ecall_open_wallet(
		eid: sgx_enclave_id_t,
		retval: *mut sgx_status_t,
		path: *const u8,
		path_len: usize,
		password: *const u8,
		password_len: usize,
		wallet_id: *mut sgx_enclave_id_t,
	) -> sgx_status_t;

	fn ecall_sign_transaction(
		eid: sgx_enclave_id_t,
		retval: *mut sgx_status_t,
		private_key: *const u8,
		transaction_data: *const u8,
		len: usize,
		signature: *mut u8,
	) -> sgx_status_t;
}

#[cfg(feature = "sgx_deps")]
impl SgxWallet {
	/// Creates a new SGX wallet
	///
	/// # Arguments
	///
	/// * `enclave` - The SGX enclave instance
	/// * `password` - The password to encrypt the wallet
	///
	/// # Returns
	///
	/// A new SGX wallet instance
	pub fn new(enclave: SgxEnclave, password: &str) -> Result<Self, sgx_status_t> {
		let mut wallet_id: sgx_enclave_id_t = 0;
		let password_bytes = password.as_bytes();
		let mut retval = sgx_status_t::SGX_SUCCESS;

		let status = unsafe {
			ecall_create_wallet(
				enclave.geteid(),
				&mut retval,
				password_bytes.as_ptr(),
				password_bytes.len(),
				&mut wallet_id,
			)
		};

		if status != sgx_status_t::SGX_SUCCESS {
			return Err(status);
		}

		if retval != sgx_status_t::SGX_SUCCESS {
			return Err(retval);
		}

		Ok(Self { enclave, wallet_id })
	}

	/// Opens an existing SGX wallet
	///
	/// # Arguments
	///
	/// * `enclave` - The SGX enclave instance
	/// * `path` - The path to the wallet data
	/// * `password` - The password to decrypt the wallet
	///
	/// # Returns
	///
	/// A new SGX wallet instance
	pub fn open(enclave: SgxEnclave, path: &Path, password: &str) -> Result<Self, sgx_status_t> {
		let mut wallet_id: sgx_enclave_id_t = 0;
		let path_str = path.to_string_lossy();
		let path_bytes = path_str.as_bytes();
		let password_bytes = password.as_bytes();
		let mut retval = sgx_status_t::SGX_SUCCESS;

		let status = unsafe {
			ecall_open_wallet(
				enclave.geteid(),
				&mut retval,
				path_bytes.as_ptr(),
				path_bytes.len(),
				password_bytes.as_ptr(),
				password_bytes.len(),
				&mut wallet_id,
			)
		};

		if status != sgx_status_t::SGX_SUCCESS {
			return Err(status);
		}

		if retval != sgx_status_t::SGX_SUCCESS {
			return Err(retval);
		}

		Ok(Self { enclave, wallet_id })
	}

	/// Signs a transaction
	///
	/// # Arguments
	///
	/// * `transaction_data` - The transaction data to sign
	///
	/// # Returns
	///
	/// The signature
	pub fn sign_transaction(&self, transaction_data: &[u8]) -> Result<[u8; 65], sgx_status_t> {
		let mut signature = [0u8; 65];
		let mut retval = sgx_status_t::SGX_SUCCESS;

		// Note: In a real implementation, we would need to securely pass the private key
		// or have the enclave store it securely. This is simplified for demonstration.
		let private_key = [0u8; 32]; // Placeholder

		let status = unsafe {
			ecall_sign_transaction(
				self.enclave.geteid(),
				&mut retval,
				private_key.as_ptr(),
				transaction_data.as_ptr(),
				transaction_data.len(),
				signature.as_mut_ptr(),
			)
		};

		if status != sgx_status_t::SGX_SUCCESS {
			return Err(status);
		}

		if retval != sgx_status_t::SGX_SUCCESS {
			return Err(retval);
		}

		Ok(signature)
	}

	/// Gets the wallet's public key
	///
	/// # Returns
	///
	/// The public key
	pub fn get_public_key(&self) -> &[u8; 64] {
		&self.public_key
	}
}

#[cfg(not(feature = "sgx_deps"))]
impl SgxWallet {
	/// Creates a new SGX wallet
	///
	/// # Arguments
	///
	/// * `enclave` - The SGX enclave instance
	/// * `password` - The password to encrypt the wallet
	///
	/// # Returns
	///
	/// A new SGX wallet instance
	pub fn new(_enclave: (), _password: &str) -> Result<Self, ()> {
		unimplemented!("SGX dependencies not available")
	}

	/// Opens an existing SGX wallet
	///
	/// # Arguments
	///
	/// * `enclave` - The SGX enclave instance
	/// * `path` - The path to the wallet data
	/// * `password` - The password to decrypt the wallet
	///
	/// # Returns
	///
	/// A new SGX wallet instance
	pub fn open(_enclave: (), _path: &Path, _password: &str) -> Result<Self, ()> {
		unimplemented!("SGX dependencies not available")
	}

	/// Signs a transaction
	///
	/// # Arguments
	///
	/// * `transaction_data` - The transaction data to sign
	///
	/// # Returns
	///
	/// The signature
	pub fn sign_transaction(&self, _transaction_data: &[u8]) -> Result<[u8; 65], ()> {
		unimplemented!("SGX dependencies not available")
	}

	/// Gets the wallet's public key
	///
	/// # Returns
	///
	/// The public key
	pub fn get_public_key(&self) -> [u8; 64] {
		unimplemented!("SGX dependencies not available")
	}
}
