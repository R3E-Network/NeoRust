# Neo CLI Documentation

## Table of Contents

1. [Introduction](#introduction)
2. [Installation](#installation)
3. [Configuration](#configuration)
4. [Command Reference](#command-reference)
   - [Wallet Commands](#wallet-commands)
   - [Blockchain Commands](#blockchain-commands)
   - [Network Commands](#network-commands)
   - [Contract Commands](#contract-commands)
   - [DeFi Commands](#defi-commands)
5. [Advanced Usage](#advanced-usage)
6. [Troubleshooting](#troubleshooting)
7. [API Integration](#api-integration)

## Introduction

Neo CLI is a comprehensive command-line tool for interacting with the Neo blockchain ecosystem. Built on the NeoRust SDK, it provides a user-friendly interface for performing a wide range of operations on the Neo blockchain.

### Key Features

- Complete wallet management functionality
- Blockchain query and monitoring capabilities
- Smart contract deployment and interaction
- Network configuration and management
- DeFi platform integration
- Secure key handling and transaction signing

## Installation

### Prerequisites

- Rust 1.63 or higher
- Cargo package manager
- (Optional) Git for cloning the repository

### Installation Methods

#### From Source

```bash
# Clone the repository
git clone https://github.com/r3-network/neo-rust.git
cd neo-rust

# Build the CLI tool
cargo build --release -p neo-cli

# Add to your PATH (optional)
cp target/release/neo-cli /usr/local/bin/
```

#### Using Cargo

```bash
cargo install neo-cli
```

## Configuration

Neo CLI requires a configuration file to store settings such as:

- RPC endpoint URLs
- Default network (MainNet, TestNet, or custom)
- Default wallet paths
- Logging preferences

### Initializing Configuration

```bash
neo-cli init [--path /custom/path/config.json]
```

Without specifying a path, the configuration file will be created in your system's default configuration directory.

### Configuration File Structure

The configuration file uses JSON format:

```json
{
  "network": {
    "default": "mainnet",
    "rpc": {
      "mainnet": "http://seed1.neo.org:10332",
      "testnet": "http://seed1.neo.org:20332"
    }
  },
  "wallet": {
    "default_path": "/path/to/wallets"
  },
  "logging": {
    "level": "info"
  }
}
```

You can edit this file manually or use the CLI to update settings.

## Command Reference

### Global Options

These options can be used with any command:

- `--config`: Specify a custom configuration file path
- `--verbose`: Enable verbose output
- `--help`: Display help information
- `--version`: Display version information

### Wallet Commands

#### Create a New Wallet

```bash
neo-cli wallet create [--name <NAME>] [--path <PATH>] [--password <PASSWORD>]
```

Options:
- `--name`: Name of the wallet (default: "wallet")
- `--path`: Custom path to save the wallet file
- `--password`: Password to encrypt the wallet (will prompt if not provided)

#### Open Wallet

```bash
neo-cli wallet open [--name <NAME>] [--path <PATH>]
```

Options:
- `--name`: Name of the wallet
- `--path`: Path to the wallet file

#### Check Wallet Balance

```bash
neo-cli wallet balance [--address <ADDRESS>] [--name <NAME>] [--path <PATH>]
```

Options:
- `--address`: Specific address to check (optional)
- `--name`: Name of the wallet
- `--path`: Path to the wallet file

#### Transfer Assets

```bash
neo-cli wallet transfer --to <ADDRESS> --asset <ASSET> --amount <AMOUNT> [--from <ADDRESS>] [--name <NAME>] [--path <PATH>]
```

Options:
- `--to`: Recipient address
- `--asset`: Asset identifier (NEO, GAS, or script hash)
- `--amount`: Amount to transfer
- `--from`: Sender address (optional, will use default if not specified)
- `--name`: Name of the wallet
- `--path`: Path to the wallet file

#### Export Wallet

```bash
neo-cli wallet export [--format <FORMAT>] [--name <NAME>] [--path <PATH>] [--output <OUTPUT_PATH>]
```

Options:
- `--format`: Export format (NEP6, WIF, etc.)
- `--name`: Name of the wallet
- `--path`: Path to the wallet file
- `--output`: Path to save the exported wallet

#### Import Wallet

```bash
neo-cli wallet import --input <INPUT_PATH> [--format <FORMAT>] [--name <NAME>] [--path <PATH>]
```

Options:
- `--input`: Path to the wallet file to import
- `--format`: Import format (NEP6, WIF, etc.)
- `--name`: Name to save the wallet as
- `--path`: Path to save the imported wallet

### Blockchain Commands

#### Get Blockchain Information

```bash
neo-cli blockchain info
```

Displays general information about the blockchain, including height, version, and consensus nodes.

#### Get Current Block Height

```bash
neo-cli blockchain height
```

Returns the current block height of the connected network.

#### Get Block Information

```bash
neo-cli blockchain block [--hash <HASH>] [--index <INDEX>]
```

Options:
- `--hash`: Block hash
- `--index`: Block index/height

#### Get Transaction Information

```bash
neo-cli blockchain tx --hash <HASH>
```

Options:
- `--hash`: Transaction hash

#### Get Asset Information

```bash
neo-cli blockchain asset --id <ASSET_ID>
```

Options:
- `--id`: Asset identifier (script hash)

### Network Commands

#### Check Network Status

```bash
neo-cli network status
```

Displays the status of the connected network, including node count and recent blocks.

#### List Connected Nodes

```bash
neo-cli network nodes
```

Lists all connected nodes and their information.

#### Switch Network

```bash
neo-cli network switch --network <NETWORK>
```

Options:
- `--network`: Network to switch to (mainnet, testnet, private)

### Contract Commands

#### Deploy Smart Contract

```bash
neo-cli contract deploy --path <NEF_PATH> --manifest <MANIFEST_PATH> [--wallet <WALLET_PATH>] [--account <ACCOUNT>]
```

Options:
- `--path`: Path to the NEF file
- `--manifest`: Path to the manifest file
- `--wallet`: Path to the wallet file
- `--account`: Account to pay for deployment

#### Invoke Contract Method

```bash
neo-cli contract invoke --contract <CONTRACT_HASH> --method <METHOD_NAME> [--params <PARAMS>] [--wallet <WALLET_PATH>] [--account <ACCOUNT>]
```

Options:
- `--contract`: Contract script hash
- `--method`: Name of the method to invoke
- `--params`: Parameters for the method (JSON format)
- `--wallet`: Path to the wallet file
- `--account`: Account to pay for the invocation

#### Get Contract Information

```bash
neo-cli contract info --contract <CONTRACT_HASH>
```

Options:
- `--contract`: Contract script hash

#### Query Contract Storage

```bash
neo-cli contract storage --contract <CONTRACT_HASH> [--key <KEY>] [--prefix <PREFIX>]
```

Options:
- `--contract`: Contract script hash
- `--key`: Specific key to query
- `--prefix`: Key prefix for querying multiple keys

### DeFi Commands

#### Perform Token Swap

```bash
neo-cli defi swap --from <TOKEN> --to <TOKEN> --amount <AMOUNT> [--max-slippage <SLIPPAGE>] [--wallet <WALLET_PATH>]
```

Options:
- `--from`: Source token (NEO, GAS, or script hash)
- `--to`: Target token (NEO, GAS, or script hash)
- `--amount`: Amount to swap
- `--max-slippage`: Maximum acceptable slippage percentage
- `--wallet`: Path to the wallet file

#### Stake Tokens

```bash
neo-cli defi stake --token <TOKEN> --amount <AMOUNT> --platform <PLATFORM> [--duration <DURATION>] [--wallet <WALLET_PATH>]
```

Options:
- `--token`: Token to stake
- `--amount`: Amount to stake
- `--platform`: Staking platform
- `--duration`: Staking duration in days
- `--wallet`: Path to the wallet file

#### Check Yield Farming Positions

```bash
neo-cli defi yield [--wallet <WALLET_PATH>] [--platform <PLATFORM>]
```

Options:
- `--wallet`: Path to the wallet file
- `--platform`: Specific platform to check

#### List Liquidity Pools

```bash
neo-cli defi pools [--platform <PLATFORM>]
```

Options:
- `--platform`: Specific platform to check

## Advanced Usage

### Working with Multi-Signature Wallets

```bash
neo-cli wallet create-multisig --min-signatures <COUNT> --pubkeys <PUBKEY1,PUBKEY2,...> [--name <NAME>]
```

### Batch Operations

For performing operations in batch mode, you can use JSON input files:

```bash
neo-cli batch --input <BATCH_FILE_PATH>
```

### Scripting Support

Neo CLI can be used in shell scripts. All commands return appropriate exit codes and structured output that can be parsed by other tools.

## Troubleshooting

### Common Issues

- **Connection problems**: Verify network settings in the configuration file
- **Transaction failures**: Check account balance and network fees
- **Wallet access issues**: Ensure the wallet password is correct and the file is not corrupted

### Logs

Logs are stored in the system's temporary directory. You can increase verbosity with the `--verbose` flag.

### Getting Help

For additional help, use the `--help` flag with any command or visit the project's GitHub repository.

## API Integration

Neo CLI can be used as a library in other Rust applications:

```rust
use neo_cli::commands::wallet;

async fn example() {
    let args = wallet::WalletArgs::Create {
        name: Some("my-wallet".to_string()),
        path: None,
        password: None,
    };
    
    let mut state = wallet::CliState::default();
    wallet::handle_wallet_command(args, &mut state).await.unwrap();
}
```

For more information on API integration, see the [API documentation](https://docs.rs/neo-cli). 