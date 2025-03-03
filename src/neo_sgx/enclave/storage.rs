#[cfg(feature = "sgx_deps")]
use sgx_types::*;
#[cfg(feature = "sgx_deps")]
use sgx_tstd::*;

#[cfg(feature = "sgx_deps")]
extern "C" {
	fn ocall_read_file(
		ret_val: *mut sgx_status_t,
		filename: *const u8,
		filename_len: usize,
		data: *mut u8,
		data_len: *mut usize,
	) -> sgx_status_t;

	fn ocall_write_file(
		ret_val: *mut sgx_status_t,
		filename: *const u8,
		filename_len: usize,
		data: *const u8,
		data_len: usize,
	) -> sgx_status_t;
}

/// Reads a file from outside the enclave
///
/// # Arguments
///
/// * `filename` - The path to the file to read
///
/// # Returns
///
/// The file contents as a byte vector
#[cfg(feature = "sgx_deps")]
pub fn read_file(filename: &str) -> Result<Vec<u8>, sgx_status_t> {
	// Allocate buffer for file data
	let mut data_buf = vec![0u8; 4096]; // Adjust size as needed
	let mut data_len = data_buf.len();

	// Call the untrusted function
	let mut retval = sgx_status_t::SGX_SUCCESS;
	let filename_bytes = filename.as_bytes();
	let status = unsafe {
		ocall_read_file(
			&mut retval,
			filename_bytes.as_ptr(),
			filename_bytes.len(),
			data_buf.as_mut_ptr(),
			&mut data_len,
		)
	};

	if status != sgx_status_t::SGX_SUCCESS {
		return Err(status);
	}

	if retval != sgx_status_t::SGX_SUCCESS {
		return Err(retval);
	}

	// Resize the buffer to the actual size
	data_buf.resize(data_len, 0);
	Ok(data_buf)
}

/// Writes data to a file outside the enclave
///
/// # Arguments
///
/// * `filename` - The path to the file to write
/// * `data` - The data to write
///
/// # Returns
///
/// Success or error
#[cfg(feature = "sgx_deps")]
pub fn write_file(filename: &str, data: &[u8]) -> Result<(), sgx_status_t> {
	let mut retval = sgx_status_t::SGX_SUCCESS;
	let filename_bytes = filename.as_bytes();
	let status = unsafe {
		ocall_write_file(
			&mut retval,
			filename_bytes.as_ptr(),
			filename_bytes.len(),
			data.as_ptr(),
			data.len(),
		)
	};

	if status != sgx_status_t::SGX_SUCCESS {
		return Err(status);
	}

	if retval != sgx_status_t::SGX_SUCCESS {
		return Err(retval);
	}

	Ok(())
}

/// Reads a file from outside the enclave (non-SGX implementation)
///
/// # Arguments
///
/// * `filename` - The path to the file to read
///
/// # Returns
///
/// The file contents as a byte vector
#[cfg(not(feature = "sgx_deps"))]
pub fn read_file(filename: &str) -> Result<Vec<u8>, ()> {
	unimplemented!("SGX dependencies not available")
}

/// Writes data to a file outside the enclave (non-SGX implementation)
///
/// # Arguments
///
/// * `filename` - The path to the file to write
/// * `data` - The data to write
///
/// # Returns
///
/// Success or error
#[cfg(not(feature = "sgx_deps"))]
pub fn write_file(filename: &str, data: &[u8]) -> Result<(), ()> {
	unimplemented!("SGX dependencies not available")
}
