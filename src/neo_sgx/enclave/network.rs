use sgx_types::*;

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

/// Sends a network request from within the enclave
///
/// # Arguments
///
/// * `url` - The URL to send the request to
/// * `method` - The HTTP method to use (e.g., "GET", "POST")
/// * `body` - The request body
///
/// # Returns
///
/// The response as a string
pub fn send_request(url: &str, method: &str, body: &str) -> Result<String, sgx_status_t> {
    // Allocate buffer for response
    let mut response_buf = vec![0u8; 4096]; // Adjust size as needed
    let mut response_len = response_buf.len();
    
    // Call the untrusted function
    let mut retval = sgx_status_t::SGX_SUCCESS;
    let status = unsafe {
        ocall_send_request(
            &mut retval,
            url.as_ptr(),
            url.len(),
            method.as_ptr(),
            method.len(),
            body.as_ptr(),
            body.len(),
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
