#![allow(dead_code, unused)]

mod api;
mod indexer;
mod parser;
mod search;

use api::create_router;
use indexer::Indexer;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    let indexer = Indexer::new();
    let cors = CorsLayer::permissive();
    let router = create_router(indexer).layer(cors);
    
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server running at http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}