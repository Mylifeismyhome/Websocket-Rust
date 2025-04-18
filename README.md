Websocket-Rust

This repository provides a Rust wrapper around a C-based WebSocket implementation, exposing it as a shared library with a C-compatible API. It enables projects written in C, C++, or other FFI-friendly languages to easily integrate WebSocket capabilities powered by Rust.
🧩 Structure

This project uses a Git submodule:

    Websocket: A C-based WebSocket implementation.

📦 Features

    Wraps a C WebSocket library with Rust.

    Exposes a C-compatible API via a shared library (.so, .dll, .dylib).

    Uses bindgen to generate C bindings for Rust.

    Enables FFI-friendly WebSocket communication.

🚀 Getting Started
1. Clone with submodules

git clone --recurse-submodules https://github.com/Mylifeismyhome/Websocket-Rust.git
cd Websocket-Rust

2. Install Prerequisites

You will need the following installed:

    Rust and Cargo: https://www.rust-lang.org/tools/install

    LLVM + Clang: https://llvm.org/

    Git

On Ubuntu/Debian:

sudo apt update
sudo apt install llvm clang build-essential git

On macOS:

brew install llvm

    Make sure LLVM is in your PATH:

    export PATH="/opt/homebrew/opt/llvm/bin:$PATH"

On Windows:

    Install LLVM via https://releases.llvm.org/ or:

choco install llvm

    Install Rust via https://rustup.rs

    Ensure cargo and clang are in your system PATH.

3. Build
On Windows:

build.bat

This compiles the Rust code and generates a C header using bindgen.
On macOS / Linux:

chmod +x build.sh
./build.sh

Same as above—builds the shared library and generates the C header.
🏗️ Output

After building, you'll find:

    target/release/websocket.dll (Windows)

    target/release/libwebsocket.so (Linux)

    target/release/libwebsocket.dylib (macOS)

    websocket.h — C header for the public API

🔧 Using the Library in C/C++

Include the header and link against the compiled shared library in your C/C++ project.
Example C usage:

#include "websocket.h"

int main() {
    ws_start("ws://echo.websocket.org");
    // your logic here
    return 0;
}

    Ensure your compiler knows where to find the header and library:

        On Linux/macOS, you might need to set LD_LIBRARY_PATH or DYLD_LIBRARY_PATH.

        On Windows, make sure the .dll is in the same directory as your executable or in your PATH.

❓ Troubleshooting

    If bindgen fails, make sure LLVM and Clang are correctly installed and accessible via PATH.

    On Windows, use the MSVC toolchain and developer command prompt if needed.

📄 License

This project is licensed under the MIT License. See the LICENSE file for more info.

Maintained by @Mylifeismyhome
Feel free to open issues or pull requests!

Let me know if you'd like a websocket.h example too or help wiring it into a CMake or Makefile project!
