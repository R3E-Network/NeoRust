# NeoRust Build & Test Scripts

This directory contains useful scripts for building and testing the NeoRust project with different feature configurations.

## Build Scripts

### Unix/Linux/MacOS:
```bash
# Build with default configuration
./scripts/build.sh

# Build with specific features
./scripts/build.sh --features ledger,aws,futures

# Build with all features
./scripts/build.sh --all-features

# Build in release mode
./scripts/build.sh --release

# Build with no default features
./scripts/build.sh --no-default-features

# Show help
./scripts/build.sh --help
```

### Windows:
```batch
# Build with default configuration
.\scripts\build.bat

# Build with specific features
.\scripts\build.bat --features ledger,aws,futures

# Build with all features
.\scripts\build.bat --all-features

# Build in release mode
.\scripts\build.bat --release

# Build with no default features
.\scripts\build.bat --no-default-features

# Show help
.\scripts\build.bat --help
```

## Test Scripts

### Unix/Linux/MacOS:
```bash
# Run tests with default features (ledger,aws,futures)
./scripts/test.sh

# Run tests with specific features
./scripts/test.sh --features ledger,aws

# Run tests with all features
./scripts/test.sh --all-features

# Run tests and show output
./scripts/test.sh --nocapture

# Run tests and continue even if some fail
./scripts/test.sh --no-fail-fast

# Show help
./scripts/test.sh --help
```

### Windows:
```batch
# Run tests with default features (ledger,aws,futures)
.\scripts\test.bat

# Run tests with specific features
.\scripts\test.bat --features ledger,aws

# Run tests with all features
.\scripts\test.bat --all-features

# Run tests and show output
.\scripts\test.bat --nocapture

# Run tests and continue even if some fail
.\scripts\test.bat --no-fail-fast

# Show help
.\scripts\test.bat --help
```

## Feature Combinations

Based on the project's needs, the following feature combinations are recommended:

1. `ledger,aws,futures` - Default set for running most tests
2. `--all-features` - For comprehensive testing (note: requires all dependencies)
3. `--no-default-features` - Minimal feature set

Note: The `sgx` feature has commented out dependencies in Cargo.toml. To use the SGX feature,
you'll need to uncomment and install the appropriate SGX SDK dependencies. 