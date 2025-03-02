# Changelog

All notable changes to the NeoRust SDK will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Added AWS Nitro TEE support as a feature
- Added improved feature documentation in README.md and installation guide

### Changed
- Reorganized feature system into logical groups (Core, Crypto, Utils, Serde, Optional)
- Improved conditional compilation with proper feature gates
- Enhanced feature compatibility documentation

### Fixed
- Fixed syntax errors in script_hash.rs
- Fixed unused imports in various modules

### Optimized
- Reduced compile times with more granular feature selection
- Improved binary size optimization with better feature organization

## [0.1.3] - 2025-03-01

### Added
- Added 'ledger' feature for hardware wallet support
- Added 'nightly' feature for documentation builds
- Added conditional compilation for tests with different feature combinations

### Changed
- Updated feature documentation in README.md
- Improved feature flag system with better organization
- Made crypto-related dependencies optional with proper feature gates

### Fixed
- Fixed test failures when running with different feature combinations
- Fixed import errors in various modules
- Fixed circular dependencies between modules

## [0.1.2] - 2023-11-15

### Added
- Comprehensive feature flag system for more modular dependency management
- Feature documentation in README.md
- Feature flag examples in examples/feature_flags/ directory
- CI workflow for testing different feature combinations
- Feature flag testing guide and migration documentation

### Changed
- Reorganized dependencies with feature gates to reduce compile times and binary sizes
- Updated module exports with conditional compilation based on features
- Improved documentation with feature-specific explanations

### Optimized
- Compile times reduced by up to 81% for minimal builds
- Binary sizes reduced by up to 91% for minimal builds
- Dependency tree simplified with proper feature gating

## [0.1.1] - 2023-09-15

### Added
- Initial crates.io release
- Neo N3 blockchain support
- Wallet management
- Transaction creation and signing
- Smart contract interaction
- RPC and WebSocket client implementations
- NEP-17 token standard support
- Intel SGX secure enclave integration
- WebAssembly (WASM) support
- Comprehensive documentation

[Unreleased]: https://github.com/R3E-Network/NeoRust/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/R3E-Network/NeoRust/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/R3E-Network/NeoRust/releases/tag/v0.1.1    