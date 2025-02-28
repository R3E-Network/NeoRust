# SGX Support

This tutorial covers using the Intel SGX (Software Guard Extensions) features of the NeoRust SDK for secure blockchain operations.

## Understanding Intel SGX

Intel SGX is a set of security-related instruction codes built into modern Intel CPUs. It allows user-level code to allocate private regions of memory, called enclaves, which are protected from processes running at higher privilege levels. Key benefits include:

- **Hardware-Level Security**: Protection of sensitive data and code from the operating system, hypervisor, BIOS, and other privileged software
- **Secure Computation**: Ability to perform computations on sensitive data within the enclave
- **Remote Attestation**: Verification that code is running in a genuine SGX enclave
- **Sealing**: Secure storage of sensitive data for later use within an enclave

## Prerequisites

Before using the SGX features of NeoRust, you need:

1. **Intel SGX-compatible hardware**
   - CPU with SGX support (check with `cpuid` or Intel's processor list)
   - SGX enabled in BIOS/UEFI

2. **Intel SGX Software Stack**
   - Intel SGX Driver
   - Intel SGX SDK v2.12
   - Intel SGX PSW (Platform Software)

3. **Rust Toolchain**
   - Rust nightly-2022-10-22 (required by the Apache Teaclave SGX SDK)
   - Install with: `rustup install nightly-2022-10-22`
   - Set as default for this project: `rustup override set nightly-2022-10-22`

For detailed installation instructions, see the [SGX Setup Guide](https://github.com/R3E-Network/NeoRust/blob/master/SGX_SETUP.md).

## Enabling SGX Support in NeoRust

To enable SGX support in your project, add the `sgx` feature to your Cargo.toml:

```toml
[dependencies]
neo = { git = "https://github.com/R3E-Network/NeoRust", features = ["sgx"] }
```

You also need to uncomment the SGX dependencies in the NeoRust Cargo.toml:

```toml
# SGX dependencies
sgx_types = { version = "=1.1.1", optional = true }
sgx_urts = { version = "=1.1.1", optional = true }
sgx_tstd = { version = "=1.1.1", optional = true }
sgx_tcrypto = { version = "=1.1.1", optional = true }

[features]
sgx = ["sgx-deps"]
sgx-deps = [
    "sgx_types",
    "sgx_urts",
    "sgx_tstd",
    "sgx_tcrypto"
]
```

## Building the SGX Components

Use the provided Makefile for SGX to build the enclave components:

```bash
make -f Makefile.sgx
```

This will build both the trusted enclave components and the untrusted application components.

## Creating an SGX Enclave Manager

The first step in using SGX features is to create an enclave manager:

```rust
use neo::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Path to the enclave shared object
    let enclave_path = "path/to/enclave.so";
    
    // Initialize the SGX enclave
    let enclave_manager = SgxEnclaveManager::new(enclave_path)?;
    println!("SGX enclave initialized successfully!");
    
    Ok(())
}
```

## Secure Wallet Management with SGX

One of the primary use cases for SGX in blockchain applications is secure wallet management:

```rust
use neo::prelude::*;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Path to the enclave shared object
    let enclave_path = "path/to/enclave.so";
    
    // Initialize the SGX enclave
    let enclave_manager = SgxEnclaveManager::new(enclave_path)?;
    
    // Create a wallet with a password
    let password = "my-secure-password";
    let wallet = enclave_manager.create_wallet(password)?;
    
    // Get the wallet's public key
    let public_key = wallet.get_public_key();
    println!("Wallet public key: {:?}", public_key);
    
    // The private key never leaves the enclave
    
    Ok(())
}
```

## Signing Transactions Securely

With SGX, you can sign transactions without exposing private keys:

```rust
use neo::prelude::*;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Path to the enclave shared object
    let enclave_path = "path/to/enclave.so";
    
    // Initialize the SGX enclave
    let enclave_manager = SgxEnclaveManager::new(enclave_path)?;
    
    // Create a wallet with a password
    let password = "my-secure-password";
    let wallet = enclave_manager.create_wallet(password)?;
    
    // Connect to a Neo N3 TestNet node
    let provider = Provider::new_http("https://testnet1.neo.coz.io:443");
    
    // Create a transaction
    let transaction = TransactionBuilder::new()
        .version(0)
        .nonce(rand::random::<u32>())
        .valid_until_block(provider.get_block_count().await? + 100)
        .script(
            ScriptBuilder::new()
                .contract_call(
                    "d2a4cff31913016155e38e474a2c06d08be276cf".parse::<ScriptHash>()?,
                    "transfer",
                    &[
                        ContractParameter::hash160(wallet.get_address().script_hash()),
                        ContractParameter::hash160("NZNos2WqTbu5oCgyfss9kUJgBXJqhuYAaj".parse::<Address>()?),
                        ContractParameter::integer(1_00000000), // 1 GAS
                        ContractParameter::any(None),
                    ],
                )
                .to_array()
        )
        .build();
    
    // Sign the transaction securely within the enclave
    let signed_tx = wallet.sign_transaction(&transaction)?;
    
    // Send the transaction
    let txid = provider.send_raw_transaction(&signed_tx).await?;
    println!("Transaction sent with ID: {}", txid);
    
    Ok(())
}
```

## Secure RPC Client

The SGX module also provides a secure RPC client for blockchain interactions:

```rust
use neo::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Path to the enclave shared object
    let enclave_path = "path/to/enclave.so";
    
    // Initialize the SGX enclave
    let enclave_manager = SgxEnclaveManager::new(enclave_path)?;
    
    // Create a secure RPC client
    let rpc_url = "https://testnet1.neo.coz.io:443";
    let rpc_client = enclave_manager.create_rpc_client(rpc_url)?;
    
    // Use the secure RPC client
    let block_count = rpc_client.get_block_count().await?;
    println!("Current block height: {}", block_count);
    
    // The RPC client encrypts sensitive data and performs secure validation
    // of responses within the enclave
    
    Ok(())
}
```

## Secure Storage

The SGX module provides secure storage for sensitive data:

```rust
use neo::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Path to the enclave shared object
    let enclave_path = "path/to/enclave.so";
    
    // Initialize the SGX enclave
    let enclave_manager = SgxEnclaveManager::new(enclave_path)?;
    
    // Create a secure storage instance
    let storage = enclave_manager.create_storage()?;
    
    // Store sensitive data
    let key = "api_key";
    let value = "my-secret-api-key";
    storage.set(key, value)?;
    
    // Retrieve sensitive data
    let retrieved_value = storage.get(key)?;
    println!("Retrieved value: {}", retrieved_value);
    
    // The data is encrypted and stored securely
    
    Ok(())
}
```

## Remote Attestation

Remote attestation allows you to verify that your code is running in a genuine SGX enclave:

```rust
use neo::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Path to the enclave shared object
    let enclave_path = "path/to/enclave.so";
    
    // Initialize the SGX enclave
    let enclave_manager = SgxEnclaveManager::new(enclave_path)?;
    
    // Generate a remote attestation quote
    let quote = enclave_manager.generate_attestation_quote()?;
    
    // Send the quote to a remote verifier
    // (implementation depends on your attestation service)
    let attestation_service_url = "https://attestation.example.com";
    let verification_result = verify_quote_with_service(attestation_service_url, &quote).await?;
    
    if verification_result.is_valid {
        println!("Remote attestation successful!");
        // Proceed with secure operations
    } else {
        println!("Remote attestation failed!");
        // Handle the failure
    }
    
    Ok(())
}

async fn verify_quote_with_service(url: &str, quote: &[u8]) -> Result<VerificationResult, Box<dyn std::error::Error>> {
    // Implementation of quote verification with a remote attestation service
    // This is just a placeholder
    Ok(VerificationResult { is_valid: true })
}

struct VerificationResult {
    is_valid: bool,
}
```

## Simulation Mode

If you don't have SGX hardware, you can still develop and test using simulation mode:

```bash
export SGX_MODE=SIM
make -f Makefile.sgx
```

In your code, you can check if you're running in simulation mode:

```rust
use neo::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Path to the enclave shared object
    let enclave_path = "path/to/enclave.so";
    
    // Initialize the SGX enclave
    let enclave_manager = SgxEnclaveManager::new(enclave_path)?;
    
    if enclave_manager.is_simulation_mode() {
        println!("Running in simulation mode. Security guarantees are not provided.");
    } else {
        println!("Running in hardware mode with full SGX protection.");
    }
    
    Ok(())
}
```

## Best Practices

1. **Minimize Enclave Code**: Keep the enclave code small to reduce the attack surface.
2. **Validate Inputs**: Always validate inputs before passing them to the enclave.
3. **Secure Key Management**: Never extract private keys from the enclave.
4. **Use Remote Attestation**: Verify the integrity of the enclave in production environments.
5. **Regular Updates**: Keep the SGX SDK and drivers updated to address security vulnerabilities.
6. **Error Handling**: Implement proper error handling for enclave operations.
7. **Testing**: Test your SGX code in both simulation and hardware modes.

## Security Considerations

When using SGX for blockchain applications:

1. **Side-Channel Attacks**: Be aware that SGX is not immune to all side-channel attacks.
2. **Enclave Interface**: The interface between the untrusted application and the enclave is a potential attack vector.
3. **Data Sealing**: Use data sealing to protect sensitive data at rest.
4. **Memory Limitations**: SGX enclaves have memory limitations; design your application accordingly.
5. **Attestation**: Use remote attestation in production to verify enclave integrity.

<!-- toc -->
