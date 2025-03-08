# PowerShell script to help migrate the SDK from src/ to crates/

# Function to migrate a module
function Migrate-Module {
    param (
        [string]$moduleName
    )
    
    $srcDir = "src\$moduleName"
    $crateName = $moduleName -replace "neo_", "neo-"
    $crateDir = "crates\$crateName"
    
    Write-Host "Migrating $moduleName to $crateDir..."
    
    # Create the crate directory if it doesn't exist
    if (-not (Test-Path $crateDir)) {
        New-Item -Path $crateDir -ItemType Directory -Force | Out-Null
    }
    
    if (-not (Test-Path "$crateDir\src")) {
        New-Item -Path "$crateDir\src" -ItemType Directory -Force | Out-Null
    }
    
    # Copy the module files to the crate
    if (Test-Path $srcDir -PathType Container) {
        Copy-Item -Path "$srcDir\*" -Destination "$crateDir\src\" -Recurse -Force
    } elseif (Test-Path "$srcDir.rs" -PathType Leaf) {
        Copy-Item -Path "$srcDir.rs" -Destination "$crateDir\src\lib.rs" -Force
    }
    
    # Create a basic Cargo.toml if it doesn't exist
    if (-not (Test-Path "$crateDir\Cargo.toml")) {
        $cargoContent = @"
[package]
name = "$crateName"
version.workspace = true
edition.workspace = true
authors.workspace = true
license.workspace = true
description = "Neo $($moduleName -replace 'neo_', '') module for the NeoRust SDK"
documentation = "https://docs.rs/$crateName"
repository.workspace = true
homepage.workspace = true
readme = "README.md"
categories.workspace = true
keywords.workspace = true

[dependencies]
# Add dependencies as needed

[features]
default = []
"@
        Set-Content -Path "$crateDir\Cargo.toml" -Value $cargoContent
    }
    
    # Create a basic README.md if it doesn't exist
    if (-not (Test-Path "$crateDir\README.md")) {
        $readmeContent = @"
# Neo $($moduleName -replace 'neo_', '')

$($moduleName -replace 'neo_', '') module for the NeoRust SDK.

## Description

This crate provides $($moduleName -replace 'neo_', '') functionality for the Neo N3 blockchain.

## Usage

```rust
use $($crateName)::*;

// Example code here
```
"@
        Set-Content -Path "$crateDir\README.md" -Value $readmeContent
    }
}

# Main script
Write-Host "Starting migration to crates-based structure..."

# Get all modules in src/
$modules = @()
Get-ChildItem -Path "src" -Directory | ForEach-Object {
    if ($_.Name -ne "src") {
        $modules += $_.Name
    }
}

Get-ChildItem -Path "src" -File | Where-Object { $_.Name -ne "lib.rs" -and $_.Name -ne "prelude.rs" } | ForEach-Object {
    $modules += $_.BaseName
}

# Migrate each module
foreach ($module in $modules) {
    Migrate-Module -moduleName $module
}

Write-Host "Migration completed!" 