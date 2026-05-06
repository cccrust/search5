use crate::indexer::{Indexer, SearchResult};
use crate::ui::create_env;
use axum::{
    extract::{Query, State},
    response::Html,
};
use minijinja::context;
use serde::Deserialize;

use crate::api::SharedIndexer;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    q: Option<String>,
    page: Option<usize>,
}

const RESULTS_PER_PAGE: usize = 10;

pub async fn index_page(State(state): State<SharedIndexer>) -> Html<String> {
    let indexer = state.lock().await;
    let stats = indexer.get_stats();

    let env = create_env();
    let template = env.get_template("index").unwrap();

    let html = template
        .render(context!(
            stats => stats,
            query => "",
        ))
        .unwrap();

    Html(html)
}

pub async fn search_page(
    State(state): State<SharedIndexer>,
    Query(query): Query<SearchQuery>,
) -> Html<String> {
    let q = query.q.as_deref().unwrap_or("").to_string();
    let page = query.page.unwrap_or(1).max(1);

    let indexer = state.lock().await;
    let stats = indexer.get_stats();

    let results = if q.is_empty() {
        vec![]
    } else {
        indexer.search(&q, 100)
    };

    let total_results = results.len();
    let total_pages = total_results.div_ceil(RESULTS_PER_PAGE);
    let start = (page - 1) * RESULTS_PER_PAGE;
    let page_results: Vec<_> = results
        .into_iter()
        .skip(start)
        .take(RESULTS_PER_PAGE)
        .collect();

    let env = create_env();
    let template = env.get_template("search").unwrap();

    let html = template
        .render(context!(
            query => q,
            results => page_results,
            current_page => page,
            total_pages => total_pages,
            total_results => total_results,
            stats => stats,
        ))
        .unwrap();

    Html(html)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ParsedDocument;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    fn create_test_indexer() -> SharedIndexer {
        let mut indexer = Indexer::new();
        indexer.index_document(ParsedDocument::new(
            "test1.html".to_string(),
            "Rust Programming".to_string(),
            "Rust is a systems programming language".to_string(),
        ));
        indexer.index_document(ParsedDocument::new(
            "test2.html".to_string(),
            "Python Basics".to_string(),
            "Python is a high-level language".to_string(),
        ));
        indexer.flush();
        Arc::new(Mutex::new(indexer))
    }

    #[tokio::test]
    async fn test_index_page() {
        let state = create_test_indexer();
        let response = index_page(State(state)).await;
        assert!(response.0.contains("搜尋引擎"));
    }

    #[tokio::test]
    async fn test_search_page_with_query() {
        let state = create_test_indexer();
        let query = SearchQuery {
            q: Some("Rust".to_string()),
            page: Some(1),
        };
        let response = search_page(State(state), Query(query)).await;
        assert!(response.0.contains("Rust Programming"));
    }

    #[tokio::test]
    async fn test_search_page_empty_query() {
        let state = create_test_indexer();
        let query = SearchQuery {
            q: None,
            page: Some(1),
        };
        let response = search_page(State(state), Query(query)).await;
        assert!(response.0.contains("請輸入關鍵字"));
    }

    #[tokio::test]
    async fn test_search_pagination() {
        let state = create_test_indexer();
        let query = SearchQuery {
            q: Some("xyz123".to_string()),
            page: Some(1),
        };
        let response = search_page(State(state), Query(query)).await;
        assert!(response.0.contains("沒有找到符合"));
    }
}
