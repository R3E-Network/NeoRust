#!/bin/bash

# Help message
show_help() {
    echo "Usage: ./scripts/test.sh [OPTIONS]"
    echo ""
    echo "Test options:"
    echo "  --all-features         Test with all features enabled"
    echo "  --no-default-features  Test with no default features"
    echo "  --features FEATURES    Test with specific features (comma-separated)"
    echo "                         Default features if not specified: futures,ledger,aws,sgx"
    echo "  --nocapture            Show test output"
    echo "  --no-fail-fast         Continue testing even if a test fails"
    echo "  --help                 Show this help message"
    echo ""
    echo "Available features:"
    echo "  futures    - Enables async/futures support"
    echo "  ledger     - Enables hardware wallet support via Ledger devices"
    echo "  aws        - Enables AWS integration"
    echo "  sgx        - Enables Intel SGX secure enclave support"
    echo "  sgx_deps   - Enables additional SGX dependencies (implies sgx)"
    echo ""
    echo "Examples:"
    echo "  ./scripts/test.sh --features futures,ledger"
    echo "  ./scripts/test.sh --features futures,ledger,aws,sgx"
    echo "  ./scripts/test.sh --all-features --nocapture"
}

# Default test flags
TEST_COMMAND="cargo test"
TEST_FLAGS=""
RUNTIME_FLAGS=""
FEATURES="futures,ledger,aws,sgx"  # Default features

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --all-features)
            TEST_FLAGS="$TEST_FLAGS --all-features"
            FEATURES=""  # Clear default features when using --all-features
            shift
            ;;
        --no-default-features)
            TEST_FLAGS="$TEST_FLAGS --no-default-features"
            FEATURES=""  # Clear default features when using --no-default-features
            shift
            ;;
        --features)
            FEATURES="$2"
            shift 2
            ;;
        --nocapture)
            RUNTIME_FLAGS="$RUNTIME_FLAGS --nocapture"
            shift
            ;;
        --no-fail-fast)
            TEST_FLAGS="$TEST_FLAGS --no-fail-fast"
            shift
            ;;
        --help)
            show_help
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Add features flag if features were specified or using default
if [ -n "$FEATURES" ]; then
    TEST_FLAGS="$TEST_FLAGS --features $FEATURES"
fi

# Execute test command
FINAL_COMMAND="$TEST_COMMAND $TEST_FLAGS"
if [ -n "$RUNTIME_FLAGS" ]; then
    FINAL_COMMAND="$FINAL_COMMAND -- $RUNTIME_FLAGS"
fi

echo "Running: $FINAL_COMMAND"
$FINAL_COMMAND 