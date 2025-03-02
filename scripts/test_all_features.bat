@echo off
setlocal enabledelayedexpansion

:: ANSI color codes for Windows 10+
set "GREEN=[0;32m"
set "YELLOW=[1;33m"
set "RED=[0;31m"
set "CYAN=[0;36m"
set "NC=[0m"

:: Check if Windows version supports ANSI colors
ver | find "10." > nul
if %ERRORLEVEL% NEQ 0 (
    :: Clear color variables if not supported
    set "GREEN="
    set "YELLOW="
    set "RED="
    set "CYAN="
    set "NC="
)

:: Title
echo %CYAN%========================================================%NC%
echo %CYAN%        NeoRust Test Suite - All Feature Combinations%NC%
echo %CYAN%========================================================%NC%

:: Initialize counters
set TOTAL_COMBINATIONS=0
set SUCCESSFUL_COMBINATIONS=0
set FAILED_COMBINATIONS=0
set FAILED_FEATURE_SETS=

:: Default features only
echo.
echo %CYAN%Testing with default features%NC%
cargo test
if %ERRORLEVEL% EQU 0 (
    echo %GREEN%Tests passed with default features%NC%
    set /a SUCCESSFUL_COMBINATIONS+=1
) else (
    echo %RED%Tests failed with default features%NC%
    set FAILED_FEATURE_SETS=!FAILED_FEATURE_SETS! default
    set /a FAILED_COMBINATIONS+=1
)
set /a TOTAL_COMBINATIONS+=1

:: All features
echo.
echo %CYAN%Testing with ALL features%NC%
echo %YELLOW%Running tests with features: crypto-standard,std,transaction,wallet,ethereum-compat,ledger,sha2,ripemd160,digest,hmac,nightly%NC%
echo %YELLOW%Description: Complete feature set%NC%
echo %YELLOW%---------------------------------------%NC%
cargo test --features crypto-standard,std,transaction,wallet,ethereum-compat,ledger,sha2,ripemd160,digest,hmac,nightly
if %ERRORLEVEL% EQU 0 (
    echo %GREEN%Tests passed with all features%NC%
    set /a SUCCESSFUL_COMBINATIONS+=1
) else (
    echo %RED%Tests failed with all features%NC%
    set FAILED_FEATURE_SETS=!FAILED_FEATURE_SETS! all
    set /a FAILED_COMBINATIONS+=1
)
set /a TOTAL_COMBINATIONS+=1

:: Minimal build with just std
echo.
echo %CYAN%Testing minimal build with just std%NC%
echo %YELLOW%Running tests with no default features but with: std%NC%
echo %YELLOW%Description: Minimal build with standard library only%NC%
echo %YELLOW%---------------------------------------%NC%
cargo test --no-default-features --features std
if %ERRORLEVEL% EQU 0 (
    echo %GREEN%Tests passed with std only%NC%
    set /a SUCCESSFUL_COMBINATIONS+=1
) else (
    echo %RED%Tests failed with std only%NC%
    set FAILED_FEATURE_SETS=!FAILED_FEATURE_SETS! std-only
    set /a FAILED_COMBINATIONS+=1
)
set /a TOTAL_COMBINATIONS+=1

:: Common feature combinations
echo.
echo %CYAN%Testing common feature combinations%NC%

:: 1. Standard Application
echo.
echo %YELLOW%Running tests with features: crypto-standard,std,transaction%NC%
echo %YELLOW%Description: Standard application features%NC%
echo %YELLOW%---------------------------------------%NC%
cargo test --features crypto-standard,std,transaction
if %ERRORLEVEL% EQU 0 (
    echo %GREEN%Tests passed with standard app features%NC%
    set /a SUCCESSFUL_COMBINATIONS+=1
) else (
    echo %RED%Tests failed with standard app features%NC%
    set FAILED_FEATURE_SETS=!FAILED_FEATURE_SETS! standard-app
    set /a FAILED_COMBINATIONS+=1
)
set /a TOTAL_COMBINATIONS+=1

:: 2. Wallet Application
echo.
echo %YELLOW%Running tests with features: crypto-standard,std,wallet,transaction%NC%
echo %YELLOW%Description: Wallet application features%NC%
echo %YELLOW%---------------------------------------%NC%
cargo test --features crypto-standard,std,wallet,transaction
if %ERRORLEVEL% EQU 0 (
    echo %GREEN%Tests passed with wallet app features%NC%
    set /a SUCCESSFUL_COMBINATIONS+=1
) else (
    echo %RED%Tests failed with wallet app features%NC%
    set FAILED_FEATURE_SETS=!FAILED_FEATURE_SETS! wallet-app
    set /a FAILED_COMBINATIONS+=1
)
set /a TOTAL_COMBINATIONS+=1

:: 3. Hardware Wallet Integration
echo.
echo %YELLOW%Running tests with features: crypto-standard,std,ledger,wallet%NC%
echo %YELLOW%Description: Hardware wallet features%NC%
echo %YELLOW%---------------------------------------%NC%
cargo test --features crypto-standard,std,ledger,wallet
if %ERRORLEVEL% EQU 0 (
    echo %GREEN%Tests passed with hardware wallet features%NC%
    set /a SUCCESSFUL_COMBINATIONS+=1
) else (
    echo %RED%Tests failed with hardware wallet features%NC%
    set FAILED_FEATURE_SETS=!FAILED_FEATURE_SETS! hardware-wallet
    set /a FAILED_COMBINATIONS+=1
)
set /a TOTAL_COMBINATIONS+=1

:: 4. Neo X / EVM compatibility
echo.
echo %YELLOW%Running tests with features: crypto-standard,std,ethereum-compat%NC%
echo %YELLOW%Description: Neo X / EVM compatibility features%NC%
echo %YELLOW%---------------------------------------%NC%
cargo test --features crypto-standard,std,ethereum-compat
if %ERRORLEVEL% EQU 0 (
    echo %GREEN%Tests passed with Neo X compatibility features%NC%
    set /a SUCCESSFUL_COMBINATIONS+=1
) else (
    echo %RED%Tests failed with Neo X compatibility features%NC%
    set FAILED_FEATURE_SETS=!FAILED_FEATURE_SETS! neox-compat
    set /a FAILED_COMBINATIONS+=1
)
set /a TOTAL_COMBINATIONS+=1

:: Custom crypto combinations
echo.
echo %CYAN%Testing custom crypto feature combinations%NC%

:: 1. Custom crypto with specific algorithms
echo.
echo %YELLOW%Running tests with features: std,sha2,ripemd160%NC%
echo %YELLOW%Description: Custom crypto with SHA2 and RIPEMD160%NC%
echo %YELLOW%---------------------------------------%NC%
cargo test --features std,sha2,ripemd160
if %ERRORLEVEL% EQU 0 (
    echo %GREEN%Tests passed with custom crypto features%NC%
    set /a SUCCESSFUL_COMBINATIONS+=1
) else (
    echo %RED%Tests failed with custom crypto features%NC%
    set FAILED_FEATURE_SETS=!FAILED_FEATURE_SETS! custom-crypto
    set /a FAILED_COMBINATIONS+=1
)
set /a TOTAL_COMBINATIONS+=1

:: Summary
echo.
echo %CYAN%========================================================%NC%
echo %CYAN%                  Test Summary%NC%
echo %CYAN%========================================================%NC%
echo Total combinations tested: %TOTAL_COMBINATIONS%
echo %GREEN%Successful combinations: %SUCCESSFUL_COMBINATIONS%%NC%

if %FAILED_COMBINATIONS% GTR 0 (
    echo %RED%Failed combinations: %FAILED_COMBINATIONS%%NC%
    echo %RED%Failed feature sets: %FAILED_FEATURE_SETS%%NC%
    exit /b 1
) else (
    echo %GREEN%All feature combinations passed!%NC%
    exit /b 0
) 