# Neo CLI Integration Tests

This directory contains integration tests for the `neo-cli` commands. These tests verify that the CLI commands work correctly by executing the actual binary and checking its output.

## Running the Tests

To run all integration tests:

```bash
cargo test --test integration_tests
```

To run a specific test:

```bash
cargo test --test integration_tests wallet_tests::test_wallet_create
```

To run all tests in a specific module:

```bash
cargo test --test integration_tests wallet_tests
```

## Test Structure

The tests are organized into modules based on the CLI command categories:

- `wallet_tests.rs`: Tests for wallet commands
- `blockchain_tests.rs`: Tests for blockchain commands
- `network_tests.rs`: Tests for network commands
- `contract_tests.rs`: Tests for contract commands
- `defi_tests.rs`: Tests for DeFi commands
- `init_tests.rs`: Tests for initialization commands

## Adding New Tests

To add a new test:

1. Identify the appropriate module for your test
2. Create a new test function with the `#[test]` attribute
3. Use the `CliTest` utility to create a test environment
4. Execute CLI commands using the `run()` or `run_with_input()` methods
5. Verify the results using the assertion helpers

Example:

```rust
#[test]
fn test_new_command() {
    let cli = CliTest::new();
    
    let output = cli.run(&["command", "subcommand", "--flag", "value"]);
    
    assert_success(&output);
    assert_output_contains(&output, "Expected output");
}
```

## Test Utilities

The `utils.rs` file provides helpful utilities for testing:

- `CliTest`: A struct that sets up a test environment
- `assert_success`: Verifies that a command executed successfully
- `assert_output_contains`: Checks if the command output contains an expected string
- `assert_output_matches`: Checks if the output matches a regex pattern

## Notes on Test Execution

These tests execute the actual `neo-cli` binary, so they require:

1. The binary to be built before testing
2. Internet connectivity for tests that interact with the Neo blockchain
3. Some tests may take longer to run due to blockchain interactions

Some blockchain-dependent tests may be marked with `#[ignore]` to skip them during regular testing. 