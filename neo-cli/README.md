# Neo Blockchain CLI

A command-line interface for interacting with the Neo blockchain, built on the NeoRust SDK.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

Neo CLI provides a comprehensive set of tools for interacting with the Neo blockchain ecosystem. It allows developers and users to manage wallets, interact with smart contracts, monitor blockchain state, configure network connections, and engage with DeFi applications.

## Features

- **Wallet Management**: Create, open, and manage Neo wallets, including key manipulation and asset transfers
- **Blockchain Operations**: Query blockchain state, blocks, transactions and more
- **Network Configuration**: Connect to different Neo networks and manage node connections
- **Smart Contract Interactions**: Deploy, invoke, and analyze smart contracts
- **DeFi Integrations**: Interact with Neo-based DeFi platforms and protocols

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

## Configuration

Neo CLI uses a configuration file to store settings like network preferences, RPC endpoints, and more. You can initialize the configuration with:

```bash
neo-cli init [--path /custom/path/config.json]
```

The default location for the configuration file is in your system's config directory under `neo-cli/config.json`.

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