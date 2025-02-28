use sgx_types::*;

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
pub fn read_file(filename: &str) -> Result<Vec<u8>, sgx_status_t> {
    // Allocate buffer for file data
    let mut data_buf = vec![0u8; 4096]; // Adjust size as needed
    let mut data_len = data_buf.len();
    
    // Call the untrusted function
    let mut retval = sgx_status_t::SGX_SUCCESS;
    let status = unsafe {
        ocall_read_file(
            &mut retval,
            filename.as_ptr(),
            filename.len(),
            data_buf.as_mut_ptr(),
            &mut data_len,
        )
    };
    
    // Check for errors
    if status != sgx_status_t::SGX_SUCCESS {
        return Err(status);
    }
    if retval != sgx_status_t::SGX_SUCCESS {
        return Err(retval);
    }
    
    // Truncate buffer to actual data length
    data_buf.truncate(data_len);
    
    Ok(data_buf)
}

/// Writes data to a file outside the enclave
///
/// # Arguments
///
/// * `filename` - The path to the file to write
/// * `data` - The data to write to the file
///
/// # Returns
///
/// `Ok(())` if successful, `Err` otherwise
pub fn write_file(filename: &str, data: &[u8]) -> Result<(), sgx_status_t> {
    // Call the untrusted function
    let mut retval = sgx_status_t::SGX_SUCCESS;
    let status = unsafe {
        ocall_write_file(
            &mut retval,
            filename.as_ptr(),
            filename.len(),
            data.as_ptr(),
            data.len(),
        )
    };
    
    // Check for errors
    if status != sgx_status_t::SGX_SUCCESS {
        return Err(status);
    }
    if retval != sgx_status_t::SGX_SUCCESS {
        return Err(retval);
    }
    
    Ok(())
}
