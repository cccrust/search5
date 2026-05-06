#!/bin/bash
echo "Cleaning old index files..."
rm -f data/index.nfts data/index.nfts.documents.json data/index.nfts.wal 2>/dev/null

echo "Building index from data/ directory..."
curl -s -X POST http://127.0.0.1:3000/index \
  -H "Content-Type: application/json" \
  -d '{"data_dir":"data/"}'
echo ""
echo "Index built successfully!"