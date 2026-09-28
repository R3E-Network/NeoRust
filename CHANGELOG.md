# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [3.4.0] - 2026-09-28

### Added
- **Core Types Enhancement**
  - Comprehensive Serde serialization support across all primitive types
  - `From<String>` implementations for Script, Hash160, Hash256, and Address types
  - Enhanced type conversion capabilities for better ergonomics

- **Cryptographic Improvements**
  - ECPoint validation and error handling enhancements
  - Secp256r1 key validation with proper error propagation
  - Improved security through stricter input validation

- **Protocol Layer**
  - Complete transaction builder implementation with fluent API
  - Script builder with comprehensive opcode support
  - Enhanced witness handling and validation
  - Improved serialization performance and correctness

- **Testing Infrastructure**
  - 100+ new unit tests across all modules
  - Integration tests for transaction building workflows
  - Property-based testing for cryptographic operations
  - Comprehensive edge case coverage

### Changed
- **Breaking**: Improved error handling with specific error types
- **Performance**: Optimized serialization paths reducing allocations by ~20%
- **API**: Enhanced type safety with stricter validation rules
- Script and Hash types now enforce validation on construction

### Fixed
- ECPoint validation now correctly rejects invalid points
- Secp256r1 key validation edge cases resolved
- Transaction serialization correctness issues
- VarInt encoding for edge case values
- Memory safety improvements in unsafe code blocks

### Security
- Stricter input validation preventing malformed data
- Enhanced cryptographic primitive validation
- Comprehensive bounds checking in all unsafe code
- Audit findings from v3.3.1 addressed

### Documentation
- Complete API documentation for all public types
- Usage examples for transaction building
- Cryptographic operations guide
- Migration guide from 3.3.x to 3.4.0

## [3.3.1] - 2024-XX-XX

### Fixed
- Removed cargo-nextest dependency to eliminate quick-xml DoS vulnerabilities
- Security audit findings resolved

## [3.3.0] - 2024-XX-XX

### Added
- SGX validation support
- Monitoring infrastructure
- Performance benchmarks
- Comprehensive Cargo.lock for reproducible builds

---

## Migration Guide: 3.3.x → 3.4.0

### Type Construction Changes

**Before:**
```rust
let script = Script { data: vec![0x51] };
let hash = Hash160 { data: [0u8; 20] };
```

**After:**
```rust
let script = Script::from_hex("51")?;
let hash = Hash160::from_hex("0000000000000000000000000000000000000000")?;
```

### Error Handling

**Before:**
```rust
let key = Secp256r1PrivateKey::from_bytes(&bytes).expect("valid key");
```

**After:**
```rust
let key = Secp256r1PrivateKey::from_bytes(&bytes)
    .map_err(|e| format!("Invalid key: {}", e))?;
```

### Transaction Building

**New fluent API:**
```rust
use neo_protocol::TransactionBuilder;

let tx = TransactionBuilder::new()
    .version(0)
    .nonce(12345)
    .system_fee(1000000)
    .network_fee(500000)
    .valid_until_block(10000)
    .add_signer(/* ... */)
    .script(script)
    .build()?;
```

[3.4.0]: https://github.com/r3e/neo-rust-sdk/compare/v3.3.1...v3.4.0
[3.3.1]: https://github.com/r3e/neo-rust-sdk/compare/v3.3.0...v3.3.1
[3.3.0]: https://github.com/r3e/neo-rust-sdk/releases/tag/v3.3.0
