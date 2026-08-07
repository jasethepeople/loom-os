#!/usr/bin/env bash
set -e

echo "Building Loom OS workspace..."
cargo build --release

echo ""
echo "Build complete. Binaries:"
ls -la target/release/core target/release/loom-cli target/release/loom-tui 2>/dev/null || true

echo ""
echo "Run the kernel:    cargo run --release -p core"
echo "Run the TUI:       cargo run --release -p loom-tui"
echo "Run the CLI:       cargo run --release -p loom-cli -- status"
