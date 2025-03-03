#!/bin/bash

# Script to update all example Cargo.toml files to remove SGX features

set -e  # Exit on error

echo "Updating example Cargo.toml files to remove SGX features..."

# Find all example Cargo.toml files
EXAMPLE_FILES=$(find examples -name "Cargo.toml")

for file in $EXAMPLE_FILES; do
  echo "Updating $file..."
  
  # Remove the sgx and sgx_deps feature lines
  sed -i'.bak' '/sgx = \["neo3\/sgx"\]/d' "$file"
  sed -i'.bak' '/sgx_deps = \["neo3\/sgx_deps", "sgx"\]/d' "$file"
  
  # Remove backup files
  rm -f "${file}.bak"
done

echo "All example Cargo.toml files updated successfully!" 