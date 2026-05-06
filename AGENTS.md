# search5 Agent Instructions

## Commands

```bash
# Full test pipeline (runs check, clippy, fmt, test)
./test.sh

# Individual commands
cargo check
cargo clippy -- -D warnings
cargo fmt -- --check
cargo test
```

## Running

```bash
cargo run
# Server runs at http://127.0.0.1:3000
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
- Run test.sh before committing