# NeoRust CLI Test Commands

This file contains example commands to test the functionality of the refactored NeoRust CLI after removing the famous contracts functionality and integrating it into the DeFi commands.

## Previous Famous Contract Commands (Now Removed)

These commands are no longer available:

```bash
# No longer available
neo-cli famous list --network mainnet
neo-cli famous show "NEO Token"
neo-cli famous invoke "NEO Token" symbol
neo-cli famous balance "NEO Token" --address NZKvXidwBhnV8rNXh2eXtpm5bH1rkofaDz
```

## New DeFi Commands (Integrated Famous Contract Functionality)

Use these commands instead:

```bash
# List famous contracts (now part of defi commands)
neo-cli defi list --network mainnet
neo-cli defi list --network testnet

# Show contract details
neo-cli defi show "NEO Token"
neo-cli defi show "GAS Token"

# Invoke contract methods
neo-cli defi invoke "NEO Token" symbol --test
neo-cli defi invoke "GAS Token" decimals --test

# Check token balances
neo-cli defi balance "NEO Token" --address NZKvXidwBhnV8rNXh2eXtpm5bH1rkofaDz
neo-cli defi balance "GAS Token" --address NZKvXidwBhnV8rNXh2eXtpm5bH1rkofaDz

# DeFi platform operations
neo-cli defi pools --platform flamingo
neo-cli defi swap-info --token-from NEO --token-to GAS --amount 1
```

## Testing Process

1. Ensure the famous commands have been completely removed:
   - Running `neo-cli famous list` should result in an "unknown command" error

2. Ensure the DeFi commands now include the famous contract functionality:
   - All the new DeFi commands listed above should work properly

3. Check for proper error handling:
   - `neo-cli defi show "NonExistentContract"` should provide a clear error message

## Integration Test Verification

To verify that the test suite has been properly updated:

```bash
# Run only the defi tests
cargo test --test integration_tests integration::defi_tests

# All tests should pass, including the ones testing famous contract functionality
```

## Documentation Verification

The following documentation files have been updated to reflect these changes:

1. `/neo-cli/docs/implementation_summary.md` - Updated to remove famous contract references
2. `/neo-cli/docs/defi.md` - New file replacing famous_contracts.md
3. `/neo-cli/README.md` - Updated to show integrated command structure
