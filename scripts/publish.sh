#!/bin/bash
set -e

# Script to publish the neo3 crate to crates.io
# Usage: ./scripts/publish.sh [version]

# Get the current version from Cargo.toml
CURRENT_VERSION=$(grep '^version =' Cargo.toml | head -n 1 | cut -d '"' -f 2)

# If a version is provided as an argument, use it
if [ $# -eq 1 ]; then
    NEW_VERSION=$1
else
    echo "Current version: $CURRENT_VERSION"
    read -p "Enter new version (leave empty to use current version): " NEW_VERSION
    if [ -z "$NEW_VERSION" ]; then
        NEW_VERSION=$CURRENT_VERSION
    fi
fi

echo "Publishing version $NEW_VERSION to crates.io"

# Update version in Cargo.toml if needed
if [ "$NEW_VERSION" != "$CURRENT_VERSION" ]; then
    echo "Updating version in Cargo.toml from $CURRENT_VERSION to $NEW_VERSION"
    sed -i "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
fi

# Build the documentation
echo "Building documentation..."
cd docs
mdbook build
cd ..

# Run tests
echo "Running tests..."
cargo test --all-features

# Verify the package
echo "Verifying package..."
cargo package --allow-dirty

# Publish to crates.io
echo "Publishing to crates.io..."
cargo publish --allow-dirty

echo "Successfully published neo3 version $NEW_VERSION to crates.io!"
