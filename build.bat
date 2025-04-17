@echo off

cargo build --no-default-features --features server --bin server
cargo build --no-default-features --features client --bin client

pause