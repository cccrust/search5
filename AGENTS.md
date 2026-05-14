# search5 Agent Instructions

## Commands

```bash
# Full test pipeline (check → clippy → fmt → test)
./test.sh

# Individual commands (order matters: lint → typecheck → test)
cargo check
cargo clippy -- -D warnings
cargo fmt -- --check
cargo test
```

## Running

```bash
# Dev server
cargo run
# Server runs at http://127.0.0.1:3000

# Production server with index auto-build
./run.sh
```

## Important: Build index before searching

Before search works, must call:
```bash
curl -X POST http://127.0.0.1:3000/index -H "Content-Type: application/json" -d '{"data_dir":"data/"}'
```

## Architecture

- `src/main.rs` - Entry point, creates Axum server
- `src/indexer/` - Builds inverted index with nanofts
- `src/search/` - Search engine with BM25 ranking
- `src/parser/` - HTML parsing with scraper
- `src/api/` - REST API routes
- `src/ui/` - Web UI handlers and templates
- `data/` - 20 HTML files to index

## API Endpoints

- `POST /index` - Build index from `data/` directory
- `GET /index/status` - Index status
- `GET /api/search?q=keyword` - REST API search
- `GET /` - Web UI home page
- `GET /search?q=keyword` - Web UI search results page

## Conventions

- `#![allow(dead_code, unused)]` in main.rs suppresses warnings
- Version docs in `_doc/v*.md`
- Cargo.toml uses `edition = "2024"` (unusual, most use 2021)
- Run test.sh before committing