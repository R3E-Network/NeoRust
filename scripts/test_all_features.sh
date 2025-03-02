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
echo -e "${CYAN}        NeoRust Test Suite - All Feature Combinations${NC}"
echo -e "${CYAN}========================================================${NC}"

# Core features to test in various combinations
CORE_FEATURES=("crypto-standard" "std" "transaction" "wallet" "ethereum-compat" "ledger")
CRYPTO_FEATURES=("sha2" "ripemd160" "digest" "hmac")
OTHER_FEATURES=("nightly")

# Function to run tests with specified features
run_test_with_features() {
    local features=$1
    local desc=$2
    echo -e "\n${YELLOW}Running tests with features: ${features}${NC}"
    echo -e "${YELLOW}Description: ${desc}${NC}"
    echo -e "${YELLOW}---------------------------------------${NC}"
    
    # Use timeout to prevent tests from hanging indefinitely
    # The timeout command isn't available on all systems, so check first
    if command -v timeout &> /dev/null; then
        timeout 300 cargo test --features "${features}" || {
            echo -e "${RED}Tests failed or timed out for features: ${features}${NC}"
            return 1
        }
    else
        cargo test --features "${features}" || {
            echo -e "${RED}Tests failed for features: ${features}${NC}"
            return 1
        }
    fi
    
    echo -e "${GREEN}Tests passed for features: ${features}${NC}"
    return 0
}

# Function to run tests with no default features
run_test_without_default_features() {
    local features=$1
    local desc=$2
    echo -e "\n${YELLOW}Running tests with no default features but with: ${features}${NC}"
    echo -e "${YELLOW}Description: ${desc}${NC}"
    echo -e "${YELLOW}---------------------------------------${NC}"
    
    if command -v timeout &> /dev/null; then
        timeout 300 cargo test --no-default-features --features "${features}" || {
            echo -e "${RED}Tests failed or timed out for features: ${features}${NC}"
            return 1
        }
    else
        cargo test --no-default-features --features "${features}" || {
            echo -e "${RED}Tests failed for features: ${features}${NC}"
            return 1
        }
    fi
    
    echo -e "${GREEN}Tests passed for no-default-features with: ${features}${NC}"
    return 0
}

# Track success/failure
TOTAL_COMBINATIONS=0
SUCCESSFUL_COMBINATIONS=0
FAILED_COMBINATIONS=0
FAILED_FEATURE_SETS=()

# Run tests with most critical feature combinations

# Default features only
echo -e "\n${CYAN}Testing with default features${NC}"
if cargo test; then
    echo -e "${GREEN}Tests passed with default features${NC}"
    ((SUCCESSFUL_COMBINATIONS++))
else
    echo -e "${RED}Tests failed with default features${NC}"
    FAILED_FEATURE_SETS+=("default")
    ((FAILED_COMBINATIONS++))
fi
((TOTAL_COMBINATIONS++))

# All features
echo -e "\n${CYAN}Testing with ALL features${NC}"
all_features=$(IFS=,; echo "${CORE_FEATURES[*]},${CRYPTO_FEATURES[*]},${OTHER_FEATURES[*]}")
if run_test_with_features "$all_features" "Complete feature set"; then
    ((SUCCESSFUL_COMBINATIONS++))
else
    FAILED_FEATURE_SETS+=("all")
    ((FAILED_COMBINATIONS++))
fi
((TOTAL_COMBINATIONS++))

# Minimal build with just std
echo -e "\n${CYAN}Testing minimal build with just std${NC}"
if run_test_without_default_features "std" "Minimal build with standard library only"; then
    ((SUCCESSFUL_COMBINATIONS++))
else
    FAILED_FEATURE_SETS+=("std-only")
    ((FAILED_COMBINATIONS++))
fi
((TOTAL_COMBINATIONS++))

# Common feature combinations
echo -e "\n${CYAN}Testing common feature combinations${NC}"

# 1. Standard Application: Default + Some extras
if run_test_with_features "crypto-standard,std,transaction" "Standard application features"; then
    ((SUCCESSFUL_COMBINATIONS++))
else
    FAILED_FEATURE_SETS+=("standard-app")
    ((FAILED_COMBINATIONS++))
fi
((TOTAL_COMBINATIONS++))

# 2. Wallet Application
if run_test_with_features "crypto-standard,std,wallet,transaction" "Wallet application features"; then
    ((SUCCESSFUL_COMBINATIONS++))
else
    FAILED_FEATURE_SETS+=("wallet-app")
    ((FAILED_COMBINATIONS++))
fi
((TOTAL_COMBINATIONS++))

# 3. Hardware Wallet Integration
if run_test_with_features "crypto-standard,std,ledger,wallet" "Hardware wallet features"; then
    ((SUCCESSFUL_COMBINATIONS++))
else
    FAILED_FEATURE_SETS+=("hardware-wallet")
    ((FAILED_COMBINATIONS++))
fi
((TOTAL_COMBINATIONS++))

# 4. Neo X / EVM compatibility
if run_test_with_features "crypto-standard,std,ethereum-compat" "Neo X / EVM compatibility features"; then
    ((SUCCESSFUL_COMBINATIONS++))
else
    FAILED_FEATURE_SETS+=("neox-compat")
    ((FAILED_COMBINATIONS++))
fi
((TOTAL_COMBINATIONS++))

# Custom crypto combinations
echo -e "\n${CYAN}Testing custom crypto feature combinations${NC}"

# 1. Custom crypto with specific algorithms
if run_test_with_features "std,sha2,ripemd160" "Custom crypto with SHA2 and RIPEMD160"; then
    ((SUCCESSFUL_COMBINATIONS++))
else
    FAILED_FEATURE_SETS+=("custom-crypto")
    ((FAILED_COMBINATIONS++))
fi
((TOTAL_COMBINATIONS++))

# Summary
echo -e "\n${CYAN}========================================================${NC}"
echo -e "${CYAN}                  Test Summary${NC}"
echo -e "${CYAN}========================================================${NC}"
echo -e "Total combinations tested: ${TOTAL_COMBINATIONS}"
echo -e "${GREEN}Successful combinations: ${SUCCESSFUL_COMBINATIONS}${NC}"

if [ $FAILED_COMBINATIONS -gt 0 ]; then
    echo -e "${RED}Failed combinations: ${FAILED_COMBINATIONS}${NC}"
    echo -e "${RED}Failed feature sets: ${FAILED_FEATURE_SETS[*]}${NC}"
    exit 1
else
    echo -e "${GREEN}All feature combinations passed!${NC}"
    exit 0
fi 