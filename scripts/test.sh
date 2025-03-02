#!/bin/bash
set -e

# ANSI color codes
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Parse arguments
ALL_FEATURES=false
ALL_EXAMPLES=false
CARGO_ARGS=""

print_usage() {
    echo "NeoRust Test Script"
    echo ""
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  --all-features      Run tests with all feature combinations"
    echo "  --all-examples      Run all examples"
    echo "  --help              Show this help message"
    echo ""
    echo "Any other arguments will be passed directly to cargo test."
}

while [ $# -gt 0 ]; do
    case "$1" in
        --all-features)
            ALL_FEATURES=true
            shift
            ;;
        --all-examples)
            ALL_EXAMPLES=true
            shift
            ;;
        --help)
            print_usage
            exit 0
            ;;
        *)
            CARGO_ARGS="$CARGO_ARGS $1"
            shift
            ;;
    esac
done

# Run the appropriate scripts based on arguments
if [ "$ALL_FEATURES" = true ]; then
    echo -e "${CYAN}Running tests with all feature combinations...${NC}"
    ./scripts/test_all_features.sh
elif [ "$ALL_EXAMPLES" = true ]; then
    echo -e "${CYAN}Running all examples...${NC}"
    ./scripts/run_all_examples.sh
else
    # Run standard cargo test with any additional args
    echo -e "${CYAN}Running standard tests...${NC}"
    cargo test $CARGO_ARGS 