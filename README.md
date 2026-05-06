# search5 - Local File Search Engine

A lightweight full-text search engine for local HTML files, built with Rust and Axum. Supports Chinese text search using n-gram indexing.

## Features

- **Full-text search** - Search through local HTML files with n-gram indexing
- **Chinese language support** - No tokenizer required, handles Chinese naturally
- **Web UI** - Clean interface with htmx + Tailwind CSS
- **REST API** - Programmatic access to search functionality
- **Persistent index** - Index persists across server restarts

## Quick Start

```bash
# Build the project
cargo build --release

# Run the server
./run.sh
```

Server runs at http://127.0.0.1:3000

## Build Index

Before searching, you need to build the index:

```bash
curl -X POST http://127.0.0.1:3000/index \
  -H "Content-Type: application/json" \
  -d '{"data_dir":"data/"}'
```

This indexes all `.html` files in the `data/` directory.

## Web Interface

- **Home page**: http://127.0.0.1:3000/
- **Search**: http://127.0.0.1:3000/search?q=keyword

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/index` | Build index from directory |
| GET | `/index/status` | Get index statistics |
| GET | `/api/search?q=keyword` | Search via API |
| GET | `/data/:file` | Serve static HTML files |

### Example

```bash
# Build index
curl -X POST http://127.0.0.1:3000/index \
  -H "Content-Type: application/json" \
  -d '{"data_dir":"data/"}'

# Search via API
curl "http://127.0.0.1:3000/api/search?q=程式"

# Check index status
curl http://127.0.0.1:3000/index/status
```

## Development

```bash
# Run tests
./test.sh

# Or individually:
cargo check
cargo clippy -- -D warnings
cargo fmt -- --check
cargo test
```

## Project Structure

```
src/
├── main.rs           # Entry point
├── indexer/          # Index building with nanofts
├── parser/           # HTML parsing
├── api/              # REST API routes
└── ui/               # Web UI handlers & templates
```

## Tech Stack

- **Backend**: Rust, Axum, tokio
- **Search**: nanofts (n-gram full-text search)
- **HTML parsing**: scraper
- **Frontend**: htmx, Tailwind CSS (via CDN)
- **Templates**: minijinja

## License

MIT