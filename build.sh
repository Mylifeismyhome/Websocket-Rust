#!/bin/bash
set -e

echo "Select build type:"
echo "[1] Debug"
echo "[2] Release"
read -p "Enter your choice (1 or 2): " CHOICE

# Translate input to build type
if [[ "$CHOICE" == "1" ]]; then
    BUILD_TYPE=Debug
    RUST_BUILD=debug
elif [[ "$CHOICE" == "2" ]]; then
    BUILD_TYPE=Release
    RUST_BUILD=release
else
    echo "Invalid choice. Please enter 1 or 2."
    exit 1
fi

echo
echo "=== Building in $BUILD_TYPE mode ==="
echo

# Build shared library using CMake
cd submodule/Websocket

mkdir -p build
cd build

cmake .. -DBUILD_SHARED=ON -DBUILD_STATIC=OFF -DBUILD_EXAMPLE=OFF -DCMAKE_BUILD_TYPE=$BUILD_TYPE
cmake --build . --config $BUILD_TYPE

cd ../../..

# Build Rust binaries
if [[ "$RUST_BUILD" == "release" ]]; then
    cargo build --release --no-default-features --features server --bin server
    cargo build --release --no-default-features --features client --bin client
else
    cargo build --no-default-features --features server --bin server
    cargo build --no-default-features --features client --bin client
fi

# Copy and rename the built shared library
echo

SRC_NAME="libLIB_SHARED"
if [[ "$OSTYPE" == "darwin"* ]]; then
    SRC_EXT=".dylib"
else
    SRC_EXT=".so"
fi

SRC_PATH="submodule/Websocket/build/lib/${SRC_NAME}${SRC_EXT}"
DEST_PATH="target/$RUST_BUILD/Websocket${SRC_EXT}"

if [[ -f "$SRC_PATH" ]]; then
    cp "$SRC_PATH" "$DEST_PATH"
    echo "Library copied successfully to: $DEST_PATH"
else
    echo "ERROR: Shared library not found at: $SRC_PATH"
fi

echo
read -n 1 -s -r -p "Press any key to continue..."
echo
