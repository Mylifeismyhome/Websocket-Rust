#!/bin/bash

set -e

cargo build --no-default-features --features server --bin server
cargo build --no-default-features --features client --bin client

read -n 1 -s -r -p "Press any key to continue..."
echo
