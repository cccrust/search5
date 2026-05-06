#!/bin/bash
cd "$(dirname "$0")"

echo "Stopping server..."
pkill -f search5 2>/dev/null || true
sleep 1

echo "Cleaning old index files..."
rm -f data/index.nfts data/index.nfts.documents.json data/index.nfts.wal 2>/dev/null

echo "Building release..."
cargo build --release

echo "Starting server..."
nohup ./target/release/search5 > /tmp/search5.log 2>&1 &
sleep 3

echo "Building index..."
curl -s -X POST http://127.0.0.1:3000/index \
  -H "Content-Type: application/json" \
  -d '{"data_dir":"data/"}'
echo ""

echo "========================================"
echo "Server running at http://127.0.0.1:3000"
echo "Test: curl http://127.0.0.1:3000/search?q=程式"
echo "========================================"