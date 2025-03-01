# NeoRust Examples

This directory contains examples demonstrating how to use the NeoRust SDK for various Neo N3 blockchain operations.

## Structure

Each subdirectory contains examples for a specific category of functionality:

- **neo_nodes**: Connecting to Neo N3 nodes and retrieving blockchain information
- **neo_wallets**: Creating and managing Neo N3 wallets
- **neo_transactions**: Building and sending transactions
- **neo_contracts**: Deploying and managing smart contracts
- **neo_smart_contracts**: Interacting with smart contracts
- **neo_nns**: Working with Neo Name Service
- **neo_nep17_tokens**: Working with NEP-17 tokens
- **neo_famous_contracts**: Interacting with well-known Neo N3 contracts
- **neo_x**: Working with Neo X (EVM compatibility layer)
- **wallets**: General wallet management examples
- **transactions**: General transaction examples
- **providers**: Provider examples
- **queries**: Query examples
- **subscriptions**: Subscription examples
- **big-numbers**: Working with big numbers
- **contracts**: Contract examples
- **events**: Event examples
- **middleware**: Middleware examples
- **sgx**: Secure enclave examples

## Running the Examples

Each example is a standalone Rust project that can be run independently. To run an example:

1. Navigate to the example directory:
   ```bash
   cd examples/neo_nodes
   ```

2. Run the example using cargo:
   ```bash
   cargo run --example connect_to_node
   ```

For examples that require specific features, you can enable them with the `--features` flag:

```bash
cargo run --example wallet_management --features ledger
```

## Example Categories

### Neo N3 Specific Examples

- **neo_nodes**: Examples for connecting to Neo N3 nodes
- **neo_wallets**: Examples for managing Neo N3 wallets
- **neo_transactions**: Examples for creating and sending Neo N3 transactions
- **neo_contracts**: Examples for deploying Neo N3 smart contracts
- **neo_smart_contracts**: Examples for interacting with Neo N3 smart contracts
- **neo_nns**: Examples for working with Neo Name Service
- **neo_nep17_tokens**: Examples for working with NEP-17 tokens
- **neo_famous_contracts**: Examples for interacting with well-known Neo N3 contracts
- **neo_x**: Examples for working with Neo X (EVM compatibility layer)

### General Examples

- **wallets**: General wallet management examples
- **transactions**: General transaction examples
- **providers**: Provider examples
- **queries**: Query examples
- **subscriptions**: Subscription examples
- **big-numbers**: Working with big numbers
- **contracts**: Contract examples
- **events**: Event examples
- **middleware**: Middleware examples
- **sgx**: Secure enclave examples

## Contributing

If you'd like to add a new example:

1. Create a new file in the appropriate example directory
2. Add the example to the Cargo.toml file in that directory
3. Ensure the example is well-documented with comments
4. Update this README.md file if necessary 