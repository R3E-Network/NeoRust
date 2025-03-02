# NeoRust Scripts

This directory contains helper scripts for building, testing, and maintaining the NeoRust project.

## Available Scripts

### Test Scripts

| Script | Platform | Description |
|--------|----------|-------------|
| `test.sh` | Unix/Linux/macOS | Main test runner with various options |
| `test.bat` | Windows | Windows version of the test runner |
| `test_all_features.sh` | Unix/Linux/macOS | Test with multiple feature combinations |
| `test_all_features.bat` | Windows | Windows version of feature matrix testing |
| `run_all_examples.sh` | Unix/Linux/macOS | Run all examples in the project |
| `run_all_examples.bat` | Windows | Windows version of example runner |

### Build Scripts

| Script | Platform | Description |
|--------|----------|-------------|
| `build.sh` | Unix/Linux/macOS | Build with various feature configurations |
| `build.bat` | Windows | Windows version of the build script |

## Usage

### Test Script

The main test script supports the following options:

```bash
./scripts/test.sh [OPTIONS]
```

#### Options:

| Option | Description |
|--------|-------------|
| `--all-features` | Run tests with all feature combinations |
| `--all-examples` | Run all examples |
| `--nocapture` | Show test output |
| `--no-fail-fast` | Continue testing even if some tests fail |
| `--help` | Show help message |
| *any cargo test options* | Passed directly to cargo test |

Examples:

```bash
# Run tests with default features
./scripts/test.sh

# Run tests with all feature combinations
./scripts/test.sh --all-features

# Run all examples
./scripts/test.sh --all-examples

# Run tests with specific cargo options
./scripts/test.sh --nocapture --no-fail-fast
```

### Build Script

The build script supports various feature configurations:

```bash
./scripts/build.sh [OPTIONS]
```

#### Options:

| Option | Description |
|--------|-------------|
| `--features FEATURES` | Build with specific features (comma-separated) |
| `--all-features` | Build with all features enabled |
| `--no-default-features` | Build without default features |
| `--release` | Build in release mode |
| `--help` | Show help message |
| *any cargo build options* | Passed directly to cargo build |

Examples:

```bash
# Build with default configuration
./scripts/build.sh

# Build with specific features
./scripts/build.sh --features ledger,crypto-standard

# Build with all features
./scripts/build.sh --all-features

# Build in release mode
./scripts/build.sh --release
```

## Feature Matrix Testing

The `test_all_features` script tests the following feature combinations:

1. **Default features only**
2. **All features enabled**
3. **Minimal build** with just `std`
4. **Standard application**: with `crypto-standard`, `std`, `transaction`
5. **Wallet application**: with `crypto-standard`, `std`, `wallet`, `transaction`
6. **Hardware wallet integration**: with `crypto-standard`, `std`, `ledger`, `wallet`
7. **Neo X / EVM compatibility**: with `crypto-standard`, `std`, `ethereum-compat`
8. **Custom crypto configurations**: with specific crypto features

This ensures that the codebase works correctly with any feature configuration you might use in your application.

## Example Runner

The `run_all_examples` script:

1. Discovers all examples in the `examples` directory
2. Compiles each example with all features
3. Runs each example and captures the output
4. Reports success/failure status for each example

This is useful for verifying that all examples work as expected, checking compatibility with the latest code changes, and demonstrating usage patterns to new users.

## Recommended Feature Combinations

Based on the project's needs, the following feature combinations are recommended:

1. `std,crypto-standard` - Default set for basic functionality
2. `std,crypto-standard,transaction,wallet` - For wallet applications
3. `std,crypto-standard,ledger` - For hardware wallet support
4. `--all-features` - For comprehensive testing (note: requires all dependencies)
5. `--no-default-features --features std` - Minimal feature set

Note: The `sgx` feature has commented out dependencies in Cargo.toml. To use the SGX feature,
you'll need to uncomment and install the appropriate SGX SDK dependencies. 