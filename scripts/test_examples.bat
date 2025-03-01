@echo off
setlocal enabledelayedexpansion

:: Script to test all examples in the NeoRust repository
:: This script will attempt to build all examples to ensure they compile

echo Testing all NeoRust examples...
echo.

:: Get the root directory of the project
set "ROOT_DIR=%~dp0.."
set "EXAMPLES_DIR=%ROOT_DIR%\examples"

:: Function to test an example directory
:test_example_dir
    set "dir=%~1"
    for %%F in ("%dir%") do set "dir_name=%%~nxF"
    
    echo Testing %dir_name% examples...
    
    :: Check if the directory has a Cargo.toml file
    if not exist "%dir%\Cargo.toml" (
        echo Error: %dir_name% does not have a Cargo.toml file
        exit /b 1
    )
    
    :: Navigate to the example directory
    pushd "%dir%"
    
    :: Try to build the examples
    cargo build --quiet
    if !errorlevel! equ 0 (
        echo ✓ %dir_name% examples built successfully
        popd
        exit /b 0
    ) else (
        echo ✗ Failed to build %dir_name% examples
        popd
        exit /b 1
    )

:: Find all example directories
for /d %%D in ("%EXAMPLES_DIR%\*") do (
    :: Skip directories that don't have a Cargo.toml file
    if not exist "%%D\Cargo.toml" (
        echo Skipping %%~nxD - no Cargo.toml file
    ) else (
        :: Test the example directory
        call :test_example_dir "%%D"
        if !errorlevel! neq 0 (
            echo Failed to build examples in %%~nxD
        )
    )
    
    echo.
)

echo Example testing completed!
endlocal 