# Intel SGX Support for NeoRust

This document provides instructions for setting up and using the Intel SGX (Software Guard Extensions) features of the NeoRust SDK.

## Overview

NeoRust provides optional support for Intel SGX, allowing you to run sensitive blockchain operations within secure enclaves. This is particularly useful for applications that require enhanced security for private key management and transaction signing.

## Prerequisites

To use the SGX features of NeoRust, you need:

1. **Intel SGX-compatible hardware**
   - CPU with SGX support (check with `cpuid` or Intel's processor list)
   - SGX enabled in BIOS/UEFI

2. **Intel SGX Software Stack**
   - Intel SGX Driver (version 2.11.0 or DCAP 1.36.2 Driver)
   - Intel SGX SDK v2.12
   - Intel SGX PSW (Platform Software)

3. **Rust Toolchain**
   - Rust nightly-2022-10-22 (required by the Apache Teaclave SGX SDK)
   - Install with: `rustup install nightly-2022-10-22`
   - Set as default for this project: `rustup override set nightly-2022-10-22`

## Installation

### 1. Install Intel SGX Driver and SDK

Follow the [official Intel SGX installation guide](https://download.01.org/intel-sgx/sgx-linux/2.12/docs/) for your platform.

For Ubuntu, you can use:

```bash
# Add Intel SGX repository
echo 'deb [arch=amd64] https://download.01.org/intel-sgx/sgx_repo/ubuntu focal main' | sudo tee /etc/apt/sources.list.d/intel-sgx.list
wget -qO - https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key | sudo apt-key add -
sudo apt update

# Install SGX packages
sudo apt install libsgx-launch libsgx-urts libsgx-epid libsgx-quote-ex libsgx-dcap-ql
sudo apt install libsgx-enclave-common-dev libsgx-dcap-ql-dev libsgx-dcap-default-qpl-dev

# Install SGX SDK
wget https://download.01.org/intel-sgx/sgx-linux/2.12/distro/ubuntu20.04-server/sgx_linux_x64_sdk_2.12.100.3.bin
chmod +x sgx_linux_x64_sdk_2.12.100.3.bin
sudo ./sgx_linux_x64_sdk_2.12.100.3.bin --prefix=/opt/intel
```

### 2. Set Environment Variables

Add the following to your `.bashrc` or `.profile`:

```bash
source /opt/intel/sgxsdk/environment
export SGX_SDK=/opt/intel/sgxsdk
export SGX_MODE=HW  # Use SIM for simulation mode without SGX hardware
export SGX_ARCH=x64
```

### 3. Enable SGX Features in NeoRust

Uncomment the SGX dependencies in `Cargo.toml`:

```toml
# SGX dependencies
sgx_types = { version = "=1.1.1", optional = true }
sgx_urts = { version = "=1.1.1", optional = true }
sgx_tstd = { version = "=1.1.1", optional = true }
sgx_tcrypto = { version = "=1.1.1", optional = true }
```

## Building with SGX Support

### 1. Build the SGX Components

Use the provided Makefile for SGX:

```bash
make -f Makefile.sgx
```

This will build both the trusted enclave components and the untrusted application components.

### 2. Build Your Application with SGX Features

When building your application, enable the SGX feature:

```bash
cargo build --features sgx
```

## Using SGX Features

The NeoRust SDK provides several SGX-enabled components:

1. **SGX Wallet**: Secure wallet implementation that keeps private keys inside the enclave
2. **SGX RPC Client**: Secure RPC client for blockchain interactions
3. **SGX Crypto**: Cryptographic operations within the enclave

Example usage:

```rust
use neo::prelude::*;

// Initialize SGX enclave
let enclave_path = "path/to/enclave.so";
let enclave_manager = SgxEnclaveManager::new(enclave_path)?;

// Create a secure wallet
let password = "my-secure-password";
let wallet = enclave_manager.create_wallet(password)?;

// Sign a transaction securely within the enclave
let transaction_data = b"Sample transaction data";
let signature = wallet.sign_transaction(transaction_data)?;
```

See the examples in `examples/sgx/` for more detailed usage.

## Simulation Mode

If you don't have SGX hardware, you can still develop and test using simulation mode:

```bash
export SGX_MODE=SIM
make -f Makefile.sgx
```

## Troubleshooting

### Common Issues

1. **Compilation errors with SGX dependencies**:
   - Ensure you're using the correct Rust nightly version (nightly-2022-10-22)
   - Check that Intel SGX SDK is properly installed and environment variables are set

2. **Enclave loading failures**:
   - Verify SGX is enabled in BIOS
   - Check SGX driver installation with `ls /dev/sgx*`
   - Try simulation mode if hardware is not available

3. **Permission issues**:
   - Ensure your user has access to SGX devices: `sudo usermod -a -G sgx <username>`

### Getting Help

If you encounter issues with the SGX integration, please:

1. Check the [Intel SGX documentation](https://download.01.org/intel-sgx/sgx-linux/2.12/docs/)
2. Refer to the [Apache Teaclave SGX SDK](https://github.com/apache/incubator-teaclave-sgx-sdk) documentation
3. Open an issue in the NeoRust repository

## Security Considerations

When using SGX for blockchain applications:

1. The enclave provides protection for data in use, but data must be protected in transit and at rest
2. Remote attestation should be used to verify enclave integrity in production environments
3. Side-channel attacks remain a concern even with SGX; follow Intel's security guidelines

## License

The SGX components of NeoRust are licensed under the same terms as the rest of the project.
