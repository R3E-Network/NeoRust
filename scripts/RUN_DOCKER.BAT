@echo off

REM Check if image exists
docker images -q my-rust-image > nul 2>&1
if %ERRORLEVEL% NEQ 0 goto build

REM Image exists, run container
docker run -v "%CD%":/workspace -v cargo-cache:/usr/local/cargo/registry -it my-rust-image
goto end

:build
REM Image does not exist, build it
docker build -t my-rust-image .
REM Run container after building
docker run -v "%CD%":/workspace -v cargo-cache:/usr/local/cargo/registry -it my-rust-image

:end