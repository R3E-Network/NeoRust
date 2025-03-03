#[cfg(feature = "sgx_deps")]
use sgx_types::*;
use std::{
	fs::{self, File},
	io::{Read, Write},
	path::Path,
	slice,
};

/// Storage functionality for the untrusted app
pub struct SgxStorage {
	#[cfg(not(feature = "sgx_deps"))]
	_private: (),
}

#[cfg(feature = "sgx_deps")]
impl SgxStorage {
	/// Creates a new SgxStorage instance
	///
	/// # Returns
	///
	/// A new SgxStorage instance
	pub fn new() -> Self {
		Self {}
	}

	/// Saves data to a file
	///
	/// # Arguments
	///
	/// * `path` - Path to save the data to
	/// * `data` - Data to save
	///
	/// # Returns
	///
	/// `true` if successful, `false` otherwise
	pub fn save_data(&self, path: &Path, data: &[u8]) -> bool {
		if let Ok(mut file) = File::create(path) {
			if let Ok(_) = file.write_all(data) {
				return true;
			}
		}
		false
	}

	/// Loads data from a file
	///
	/// # Arguments
	///
	/// * `path` - Path to load the data from
	///
	/// # Returns
	///
	/// The loaded data, or None if an error occurred
	pub fn load_data(&self, path: &Path) -> Option<Vec<u8>> {
		if let Ok(mut file) = File::open(path) {
			let mut data = Vec::new();
			if let Ok(_) = file.read_to_end(&mut data) {
				return Some(data);
			}
		}
		None
	}
}

#[cfg(not(feature = "sgx_deps"))]
impl SgxStorage {
	/// Creates a new SgxStorage instance
	///
	/// # Returns
	///
	/// A new SgxStorage instance
	pub fn new() -> Self {
		Self { _private: () }
	}

	/// Placeholder for saving data to a file
	pub fn save_data(&self, _path: &Path, _data: &[u8]) -> bool {
		unimplemented!("SGX dependencies not available")
	}

	/// Placeholder for loading data from a file
	pub fn load_data(&self, _path: &Path) -> Option<Vec<u8>> {
		unimplemented!("SGX dependencies not available")
	}
}

/// Reads a file from the untrusted app
///
/// # Arguments
///
/// * `filename` - The path to the file to read
/// * `filename_len` - The length of the filename string
/// * `data` - Buffer to store the file contents
/// * `data_len` - Pointer to store the length of the data
///
/// # Returns
///
/// SGX status code
#[cfg(feature = "sgx_deps")]
#[no_mangle]
pub extern "C" fn ocall_read_file(
	ret_val: *mut sgx_status_t,
	filename: *const u8,
	filename_len: usize,
	data: *mut u8,
	data_len: *mut usize,
) -> sgx_status_t {
	let mut retval = sgx_status_t::SGX_SUCCESS;

	// Convert raw pointer to Rust string
	let filename_slice = unsafe { slice::from_raw_parts(filename, filename_len) };
	let filename_str = match std::str::from_utf8(filename_slice) {
		Ok(s) => s,
		Err(_) => {
			unsafe { *ret_val = sgx_status_t::SGX_ERROR_UNEXPECTED };
			return sgx_status_t::SGX_SUCCESS;
		},
	};

	// Read file
	match File::open(filename_str) {
		Ok(mut file) => {
			let mut contents = Vec::new();
			match file.read_to_end(&mut contents) {
				Ok(_) => {
					let max_len = unsafe { *data_len };

					if contents.len() <= max_len {
						unsafe {
							std::ptr::copy_nonoverlapping(contents.as_ptr(), data, contents.len());
							*data_len = contents.len();
						}
					} else {
						unsafe {
							std::ptr::copy_nonoverlapping(contents.as_ptr(), data, max_len);
							*data_len = max_len;
						}
						retval = sgx_status_t::SGX_ERROR_UNEXPECTED;
					}
				},
				Err(_) => {
					retval = sgx_status_t::SGX_ERROR_UNEXPECTED;
				},
			}
		},
		Err(_) => {
			retval = sgx_status_t::SGX_ERROR_UNEXPECTED;
		},
	}

	unsafe { *ret_val = retval };
	sgx_status_t::SGX_SUCCESS
}

/// Writes data to a file from the untrusted app
///
/// # Arguments
///
/// * `filename` - The path to the file to write
/// * `filename_len` - The length of the filename string
/// * `data` - The data to write to the file
/// * `data_len` - The length of the data
///
/// # Returns
///
/// SGX status code
#[cfg(feature = "sgx_deps")]
#[no_mangle]
pub extern "C" fn ocall_write_file(
	ret_val: *mut sgx_status_t,
	filename: *const u8,
	filename_len: usize,
	data: *const u8,
	data_len: usize,
) -> sgx_status_t {
	let mut retval = sgx_status_t::SGX_SUCCESS;

	// Convert raw pointer to Rust string
	let filename_slice = unsafe { slice::from_raw_parts(filename, filename_len) };
	let filename_str = match std::str::from_utf8(filename_slice) {
		Ok(s) => s,
		Err(_) => {
			unsafe { *ret_val = sgx_status_t::SGX_ERROR_UNEXPECTED };
			return sgx_status_t::SGX_SUCCESS;
		},
	};

	// Create parent directories if they don't exist
	if let Some(parent) = Path::new(filename_str).parent() {
		if !parent.exists() {
			if let Err(_) = fs::create_dir_all(parent) {
				unsafe { *ret_val = sgx_status_t::SGX_ERROR_UNEXPECTED };
				return sgx_status_t::SGX_SUCCESS;
			}
		}
	}

	// Write file
	match File::create(filename_str) {
		Ok(mut file) => {
			let data_slice = unsafe { slice::from_raw_parts(data, data_len) };
			if let Err(_) = file.write_all(data_slice) {
				retval = sgx_status_t::SGX_ERROR_UNEXPECTED;
			}
		},
		Err(_) => {
			retval = sgx_status_t::SGX_ERROR_UNEXPECTED;
		},
	}

	unsafe { *ret_val = retval };
	sgx_status_t::SGX_SUCCESS
}

/// Logs a message from the enclave
///
/// # Arguments
///
/// * `message` - The message to log
/// * `message_len` - The length of the message
///
/// # Returns
///
/// SGX status code
#[cfg(feature = "sgx_deps")]
#[no_mangle]
pub extern "C" fn ocall_log(message: *const u8, message_len: usize) -> sgx_status_t {
	// Convert raw pointer to Rust string
	let message_slice = unsafe { slice::from_raw_parts(message, message_len) };
	if let Ok(message_str) = std::str::from_utf8(message_slice) {
		println!("[SGX Enclave] {}", message_str);
	}

	sgx_status_t::SGX_SUCCESS
}
