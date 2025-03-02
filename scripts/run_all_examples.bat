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
echo %CYAN%            NeoRust - Run All Examples%NC%
echo %CYAN%========================================================%NC%

:: Create a temporary file to store example names
set "TEMP_FILE=%TEMP%\neo_examples.txt"
if exist "%TEMP_FILE%" del "%TEMP_FILE%"

:: Get all example files from examples directory
echo %YELLOW%Discovering examples...%NC%
if exist "examples" (
    :: List all .rs files and extract example names
    for /r "examples" %%f in (*.rs) do (
        set "FULL_PATH=%%f"
        set "EXAMPLE_NAME=!FULL_PATH:*examples\=!"
        set "EXAMPLE_NAME=!EXAMPLE_NAME:.rs=!"
        :: Replace directory separators with underscores
        set "EXAMPLE_NAME=!EXAMPLE_NAME:\=_!"
        echo !EXAMPLE_NAME!>>"%TEMP_FILE%"
    )
) else (
    echo %RED%Examples directory not found!%NC%
    exit /b 1
)

:: Initialize counters
set TOTAL_EXAMPLES=0
set SUCCESSFUL_EXAMPLES=0
set FAILED_EXAMPLES=

:: Run each example
for /f "tokens=*" %%a in (%TEMP_FILE%) do (
    set EXAMPLE=%%a
    echo.
    echo %CYAN%Running example: !EXAMPLE!%NC%
    echo %YELLOW%----------------------------------------%NC%
    
    :: Build example with all features (to ensure it compiles)
    cargo build --example "!EXAMPLE!" --all-features
    if !ERRORLEVEL! EQU 0 (
        :: Run with all features
        cargo run --example "!EXAMPLE!" --all-features
        if !ERRORLEVEL! EQU 0 (
            echo %GREEN%Example !EXAMPLE! ran successfully%NC%
            set /a SUCCESSFUL_EXAMPLES+=1
        ) else (
            echo %RED%Example !EXAMPLE! failed%NC%
            set FAILED_EXAMPLES=!FAILED_EXAMPLES! !EXAMPLE!
        )
    ) else (
        echo %RED%Failed to build example: !EXAMPLE!%NC%
        set FAILED_EXAMPLES=!FAILED_EXAMPLES! !EXAMPLE!
    )
    
    set /a TOTAL_EXAMPLES+=1
)

:: Clean up temp file
if exist "%TEMP_FILE%" del "%TEMP_FILE%"

:: Summary
echo.
echo %CYAN%========================================================%NC%
echo %CYAN%                 Examples Summary%NC%
echo %CYAN%========================================================%NC%
echo Total examples: %TOTAL_EXAMPLES%
echo %GREEN%Successful examples: %SUCCESSFUL_EXAMPLES%%NC%

if not "!FAILED_EXAMPLES!"=="" (
    echo %RED%Failed examples: !FAILED_EXAMPLES!%NC%
    exit /b 1
) else (
    echo %GREEN%All examples ran successfully!%NC%
    exit /b 0
) 