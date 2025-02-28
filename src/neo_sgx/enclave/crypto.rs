use sgx_tcrypto::*;
use sgx_types::*;

/// Generates a new keypair for use with Neo blockchain
///
/// # Returns
///
/// A tuple containing the private key (32 bytes) and public key (64 bytes)
pub fn generate_keypair() -> Result<([u8; 32], [u8; 64]), sgx_status_t> {
	// Create a random private key
	let mut private_key = [0u8; 32];
	let result = sgx_read_rand(&mut private_key);
	if result != sgx_status_t::SGX_SUCCESS {
		return Err(result);
	}

	// Derive public key from private key using ECC
	let ecc_handle = SgxEccHandle::new();
	let public_key = match ecc_handle.create_key_pair() {
		Ok((priv_k, pub_k)) => {
			// Copy the generated private key to our buffer
			private_key.copy_from_slice(&priv_k[..32]);
			pub_k
		},
		Err(err) => return Err(err),
	};

	// Convert the public key to the expected format
	let mut public_key_bytes = [0u8; 64];
	public_key_bytes.copy_from_slice(&public_key[..64]);

	Ok((private_key, public_key_bytes))
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
pub fn sign_message(private_key: &[u8; 32], message: &[u8]) -> Result<[u8; 65], sgx_status_t> {
	// Hash the message first (SHA-256)
	let mut message_hash = [0u8; 32];
	let result = sgx_sha256_msg(message, message.len() as u32, &mut message_hash);
	if result != sgx_status_t::SGX_SUCCESS {
		return Err(result);
	}

	// Create ECC handle
	let ecc_handle = SgxEccHandle::new();

	// Sign the message hash
	let signature = match ecc_handle.ecdsa_sign_slice(&message_hash, private_key) {
		Ok(sig) => sig,
		Err(err) => return Err(err),
	};

	// Convert to the expected format (r, s, v)
	let mut signature_bytes = [0u8; 65];
	signature_bytes[0..32].copy_from_slice(&signature[0..32]); // r
	signature_bytes[32..64].copy_from_slice(&signature[32..64]); // s
	signature_bytes[64] = 0; // v (recovery id, set to 0 for simplicity)

	Ok(signature_bytes)
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
	public_key: &[u8; 64],
	message: &[u8],
	signature: &[u8; 65],
) -> Result<bool, sgx_status_t> {
	// Hash the message first (SHA-256)
	let mut message_hash = [0u8; 32];
	let result = sgx_sha256_msg(message, message.len() as u32, &mut message_hash);
	if result != sgx_status_t::SGX_SUCCESS {
		return Err(result);
	}

	// Create ECC handle
	let ecc_handle = SgxEccHandle::new();

	// Extract r and s from the signature
	let mut sig = [0u8; 64];
	sig[0..32].copy_from_slice(&signature[0..32]); // r
	sig[32..64].copy_from_slice(&signature[32..64]); // s

	// Verify the signature
	match ecc_handle.ecdsa_verify_slice(&message_hash, public_key, &sig) {
		Ok(valid) => Ok(valid),
		Err(err) => Err(err),
	}
}
