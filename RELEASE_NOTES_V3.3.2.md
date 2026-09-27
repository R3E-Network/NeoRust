# Release Notes - Neo Rust SDK v3.3.2

**Release Date**: September 27, 2026  
**Type**: Security Hotfix Release  
**Priority**: 🔴 CRITICAL - Immediate Upgrade Recommended

---

## 🚨 Security Advisory

### Critical Security Fix: RUSTSEC-2026-0285 (CVE-2025-61730)

This release addresses a **critical security vulnerability** in the rustls TLS library that could allow TLS 1.3 handshake messages to be incorrectly accepted across encryption level boundaries.

**Affected Versions**: All versions using rustls 0.23.44  
**Fixed Version**: rustls 0.23.45  
**Severity**: Medium (CVSS 5.3)

**What was the issue?**

The rustls library accepted TLS 1.3 handshake messages sent at the wrong encryption level when they followed a key-changing message in the same record. For example, a plaintext `EncryptedExtensions` message packed into the same record as the `ServerHello` was accepted.

RFC 8446 section 5.1 requires that handshake messages do not span key changes, and implementations must terminate the connection with an "unexpected_message" alert if they do.

**Impact Assessment**

- The handshake transcript remains authenticated, preventing network-position attackers from altering or completing handshakes
- A peer could send handshake messages that should be encrypted in plaintext without rustls rejecting the connection
- This is functionally the same bug as Go's GO-2026-4340

**Recommendation**: All users should upgrade to v3.3.2 immediately.

---

## 📦 What's Changed

### Security Updates
- ✅ Update rustls from 0.23.44 to 0.23.45 (fixes RUSTSEC-2026-0285)
- ✅ Security fix applies across all transport layers:
  - `reqwest` HTTP client
  - `hyper-rustls` HTTP/2 backend
  - `tokio-rustls` async TLS
  - `rustls-platform-verifier` certificate validation

### Maintenance
- ✅ Remove obsolete RUSTSEC-2024-0436 from deny.toml (paste crate no longer in dependency tree)
- ✅ Clean up dependency ignore list

### Version Updates
- `neo3`: 3.3.1 → 3.3.2
- `rustls`: 0.23.44 → 0.23.45

---

## 🔄 Migration Guide

### For Library Users

**Upgrade is straightforward - update your Cargo.toml:**

```toml
[dependencies]
neo3 = "3.3.2"  # Previously: "3.3.1"
```

Then run:
```bash
cargo update
cargo build
```

**No API changes** - this is a drop-in replacement with only dependency updates.

### For Application Developers

1. Update your `Cargo.toml` to use `neo3 = "3.3.2"`
2. Run `cargo update` to refresh Cargo.lock
3. Rebuild your application
4. Run your test suite to verify everything works
5. Deploy the updated version

**Breaking Changes**: None  
**Deprecations**: None  
**New Features**: None (security hotfix only)

---

## ✅ Verification Steps

After upgrading, verify the security fix:

```bash
# Check rustls version in dependency tree
cargo tree | grep rustls

# Expected output:
# rustls v0.23.45

# Run security audit
cargo audit

# Should show no vulnerabilities (only warnings for unmaintained crates)

# Run cargo deny check
cargo deny check advisories

# Should show "advisories ok"
```

---

## 📊 Compatibility

- **Rust Version**: 1.95.0+ (MSRV unchanged)
- **Neo N3**: Full compatibility maintained
- **Platform Support**: Linux, macOS, Windows (unchanged)
- **Feature Flags**: All existing features unchanged
- **API Compatibility**: 100% backward compatible

---

## 🔍 Full Changelog

### Security
- **CRITICAL**: Update rustls to 0.23.45 to fix RUSTSEC-2026-0285 (CVE-2025-61730)
  - Vulnerability: TLS 1.3 handshake messages incorrectly accepted across encryption level boundaries
  - Impact: Medium severity - handshake transcript authenticated but plaintext messages could be accepted
  - Fixed by updating rustls across all transport layers

### Fixed
- Remove obsolete RUSTSEC-2024-0436 from deny.toml ignore list

### Changed
- Update dependency chain to use rustls 0.23.45 for secure TLS 1.3 handshake validation
- Bump version to 3.3.2 for security hotfix release

---

## 📚 Additional Resources

- **Security Advisory**: https://rustsec.org/advisories/RUSTSEC-2026-0285
- **CVE Details**: https://github.com/rustls/rustls/security/advisories/GHSA-2mjx-qc3c-rqvc
- **Go Equivalent**: GO-2026-4340 (CVE-2025-61730)
- **RFC 8446 Section 5.1**: TLS 1.3 key change requirements

---

## 🆘 Support

If you encounter any issues with this release:

1. **GitHub Issues**: https://github.com/r3e-network/neo-rust-sdk/issues
2. **Email**: jimmy@r3e.network
3. **Documentation**: https://docs.rs/neo3

---

## 🙏 Acknowledgments

Thanks to the rustls team for the rapid security fix and clear disclosure.

---

## 📅 Previous Release (v3.3.1)

For the full feature set and previous changes, see:
- [CHANGELOG.md](./CHANGELOG.md)
- [Release v3.3.1](https://github.com/r3e-network/neo-rust-sdk/releases/tag/v3.3.1)

---

**Release Manager**: R3E Network Team  
**Release Date**: 2026-09-27  
**Git Tag**: v3.3.2
