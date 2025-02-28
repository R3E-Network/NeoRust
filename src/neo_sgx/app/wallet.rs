use sgx_types::*;
#[cfg(feature = "sgx_deps")]
use sgx_urts::SgxEnclave;

/// Wrapper for wallet operations in the untrusted app
pub struct SgxWallet {
	enclave: SgxEnclave,
	public_key: [u8; 64],
}

extern "C" {
	fn ecall_create_wallet(
		eid: sgx_enclave_id_t,
		retval: *mut sgx_status_t,
		password: *const u8,
		password_len: usize,
		wallet_data: *mut u8,
		wallet_data_len: *mut usize,
	) -> sgx_status_t;

	fn ecall_open_wallet(
		eid: sgx_enclave_id_t,
		retval: *mut sgx_status_t,
		wallet_data: *const u8,
		wallet_data_len: usize,
		password: *const u8,
		password_len: usize,
		result: *mut i32,
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
		let mut wallet_data = vec![0u8; 4096]; // Adjust size as needed
		let mut wallet_data_len = wallet_data.len();
		let mut retval = sgx_status_t::SGX_SUCCESS;

		let status = unsafe {
			ecall_create_wallet(
				enclave.geteid(),
				&mut retval,
				password.as_ptr(),
				password.len(),
				wallet_data.as_mut_ptr(),
				&mut wallet_data_len,
			)
		};

		if status != sgx_status_t::SGX_SUCCESS {
			return Err(status);
		}

		if retval != sgx_status_t::SGX_SUCCESS {
			return Err(retval);
		}

		// Extract public key from wallet data
		// For simplicity, assume the first 64 bytes are the public key
		let mut public_key = [0u8; 64];
		public_key.copy_from_slice(&wallet_data[0..64]);

		Ok(Self { enclave, public_key })
	}

	/// Opens an existing SGX wallet
	///
	/// # Arguments
	///
	/// * `enclave` - The SGX enclave instance
	/// * `wallet_data` - The encrypted wallet data
	/// * `password` - The password to decrypt the wallet
	///
	/// # Returns
	///
	/// A new SGX wallet instance
	pub fn open(
		enclave: SgxEnclave,
		wallet_data: &[u8],
		password: &str,
	) -> Result<Self, sgx_status_t> {
		let mut result = 0i32;
		let mut retval = sgx_status_t::SGX_SUCCESS;

		let status = unsafe {
			ecall_open_wallet(
				enclave.geteid(),
				&mut retval,
				wallet_data.as_ptr(),
				wallet_data.len(),
				password.as_ptr(),
				password.len(),
				&mut result,
			)
		};

		if status != sgx_status_t::SGX_SUCCESS {
			return Err(status);
		}

		if retval != sgx_status_t::SGX_SUCCESS {
			return Err(retval);
		}

		if result == 0 {
			return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
		}

		// Extract public key from wallet data
		// For simplicity, assume the first 64 bytes are the public key
		let mut public_key = [0u8; 64];
		public_key.copy_from_slice(&wallet_data[0..64]);

		Ok(Self { enclave, public_key })
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
