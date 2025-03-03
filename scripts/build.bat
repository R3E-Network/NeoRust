@echo off
setlocal enabledelayedexpansion

:: Colors for output (Windows)
set "GREEN=[32m"
set "RED=[31m"
set "YELLOW=[33m"
set "NC=[0m"

echo %YELLOW%NeoRust Build Script%NC%
echo.

:: Help output
if "%1"=="-h" goto :help
if "%1"=="--help" goto :help
goto :main

:help
echo Usage: .\scripts\build.bat [options]
echo.
echo Options:
echo   --features    - Comma-separated list of features to enable
echo                   Available features:
echo   futures     - Enables async/futures support
echo   ledger      - Enables Ledger hardware wallet support
echo   aws         - Enables AWS KMS support
echo   --release     - Build in release mode
echo   -h, --help    - Show this help message
echo.
echo Examples:
echo   .\scripts\build.bat --features futures,ledger,aws
echo   .\scripts\build.bat --release
exit /b 0

:main
:: Default features
set "FEATURES=futures,ledger,aws"
set "BUILD_MODE=debug"

:: Parse arguments
:parse_args
if "%1"=="" goto :build
if "%1"=="--features" (
    set "FEATURES=%2"
    shift
    shift
    goto :parse_args
)
if "%1"=="--release" (
    set "BUILD_MODE=release"
    shift
    goto :parse_args
)
echo Unknown option: %1
echo Use --help to see available options
exit /b 1

:build
:: Display build settings
echo %YELLOW%Building NeoRust with features: %GREEN%%FEATURES%%NC%
echo %YELLOW%Build mode: %GREEN%%BUILD_MODE%%NC%
echo.

:: Build command based on settings
if "%BUILD_MODE%"=="release" (
    cargo build --release --features "%FEATURES%"
    echo %GREEN%Release build completed successfully!%NC%
) else (
    cargo build --features "%FEATURES%"
    echo %GREEN%Debug build completed successfully!%NC%
)

exit /b 0 