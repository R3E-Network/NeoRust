# Feature Flag System Documentation Index

This document provides an index of all documentation related to the feature flag system in the NeoRust SDK.

## Core Documentation

- [README.md Feature Flag Section](../README.md#feature-flag-system): Main documentation for feature flags, including feature groups, examples, and performance benefits.

## Implementation Documents

- [Feature Flag Implementation Summary](feature_flag_implementation_summary.md): Overview of the feature flag implementation, changes made, and benefits.
- [Performance Report](performance_report.md): Detailed metrics on performance improvements including compile times and binary sizes.
- [Migration Guide](migration_guide.md): Guide for users to transition to the new feature flag system.
- [Feature Flag Testing Guide](feature_flag_testing_guide.md): Instructions for testing and maintaining the feature flag system.

## Examples

- [Minimal Wallet Example](../examples/feature_flags/minimal_wallet.rs): Demonstrates creating a wallet with minimal features.
- [NEP-17 Token Example](../examples/feature_flags/nep17_token.rs): Shows how to interact with NEP-17 tokens.
- [Example README](../examples/feature_flags/README.md): Guide to using the feature flag examples.

## CI/Testing

- [Feature Combinations Workflow](../.github/workflows/feature-combinations.yml): GitHub Actions workflow for testing different feature combinations.
- [PR Template for Feature Flags](../.github/PULL_REQUEST_TEMPLATE/feature_flags.md): Pull request template for feature flag changes.

## Feature Structure Summary

### Core Feature Groups
- `std`: Standard library support
- `transaction`: Transaction creation and signing
- `wallet`: Wallet and account management
- `contract`: Smart contract interaction
- `http-client`: HTTP-based RPC client
- `ws-client`: WebSocket client

### Token Standards
- `nep17`: NEP-17 fungible token standard
- `nep11`: NEP-11 non-fungible token standard

### Integration Features
- `sgx`: Intel SGX secure enclave integration
- `wasm`: WebAssembly support
- `ledger`: Ledger hardware wallet support
- `aws`: AWS KMS integration
- `ethereum-compat`: Ethereum compatibility for Neo X 