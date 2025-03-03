#[cfg(feature = "sgx_deps")]
use reqwest::blocking::Client;
#[cfg(feature = "sgx_deps")]
use sgx_types::*;
use std::{
	ffi::{CStr, CString},
	slice,
};

/// Network-related functionality for SGX operations
pub struct SgxNetwork {
	#[cfg(feature = "sgx_deps")]
	client: Client,
	#[cfg(not(feature = "sgx_deps"))]
	_private: (),
}

#[cfg(feature = "sgx_deps")]
impl SgxNetwork {
	/// Creates a new SgxNetwork instance
	///
	/// # Returns
	///
	/// A new SgxNetwork instance
	pub fn new() -> Self {
		Self {
			client: Client::new(),
		}
	}

	// Implement any other methods using the client
}

#[cfg(not(feature = "sgx_deps"))]
impl SgxNetwork {
	/// Creates a new SgxNetwork instance
	///
	/// # Returns
	///
	/// A new SgxNetwork instance
	pub fn new() -> Self {
		Self { _private: () }
	}

	// Placeholder implementations for the non-SGX version
}

#[cfg(feature = "sgx_deps")]
/// Sends a network request from the untrusted app
///
/// # Arguments
///
/// * `eid` - Enclave ID
/// * `ret_val` - Return value
/// * `method` - HTTP method (GET, POST, PUT, DELETE)
/// * `url` - Request URL
/// * `body` - Request body
/// * `response` - Buffer to store the response
/// * `response_len` - Length of the response buffer
/// * `response_len_out` - Actual length of the response
///
/// # Returns
///
/// SGX status
pub unsafe fn send_request(
	eid: sgx_enclave_id_t,
	ret_val: *mut sgx_status_t,
	method: *const u8,
	url: *const u8,
	body: *const u8,
	response: *mut u8,
	response_len: usize,
	response_len_out: *mut usize,
) -> sgx_status_t {
	if method.is_null() || url.is_null() || body.is_null() || response.is_null() {
		unsafe { *ret_val = sgx_status_t::SGX_ERROR_UNEXPECTED };
		return sgx_status_t::SGX_SUCCESS;
	}

	// Convert raw pointers to Rust strings
	let method_str = CStr::from_ptr(method as *const i8).to_str().unwrap_or("");
	let url_str = CStr::from_ptr(url as *const i8).to_str().unwrap_or("");
	let body_str = CStr::from_ptr(body as *const i8).to_str().unwrap_or("");

	let network = SgxNetwork::new();

	// Send request based on method
	let client = match method_str {
		"GET" => network.client.get(url_str).send(),
		"POST" => network.client.post(url_str).body(body_str.to_string()).send(),
		"PUT" => network.client.put(url_str).body(body_str.to_string()).send(),
		"DELETE" => network.client.delete(url_str).send(),
		_ => {
			unsafe { *ret_val = sgx_status_t::SGX_ERROR_UNEXPECTED };
			return sgx_status_t::SGX_SUCCESS;
		}
	};

	// Process response
	match client {
		Ok(res) => match res.text() {
			Ok(text) => {
				// Copy response to buffer
				let text_bytes = text.as_bytes();
				let len = std::cmp::min(text_bytes.len(), response_len);
				unsafe {
					slice::from_raw_parts_mut(response, len).copy_from_slice(&text_bytes[..len]);
					*response_len_out = len;
					*ret_val = sgx_status_t::SGX_SUCCESS;
				}
			}
			Err(_) => {
				unsafe { *ret_val = sgx_status_t::SGX_ERROR_UNEXPECTED };
			}
		},
		Err(_) => {
			unsafe { *ret_val = sgx_status_t::SGX_ERROR_UNEXPECTED };
		}
	}

	sgx_status_t::SGX_SUCCESS
}
