@echo off
setlocal enabledelayedexpansion

:: Default build flags
set BUILD_COMMAND=cargo build
set BUILD_FLAGS=
set RELEASE_MODE=

:: Parse arguments
:parse_args
if "%~1"=="" goto execute_command
if "%~1"=="--all-features" (
    set BUILD_FLAGS=!BUILD_FLAGS! --all-features
    shift
    goto parse_args
)
if "%~1"=="--no-default-features" (
    set BUILD_FLAGS=!BUILD_FLAGS! --no-default-features
    shift
    goto parse_args
)
if "%~1"=="--features" (
    set BUILD_FLAGS=!BUILD_FLAGS! --features %~2
    shift
    shift
    goto parse_args
)
if "%~1"=="--release" (
    set RELEASE_MODE=--release
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

:execute_command
:: Execute build command
set FINAL_COMMAND=%BUILD_COMMAND% %RELEASE_MODE% %BUILD_FLAGS% --verbose
echo Running: %FINAL_COMMAND%
%FINAL_COMMAND%
exit /b

:show_help
echo Usage: .\scripts\build.bat [OPTIONS]
echo.
echo Build options:
echo   --all-features         Build with all features enabled
echo   --no-default-features  Build with no default features
echo   --features FEATURES    Build with specific features (comma-separated)
echo   --release              Build in release mode
echo   --help                 Show this help message
echo.
echo Examples:
echo   .\scripts\build.bat --features ledger,aws,futures
echo   .\scripts\build.bat --all-features --release
exit /b 