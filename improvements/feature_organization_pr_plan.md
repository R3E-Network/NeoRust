# Feature Organization Improvement - Pull Request Plan

## Overview

This document outlines the plan for implementing the enhanced dependency management and feature organization for the NeoRust SDK. The goal is to create a more modular and flexible feature flag system that reduces compile times, binary sizes, and dependency conflicts.

## Current State Analysis

The current `Cargo.toml` has several limitations:

1. Limited use of feature flags to control optional functionality
2. Many dependencies are included by default regardless of use case
3. Features are not organized by functional area
4. Some features have overlapping or unclear purposes
5. Missing documentation on what each feature provides

## Pull Request Scope

The PR will:

1. Reorganize all dependencies under appropriate feature flags
2. Create logical feature groups based on functionality
3. Ensure backward compatibility with existing feature flags
4. Add comprehensive documentation for each feature
5. Update examples to demonstrate feature usage

## Implementation Details

### 1. Feature Hierarchy

We will implement a hierarchical feature system with the following structure:

```
default = ["std"]
std = ["serde", "json"]

# Core functionality groups
transaction = ["neo-crypto"]
wallet = ["transaction", "key-management"]
contract = ["transaction", "vm"]
node-client = ["http-client", "json-rpc"]

# Integration features
sgx = ["sgx-runtime", "sgx-crypto"]
wasm = ["wasm-bindgen", "js-sys"]
ledger = ["ledger-transport"]
aws = ["aws-kms"]

# Transport/serialization features
serde = ["dep:serde", "serde_json"]
json = ["serde_json"]
json-rpc = ["jsonrpc-core", "node-client"]
http-client = ["reqwest"]

# Utility features
key-management = ["neo-crypto"]
vm = ["neo-vm"]
neo-crypto = ["crypto-primitives"]
```

### 2. Dependency Reorganization in Cargo.toml

```toml
[dependencies]
# Core dependencies - always included
primitive-types = "0.12"
hex = "0.4"
thiserror = "1.0"

# Optional dependencies with feature gates
serde = { version = "1.0", optional = true }
serde_json = { version = "1.0", optional = true }
jsonrpc-core = { version = "18.0", optional = true }
reqwest = { version = "0.11", optional = true, features = ["json"] }

# SGX-related dependencies
sgx_types = { version = "1.1.6", optional = true }
sgx_urts = { version = "1.1.6", optional = true }

# WASM-related dependencies
wasm-bindgen = { version = "0.2", optional = true }
js-sys = { version = "0.3", optional = true }

# Ledger
ledger-transport = { version = "0.7", optional = true }

# AWS
aws-sdk-kms = { version = "0.24", optional = true }

[features]
default = ["std"]
std = ["serde", "json"]

# Feature groups - detailed above
transaction = ["neo-crypto"]
wallet = ["transaction", "key-management"]
contract = ["transaction", "vm"]
node-client = ["http-client", "json-rpc"]

sgx = ["sgx_types", "sgx_urts", "sgx-runtime", "sgx-crypto"]
wasm = ["wasm-bindgen", "js-sys"]
ledger = ["ledger-transport"]
aws = ["aws-sdk-kms"]

# Detailed features - see hierarchy above
serde = ["dep:serde", "serde_json"]
json = ["serde_json"]
json-rpc = ["jsonrpc-core", "node-client"]
http-client = ["reqwest"]
key-management = ["neo-crypto"]
vm = ["neo-vm"]
neo-crypto = ["crypto-primitives"]
```

### 3. Documentation Updates

We will add detailed feature documentation in:

1. `Cargo.toml` comments
2. Updated README.md section on features
3. API documentation for modules affected by features

Example README section:

```markdown
## Feature Flags

NeoRust uses a flexible feature flag system to allow you to include only the functionality you need. 
The main feature groups are:

- **transaction**: Support for creating and signing transactions
- **wallet**: Wallet management functionality (includes transaction)
- **contract**: Smart contract interaction (includes transaction and VM)
- **node-client**: Neo N3 node communication (includes HTTP client and JSON-RPC)

Integration features:
- **sgx**: Intel SGX secure enclave support
- **wasm**: WebAssembly support for browser environments
- **ledger**: Ledger hardware wallet support
- **aws**: AWS KMS integration

You can enable features in your Cargo.toml:

```toml
neo = { version = "0.X.0", features = ["wallet", "node-client"] }
```

For minimal binary size, specify only what you need:

```toml
neo = { version = "0.X.0", default-features = false, features = ["transaction"] }
```
```

### 4. Build Test Matrix

We will create a CI test matrix that tests various feature combinations to ensure all feature combinations work correctly:

- Default features
- No default features + minimal core
- Each major feature group independently
- Common combinations (wallet + node-client, contract + node-client)
- All features

## Migration Guide

For users of the existing SDK, we'll provide a migration guide:

```markdown
## Feature Flag Migration Guide

### Changes from Previous Versions

If you were using previous versions of NeoRust, here's how to migrate to the new feature system:

| Old Feature | New Feature(s) | Notes |
|-------------|---------------|-------|
| futures | Not needed | Now part of core |
| ledger | ledger | Same functionality |
| aws | aws | Same functionality |
| sgx | sgx | Same functionality |

### Common Use Cases

- **DApp Backend**: `features = ["wallet", "node-client", "contract"]`
- **Wallet Application**: `features = ["wallet", "node-client"]`
- **Contract Deployment Tool**: `features = ["contract", "node-client"]`
- **Browser Web App**: `features = ["wasm", "wallet", "node-client"]`
- **Hardware Wallet Integration**: `features = ["ledger", "wallet"]`
```

## Testing Approach

1. Unit tests for each feature combination
2. Binary size comparison before/after
3. Compilation time measurements before/after
4. Integration tests for each major feature group

## PR Timeline

1. Initial PR with feature reorganization (1 week)
2. Documentation updates (3 days)
3. Example updates (2 days)
4. Testing and refinement (4 days)

Total: ~2 weeks

## Future Considerations

After this PR is merged:

1. Consider further modularization into separate crates
2. Add more granular features for specific use cases
3. Create feature preset profiles for common scenarios 