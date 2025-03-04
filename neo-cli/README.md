# Neo Blockchain CLI

A command-line interface for interacting with the Neo blockchain, built on the NeoRust SDK.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

Neo CLI provides a comprehensive set of tools for interacting with the Neo blockchain ecosystem. It allows developers and users to manage wallets, interact with smart contracts, monitor blockchain state, configure network connections, engage with DeFi applications and well-known contracts, and leverage NeoFS decentralized storage.

## Features

- **Wallet Management**: Create, open, and manage Neo wallets, including key manipulation and asset transfers
- **Blockchain Operations**: Query blockchain state, blocks, transactions and more
- **Network Configuration**: Connect to different Neo networks and manage node connections
- **Smart Contract Interactions**: Deploy, invoke, and analyze smart contracts
- **DeFi Integrations**: Interact with Neo-based DeFi platforms, protocols, and well-known contracts
- **NeoFS Integration**: Store and retrieve data from Neo's decentralized storage

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/r3-network/neo-rust.git
cd neo-rust

# Build the CLI tool
cargo build --release -p neo-cli

# Run the CLI
./target/release/neo-cli --help
```

### Using Cargo

```bash
cargo install neo-cli
```

## Quick Start

```bash
# Initialize configuration
neo-cli init

# Create a new wallet
neo-cli wallet create --name my-wallet

# Check wallet balance
neo-cli wallet balance --name my-wallet

# Get blockchain height
neo-cli blockchain height

# Deploy a smart contract
neo-cli contract deploy --path ./my-contract.nef --manifest ./my-contract.manifest.json

# List famous contracts on mainnet
neo-cli defi list --network mainnet

# Check NeoFS connection status
neo-cli fs status
```

## Command Reference

### Wallet Commands

- `neo-cli wallet create`: Create a new wallet
- `neo-cli wallet open`: Open an existing wallet
- `neo-cli wallet balance`: Check wallet balance
- `neo-cli wallet transfer`: Transfer assets between accounts
- `neo-cli wallet export`: Export wallet keys and certificates
- `neo-cli wallet import`: Import wallet keys

### Blockchain Commands

- `neo-cli blockchain info`: Display general blockchain information
- `neo-cli blockchain height`: Get the current blockchain height
- `neo-cli blockchain block`: Get block information
- `neo-cli blockchain tx`: Get transaction details
- `neo-cli blockchain asset`: Get asset information

### Network Commands

- `neo-cli network status`: Check network status
- `neo-cli network nodes`: List connected nodes
- `neo-cli network switch`: Switch between different Neo networks

### Contract Commands

- `neo-cli contract deploy`: Deploy a smart contract
- `neo-cli contract invoke`: Invoke a smart contract method
- `neo-cli contract info`: Display contract information
- `neo-cli contract storage`: Query contract storage

### DeFi Commands

- `neo-cli defi swap`: Perform token swaps
- `neo-cli defi stake`: Stake tokens
- `neo-cli defi yield`: Check yield farming positions
- `neo-cli defi pools`: List liquidity pools
- `neo-cli defi list`: List all well-known contracts on a network
- `neo-cli defi show`: Show details of a specific contract
- `neo-cli defi invoke`: Invoke a method on a well-known contract
- `neo-cli defi balance`: Query token balance from a contract

```bash
# List all well-known contracts on mainnet
neo-cli defi list --network mainnet

# Show details for Flamingo Finance contract
neo-cli defi show "Flamingo Finance"

# Invoke a method on a contract
neo-cli defi invoke ghostmarket "balanceOf" --args '[{"type":"Address","value":"NZKvXidwBhnV8rNXh2eXtpm5bH1rkofaDz"}]'

# Check token balance
neo-cli defi balance "FLM Token" --address NZKvXidwBhnV8rNXh2eXtpm5bH1rkofaDz

# Perform a token swap
neo-cli defi swap --token-from NEO --token-to GAS --amount 1
```

### NeoFS Commands

- `neo-cli fs container`: Create, list, get info, and delete containers
- `neo-cli fs object`: Upload, download, delete objects
- `neo-cli fs acl`: Manage access control for containers and objects
- `neo-cli fs endpoints`: Manage and get information about NeoFS endpoints
- `neo-cli fs status`: Show NeoFS connection status

```bash
# List all available NeoFS endpoints for mainnet
neo-cli fs endpoints list --network mainnet

# Test connection to a specific endpoint
neo-cli fs endpoints test --endpoint grpc.mainnet.fs.neo.org:8082

# Get detailed information about an endpoint
neo-cli fs endpoints info --endpoint grpc.mainnet.fs.neo.org:8082

# Create a container
neo-cli fs container create --config container-config.json

# Upload a file
neo-cli fs object put --container CID --file path/to/file

# Download a file
neo-cli fs object get --container CID --id OID --output path/to/save
```

## Configuration

Neo CLI uses a configuration file to store settings like network preferences, RPC endpoints, and more. You can initialize the configuration with:

```bash
neo-cli init [--path /custom/path/config.json]
```

The default location for the configuration file is in your system's config directory under `neo-cli/config.json`.

## Testing

The Neo CLI includes comprehensive automated tests to ensure functionality and help with development.

### Running Tests

To run all tests:

```bash
cargo test
```

To run only specific test categories:

```bash
# Run only DeFi tests (including well-known contracts)
cargo test --test integration_tests integration::defi_tests

# Run only NeoFS tests
cargo test --test integration_tests integration::fs_tests
```

### Test Structure

- **Unit Tests**: Test individual functions and components
- **Integration Tests**: Test the CLI commands from a user perspective
  - `defi_tests.rs`: Tests for DeFi and well-known contract commands
  - `fs_tests.rs`: Tests for NeoFS storage operations
  - `blockchain_tests.rs`: Tests for blockchain query commands
  - `wallet_tests.rs`: Tests for wallet management commands

### Writing New Tests

When adding new features to the CLI, follow this pattern for testing:

1. Create unit tests for new functions in the source files
2. Add integration tests in the appropriate test module
3. Run the tests to verify functionality
4. Ensure both success cases and error handling are tested

## Development

### Building from Source

```bash
cargo build [--release]
```

### Running Tests

```bash
cargo test -p neo-cli
```

## License

MIT License

## Credits

Developed by the R3E Network team

---
Copyright © 2020-2025 R3E Network. All rights reserved.