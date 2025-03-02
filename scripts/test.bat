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

:: Parse arguments
set ALL_FEATURES=false
set ALL_EXAMPLES=false
set CARGO_ARGS=

:parse_args
if "%~1"=="" goto :run_tests
if "%~1"=="--all-features" (
    set ALL_FEATURES=true
    shift
    goto :parse_args
)
if "%~1"=="--all-examples" (
    set ALL_EXAMPLES=true
    shift
    goto :parse_args
)
if "%~1"=="--help" (
    call :print_usage
    exit /b 0
)
set CARGO_ARGS=%CARGO_ARGS% %1
shift
goto :parse_args

:print_usage
echo NeoRust Test Script
echo.
echo Usage: test.bat [options]
echo.
echo Options:
echo   --all-features      Run tests with all feature combinations
echo   --all-examples      Run all examples
echo   --help              Show this help message
echo.
echo Any other arguments will be passed directly to cargo test.
exit /b 0

:run_tests
:: Run the appropriate scripts based on arguments
if "%ALL_FEATURES%"=="true" (
    echo %CYAN%Running tests with all feature combinations...%NC%
    call scripts\test_all_features.bat
) else if "%ALL_EXAMPLES%"=="true" (
    echo %CYAN%Running all examples...%NC%
    call scripts\run_all_examples.bat
) else (
    :: Run standard cargo test with any additional args
    echo %CYAN%Running standard tests...%NC%
    cargo test %CARGO_ARGS%
)

exit /b %ERRORLEVEL% 