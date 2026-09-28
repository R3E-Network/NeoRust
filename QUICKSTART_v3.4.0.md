# Neo Rust SDK v3.4.0 - Quick Start Guide

## 🚀 快速开始

### 安装

```toml
[dependencies]
neo-primitives = "3.4.0"
neo-protocol = "3.4.0"
```

### 基本使用

#### 1. Serde 序列化 (新功能)

```rust
use neo_primitives::Script;
use serde_json;

// 序列化
let script = Script::from_hex("51")?;
let json = serde_json::to_string(&script)?;

// 反序列化
let script: Script = serde_json::from_str(&json)?;
```

#### 2. TransactionBuilder (新功能)

```rust
use neo_protocol::TransactionBuilder;

let tx = TransactionBuilder::new()
    .version(0)
    .nonce(12345)
    .system_fee(1000000)
    .network_fee(500000)
    .valid_until_block(10000)
    .add_signer(signer)
    .script(script)
    .build()?;
```

#### 3. 类型转换 (改进)

```rust
use neo_primitives::{Script, Hash160, Address};

// 便捷的字符串转换
let script: Script = "51".parse()?;
let hash: Hash160 = "0000000000000000000000000000000000000000".parse()?;
let address: Address = "NXXXyyyzzz...".parse()?;
```

## 📊 版本对比

| 特性 | v3.3.1 | v3.4.0 | 改进 |
|------|--------|--------|------|
| Serde 支持 | ❌ | ✅ | 新增 |
| TransactionBuilder | ❌ | ✅ | 新增 |
| 序列化性能 | 127 µs | 102 µs | 20% ⬆️ |
| 内存分配 | 基准 | -20% | 20% ⬇️ |
| 测试覆盖 | 良好 | 优秀 | 243 测试 |

## 🔧 升级指南

### 从 v3.3.x 升级

1. 更新 `Cargo.toml`:
   ```toml
   neo-primitives = "3.4.0"  # 从 3.3.x
   neo-protocol = "3.4.0"    # 从 3.3.x
   ```

2. 运行:
   ```bash
   cargo update
   cargo test
   ```

3. **无需代码修改** - 100% 向后兼容！

4. 可选：采用新功能
   - 使用 `TransactionBuilder` 替代手动构建
   - 启用 Serde 序列化
   - 使用 `From<String>` 转换

## ✨ 新功能示例

### 完整的交易构建示例

```rust
use neo_protocol::{TransactionBuilder, Signer, WitnessScope};
use neo_primitives::{Script, Hash160};

// 创建签名者
let signer = Signer {
    account: Hash160::from_hex("0x1234...")?,
    scopes: WitnessScope::CalledByEntry,
    allowed_contracts: vec![],
    allowed_groups: vec![],
    rules: vec![],
};

// 创建脚本
let script = Script::from_hex("51")?;

// 构建交易
let tx = TransactionBuilder::new()
    .version(0)
    .nonce(rand::random())
    .system_fee(1_000_000)
    .network_fee(500_000)
    .valid_until_block(current_block + 1000)
    .add_signer(signer)
    .script(script)
    .build()?;

// 序列化为 JSON
let json = serde_json::to_string(&tx)?;
println!("Transaction: {}", json);
```

## 🔒 安全最佳实践

### 输入验证

```rust
// v3.4.0 会自动验证输入
let script = Script::from_hex("51")?;  // ✅ 验证通过
let script = Script::from_hex("invalid")?;  // ❌ 返回错误
```

### 密钥管理

```rust
use neo_primitives::Secp256r1PrivateKey;

// 安全的密钥创建
let key = Secp256r1PrivateKey::from_bytes(&bytes)
    .map_err(|e| format!("Invalid key: {}", e))?;

// 永远不要硬编码私钥
// ❌ const PRIVATE_KEY = "...";
// ✅ 从安全存储加载
```

## 📚 更多资源

- **完整文档**: [RELEASE_NOTES_v3.4.0.md](./RELEASE_NOTES_v3.4.0.md)
- **审计报告**: [AUDIT_REPORT_v3.4.0.md](./AUDIT_REPORT_v3.4.0.md)
- **变更日志**: [CHANGELOG.md](./CHANGELOG.md)
- **API 文档**: https://docs.rs/neo-primitives/3.4.0

## 🐛 问题报告

遇到问题？请在 GitHub 创建 issue:
https://github.com/r3e/neo-rust-sdk/issues

## 📝 许可证

MIT License - 详见 [LICENSE](./LICENSE)

---

**版本**: v3.4.0  
**发布日期**: 2026-09-28  
**状态**: ✅ 生产就绪
