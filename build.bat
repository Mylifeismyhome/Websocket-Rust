@echo off
setlocal enabledelayedexpansion

:: Ask user to choose build configuration
echo Select build type:
echo [1] Debug
echo [2] Release
set /p CHOICE=Enter your choice (1 or 2): 

:: Translate choice into build type
if "%CHOICE%"=="1" (
    set BUILD_TYPE=debug
) else if "%CHOICE%"=="2" (
    set BUILD_TYPE=release
) else (
    echo Invalid choice. Please enter 1 or 2.
    pause
    exit /b 1
)

echo.
echo === Building in %BUILD_TYPE% mode ===
echo.

:: Build shared library using CMake
cd submodule\Websocket

if not exist build (
    mkdir build
)
cd build

cmake .. -DBUILD_SHARED=ON -DBUILD_STATIC=OFF -DBUILD_EXAMPLE=OFF -DCMAKE_BUILD_TYPE=%BUILD_TYPE%
cmake --build . --config %BUILD_TYPE%

cd ..\..\..

:: Build Rust binaries
set LIBCLANG_PATH=C:\Software\LLVM\bin

bindgen submodule\Websocket\include\websocket\api\websocket_c_api.h -o src\bindings.rs --with-derive-default -- -xc++ -std=c++17 -DWEBSOCKET_C_API -Isubmodule\Websocket\include\

if "%BUILD_TYPE%"=="release" (
    cargo build --release --no-default-features --features server --bin server
    cargo build --release --no-default-features --features client --bin client
) else (
    cargo build --no-default-features --features server --bin server
    cargo build --no-default-features --features client --bin client
)

:: Copy the built DLL to Rust target output
set DLL_NAME_SOURCE=LIB_SHARED.dll
set DLL_NAME_TARGET=Websocket.dll
set DLL_SOURCE=submodule\Websocket\build\bin\%BUILD_TYPE%\%DLL_NAME_SOURCE%
set DLL_DEST=target\%BUILD_TYPE%\%DLL_NAME_TARGET%

echo.
if exist %DLL_SOURCE% (
    copy /Y %DLL_SOURCE% %DLL_DEST% >nul
    echo DLL copied successfully to: %DLL_DEST%
) else (
    echo ERROR: DLL not found at path: %DLL_SOURCE%
)

echo.
pause
