use sgx_types::*;
use sgx_urts::SgxEnclave;

/// Wrapper for cryptographic operations in the untrusted app
pub struct SgxCrypto {
	enclave: SgxEnclave,
}

extern "C" {
	fn ecall_generate_keypair(
		eid: sgx_enclave_id_t,
		retval: *mut sgx_status_t,
		private_key: *mut u8,
		public_key: *mut u8,
	) -> sgx_status_t;

	fn ecall_sign_message(
		eid: sgx_enclave_id_t,
		retval: *mut sgx_status_t,
		private_key: *const u8,
		message: *const u8,
		len: usize,
		signature: *mut u8,
	) -> sgx_status_t;

	fn ecall_verify_signature(
		eid: sgx_enclave_id_t,
		retval: *mut sgx_status_t,
		public_key: *const u8,
		message: *const u8,
		len: usize,
		signature: *const u8,
		result: *mut i32,
	) -> sgx_status_t;
}

impl SgxCrypto {
	/// Creates a new SgxCrypto instance
	///
	/// # Arguments
	///
	/// * `enclave` - The SGX enclave instance
	///
	/// # Returns
	///
	/// A new SgxCrypto instance
	pub fn new(enclave: SgxEnclave) -> Self {
		Self { enclave }
	}

	/// Generates a new keypair
	///
	/// # Returns
	///
	/// A tuple containing the private key (32 bytes) and public key (64 bytes)
	pub fn generate_keypair(&self) -> Result<([u8; 32], [u8; 64]), sgx_status_t> {
		let mut private_key = [0u8; 32];
		let mut public_key = [0u8; 64];
		let mut retval = sgx_status_t::SGX_SUCCESS;

		let status = unsafe {
			ecall_generate_keypair(
				self.enclave.geteid(),
				&mut retval,
				private_key.as_mut_ptr(),
				public_key.as_mut_ptr(),
			)
		};

		if status != sgx_status_t::SGX_SUCCESS {
			return Err(status);
		}

		if retval != sgx_status_t::SGX_SUCCESS {
			return Err(retval);
		}

		Ok((private_key, public_key))
	}

	/// Signs a message using the provided private key
	///
	/// # Arguments
	///
	/// * `private_key` - A 32-byte private key
	/// * `message` - The message to sign
	///
	/// # Returns
	///
	/// A 65-byte signature
	pub fn sign_message(
		&self,
		private_key: &[u8; 32],
		message: &[u8],
	) -> Result<[u8; 65], sgx_status_t> {
		let mut signature = [0u8; 65];
		let mut retval = sgx_status_t::SGX_SUCCESS;

		let status = unsafe {
			ecall_sign_message(
				self.enclave.geteid(),
				&mut retval,
				private_key.as_ptr(),
				message.as_ptr(),
				message.len(),
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

	/// Verifies a signature against a message and public key
	///
	/// # Arguments
	///
	/// * `public_key` - A 64-byte public key
	/// * `message` - The message that was signed
	/// * `signature` - A 65-byte signature
	///
	/// # Returns
	///
	/// `true` if the signature is valid, `false` otherwise
	pub fn verify_signature(
		&self,
		public_key: &[u8; 64],
		message: &[u8],
		signature: &[u8; 65],
	) -> Result<bool, sgx_status_t> {
		let mut result = 0i32;
		let mut retval = sgx_status_t::SGX_SUCCESS;

		let status = unsafe {
			ecall_verify_signature(
				self.enclave.geteid(),
				&mut retval,
				public_key.as_ptr(),
				message.as_ptr(),
				message.len(),
				signature.as_ptr(),
				&mut result,
			)
		};

		if status != sgx_status_t::SGX_SUCCESS {
			return Err(status);
		}

		if retval != sgx_status_t::SGX_SUCCESS {
			return Err(retval);
		}

		Ok(result != 0)
	}
}
