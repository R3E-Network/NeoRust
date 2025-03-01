# Feature Flag Testing Guide

This guide provides instructions for testing the feature flag system and maintaining it as the SDK evolves.

## Testing Feature Combinations

### Using the CI Workflow

The repository includes a CI workflow in `.github/workflows/feature-combinations.yml` that automatically tests multiple feature combinations. This workflow:

1. Builds and tests the SDK with various feature combinations
2. Measures compile times for different combinations
3. Measures binary sizes for different combinations

To run this workflow manually:
1. Go to the GitHub repository
2. Navigate to the Actions tab
3. Select "Feature Combinations Test" from the workflows list
4. Click "Run workflow"

### Testing Locally

To test feature combinations locally:

```bash
# Test with default features
cargo test

# Test with minimal features
cargo test --no-default-features --features="std"

# Test with specific feature combinations
cargo test --no-default-features --features="wallet,crypto-standard"
cargo test --no-default-features --features="transaction,crypto-standard"
cargo test --no-default-features --features="http-client"

# Test with all features
cargo test --all-features
```

To measure compile times:

```bash
# Install cargo-timings tool
cargo install cargo-timings

# Measure compile time for a specific feature combination
cargo clean
cargo +stable timings build --no-default-features --features="wallet,crypto-standard"

# View the results
open target/cargo-timings/cargo-timing.html
```

## Adding New Features

When adding a new feature to the SDK, follow these steps:

1. **Update Cargo.toml**:
   - Add the feature to the appropriate feature group
   - Make sure dependencies are properly gated behind the feature

   ```toml
   [features]
   # Add your feature to the appropriate group
   my-new-feature = ["dependency1", "dependency2"]
   
   [dependencies]
   # Make sure dependencies are optional
   dependency1 = { version = "1.0", optional = true }
   dependency2 = { version = "2.0", optional = true }
   ```

2. **Update module code**:
   - Use conditional compilation with feature gates

   ```rust
   // Core functionality - always available
   pub use core::*;
   
   // Feature-specific functionality
   #[cfg(feature = "my-new-feature")]
   #[cfg_attr(docsrs, doc(cfg(feature = "my-new-feature")))]
   pub use my_new_module::*;
   
   // Core module - always available
   mod core;
   
   // Feature-specific module
   #[cfg(feature = "my-new-feature")]
   mod my_new_module;
   ```

3. **Update documentation**:
   - Add feature documentation to README.md
   - Document the feature in module-level comments
   - Create examples demonstrating the feature

4. **Add tests**:
   - Add tests for the new feature
   - Ensure tests are gated behind the feature flag
   
   ```rust
   #[cfg(feature = "my-new-feature")]
   #[test]
   fn test_my_new_feature() {
       // Test code
   }
   ```

5. **Update CI configuration**:
   - Add the new feature to the feature matrix in `.github/workflows/feature-combinations.yml`
   - Create combinations with other relevant features

## Maintaining Feature Compatibility

As the SDK evolves, it's important to maintain feature compatibility:

1. **Avoid implicit dependencies**: Don't assume a feature is enabled in code that doesn't explicitly require it.

2. **Test all combinations**: Regularly test different feature combinations to ensure they continue to work.

3. **Document breaking changes**: If a feature's behavior or requirements change, document this clearly.

4. **Use feature inheritance**: If a feature depends on another feature, make sure it's explicitly included:

   ```toml
   # Good: feature-b explicitly includes feature-a
   feature-b = ["feature-a", "some-dependency"]
   
   # Bad: feature-b implicitly depends on feature-a
   feature-b = ["some-dependency"]
   ```

5. **Be careful with optional dependencies**: Ensure optional dependencies are actually optional by checking that the code compiles without them.

## Common Pitfalls

1. **Missing feature gates**: Forgetting to gate code behind a feature flag, leading to compilation errors when the feature is disabled.

2. **Circular dependencies**: Creating circular dependencies between features.

3. **Undocumented dependencies**: Adding features that depend on other features without documenting or enforcing this relationship.

4. **Test coverage**: Not testing all feature combinations, leading to issues in specific configurations.

5. **Performance regressions**: Adding features that significantly impact compile times or binary sizes without documenting this impact.

## Feature Flag Checklist

Use this checklist when adding or modifying features:

- [ ] Feature is properly defined in Cargo.toml
- [ ] Dependencies are properly gated
- [ ] Code is properly gated with `#[cfg(feature = "...")]`
- [ ] Documentation is updated
- [ ] Tests are added and gated
- [ ] CI configuration is updated
- [ ] Feature combinations are tested
- [ ] Performance impact is measured and documented 