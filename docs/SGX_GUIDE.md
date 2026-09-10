# SGX Support Guide - NeoRust (experimental)
> Note: This guide documents an experimental SGX/no_std surface. Host builds with `sgx` compile, but enclave builds require an SGX target/toolchain and deployment-specific validation.

## Overview

NeoRust exposes compile gates and scaffolding for Intel SGX (Software Guard Extensions). Treat this as an integration starting point, not certified enclave support. The general SDK still depends on `std` for normal host targets, and `no_std` is not certified for arbitrary embedded or WASM targets.

> Security note: The SGX module is still evolving. Quote parsing and certificate-chain checks do
> not establish a complete attestation verdict. Production deployments must integrate Intel DCAP
> QVL and validate current PCK CRLs, TCB Info, QE Identity, and the applicable collateral before
> trusting a quote.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Features](#features)
3. [Setup](#setup)
4. [Architecture](#architecture)
5. [Usage](#usage)
6. [Security Boundary](#security-boundary)
7. [Performance Considerations](#performance-considerations)
8. [Troubleshooting](#troubleshooting)

## Prerequisites

### Hardware Requirements
- Intel CPU with SGX support (6th generation Core or newer)
- SGX enabled in BIOS
- At least 128MB EPC (Enclave Page Cache) memory

### Software Requirements
- Ubuntu 18.04/20.04/22.04 or Windows 10/11
- Intel SGX SDK 2.15+
- Intel SGX PSW (Platform Software)
- Rust 1.95+ for the SDK host build

### Installation

#### Ubuntu
```bash
# Add Intel SGX repository
echo 'deb [arch=amd64] https://download.01.org/intel-sgx/sgx_repo/ubuntu focal main' | sudo tee /etc/apt/sources.list.d/intel-sgx.list
wget -qO - https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key | sudo apt-key add -

# Install SGX packages
sudo apt-get update
sudo apt-get install libsgx-epid libsgx-quote-ex libsgx-dcap-ql libsgx-dcap-ql-dev

# Install SGX SDK
wget https://download.01.org/intel-sgx/latest/linux-latest/distro/ubuntu20.04-server/sgx_linux_x64_sdk_2.19.100.3.bin
chmod +x sgx_linux_x64_sdk_2.19.100.3.bin
./sgx_linux_x64_sdk_2.19.100.3.bin
```

## Features

### Experimental API Surface

- Host-compilable quote parsing and certificate-path validation primitives
- Enclave crypto, storage, networking, and integration scaffolding exported only for an SGX target
- SGX quote v3 parsing and measurement extraction helpers
- PCK certificate-path and caller-supplied CRL validation primitives
- Feature gates for deployment-specific SGX toolchains

These APIs do not prove that keys were generated or retained in hardware, that stored data is
rollback protected, or that an enclave is trustworthy. Those properties depend on the enclave
implementation, platform configuration, fresh collateral, and a complete Intel DCAP QVL verdict.

## Setup

### 1. Add SGX Dependencies

```toml
# Cargo.toml
[dependencies]
neo3 = { version = "3.3.1", features = ["sgx", "no_std"] }

[target.'cfg(target_env = "sgx")'.dependencies]
sgx_tstd = { git = "https://github.com/apache/teaclave-sgx-sdk.git", rev = "v2.0.0" }
sgx_types = { git = "https://github.com/apache/teaclave-sgx-sdk.git", rev = "v2.0.0" }
```

### 2. Configure Enclave

Create `Enclave.config.xml`:

```xml
<EnclaveConfiguration>
    <ProdID>0</ProdID>
    <ISVSVN>0</ISVSVN>
    <StackMaxSize>0x400000</StackMaxSize>
    <HeapMaxSize>0x10000000</HeapMaxSize>
    <TCSNum>10</TCSNum>
    <TCSPolicy>1</TCSPolicy>
    <DisableDebug>0</DisableDebug>
    <MiscSelect>0</MiscSelect>
    <MiscMask>0xFFFFFFFF</MiscMask>
</EnclaveConfiguration>
```

### 3. Create Enclave EDL

Create `Enclave.edl`:

```c
enclave {
    from "sgx_tstd.edl" import *;
    
    trusted {
        public sgx_status_t neo_sign_transaction(
            [in, size=data_len] const uint8_t* data,
            size_t data_len,
            [out, size=64] uint8_t* signature
        );
    };
    
    untrusted {
        sgx_status_t ocall_neo_rpc_request(
            [in, size=request_len] const uint8_t* request,
            size_t request_len,
            [out, size=response_len] uint8_t* response,
            size_t response_len,
            [out] size_t* actual_response_len
        );
    };
};
```

## Architecture

### Component Overview

The following is a target integration architecture, not a statement that this SDK has established
the depicted trust boundary. The application must build, deploy, and validate the enclave and its
complete attestation flow.

```
┌─────────────────────────────────────┐
│         Untrusted World             │
│  ┌─────────────────────────────┐    │
│  │    Neo Application          │    │
│  │  - Transaction Builder       │    │
│  │  - RPC Client               │    │
│  └──────────┬──────────────────┘    │
│             │ ECALL/OCALL            │
├─────────────┼───────────────────────┤
│             │                        │
│  ┌──────────▼──────────────────┐    │
│  │     SGX Enclave             │    │
│  │  ┌───────────────────────┐  │    │
│  │  │   NeoRust SGX Module  │  │    │
│  │  │  - Crypto Operations  │  │    │
│  │  │  - Key Management     │  │    │
│  │  │  - Secure Storage     │  │    │
│  │  └───────────────────────┘  │    │
│  └─────────────────────────────┘    │
│         Trusted World               │
└─────────────────────────────────────┘
```

### Module Structure

The `neo_sgx` tree below is enclave-target scaffolding compiled only when both the `sgx` feature and
`target_env = "sgx"` are active.

```rust
neo3/
├── neo_sgx/
│   ├── mod.rs         // Main SGX module
│   ├── allocator.rs   // no_std memory allocator
│   ├── crypto.rs      // SGX crypto operations
│   ├── enclave.rs     // Enclave management
│   ├── networking.rs  // Secure networking
│   ├── storage.rs     // Sealed storage
│   └── attestation.rs // Remote attestation
```

## Usage

The snippets in this section are enclave-target integration sketches. They are not host-runnable
examples or a complete attestation protocol.

### Basic Example

```rust
#![no_std]
#![feature(rustc_private)]

extern crate sgx_tstd as std;
extern crate neo3;

use neo3::neo_sgx::prelude::*;
use neo3::neo_sgx::crypto::SgxCrypto;
use sgx_types::*;

#[no_mangle]
pub extern "C" fn sign_neo_transaction(
    data: *const u8,
    data_len: usize,
    signature: *mut u8,
) -> sgx_status_t {
    // Initialize crypto
    let crypto = match SgxCrypto::new() {
        Ok(c) => c,
        Err(_) => return sgx_status_t::SGX_ERROR_UNEXPECTED,
    };
    
    // Get transaction data
    let tx_data = unsafe {
        std::slice::from_raw_parts(data, data_len)
    };
    
    // Sign with sealed private key
    let private_key = retrieve_sealed_key();
    match crypto.sign_ecdsa(tx_data, &private_key) {
        Ok(sig) => {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    sig.as_ptr(),
                    signature,
                    64
                );
            }
            sgx_status_t::SGX_SUCCESS
        }
        Err(_) => sgx_status_t::SGX_ERROR_UNEXPECTED,
    }
}
```

### Key Management

```rust
use neo3::neo_sgx::crypto::SgxKeyManager;
use neo3::neo_sgx::storage::SecureStorage;

fn manage_keys() -> Result<(), SgxError> {
    let mut key_manager = SgxKeyManager::new();
    let mut storage = SecureStorage::new();
    
    // Generate new key
    let crypto = SgxCrypto::new()?;
    let private_key = crypto.random_bytes(32)?;
    
    // Seal and store key
    let key_id = [0u8; 32]; // Unique key identifier
    storage.store(&key_id, &private_key)?;
    
    // Retrieve and unseal key
    let unsealed_key = storage.retrieve(&key_id)?;
    
    Ok(())
}
```

### Remote Attestation

```rust
use neo3::neo_sgx::attestation::RemoteAttestation;

fn perform_attestation() -> Result<Vec<u8>, SgxError> {
    let mut attestation = RemoteAttestation::new();
    
    // Initialize with service provider's public key
    let sp_pub_key = [0u8; 64]; // SP's public key
    attestation.init_attestation(&sp_pub_key)?;

    // Legacy EPID flows require a SPID for quote generation.
    // (DCAP/ECDSA-based attestation uses a different flow.)
    attestation.configure_spid([0u8; 16]);
    
    // Generate quote with user data
    let user_data = b"NEO_WALLET_V1";
    let quote = attestation.generate_quote(user_data)?;
    
    // Quote can be sent to service provider for verification.
    // Note: QuoteVerifier::verify_quote is not implemented in the SDK yet.
    Ok(quote)
}
```

### Secure Networking

```rust
use neo3::neo_sgx::networking::{SgxNetworking, SecureChannel};

async fn secure_rpc_call() -> Result<Vec<u8>, SgxError> {
    let mut networking = SgxNetworking::new();
    
    // Establish secure channel
    let remote_id = [0u8; 16];
    let mut channel = networking.establish_channel(&remote_id)?;
    
    // Complete handshake
    let remote_pubkey = [0u8; 64]; // Remote's public key
    channel.complete_handshake(&remote_pubkey)?;
    
    // Send encrypted RPC request
    let request = b"{'method': 'getblockcount'}";
    let encrypted = networking.send_secure(&mut channel, request)?;
    
    // Receive encrypted response
    let response = networking.receive_secure(&mut channel, &encrypted)?;
    
    Ok(response)
}
```

## Security Boundary

- The SDK does not establish memory isolation, hardware key provenance, non-extractability, or
  sealed-storage rollback protection by itself.
- The storage and networking modules are integration scaffolding and have not been certified as
  side-channel, replay, or rollback resistant.
- Quote parsing and PCK path validation are local primitives, not an attestation verdict. A complete
  verifier must validate the quote/QE binding, QE Identity, TCB Info, current revocation collateral,
  and a caller challenge bound to REPORTDATA.
- Enclave behavior must be tested with the exact SGX runtime, signing policy, and deployment target.

## Performance Considerations

No SGX performance baseline is published for this SDK. Measure enclave transitions, EPC pressure,
cryptographic operations, and sealing on the actual deployment hardware and toolchain.

### Optimization Tips
1. **Batch Operations**: Minimize ECALL/OCALL transitions
2. **Cache Data**: Keep frequently used data in enclave
3. **Async Operations**: Use async for I/O operations
4. **Memory Management**: Implement efficient allocators

## Troubleshooting

### Common Issues

#### SGX Not Available
```
Error: SGX is not supported on this platform
```
**Solution**: Check CPU support and BIOS settings

#### Enclave Creation Failed
```
Error: Failed to create enclave: SGX_ERROR_NO_DEVICE
```
**Solution**: Install SGX driver and PSW

#### Out of EPC Memory
```
Error: SGX_ERROR_OUT_OF_EPC
```
**Solution**: Reduce heap size or optimize memory usage

#### Attestation Failed
```
Error: Quote verification failed
```
**Solution**: Integrate Intel DCAP QVL, supply current collateral, and inspect the verifier's
supplemental result. The SDK's parser and certificate-chain checks are not a complete attestation
verdict.

### Debug Mode

Enable debug mode for development:

```rust
let config = EnclaveConfig {
    debug: true,
    ..Default::default()
};
```

**Warning**: Disable debug mode in production!

## Best Practices

1. **Minimize Enclave Size**: Keep enclave code minimal
2. **Validate Inputs**: Always validate untrusted inputs
3. **Handle Errors**: Proper error handling for all operations
4. **Secure Channels**: Use verified attestation (external verifier) before sensitive operations
5. **Regular Updates**: Keep SGX SDK and PSW updated
6. **Production Config**: Disable debug, enable ASLR
7. **Monitoring**: Log enclave events for audit

## Migration Guide

### From Standard to SGX

1. Add SGX features to Cargo.toml
2. Replace std with sgx_tstd
3. Implement ECALL interfaces
4. Move sensitive operations to enclave
5. Add attestation flow
6. Test with SGX simulator
7. Deploy with production settings

## Resources

- [Intel SGX Developer Guide](https://software.intel.com/content/www/us/en/develop/topics/software-guard-extensions.html)
- [Teaclave SGX SDK](https://github.com/apache/teaclave-sgx-sdk)
- [SGX Developer Reference](https://download.01.org/intel-sgx/latest/linux-latest/docs/)
- [NeoRust SGX Examples](https://github.com/r3e-network/neo-rust-sdk/tree/master/examples/sgx_enclave)

## Summary

NeoRust exposes experimental SGX integration scaffolding plus quote-parsing and PCK path-validation
primitives. It does not ship a complete enclave implementation or Intel DCAP verifier. Enclave and
attestation behavior must be validated with the target SGX toolchain, a complete Intel DCAP QVL,
fresh deployment collateral, and an application-specific challenge policy.
