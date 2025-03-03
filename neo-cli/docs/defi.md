# DeFi Commands in Neo CLI

This document describes how to use the Neo CLI to interact with decentralized finance (DeFi) platforms and well-known contracts on the Neo N3 blockchain.

## Overview

The `defi` command group in Neo CLI provides a comprehensive interface for interacting with:

1. DeFi platforms like Flamingo, NeoSwap, and NeoCompound
2. Well-known ("famous") contracts on the Neo N3 blockchain (e.g., NEO, GAS, FLM tokens)

## Available Commands

### DeFi Platform Commands

#### 1. List Liquidity Pools

```
neo-cli defi pools --platform <platform_name>
```

Lists available liquidity pools on the specified DeFi platform.

**Options:**
- `--platform` - The name of the DeFi platform (e.g., flamingo)

**Example:**
```
neo-cli defi pools --platform flamingo
```

#### 2. Get Swap Information

```
neo-cli defi swap-info --token-from <token> --token-to <token> --amount <amount>
```

Retrieves information about a potential swap between two tokens.

**Options:**
- `--token-from` - The source token (e.g., NEO)
- `--token-to` - The destination token (e.g., GAS)
- `--amount` - The amount of source tokens to swap

**Example:**
```
neo-cli defi swap-info --token-from NEO --token-to GAS --amount 1
```

#### 3. Execute a Swap

```
neo-cli defi swap --token-from <token> --token-to <token> --amount <amount> [--slippage <percentage>]
```

Executes a token swap on the appropriate DeFi platform.

**Options:**
- `--token-from` - The source token
- `--token-to` - The destination token
- `--amount` - The amount of source tokens to swap
- `--slippage` - Maximum allowed slippage percentage (default: 0.5)
- `--wallet` - Path to the wallet file
- `--password` - Wallet password

**Example:**
```
neo-cli defi swap --token-from NEO --token-to GAS --amount 1 --slippage 1.0 --wallet my-wallet.json --password mypassword
```

#### 4. Add Liquidity

```
neo-cli defi add-liquidity --token-a <token> --token-b <token> --amount-a <amount> --amount-b <amount>
```

Adds liquidity to a pool on a DeFi platform.

**Options:**
- `--token-a` - The first token
- `--token-b` - The second token
- `--amount-a` - The amount of first token to add
- `--amount-b` - The amount of second token to add
- `--wallet` - Path to the wallet file
- `--password` - Wallet password

**Example:**
```
neo-cli defi add-liquidity --token-a NEO --token-b GAS --amount-a 1 --amount-b 5 --wallet my-wallet.json --password mypassword
```

#### 5. Remove Liquidity

```
neo-cli defi remove-liquidity --token-a <token> --token-b <token> --percent <percentage>
```

Removes liquidity from a pool on a DeFi platform.

**Options:**
- `--token-a` - The first token in the pool
- `--token-b` - The second token in the pool
- `--percent` - The percentage of liquidity to remove (1-100)
- `--wallet` - Path to the wallet file
- `--password` - Wallet password

**Example:**
```
neo-cli defi remove-liquidity --token-a NEO --token-b GAS --percent 50 --wallet my-wallet.json --password mypassword
```

### Famous Contract Commands

#### 1. List Famous Contracts

```
neo-cli defi list [--network <network>]
```

Lists all well-known contracts on the specified network.

**Options:**
- `--network` - The network to use (mainnet or testnet, default: mainnet)

**Example:**
```
neo-cli defi list --network testnet
```

#### 2. Show Contract Details

```
neo-cli defi show <contract>
```

Shows detailed information about a specific contract.

**Arguments:**
- `<contract>` - The contract name or script hash (e.g., "NEO Token" or "0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5")

**Example:**
```
neo-cli defi show "NEO Token"
```

#### 3. Invoke Contract Method

```
neo-cli defi invoke <contract> <method> [<params>...] [--test] [--rpc <rpc_url>]
```

Invokes a method on a contract.

**Arguments:**
- `<contract>` - The contract name or script hash
- `<method>` - The method name to invoke
- `<params>` - Optional parameters for the method (in JSON format)

**Options:**
- `--test` - Perform a test invocation (no blockchain changes)
- `--rpc` - Custom RPC URL to use
- `--wallet` - Path to the wallet file
- `--password` - Wallet password

**Examples:**
```
# Test invoke the symbol method of the NEO token contract
neo-cli defi invoke "NEO Token" symbol --test

# Invoke the transfer method with parameters
neo-cli defi invoke "NEO Token" transfer '[
  "NZKvXidwBhnV8rNXh2eXtpm5bH1rkofaDz",
  "NgaiKFjurmNmiRzDRQGs44yzByXuSkdGPF",
  10,
  null
]' --wallet my-wallet.json --password mypassword
```

#### 4. Check Token Balance

```
neo-cli defi balance <contract> [--address <address>] [--rpc <rpc_url>]
```

Checks the token balance for an address.

**Arguments:**
- `<contract>` - The contract name or script hash

**Options:**
- `--address` - The address to check (if not provided, will use the default address from the wallet)
- `--rpc` - Custom RPC URL to use
- `--wallet` - Path to the wallet file
- `--password` - Wallet password

**Example:**
```
neo-cli defi balance "NEO Token" --address NZKvXidwBhnV8rNXh2eXtpm5bH1rkofaDz
```

## Examples

### A Complete DeFi Workflow

Here's a complete workflow using the DeFi commands:

1. List available pools on Flamingo:
   ```
   neo-cli defi pools --platform flamingo
   ```

2. Get information about a potential swap:
   ```
   neo-cli defi swap-info --token-from NEO --token-to GAS --amount 1
   ```

3. Execute the swap:
   ```
   neo-cli defi swap --token-from NEO --token-to GAS --amount 1 --wallet my-wallet.json --password mypassword
   ```

4. Check your token balance after the swap:
   ```
   neo-cli defi balance "GAS Token" --wallet my-wallet.json --password mypassword
   ```

### Interacting with a Famous Contract

Here's an example of interacting with the NEO token contract:

1. Get detailed information about the NEO token contract:
   ```
   neo-cli defi show "NEO Token"
   ```

2. Check the token symbol (test invocation):
   ```
   neo-cli defi invoke "NEO Token" symbol --test
   ```

3. Check your NEO balance:
   ```
   neo-cli defi balance "NEO Token" --wallet my-wallet.json --password mypassword
   ```

4. Transfer some NEO to another address:
   ```
   neo-cli defi invoke "NEO Token" transfer '[
     "Your_Address",
     "Destination_Address",
     10,
     null
   ]' --wallet my-wallet.json --password mypassword
   ```

## Advanced Usage

### Working with Custom RPC Endpoints

You can specify a custom RPC endpoint for any command:

```
neo-cli defi show "NEO Token" --rpc http://custom-neo-node.example.com:10332
```

### Interacting with a Contract by Script Hash

If you know the script hash of a contract that's not in the famous contracts list, you can still use it:

```
neo-cli defi invoke 0xef4073a0f2b305a38ec4050e4d3d28bc40ea63f5 symbol --test
```

## Troubleshooting

### Common Issues:

1. **Contract not found** - Make sure you're using the correct contract name or script hash, and that you're on the right network (mainnet/testnet).

2. **Method not found** - Verify that the method name is correct and exists on the contract.

3. **Parameter conversion error** - Ensure your JSON parameters are correctly formatted.

4. **Insufficient funds** - Make sure your wallet has enough funds for the operation, including GAS for the transaction fee.

5. **RPC Connection failed** - Verify your network connection and that the RPC URL is correct.

## Conclusion

The DeFi commands in Neo CLI provide a powerful interface for interacting with decentralized finance platforms and well-known contracts on the Neo N3 blockchain. Whether you're swapping tokens, providing liquidity, or simply checking balances, the CLI offers a comprehensive set of tools for your needs.
