use axum::{
    Json, Router,
    extract::{Path, Query, State},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::indexer::{IndexStats, Indexer, SearchResult};

pub type SharedIndexer = Arc<Mutex<Indexer>>;

#[derive(Debug, Deserialize)]
pub struct IndexRequest {
    pub data_dir: String,
}

#[derive(Debug, Serialize)]
pub struct IndexResponse {
    pub indexed: usize,
    pub documents: usize,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    10
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusResponse {
    pub documents: usize,
    pub indexed: usize,
}

pub fn create_router(indexer: Indexer) -> Router {
    let shared = Arc::new(Mutex::new(indexer));

    Router::new()
        .route("/index", post(index_handler))
        .route("/api/search", get(search_handler))
        .route("/index/status", get(status_handler))
        .with_state(shared)
}

pub fn create_router_with_state(shared: SharedIndexer) -> Router {
    Router::new()
        .route("/index", post(index_handler))
        .route("/api/search", get(search_handler))
        .route("/index/status", get(status_handler))
        .with_state(shared)
}

#[axum::debug_handler]
async fn index_handler(
    State(state): State<SharedIndexer>,
    Json(req): Json<IndexRequest>,
) -> Json<IndexResponse> {
    let mut indexer = state.lock().await;

    let stats = match indexer.index_directory(std::path::Path::new(&req.data_dir)) {
        Ok(s) => s,
        Err(e) => {
            return Json(IndexResponse {
                indexed: 0,
                documents: 0,
            });
        }
    };

    Json(IndexResponse {
        indexed: stats.indexed,
        documents: stats.documents,
    })
}

#[axum::debug_handler]
async fn search_handler(
    State(state): State<SharedIndexer>,
    Query(query): Query<SearchQuery>,
) -> Json<SearchResponse> {
    let mut indexer = state.lock().await;

    let results = indexer.search(&query.q, query.limit);

    Json(SearchResponse { results })
}

#[axum::debug_handler]
async fn status_handler(State(state): State<SharedIndexer>) -> Json<StatusResponse> {
    let indexer = state.lock().await;
    let stats = indexer.get_stats();

    Json(StatusResponse {
        documents: stats.documents,
        indexed: stats.indexed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;
    use tower::util::ServiceExt;

    fn create_test_indexer() -> Indexer {
        let mut indexer = Indexer::new();
        indexer.index_document(crate::parser::ParsedDocument::new(
            "test1.html".to_string(),
            "Test Title 1".to_string(),
            "Test content one".to_string(),
        ));
        indexer.index_document(crate::parser::ParsedDocument::new(
            "test2.html".to_string(),
            "Test Title 2".to_string(),
            "Test content two".to_string(),
        ));
        indexer.flush();
        indexer
    }

    #[tokio::test]
    async fn test_search_handler() {
        let indexer = create_test_indexer();
        let router = create_router(indexer);

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/api/search?q=test&limit=5")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status().is_success());

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let response: SearchResponse = serde_json::from_slice(&body).unwrap();

        assert!(!response.results.is_empty());
    }

    #[tokio::test]
    async fn test_status_handler() {
        let indexer = create_test_indexer();
        let router = create_router(indexer);

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/index/status")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status().is_success());

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let response: StatusResponse = serde_json::from_slice(&body).unwrap();

        assert_eq!(response.documents, 2);
    }

    #[tokio::test]
    async fn test_search_empty_query() {
        let indexer = create_test_indexer();
        let router = create_router(indexer);

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/api/search?q=")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status().is_success());

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let response: SearchResponse = serde_json::from_slice(&body).unwrap();

        assert!(response.results.is_empty());
    }

    #[tokio::test]
    async fn test_search_with_chinese() {
        let mut indexer = Indexer::new();
        indexer.index_document(crate::parser::ParsedDocument::new(
            "test.html".to_string(),
            "測試標題".to_string(),
            "中文測試內容".to_string(),
        ));
        indexer.flush();

        let router = create_router(indexer);

        let response = router
            .oneshot(
                Request::builder()
                    .uri("/api/search?q=中文")
                    .body(axum::body::Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert!(response.status().is_success());

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let response: SearchResponse = serde_json::from_slice(&body).unwrap();

        assert!(!response.results.is_empty());
    }
}
