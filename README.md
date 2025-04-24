# Websocket-Rust

This repository integrates a C-based WebSocket implementation into a Rust project using FFI. The core WebSocket logic is implemented entirely in C and included as a Git submodule. Rust bindings are generated from the C headers using `bindgen`.

## 🧩 Structure

This project uses a Git submodule:

- [`Websocket`](https://github.com/Mylifeismyhome/Websocket): A C-based WebSocket implementation with a C API.

## 📦 Features

- Integrates a C WebSocket library directly into Rust.
- Uses `bindgen` to generate Rust FFI bindings from C headers.
- Builds a shared Rust library using the C backend.

## 🚀 Getting Started

### 1. Clone with submodules

```bash
git clone --recurse-submodules https://github.com/Mylifeismyhome/Websocket-Rust.git
cd Websocket-Rust
```

### 2. Install Prerequisites

You will need the following installed:

- [Rust and Cargo](https://www.rust-lang.org/tools/install)
- [LLVM + Clang](https://llvm.org/) (required for `bindgen`)

#### On Ubuntu/Debian:

```bash
sudo apt update
sudo apt install llvm clang build-essential git
```

#### On macOS:

```bash
brew install llvm
```

Make sure LLVM is in your PATH:

```bash
export PATH="/opt/homebrew/opt/llvm/bin:$PATH"
```

#### On Windows:

Install LLVM via [releases.llvm.org](https://releases.llvm.org/) or:

```bash
choco install llvm
```

Install Rust via [rustup.rs](https://rustup.rs)

Ensure `cargo` and `clang` are in your system PATH.

### 3. Build

#### On Windows:

```bash
build.bat
```

#### On macOS / Linux:

```bash
chmod +x build.sh
./build.sh
```

## ❓ Troubleshooting

- If `bindgen` fails, make sure LLVM and Clang are correctly installed and accessible via PATH.
- On Windows, use the MSVC toolchain and developer command prompt if needed.

## 📄 License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for more info.
