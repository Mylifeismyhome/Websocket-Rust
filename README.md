# Websocket-Rust

This repository provides a Rust wrapper around a C-based WebSocket implementation, exposing it as a shared library with a C-compatible API. It enables projects written in C, C++, or other FFI-friendly languages to easily integrate WebSocket capabilities powered by Rust.

## 🧩 Structure

This project uses a Git submodule:
- [`Websocket`](https://github.com/Mylifeismyhome/Websocket): A C-based WebSocket implementation.

## 📦 Features

- Wraps a C WebSocket library with Rust.
- Exposes a C-compatible API via a shared library (`.so`, `.dll`, `.dylib`).
- Uses [`bindgen`](https://docs.rs/bindgen/latest/bindgen/) to generate C bindings for Rust.
- Enables FFI-friendly WebSocket communication.

## 🚀 Getting Started

### 1. Clone with submodules

```bash
git clone --recurse-submodules https://github.com/Mylifeismyhome/Websocket-Rust.git
cd Websocket-Rust
```
