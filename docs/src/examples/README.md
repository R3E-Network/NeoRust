# NeoRust Examples

This section contains various examples demonstrating how to use the NeoRust SDK for different Neo blockchain operations.

## Basic Examples

- [Connecting to a Neo Node](basic-connection.md): How to establish connections to Neo N3 nodes
- [Account Creation](account-creation.md): Creating and managing Neo accounts

## Wallet Management

- [Wallet Creation](wallet-creation.md): Creating and managing Neo wallets
- [Message Signing](message-signing.md): Signing messages with Neo wallets

## Transactions

- [Simple Transfer](simple-transfer.md): Transferring NEO and GAS between accounts
- [Multi-signature Transactions](multi-signature.md): Creating and signing multi-signature transactions

## Smart Contracts

- [Contract Invocation](contract-invocation.md): Calling smart contract methods
- [Contract Deployment](contract-deployment.md): Deploying new smart contracts
- [NEP-17 Tokens](nep17-tokens.md): Working with NEP-17 token standard

## Neo X Integration

- [Neo X Connection](neo-x-connection.md): Connecting to Neo X EVM-compatible chain
- [Bridge Operations](bridge-operations.md): Transferring assets between Neo N3 and Neo X
- [EVM Contracts](evm-contracts.md): Working with EVM contracts on Neo X

## Advanced

- [Oracle Usage](oracle-usage.md): Interacting with Neo Oracle service
- [Custom Signatures](custom-signatures.md): Implementing custom signature schemes
- [RPC Extensions](rpc-extensions.md): Using extended RPC capabilities

## Running the Examples

Each example can be run from the root of the repository using:

```bash
cargo run --example <example_name>
```

For example:

```bash
cargo run --example wallet_creation
```

Or navigate to the specific example directory and run:

```bash
cargo run
```

## Example Code Structure

Most examples follow this structure:

1. **Setup**: Establishing connection to Neo nodes
2. **Account preparation**: Loading or creating accounts
3. **Main operation**: Performing the specific blockchain operation
4. **Verification**: Checking the results of the operation

## Adding Your Own Examples

If you've created a useful example and would like to contribute it, please follow these steps:

1. Create a new directory under `/examples` with a descriptive name
2. Add your Rust code and a proper `Cargo.toml` file
3. Document your example with clear comments
4. Create a pull request to the NeoRust repository

## Full Example List

Here's a list of all available examples in the repository:

- [basic_connection](https://github.com/username/NeoRust/tree/main/examples/basic_connection)
- [wallet_management](https://github.com/username/NeoRust/tree/main/examples/wallet_management)
- [message_signing](https://github.com/username/NeoRust/tree/main/examples/message_signing)
- [neo_x](https://github.com/username/NeoRust/tree/main/examples/neo_x)
- [simple_transfer](https://github.com/username/NeoRust/tree/main/examples/simple_transfer)
- [contract_invocation](https://github.com/username/NeoRust/tree/main/examples/contract_invocation)