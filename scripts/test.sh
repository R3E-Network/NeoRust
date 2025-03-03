#!/bin/bash

# Script to run tests with various feature combinations
# By default, runs with the futures feature enabled

set -e  # Exit on first error

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}NeoRust Test Script${NC}"
echo

# Help output
if [[ "$1" == "-h" || "$1" == "--help" ]]; then
  echo "Usage: ./scripts/test.sh [options]"
  echo
  echo "Options:"
  echo "  --features    - Comma-separated list of features to enable"
  echo "                  Available features:"
  echo "  futures     - Enables async/futures support"
  echo "  ledger      - Enables Ledger hardware wallet support"
  echo "  aws         - Enables AWS KMS support"
  echo "  --nocapture   - Shows test output (passes through to cargo test)"
  echo "  --release     - Build in release mode"
  echo "  -h, --help    - Show this help message"
  echo
  exit 0
fi

# Default features (if none specified)
FEATURES="futures,ledger,aws"  # Default features

# Parse arguments
CARGO_ARGS=()
NOCAPTURE=""
RELEASE=""

while [[ $# -gt 0 ]]; do
  case $1 in
    --features)
      FEATURES="$2"
      shift 2
      ;;
    --nocapture)
      NOCAPTURE="--nocapture"
      shift
      ;;
    --release)
      RELEASE="--release"
      shift
      ;;
    *)
      CARGO_ARGS+=("$1")
      shift
      ;;
  esac
done

# Function to display features
display_features() {
  echo -e "${YELLOW}Running tests with features: ${GREEN}$FEATURES${NC}"
  if [[ -n "$RELEASE" ]]; then
    echo -e "${YELLOW}Build mode: ${GREEN}release${NC}"
  else
    echo -e "${YELLOW}Build mode: ${GREEN}debug${NC}"
  fi
  echo
}

# Run the tests
run_tests() {
  display_features
  
  if [[ -n "$NOCAPTURE" ]]; then
    echo -e "${YELLOW}Running tests with output displayed...${NC}"
    cargo test $RELEASE --features "$FEATURES" "${CARGO_ARGS[@]}" -- --nocapture
  else
    echo -e "${YELLOW}Running tests...${NC}"
    cargo test $RELEASE --features "$FEATURES" "${CARGO_ARGS[@]}"
  fi

  echo -e "${GREEN}Tests completed successfully!${NC}"
}

# Execute tests
run_tests 