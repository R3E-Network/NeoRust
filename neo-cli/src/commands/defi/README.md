# Neo CLI DeFi Module

This module provides commands for interacting with various DeFi protocols on the Neo N3 blockchain. **Note that this module is currently in early development with many placeholder implementations.**

## Overview

The DeFi module is being developed to support:

1. **Token Operations**
   - View token information (`token`)
   - Check token balances (`balance`)
   - Transfer tokens (`transfer`)

2. **Flamingo Finance** (Placeholder implementations)
   - Token swapping
   - Liquidity provision
   - Staking
   - Rewards claiming

3. **NeoBurger** (Placeholder implementations)
   - Wrap NEO to bNEO
   - Unwrap bNEO to NEO
   - Claim GAS rewards
   - Check exchange rates

4. **NeoCompound** (Placeholder implementations)
   - Yield farming operations

5. **GrandShare** (Placeholder implementations)
   - Funding platform operations

## Current Status

This module is in **early development**. Most contract interactions are currently implemented as placeholders that demonstrate the intended functionality but do not yet execute actual blockchain transactions.

## Usage

All DeFi commands use the following format:

```
neo-cli defi [OPTIONS] <SUBCOMMAND>
```

### Options

- `-w, --wallet <WALLET>` - Path to wallet file
- `-p, --password <PASSWORD>` - Wallet password

### Available Commands

Currently available commands (some with limited functionality):

```
neo-cli defi token <CONTRACT>
neo-cli defi balance <CONTRACT> [--address <ADDRESS>]
neo-cli defi transfer <TOKEN> <TO> <AMOUNT> [--data <DATA>]
```

## Development Roadmap

1. Complete token operations implementation
2. Implement Flamingo Finance integration
3. Add NeoBurger support
4. Integrate NeoCompound functionality
5. Develop GrandShare operations

## Technical Requirements

- Connection to a Neo N3 RPC node
- Properly configured wallet
- Neo SDK v0.1.8 or later with the following features:
  - `futures` - Required for async operations
  - `ledger` - Optional for hardware wallet support
  - `aws` - Optional for AWS integration

## Feature Support

The DeFi module works with the following Neo SDK features:
- `futures` - Required for all DeFi operations (enables async/await support)
- `ledger` - Optional, enables hardware wallet support for signing transactions
- `aws` - Optional, enables AWS integration for cloud deployments

## Contributing

This module welcomes contributions. Areas particularly needing attention:
- Contract interaction implementations
- Unit tests
- Documentation improvements
- Error handling

## Important Notes

1. All contract interactions require a wallet with sufficient funds
2. Commands that modify state require a password
3. Token amounts use the token's native decimal places
4. The module offers simulation capabilities for testing without execution
5. All interactions are subject to network fees

## Contract Addresses

The module uses the following contract addresses for supported networks:

### MainNet

- **NEO**: ef4073a0f2b305a38ec4050e4d3d28bc40ea63f5
- **GAS**: d2a4cff31913016155e38e474a2c06d08be276cf
- **FLM**: 4d9eab13620fe3569ba3b0e56e2877739e4145e3
- **bNEO**: 48c40d4666f93408be1bef038b6722404f5c4a5a

### TestNet

- **NEO**: a0a698c4e1a5884a5d1221d144d8ff878f530572
- **GAS**: 6f8c4f86cf2bfd83171c43027f2af6b97fd879c9
- **FLM**: f7848bdb22bb4e1580642521295e2a75a4107746
- **bNEO**: cc246a0e8126fa7fb93e4d9ff18c61660458ef89

## Error Handling

The module provides detailed error messages for common issues including:

- Invalid addresses or script hashes
- Insufficient funds
- Network connection problems
- Contract execution failures

## Development Status

This module is currently in development and some features may be limited to placeholders. Contributions are welcome. 