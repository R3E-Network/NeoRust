#!/bin/bash

# Script to build NeoRust with various feature combinations

set -e  # Exit on first error

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}NeoRust Build Script${NC}"
echo

# Help output
if [[ "$1" == "-h" || "$1" == "--help" ]]; then
  echo "Usage: ./scripts/build.sh [options]"
  echo
  echo "Options:"
  echo "  --features    - Comma-separated list of features to enable"
  echo "                  Available features:"
  echo "  futures     - Enables async/futures support"
  echo "  ledger      - Enables Ledger hardware wallet support"
  echo "  aws         - Enables AWS KMS support"
  echo "  --release     - Build in release mode"
  echo "  -h, --help    - Show this help message"
  echo
  echo "Examples:"
  echo "  ./scripts/build.sh --features futures,ledger,aws"
  echo "  ./scripts/build.sh --release"
  exit 0
fi

# Default features
FEATURES="futures,ledger,aws"
BUILD_MODE="debug"

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --features)
      FEATURES="$2"
      shift 2
      ;;
    --release)
      BUILD_MODE="release"
      shift
      ;;
    *)
      echo "Unknown option: $1"
      echo "Use --help to see available options"
      exit 1
      ;;
  esac
done

# Display build settings
echo -e "${YELLOW}Building NeoRust with features: ${GREEN}$FEATURES${NC}"
echo -e "${YELLOW}Build mode: ${GREEN}$BUILD_MODE${NC}"
echo

# Build command based on settings
if [ "$BUILD_MODE" = "release" ]; then
  cargo build --release --features "$FEATURES"
  echo -e "${GREEN}Release build completed successfully!${NC}"
else
  cargo build --features "$FEATURES"
  echo -e "${GREEN}Debug build completed successfully!${NC}"
fi 