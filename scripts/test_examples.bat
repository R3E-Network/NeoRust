@echo off
setlocal enabledelayedexpansion

:: Script to test all examples in the NeoRust repository
:: This script will attempt to build all examples to ensure they compile

echo Testing all NeoRust examples...
echo.

:: Get the root directory of the project
set "ROOT_DIR=%~dp0.."
set "EXAMPLES_DIR=%ROOT_DIR%\examples"

:: Define feature combinations to test
set "FEATURE_SETS="
set "FEATURE_SETS=!FEATURE_SETS! """
set "FEATURE_SETS=!FEATURE_SETS! "futures""
set "FEATURE_SETS=!FEATURE_SETS! "futures,ledger""
set "FEATURE_SETS=!FEATURE_SETS! "futures,aws""
set "FEATURE_SETS=!FEATURE_SETS! "futures,sgx""
set "FEATURE_SETS=!FEATURE_SETS! "futures,ledger,aws""
set "FEATURE_SETS=!FEATURE_SETS! "futures,ledger,aws,sgx""

:: Function to test an example directory with specific features
:test_example_dir
    set "dir=%~1"
    set "features=%~2"
    for %%F in ("%dir%") do set "dir_name=%%~nxF"
    
    if "%features%"=="" (
        echo Testing %dir_name% examples with no features...
    ) else (
        echo Testing %dir_name% examples with features: %features%...
    )
    
    :: Check if the directory has a Cargo.toml file
    if not exist "%dir%\Cargo.toml" (
        echo Error: %dir_name% does not have a Cargo.toml file
        exit /b 1
    )
    
    :: Navigate to the example directory
    pushd "%dir%"
    
    :: Try to build the examples with the specified features
    if "%features%"=="" (
        :: Build with no features
        cargo build --quiet --no-default-features
    ) else (
        :: Build with the specified features
        cargo build --quiet --no-default-features --features %features%
    )
    
    if !errorlevel! equ 0 (
        if "%features%"=="" (
            echo ✓ %dir_name% examples built successfully with no features
        ) else (
            echo ✓ %dir_name% examples built successfully with features: %features%
        )
        popd
        exit /b 0
    ) else (
        if "%features%"=="" (
            echo ✗ Failed to build %dir_name% examples with no features
        ) else (
            echo ✗ Failed to build %dir_name% examples with features: %features%
        )
        popd
        exit /b 1
    )

:: Find all example directories
for /d %%D in ("%EXAMPLES_DIR%\*") do (
    :: Skip directories that don't have a Cargo.toml file
    if not exist "%%D\Cargo.toml" (
        echo Skipping %%~nxD - no Cargo.toml file
    ) else (
        :: Test the example directory with different feature combinations
        for %%F in (%FEATURE_SETS%) do (
            call :test_example_dir "%%D" %%F
            echo.
        )
        
        echo Testing completed for %%~nxD!
        echo ---------------------------------------
        echo.
    )
)

echo All example testing completed!
endlocal 