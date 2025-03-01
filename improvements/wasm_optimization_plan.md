# WASM Optimization Implementation Plan

## Overview

This document outlines the plan for optimizing the NeoRust SDK for WebAssembly (WASM) environments, with a focus on improving performance, reducing bundle size, and enhancing compatibility with web-based applications.

## Current State Analysis

The current WebAssembly support in NeoRust has several limitations:

1. Large compiled WASM bundle size due to unnecessary inclusions
2. Limited browser-specific optimizations
3. Non-optimal compatibility with JavaScript async patterns
4. Missing TypeScript type definitions
5. Limited documentation for web developers
6. Potential performance bottlenecks in crypto operations

## Implementation Goals

1. Reduce WASM bundle size by at least 40%
2. Improve runtime performance for key operations
3. Provide better integration with JavaScript/TypeScript
4. Create a streamlined API surface for web developers
5. Enhance documentation with web-specific examples

## Technical Implementation Plan

### 1. WASM-specific Feature Flag Structure

Update the feature flag system to include more granular WASM-specific features:

```toml
[features]
# WASM-specific features
wasm = ["wasm-bindgen", "js-sys", "web-sys"]
wasm-browser = ["wasm", "web-sys/Window", "web-sys/Document", "web-sys/Element"]
wasm-crypto = ["wasm", "wasm-bindgen-rayon"]
wasm-no-modules = ["wasm", "no-modules-support"]
wasm-nodejs = ["wasm", "nodejs-support"]
```

### 2. JavaScript API Surface Optimization

Revise the exported API surface to be more idiomatic for JavaScript/TypeScript developers:

```rust
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl Neo3Client {
    #[wasm_bindgen(constructor)]
    pub fn new_browser(rpc_url: &str) -> Result<Neo3Client, JsError> {
        // Browser-optimized client initialization
    }
    
    #[wasm_bindgen]
    pub async fn get_block_count_js(&self) -> Result<u32, JsError> {
        self.get_block_count().await.map_err(|e| JsError::new(&e.to_string()))
    }
    
    // Additional methods with JS-friendly signatures...
}
```

### 3. Size Optimization Techniques

Implement the following size optimization techniques:

1. **Dead Code Elimination**:
   - Use `wasm-opt` to strip unused code
   - Implement `#[cfg(not(target_arch = "wasm32"))]` to exclude non-WASM code

2. **Targeted Dependency Selection**:
   - Replace heavy crypto libraries with browser-native Web Crypto API where possible
   - Use conditional compilation for platform-specific dependencies

3. **Code Splitting**:
   - Create separate WASM modules for rarely used functionality
   - Implement dynamic loading for optional components

Example implementation:

```rust
#[cfg(all(feature = "wasm", feature = "wasm-crypto-native"))]
mod crypto {
    // Use browser's WebCrypto API
    use web_sys::{SubtleCrypto, Crypto};
    use wasm_bindgen::prelude::*;

    pub async fn sha256(data: &[u8]) -> Result<[u8; 32], JsError> {
        let window = web_sys::window().ok_or_else(|| JsError::new("No window found"))?;
        let crypto = window.crypto().map_err(|_| JsError::new("No crypto support"))?;
        // Use browser's native SHA-256 implementation
        // ...
    }
}

#[cfg(all(feature = "wasm", not(feature = "wasm-crypto-native")))]
mod crypto {
    // Use Rust implementation
    use sha2::{Sha256, Digest};
    
    pub async fn sha256(data: &[u8]) -> Result<[u8; 32], JsError> {
        // Use Rust SHA-256 implementation
        // ...
    }
}
```

### 4. Performance Optimizations

Implement the following performance optimizations:

1. **Memory Management**:
   - Minimize copying between JS and WASM boundary
   - Use `Uint8Array` views where possible instead of copying

2. **Threading Support** (where available):
   - Implement `wasm-bindgen-rayon` for parallelizable operations
   - Add thread pool support for browsers that support Web Workers

3. **Lazy Initialization**:
   - Defer expensive setup until needed
   - Cache results of expensive operations

Example implementation:

```rust
#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl SignatureManager {
    #[wasm_bindgen]
    pub fn verify_signature_js(
        &self, 
        message: &[u8], 
        signature: &[u8], 
        public_key: &[u8]
    ) -> Promise {
        // Create a Promise to handle async verification
        let message = message.to_vec();
        let signature = signature.to_vec();
        let public_key = public_key.to_vec();
        
        future_to_promise(async move {
            // Use Web Crypto API for verification when available
            #[cfg(feature = "wasm-crypto-native")]
            {
                if let Some(result) = try_web_crypto_verify(&message, &signature, &public_key).await {
                    return Ok(JsValue::from_bool(result));
                }
            }
            
            // Fall back to Rust implementation
            let result = verify_signature_internal(&message, &signature, &public_key)?;
            Ok(JsValue::from_bool(result))
        })
    }
}
```

### 5. TypeScript Integration

Create comprehensive TypeScript type definitions:

1. Generate TypeScript types for all exported APIs
2. Implement custom .d.ts files for improved developer experience
3. Add JSDoc comments for better IDE integration

Example:

```typescript
// neo3.d.ts
export class Neo3Client {
  /**
   * Creates a new Neo3Client instance optimized for browser environments
   * @param rpcUrl - URL of the Neo N3 RPC server
   */
  constructor(rpcUrl: string);
  
  /**
   * Gets the current block count from the Neo N3 blockchain
   * @returns A promise that resolves to the current block count
   */
  getBlockCount(): Promise<number>;
  
  // Additional type definitions...
}
```

### 6. Browser-specific API Enhancements

Add browser-specific convenience APIs:

1. **Local Storage Integration**:
   - Wallet persistence using browser's localStorage/IndexedDB
   - Encrypted key storage with Web Crypto API

2. **Browser Event System**:
   - Implement custom event listeners for blockchain updates
   - Support for subscription-based pattern

3. **Progressive Web App Support**:
   - Service worker integration for offline capability
   - Push notification support for transaction confirmations

Example implementation:

```rust
#[cfg(all(feature = "wasm", feature = "wasm-browser"))]
#[wasm_bindgen]
impl Neo3Wallet {
    #[wasm_bindgen]
    pub async fn save_to_browser_storage(&self, key_name: &str, password: &str) -> Result<(), JsError> {
        // Encrypt wallet using Web Crypto API
        let encrypted = self.encrypt_with_web_crypto(password).await?;
        
        // Save to localStorage
        let window = web_sys::window().ok_or_else(|| JsError::new("No window found"))?;
        let storage = window.local_storage().map_err(|_| JsError::new("Cannot access localStorage"))?;
        let storage = storage.ok_or_else(|| JsError::new("No localStorage available"))?;
        
        storage.set_item(key_name, &encrypted)
            .map_err(|_| JsError::new("Failed to save to localStorage"))?;
            
        Ok(())
    }
    
    #[wasm_bindgen]
    pub async fn load_from_browser_storage(key_name: &str, password: &str) -> Result<Neo3Wallet, JsError> {
        // Load from localStorage
        let window = web_sys::window().ok_or_else(|| JsError::new("No window found"))?;
        let storage = window.local_storage().map_err(|_| JsError::new("Cannot access localStorage"))?;
        let storage = storage.ok_or_else(|| JsError::new("No localStorage available"))?;
        
        let encrypted = storage.get_item(key_name)
            .map_err(|_| JsError::new("Failed to load from localStorage"))?
            .ok_or_else(|| JsError::new("No wallet found with that name"))?;
            
        // Decrypt using Web Crypto API
        let wallet_data = Neo3Wallet::decrypt_with_web_crypto(&encrypted, password).await?;
        Neo3Wallet::from_decrypted_data(wallet_data)
    }
}
```

### 7. Examples and Documentation

Create WASM-specific examples and documentation:

1. **Example Applications**:
   - Simple wallet web application
   - Block explorer component
   - NFT gallery application

2. **Integration Examples**:
   - React/Vue/Angular integration examples
   - TypeScript usage patterns
   - Webpack/Rollup configuration guides

3. **Performance Guides**:
   - Best practices for minimizing bundle size
   - Benchmarking and profiling instructions
   - Optimization techniques for web developers

## Testing Strategy

1. **Size Testing**:
   - Measure WASM bundle size before and after optimizations
   - Track size impact of individual features

2. **Performance Testing**:
   - Benchmark key operations in browser environments
   - Compare against non-WASM implementations
   - Test on various browsers and devices

3. **Integration Testing**:
   - Test with popular web frameworks
   - Verify TypeScript type correctness
   - Test in various bundler configurations

4. **Compatibility Testing**:
   - Test across Chrome, Firefox, Safari, Edge
   - Test in Node.js environments
   - Test with different WASM feature sets

## Implementation Phases

### Phase 1: Foundational Work (2 weeks)
- Update feature flag system for WASM
- Implement basic size optimizations
- Create initial TypeScript type definitions

### Phase 2: Core Optimizations (3 weeks)
- Implement browser-native crypto integrations
- Develop optimized memory management
- Create browser storage integrations

### Phase 3: Documentation and Examples (1 week)
- Develop example applications
- Create integration guides
- Complete TypeScript documentation

## Success Metrics

1. **Size Reduction**: 40% reduction in main WASM bundle size
2. **Performance**: 2x improvement in key crypto operations
3. **Developer Experience**: Comprehensive TypeScript definitions with 100% API coverage
4. **Compatibility**: Successful tests across 4 major browsers
5. **Documentation**: Complete web developer guide with at least 5 example applications

## Future Considerations

1. **WebGPU Integration**: For future optimization of cryptographic operations
2. **WebTransport**: For direct P2P communication between browsers
3. **SharedArrayBuffer**: For improved multi-threading when browser support improves
4. **Component Model**: Adoption of the WASM Component Model when standardized 