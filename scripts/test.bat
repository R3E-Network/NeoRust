@echo off
setlocal enabledelayedexpansion

:: Default test flags
set TEST_COMMAND=cargo test
set TEST_FLAGS=
set RUNTIME_FLAGS=
set FEATURES=futures,ledger,aws

:: Parse arguments
:parse_args
if "%~1"=="" goto prepare_command
if "%~1"=="--all-features" (
    set TEST_FLAGS=!TEST_FLAGS! --all-features
    set FEATURES=
    shift
    goto parse_args
)
if "%~1"=="--no-default-features" (
    set TEST_FLAGS=!TEST_FLAGS! --no-default-features
    set FEATURES=
    shift
    goto parse_args
)
if "%~1"=="--features" (
    set FEATURES=%~2
    shift
    shift
    goto parse_args
)
if "%~1"=="--nocapture" (
    set RUNTIME_FLAGS=!RUNTIME_FLAGS! --nocapture
    shift
    goto parse_args
)
if "%~1"=="--no-fail-fast" (
    set TEST_FLAGS=!TEST_FLAGS! --no-fail-fast
    shift
    goto parse_args
)
if "%~1"=="--help" (
    call :show_help
    exit /b 0
)
echo Unknown option: %~1
call :show_help
exit /b 1

:prepare_command
:: Add features flag if features were specified or using default
if not "!FEATURES!"=="" (
    set TEST_FLAGS=!TEST_FLAGS! --features !FEATURES!
)

:: Execute test command
set FINAL_COMMAND=%TEST_COMMAND% %TEST_FLAGS%
if not "!RUNTIME_FLAGS!"=="" (
    set FINAL_COMMAND=!FINAL_COMMAND! -- !RUNTIME_FLAGS!
)

echo Running: !FINAL_COMMAND!
!FINAL_COMMAND!
exit /b

:show_help
echo Usage: .\scripts\test.bat [OPTIONS]
echo.
echo Test options:
echo   --all-features         Test with all features enabled
echo   --no-default-features  Test with no default features
echo   --features FEATURES    Test with specific features (comma-separated)
echo                          Default features if not specified: futures,ledger,aws
echo   --nocapture            Show test output
echo   --no-fail-fast         Continue testing even if a test fails
echo   --help                 Show this help message
echo.
echo Available features:
echo   futures    - Enables async/futures support
echo   ledger     - Enables hardware wallet support via Ledger devices
echo   aws        - Enables AWS integration
echo   sgx        - Enables Intel SGX secure enclave support (not included by default)
echo   sgx_deps   - Enables additional SGX dependencies (implies sgx)
echo.
echo Examples:
echo   .\scripts\test.bat --features futures,ledger
echo   .\scripts\test.bat --features futures,ledger,aws
echo   .\scripts\test.bat --all-features --nocapture
exit /b 