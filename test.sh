#!/bin/bash

set -e

echo "=== Running cargo check ==="
cargo check

echo ""
echo "=== Running cargo clippy ==="
cargo clippy -- -D warnings

echo ""
echo "=== Running cargo fmt ==="
cargo fmt -- --check

echo ""
echo "=== Running unit tests ==="
cargo test

echo ""
echo "=== All tests passed ==="