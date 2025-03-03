#!/bin/bash

# Script to test all examples in the NeoRust repository
# This script will attempt to build all examples to ensure they compile

set -e  # Exit on error

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Testing all NeoRust examples...${NC}"
echo

# Get the root directory of the project
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EXAMPLES_DIR="$ROOT_DIR/examples"

# Define feature combinations to test
FEATURE_SETS=(
    ""  # No features
    "futures"
)

# Function to test an example directory with specific features
test_example_dir() {
    local dir=$1
    local features=$2
    local dir_name=$(basename "$dir")
    
    local feature_text=""
    if [ -n "$features" ]; then
        feature_text=" with features: $features"
    else
        feature_text=" with no features"
    fi
    
    echo -e "${YELLOW}Testing $dir_name examples$feature_text...${NC}"
    
    # Check if the directory has a Cargo.toml file
    if [ ! -f "$dir/Cargo.toml" ]; then
        echo -e "${RED}Error: $dir_name does not have a Cargo.toml file${NC}"
        return 1
    fi
    
    # Navigate to the example directory
    cd "$dir"
    
    # Try to build the examples with the specified features
    if [ -n "$features" ]; then
        if cargo build --quiet --no-default-features --features "$features"; then
            echo -e "${GREEN}✓ $dir_name examples built successfully$feature_text${NC}"
            return 0
        else
            echo -e "${RED}✗ Failed to build $dir_name examples$feature_text${NC}"
            return 1
        fi
    else
        # Build with no features
        if cargo build --quiet --no-default-features; then
            echo -e "${GREEN}✓ $dir_name examples built successfully$feature_text${NC}"
            return 0
        else
            echo -e "${RED}✗ Failed to build $dir_name examples$feature_text${NC}"
            return 1
        fi
    fi
}

# Find all example directories
find "$EXAMPLES_DIR" -maxdepth 1 -mindepth 1 -type d | while read -r dir; do
    # Skip directories that don't have a Cargo.toml file
    if [ ! -f "$dir/Cargo.toml" ]; then
        echo -e "${YELLOW}Skipping $(basename "$dir") - no Cargo.toml file${NC}"
        continue
    fi
    
    # Test the example directory with different feature combinations
    for features in "${FEATURE_SETS[@]}"; do
        if ! test_example_dir "$dir" "$features"; then
            echo -e "${RED}Failed to build examples in $(basename "$dir") with features: $features${NC}"
        fi
        echo
    done
    
    echo -e "${GREEN}Testing completed for $(basename "$dir")!${NC}"
    echo -e "${YELLOW}---------------------------------------${NC}"
    echo
done

echo -e "${GREEN}All example testing completed!${NC}" 