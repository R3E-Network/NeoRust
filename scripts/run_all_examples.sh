#!/bin/bash
set -e

# ANSI color codes
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Title
echo -e "${CYAN}========================================================${NC}"
echo -e "${CYAN}            NeoRust - Run All Examples${NC}"
echo -e "${CYAN}========================================================${NC}"

# Get all examples from cargo metadata
echo -e "${YELLOW}Discovering examples...${NC}"
EXAMPLES=$(cargo metadata --no-deps --format-version=1 | 
           jq -r '.packages[0].targets[] | select(.kind[] == "example") | .name')

if [ -z "$EXAMPLES" ]; then
    # If jq command failed or no examples found, try listing files directly
    echo -e "${YELLOW}Using direct directory lookup...${NC}"
    if [ -d "examples" ]; then
        EXAMPLES=$(find examples -name "*.rs" -not -path "*/\.*" | 
                  sed -E 's|^examples/||g' | 
                  sed -E 's|/|_|g' | 
                  sed -E 's|\.rs$||g')
    fi
fi

if [ -z "$EXAMPLES" ]; then
    echo -e "${RED}No examples found!${NC}"
    exit 1
fi

# Counter variables
TOTAL_EXAMPLES=0
SUCCESSFUL_EXAMPLES=0
FAILED_EXAMPLES=()

# Run each example
for example in $EXAMPLES; do
    echo -e "\n${CYAN}Running example: ${example}${NC}"
    echo -e "${YELLOW}----------------------------------------${NC}"
    
    # Use timeout to prevent hanging examples
    if command -v timeout &> /dev/null; then
        # Build example with all features (to ensure it compiles)
        timeout 300 cargo build --example "$example" --all-features || {
            echo -e "${RED}Failed to build example: ${example}${NC}"
            FAILED_EXAMPLES+=("$example")
            ((TOTAL_EXAMPLES++))
            continue
        }
        
        # Run with all features (with a timeout)
        if timeout 60 cargo run --example "$example" --all-features; then
            echo -e "${GREEN}Example ${example} ran successfully${NC}"
            ((SUCCESSFUL_EXAMPLES++))
        else
            echo -e "${RED}Example ${example} failed${NC}"
            FAILED_EXAMPLES+=("$example")
        fi
    else
        # Build example with all features (to ensure it compiles)
        if cargo build --example "$example" --all-features; then
            # Run with all features
            if cargo run --example "$example" --all-features; then
                echo -e "${GREEN}Example ${example} ran successfully${NC}"
                ((SUCCESSFUL_EXAMPLES++))
            else
                echo -e "${RED}Example ${example} failed${NC}"
                FAILED_EXAMPLES+=("$example")
            fi
        else
            echo -e "${RED}Failed to build example: ${example}${NC}"
            FAILED_EXAMPLES+=("$example")
        fi
    fi
    
    ((TOTAL_EXAMPLES++))
done

# Summary
echo -e "\n${CYAN}========================================================${NC}"
echo -e "${CYAN}                 Examples Summary${NC}"
echo -e "${CYAN}========================================================${NC}"
echo -e "Total examples: ${TOTAL_EXAMPLES}"
echo -e "${GREEN}Successful examples: ${SUCCESSFUL_EXAMPLES}${NC}"

if [ ${#FAILED_EXAMPLES[@]} -gt 0 ]; then
    echo -e "${RED}Failed examples: ${#FAILED_EXAMPLES[@]}${NC}"
    echo -e "${RED}Failed example names: ${FAILED_EXAMPLES[*]}${NC}"
    exit 1
else
    echo -e "${GREEN}All examples ran successfully!${NC}"
    exit 0
fi 