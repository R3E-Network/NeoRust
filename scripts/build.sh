#!/bin/bash

# Help message
show_help() {
    echo "Usage: ./scripts/build.sh [OPTIONS]"
    echo ""
    echo "Build options:"
    echo "  --all-features         Build with all features enabled"
    echo "  --no-default-features  Build with no default features"
    echo "  --features FEATURES    Build with specific features (comma-separated)"
    echo "  --release              Build in release mode"
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
    echo "  ./scripts/build.sh --features futures,ledger"
    echo "  ./scripts/build.sh --features futures,ledger,aws,sgx"
    echo "  ./scripts/build.sh --all-features --release"
}

# Default build flags
BUILD_COMMAND="cargo build"
BUILD_FLAGS=""
RELEASE_MODE=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --all-features)
            BUILD_FLAGS="$BUILD_FLAGS --all-features"
            shift
            ;;
        --no-default-features)
            BUILD_FLAGS="$BUILD_FLAGS --no-default-features"
            shift
            ;;
        --features)
            BUILD_FLAGS="$BUILD_FLAGS --features $2"
            shift 2
            ;;
        --release)
            RELEASE_MODE="--release"
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

# Execute build command
FINAL_COMMAND="$BUILD_COMMAND $RELEASE_MODE $BUILD_FLAGS --verbose"
echo "Running: $FINAL_COMMAND"
$FINAL_COMMAND 