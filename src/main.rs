#![allow(dead_code, unused)]

mod api;
mod indexer;
mod parser;
mod search;
mod ui;

use api::{SharedIndexer, create_router_with_state};
use axum::{extract::Path, response::Html, routing::get};
use indexer::Indexer;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    let indexer = Indexer::new_persistent("data/index.nfts");
    let shared: SharedIndexer = Arc::new(Mutex::new(indexer));

    let cors = CorsLayer::permissive();

    let api_router = create_router_with_state(shared.clone()).layer(cors.clone());

    let ui_router = axum::Router::new()
        .route("/", axum::routing::get(ui::index_page))
        .route("/search", axum::routing::get(ui::search_page))
        .route("/data/:file", get(static_file_handler))
        .with_state(shared);

    async fn static_file_handler(Path(file): Path<String>) -> Html<String> {
        let path = std::path::Path::new("data").join(&file);
        if path.exists()
            && let Ok(content) = std::fs::read_to_string(&path)
        {
            return Html(content);
        }
        Html("<h1>404 - File not found</h1>".to_string())
    }

    let router = api_router.merge(ui_router);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
