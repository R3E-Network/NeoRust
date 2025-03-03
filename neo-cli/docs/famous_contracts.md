# Working with Famous Neo N3 Contracts

This guide explains how to use the Neo CLI to interact with well-known contracts on the Neo N3 blockchain.

## Overview

The Neo blockchain ecosystem has several important contracts that provide key functionality. The `neo-cli` tool provides direct access to these contracts through the `famous` command group, allowing you to:

1. List all available famous contracts
2. Get detailed information about specific contracts
3. Invoke methods on these contracts
4. Check token balances

## Available Commands

### Listing Famous Contracts

To see all famous contracts available on a network:

```bash
# List mainnet contracts
neo-cli famous list --network mainnet

# List testnet contracts
neo-cli famous list --network testnet
```

This will display information about each contract, including its name, script hash, and description.

### Viewing Contract Details

To see detailed information about a specific contract:

```bash
# You can use the contract name (case-insensitive)
neo-cli famous show "flamingo finance"

# Or you can use the script hash
neo-cli famous show 0x1a4e5b62b908c758417eb525ecba58752a947f2b
```

This command will show:
- Contract name and description
- Script hash
- Network (mainnet or testnet)
- Contract type
- Methods and parameters (if available)

### Invoking Contract Methods

To call a method on a famous contract:

```bash
# Test invoke (simulate, don't execute)
neo-cli famous invoke "Neo Name Service" "ownerOf" --args '["neo.neo"]'

# Execute the invocation (requires a wallet)
neo-cli famous invoke "FLM Token" "transfer" --args '[
  {"type":"Hash160","value":"0x1a4e5b62b908c758417eb525ecba58752a947f2b"},
  {"type":"Hash160","value":"NZKvXidwBhnV8rNXh2eXtpm5bH1rkofaDz"},
  {"type":"Integer","value":"1000000"},
  {"type":"Any","value":null}
]' --execute
```

The `--args` parameter accepts a JSON array of arguments. Each argument can be a simple JSON value or an object with explicit type information.

### Checking Token Balances

To check the balance of a token contract:

```bash
# Check balance for the currently loaded account
neo-cli famous balance "FLM Token"

# Check balance for a specific address
neo-cli famous balance "GAS Token" --address NZKvXidwBhnV8rNXh2eXtpm5bH1rkofaDz
```

## Available Famous Contracts

### Mainnet

| Contract Name       | Description                         |
|---------------------|-------------------------------------|
| NEO Token           | Native governance token             |
| GAS Token           | Native utility token                |
| Flamingo Finance    | DeFi platform                       |
| FLM Token           | Flamingo governance token           |
| GhostMarket         | NFT marketplace                     |
| NeoBurger DAO       | Governance platform                 |
| NeoCompound         | GAS staking platform                |
| Neo Name Service    | Domain name service                 |
| Poly Network Bridge | Cross-chain bridge                  |

### Testnet

| Contract Name       | Description                         |
|---------------------|-------------------------------------|
| Testnet NNS         | Name service on testnet             |
| Testnet Faucet      | Token distribution                  |

## Working with JSON Arguments

When invoking contract methods, you'll need to provide arguments in JSON format. Here are some examples:

### Simple Values

For simple values, you can use plain JSON:

```bash
--args '["hello", 123, true]'
```

### Typed Values

For more complex Neo types, use the object format with type information:

```bash
--args '[
  {"type":"Hash160","value":"0x1a4e5b62b908c758417eb525ecba58752a947f2b"},
  {"type":"Address","value":"NZKvXidwBhnV8rNXh2eXtpm5bH1rkofaDz"},
  {"type":"Integer","value":1000}
]'
```

Supported types include:
- `Hash160` (script hash)
- `Address` (Neo address)
- `Integer` (number)
- `String` (text)
- `Boolean` (true/false)
- `ByteArray` (hex string starting with "0x")
- `Any` (null value)

## Examples

### Transfer NEO tokens

```bash
neo-cli famous invoke "NEO Token" "transfer" --args '[
  {"type":"Hash160","value":"NZKvXidwBhnV8rNXh2eXtpm5bH1rkofaDz"},
  {"type":"Integer","value":"100000000"},
  {"type":"Any","value":null}
]' --execute
```

### Check if a domain is available on NNS

```bash
neo-cli famous invoke "Neo Name Service" "isAvailable" --args '["example.neo"]'
```

### Get token metadata

```bash
neo-cli famous invoke "FLM Token" "symbol"
neo-cli famous invoke "FLM Token" "decimals"
neo-cli famous invoke "FLM Token" "totalSupply"
```
