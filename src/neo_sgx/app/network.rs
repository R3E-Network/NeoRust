use sgx_types::*;
use std::ffi::{CStr, CString};
use std::slice;
use reqwest::blocking::Client;

/// Sends a network request from the untrusted app
///
/// # Arguments
///
/// * `url` - The URL to send the request to
/// * `url_len` - The length of the URL string
/// * `method` - The HTTP method to use (e.g., "GET", "POST")
/// * `method_len` - The length of the method string
/// * `body` - The request body
/// * `body_len` - The length of the body string
/// * `response` - Buffer to store the response
/// * `response_len` - Pointer to store the length of the response
///
/// # Returns
///
/// SGX status code
#[no_mangle]
pub extern "C" fn ocall_send_request(
    ret_val: *mut sgx_status_t,
    url: *const u8,
    url_len: usize,
    method: *const u8,
    method_len: usize,
    body: *const u8,
    body_len: usize,
    response: *mut u8,
    response_len: *mut usize,
) -> sgx_status_t {
    let mut retval = sgx_status_t::SGX_SUCCESS;
    
    // Convert raw pointers to Rust strings
    let url_slice = unsafe { slice::from_raw_parts(url, url_len) };
    let method_slice = unsafe { slice::from_raw_parts(method, method_len) };
    let body_slice = unsafe { slice::from_raw_parts(body, body_len) };
    
    let url_str = match std::str::from_utf8(url_slice) {
        Ok(s) => s,
        Err(_) => {
            unsafe { *ret_val = sgx_status_t::SGX_ERROR_UNEXPECTED };
            return sgx_status_t::SGX_SUCCESS;
        }
    };
    
    let method_str = match std::str::from_utf8(method_slice) {
        Ok(s) => s,
        Err(_) => {
            unsafe { *ret_val = sgx_status_t::SGX_ERROR_UNEXPECTED };
            return sgx_status_t::SGX_SUCCESS;
        }
    };
    
    let body_str = match std::str::from_utf8(body_slice) {
        Ok(s) => s,
        Err(_) => {
            unsafe { *ret_val = sgx_status_t::SGX_ERROR_UNEXPECTED };
            return sgx_status_t::SGX_SUCCESS;
        }
    };
    
    // Create HTTP client
    let client = match Client::new() {
        Ok(c) => c,
        Err(_) => {
            unsafe { *ret_val = sgx_status_t::SGX_ERROR_UNEXPECTED };
            return sgx_status_t::SGX_SUCCESS;
        }
    };
    
    // Send request based on method
    let result = match method_str {
        "GET" => client.get(url_str).send(),
        "POST" => client.post(url_str).body(body_str.to_string()).send(),
        "PUT" => client.put(url_str).body(body_str.to_string()).send(),
        "DELETE" => client.delete(url_str).send(),
        _ => {
            unsafe { *ret_val = sgx_status_t::SGX_ERROR_UNEXPECTED };
            return sgx_status_t::SGX_SUCCESS;
        }
    };
    
    // Process response
    match result {
        Ok(res) => {
            match res.text() {
                Ok(text) => {
                    let text_bytes = text.as_bytes();
                    let max_len = unsafe { *response_len };
                    
                    if text_bytes.len() <= max_len {
                        unsafe {
                            std::ptr::copy_nonoverlapping(text_bytes.as_ptr(), response, text_bytes.len());
                            *response_len = text_bytes.len();
                        }
                    } else {
                        unsafe {
                            std::ptr::copy_nonoverlapping(text_bytes.as_ptr(), response, max_len);
                            *response_len = max_len;
                        }
                        retval = sgx_status_t::SGX_ERROR_UNEXPECTED;
                    }
                }
                Err(_) => {
                    retval = sgx_status_t::SGX_ERROR_UNEXPECTED;
                }
            }
        }
        Err(_) => {
            retval = sgx_status_t::SGX_ERROR_UNEXPECTED;
        }
    }
    
    unsafe { *ret_val = retval };
    sgx_status_t::SGX_SUCCESS
}
