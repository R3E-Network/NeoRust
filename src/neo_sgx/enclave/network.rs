#[cfg(feature = "sgx_deps")]
use sgx_types::*;
#[cfg(feature = "sgx_deps")]
use sgx_tstd::*;

#[cfg(feature = "sgx_deps")]
extern "C" {
	fn ocall_send_request(
		ret_val: *mut sgx_status_t,
		url: *const u8,
		url_len: usize,
		method: *const u8,
		method_len: usize,
		body: *const u8,
		body_len: usize,
		response: *mut u8,
		response_len: *mut usize,
	) -> sgx_status_t;
}

/// Network-related functionality for the enclave
pub struct EnclaveNetwork {}

#[cfg(feature = "sgx_deps")]
impl EnclaveNetwork {
	/// Creates a new EnclaveNetwork instance
	///
	/// # Returns
	///
	/// A new EnclaveNetwork instance
	pub fn new() -> Self {
		Self {}
	}

	/// Sends a request to a URL
	///
	/// # Arguments
	///
	/// * `url` - The URL to send the request to
	/// * `method` - The HTTP method to use
	/// * `body` - The request body
	///
	/// # Returns
	///
	/// The response as a string
	pub fn send_request(&self, url: &str, method: &str, body: &str) -> Result<String, sgx_status_t> {
		// Allocate buffer for response
		let mut response_buf = vec![0u8; 4096]; // Adjust size as needed
		let mut response_len = response_buf.len();

		// Call the untrusted function
		let mut retval = sgx_status_t::SGX_SUCCESS;
		let url_bytes = url.as_bytes();
		let method_bytes = method.as_bytes();
		let body_bytes = body.as_bytes();
        
		let status = unsafe {
			ocall_send_request(
				&mut retval,
				url_bytes.as_ptr(),
				url_bytes.len(),
				method_bytes.as_ptr(),
				method_bytes.len(),
				body_bytes.as_ptr(),
				body_bytes.len(),
				response_buf.as_mut_ptr(),
				&mut response_len,
			)
		};

		// Check for errors
		if status != sgx_status_t::SGX_SUCCESS {
			return Err(status);
		}
		if retval != sgx_status_t::SGX_SUCCESS {
			return Err(retval);
		}

		// Convert response to string
		response_buf.truncate(response_len);
		let response_str = match std::str::from_utf8(&response_buf) {
			Ok(s) => s.to_string(),
			Err(_) => return Err(sgx_status_t::SGX_ERROR_UNEXPECTED),
		};

		Ok(response_str)
	}
}

#[cfg(not(feature = "sgx_deps"))]
impl EnclaveNetwork {
	/// Creates a new EnclaveNetwork instance
	///
	/// # Returns
	///
	/// A new EnclaveNetwork instance
	pub fn new() -> Self {
		Self {}
	}

	/// Sends a request to a URL (non-SGX implementation)
	///
	/// # Arguments
	///
	/// * `url` - The URL to send the request to
	/// * `method` - The HTTP method to use
	/// * `body` - The request body
	///
	/// # Returns
	///
	/// The response as a string
	pub fn send_request(&self, _url: &str, _method: &str, _body: &str) -> Result<String, ()> {
		unimplemented!("SGX dependencies not available")
	}
}
