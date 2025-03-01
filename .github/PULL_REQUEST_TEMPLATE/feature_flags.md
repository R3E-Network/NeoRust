# Feature Flag System - Pull Request

## Description

<!-- Describe the changes you've made to the feature flag system -->

## Type of Change

- [ ] New feature flag addition
- [ ] Modification to existing feature flags
- [ ] Reorganization of feature flags
- [ ] Documentation update for feature flags
- [ ] Bug fix for feature flags
- [ ] Other (please describe)

## Feature Flag Checklist

- [ ] All new functionality is properly gated behind appropriate feature flags
- [ ] Feature documentation has been updated in module-level comments
- [ ] README.md feature section has been updated (if applicable)
- [ ] Dependencies are properly organized in Cargo.toml
- [ ] No unnecessary dependencies are pulled in without feature gates
- [ ] Feature flags maintain backward compatibility (or breaking changes are documented)
- [ ] Examples demonstrate proper feature flag usage
- [ ] Tests run correctly with different feature combinations

## Module Updates

List all modules that were updated with feature flags:

- [ ] lib.rs
- [ ] neo_crypto
- [ ] neo_contract
- [ ] neo_clients
- [ ] neo_wallets
- [ ] neo_protocol
- [ ] neo_builder
- [ ] neo_types
- [ ] neo_codec
- [ ] neo_config
- [ ] neo_utils
- [ ] neo_x
- [ ] neo_sgx
- [ ] Other: <!-- Please specify -->

## Feature Combinations Tested

List the feature combinations you tested:

- [ ] Default features
- [ ] Minimal build (specify features: <!-- Please list features -->)
- [ ] Custom combination (specify features: <!-- Please list features -->)
- [ ] All features

## Performance Impact

If applicable, provide information about the performance impact:

- Compile time change: <!-- Please specify -->
- Binary size change: <!-- Please specify -->
- Other performance metrics: <!-- Please specify -->

## Additional Information

<!-- Any additional information about your changes --> 