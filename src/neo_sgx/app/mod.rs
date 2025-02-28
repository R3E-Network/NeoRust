pub mod crypto;
pub mod network;
pub mod storage;
pub mod wallet;

use sgx_types::*;
use sgx_urts::SgxEnclave;
use std::path::Path;

pub use crypto::*;
pub use network::*;
pub use storage::*;
pub use wallet::*;

/// Main SGX enclave wrapper for the untrusted app
pub struct SgxEnclaveManager {
    enclave: SgxEnclave,
}

impl SgxEnclaveManager {
    /// Creates a new SGX enclave manager
    ///
    /// # Arguments
    ///
    /// * `enclave_path` - Path to the enclave shared object file
    ///
    /// # Returns
    ///
    /// A new SGX enclave manager
    pub fn new(enclave_path: &str) -> Result<Self, sgx_status_t> {
        let mut launch_token: sgx_launch_token_t = [0; 1024];
        let mut launch_token_updated: i32 = 0;
        let mut misc_attr = sgx_misc_attribute_t {
            secs_attr: sgx_attributes_t { flags: 0, xfrm: 0 },
            misc_select: 0,
        };
        
        // Check if enclave file exists
        if !Path::new(enclave_path).exists() {
            println!("Error: Enclave file not found at {}", enclave_path);
            return Err(sgx_status_t::SGX_ERROR_UNEXPECTED);
        }
        
        // Create the enclave
        let enclave = match SgxEnclave::create(
            enclave_path,
            sgx_debug_flag_t::SGX_DEBUG_FLAG_ON as i32,
            &mut launch_token,
            &mut launch_token_updated,
            &mut misc_attr,
        ) {
            Ok(enclave) => enclave,
            Err(sgx_status) => {
                println!("Error: Failed to create enclave: {:?}", sgx_status);
                return Err(sgx_status);
            }
        };
        
        Ok(Self { enclave })
    }
    
    /// Gets a reference to the enclave
    ///
    /// # Returns
    ///
    /// A reference to the enclave
    pub fn get_enclave(&self) -> &SgxEnclave {
        &self.enclave
    }
    
    /// Creates a new SGX crypto instance
    ///
    /// # Returns
    ///
    /// A new SGX crypto instance
    pub fn create_crypto(&self) -> SgxCrypto {
        SgxCrypto::new(self.enclave.clone())
    }
    
    /// Creates a new SGX wallet instance
    ///
    /// # Returns
    ///
    /// A new SGX wallet instance
    pub fn create_wallet(&self, password: &str) -> Result<SgxWallet, sgx_status_t> {
        SgxWallet::new(self.enclave.clone(), password)
    }
}

impl Drop for SgxEnclaveManager {
    fn drop(&mut self) {
        // Enclave will be automatically destroyed when SgxEnclave is dropped
    }
}
