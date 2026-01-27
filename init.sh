#!/usr/bin/env bash
set -euo pipefail

# Check Rust toolchain
if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo not found. Please install Rust toolchain."
  exit 1
fi
if ! command -v rustc >/dev/null 2>&1; then
  echo "rustc not found. Please install Rust toolchain."
  exit 1
fi

echo "Building project..."
cargo build --verbose

# Optional tests
if command -v cargo >/dev/null 2>&1; then
  echo "Running tests (optional)..."
  cargo test --verbose
fi

echo "Setup complete. To run the CLI, use: ./target/debug/musicctl --help"
