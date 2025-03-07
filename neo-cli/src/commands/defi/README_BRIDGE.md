# Neo X Bridge Commands

This module provides functionality for transferring tokens between Neo N3 and Neo X networks using the official bridge contracts.

## Network Compatibility

Bridge operations have specific network requirements:

- **Deposits**: Must be initiated from a Neo N3 network (Mainnet or Testnet)
- **Withdrawals**: Must be initiated from the Neo X network
- **Address formats**: Each operation requires the correct address format for the destination network

All bridge commands include automatic validation to ensure operations are performed on the correct network with valid addresses.

## Available Commands

### Bridge Deposit

Transfer tokens from Neo N3 to Neo X.

```
neo defi bridge deposit --token NEO --amount 1 --destination NeoXAddress
```

Parameters:
- `--token`: Token symbol or contract address to deposit (currently only NEO and GAS are supported)
- `--amount`: Amount to deposit
- `--destination`: Destination address on Neo X

### Bridge Withdraw

Shows instructions for withdrawing tokens from Neo X to Neo N3.
Note: Withdrawals must be initiated from Neo X.

```
neo defi bridge withdraw --token NEO --amount 1 --destination N3Address
```

Parameters:
- `--token`: Token symbol or contract address to withdraw
- `--amount`: Amount to withdraw
- `--destination`: Destination address on Neo N3

### Bridge Fee

Check the current bridge fee for a specific token.

```
neo defi bridge fee --token NEO
```

Parameters:
- `--token`: Token symbol or contract address

### Bridge Cap

Check the current bridge capacity for a specific token.

```
neo defi bridge cap --token NEO
```

Parameters:
- `--token`: Token symbol or contract address

## Network Compatibility

- Bridge commands must be executed while connected to a Neo N3 network.
- The SDK will automatically determine whether to use mainnet or testnet bridge contracts.
- Token addresses are automatically resolved to their correct values on each network.

## Supported Tokens

- NEO: Native governance token
- GAS: Native utility token
- NEOX: Neo X native token (equivalent to GAS on Neo X)

## Transaction Signing

All transactions require a wallet with sufficient funds to cover gas costs. Use the `--wallet` and `--password` parameters to specify your wallet details.

## Example Usage

```
# Connect to Neo N3 TestNet
neo network connect testnet

# Check your balance
neo defi balance NEO

# Deposit 1 NEO to Neo X
neo defi bridge deposit --token NEO --amount 1 --destination NXNeoXTestAddress

# Check the fee for bridging GAS
neo defi bridge fee --token GAS
```

## Network Utilities

### Network Status Command

To get a comprehensive view of your network connectivity and token compatibility across Neo N3 and Neo X:

```
neo defi network-status
```

This command provides detailed information about:
- Current network connection status
- Token support on your current network
- Cross-network token compatibility
- Bridge contract status and availability
- Automatic Neo X chain connectivity testing

### Address Conversion

Convert between Neo N3 and Neo X address formats with a single command:

```
neo defi address-convert --address <ADDRESS>
```

This utility:
- Automatically detects input address format (Neo N3 or Neo X)
- Displays the equivalent address in the other network format
- Validates address correctness for each network
- Shows script hash equivalents

Utility for cross-network operations when you need to prepare destination addresses for the bridge.
