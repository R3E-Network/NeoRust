@echo off
setlocal enabledelayedexpansion

:: Colors for output (Windows)
set "GREEN=[32m"
set "RED=[31m"
set "YELLOW=[33m"
set "NC=[0m"

echo %YELLOW%Testing all NeoRust examples...%NC%
echo.

:: Get the root directory of the project
set "ROOT_DIR=%~dp0.."
set "EXAMPLES_DIR=%ROOT_DIR%\examples"

:: Define feature combinations to test
set FEATURE_SETS[0]=
set FEATURE_SETS[1]=futures

:: Function to test an example directory with specific features
:test_example_dir
set "dir=%~1"
set "features=%~2"
for %%F in ("%dir%") do set "dir_name=%%~nxF"

set "feature_text="
if not "%features%"=="" (
    set "feature_text= with features: %features%"
) else (
    set "feature_text= with no features"
)

echo %YELLOW%Testing %dir_name% examples%feature_text%...%NC%

:: Check if the directory has a Cargo.toml file
if not exist "%dir%\Cargo.toml" (
    echo %RED%Error: %dir_name% does not have a Cargo.toml file%NC%
    exit /b 1
)

:: Navigate to the example directory
pushd "%dir%"

:: Try to build the examples with the specified features
if not "%features%"=="" (
    cargo build --quiet --no-default-features --features "%features%"
    if !errorlevel! equ 0 (
        echo %GREEN%✓ %dir_name% examples built successfully%feature_text%%NC%
        popd
        exit /b 0
    ) else (
        echo %RED%✗ Failed to build %dir_name% examples%feature_text%%NC%
        popd
        exit /b 1
    )
) else (
    :: Build with no features
    cargo build --quiet --no-default-features
    if !errorlevel! equ 0 (
        echo %GREEN%✓ %dir_name% examples built successfully%feature_text%%NC%
        popd
        exit /b 0
    ) else (
        echo %RED%✗ Failed to build %dir_name% examples%feature_text%%NC%
        popd
        exit /b 1
    )
)
popd
exit /b 0

:: Main script
for /d %%D in ("%EXAMPLES_DIR%\*") do (
    :: Skip directories that don't have a Cargo.toml file
    if not exist "%%D\Cargo.toml" (
        echo %YELLOW%Skipping %%~nxD - no Cargo.toml file%NC%
        echo.
        goto :continue
    )
    
    :: Test the example directory with different feature combinations
    for /L %%i in (0,1,1) do (
        call :test_example_dir "%%D" "!FEATURE_SETS[%%i]!"
        if !errorlevel! neq 0 (
            echo %RED%Failed to build examples in %%~nxD with features: !FEATURE_SETS[%%i]!%NC%
        )
        echo.
    )
    
    echo %GREEN%Testing completed for %%~nxD!%NC%
    echo %YELLOW%---------------------------------------%NC%
    echo.
    
    :continue
)

echo %GREEN%All example testing completed!%NC%
endlocal 