#!/bin/bash

# Script to help migrate the SDK from src/ to crates/

# Function to migrate a module
migrate_module() {
    local module_name=$1
    local src_dir="src/${module_name}"
    local crate_dir="crates/neo-${module_name#neo_}"
    
    echo "Migrating ${module_name} to ${crate_dir}..."
    
    # Create the crate directory if it doesn't exist
    mkdir -p "${crate_dir}/src"
    
    # Copy the module files to the crate
    if [ -d "${src_dir}" ]; then
        cp -r "${src_dir}"/* "${crate_dir}/src/"
    elif [ -f "${src_dir}.rs" ]; then
        cp "${src_dir}.rs" "${crate_dir}/src/lib.rs"
    fi
    
    # Create a basic Cargo.toml if it doesn't exist
    if [ ! -f "${crate_dir}/Cargo.toml" ]; then
        cat > "${crate_dir}/Cargo.toml" << EOF
[package]
name = "neo-${module_name#neo_}"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
description = "Neo ${module_name#neo_} module for the NeoRust SDK"
documentation = "https://docs.rs/neo-${module_name#neo_}"
repository.workspace = true
homepage.workspace = true
readme = "README.md"
categories.workspace = true
keywords.workspace = true

[dependencies]
# Add dependencies as needed

[features]
default = []
EOF
    fi
    
    # Create a basic README.md if it doesn't exist
    if [ ! -f "${crate_dir}/README.md" ]; then
        cat > "${crate_dir}/README.md" << EOF
# Neo ${module_name#neo_}

${module_name#neo_} module for the NeoRust SDK.

## Description

This crate provides ${module_name#neo_} functionality for the Neo N3 blockchain.

## Usage

\`\`\`rust
use neo_${module_name#neo_}::*;

// Example code here
\`\`\`
EOF
    fi
}

# Main script
echo "Starting migration to crates-based structure..."

# Get all modules in src/
modules=$(find src -maxdepth 1 -type d -not -path "src" -o -name "*.rs" -not -name "lib.rs" -not -name "prelude.rs" | sed 's|^src/||' | sed 's|\.rs$||')

# Migrate each module
for module in $modules; do
    migrate_module "$module"
done

echo "Migration completed!" 