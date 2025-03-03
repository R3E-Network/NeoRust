@echo off
setlocal enabledelayedexpansion

:: Colors for output (Windows)
set "GREEN=[32m"
set "RED=[31m"
set "YELLOW=[33m"
set "NC=[0m"

echo %YELLOW%NeoRust Test Script%NC%
echo.

:: Help output
if "%1"=="-h" goto :help
if "%1"=="--help" goto :help
goto :main

:help
echo Usage: .\scripts\test.bat [options]
echo.
echo Options:
echo   --features    - Comma-separated list of features to enable
echo                   Available features:
echo   futures     - Enables async/futures support
echo   ledger      - Enables Ledger hardware wallet support
echo   aws         - Enables AWS KMS support
echo   --nocapture   - Shows test output (passes through to cargo test)
echo   --release     - Build in release mode
echo   -h, --help    - Show this help message
echo.
echo Examples:
echo   .\scripts\test.bat --features futures,ledger,aws
echo   .\scripts\test.bat --nocapture
exit /b 0

:main
:: Default features
set "FEATURES=futures,ledger,aws"
set "NOCAPTURE="
set "RELEASE="

:: Parse arguments
:parse_args
if "%1"=="" goto :run_tests
if "%1"=="--features" (
    set "FEATURES=%2"
    shift
    shift
    goto :parse_args
)
if "%1"=="--nocapture" (
    set "NOCAPTURE=--nocapture"
    shift
    goto :parse_args
)
if "%1"=="--release" (
    set "RELEASE=--release"
    shift
    goto :parse_args
)
echo Unknown option: %1
echo Use --help to see available options
exit /b 1

:run_tests
:: Display features
echo %YELLOW%Running tests with features: %GREEN%%FEATURES%%NC%
if not "%RELEASE%"=="" (
    echo %YELLOW%Build mode: %GREEN%release%NC%
) else (
    echo %YELLOW%Build mode: %GREEN%debug%NC%
)
echo.

:: Run the tests
if not "%NOCAPTURE%"=="" (
    echo %YELLOW%Running tests with output displayed...%NC%
    cargo test %RELEASE% --features "%FEATURES%" -- --nocapture
) else (
    echo %YELLOW%Running tests...%NC%
    cargo test %RELEASE% --features "%FEATURES%"
)

echo %GREEN%Tests completed successfully!%NC%

exit /b 0 