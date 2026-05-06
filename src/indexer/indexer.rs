use crate::indexer::{IndexStats, SearchResult};
use crate::parser::{HtmlParser, ParsedDocument};
use nanofts::{EngineConfig, UnifiedEngine};
use std::collections::HashMap;
use std::path::Path;

pub struct Indexer {
    engine: UnifiedEngine,
    parser: HtmlParser,
}

impl Indexer {
    pub fn new() -> Self {
        let config = EngineConfig::memory_only();
        let engine = UnifiedEngine::new(config).expect("Failed to create engine");
        Self {
            engine,
            parser: HtmlParser::new(),
        }
    }

    pub fn new_persistent(path: &str) -> Self {
        let config = EngineConfig::persistent(path)
            .with_max_chinese_length(4)
            .with_min_term_length(2);
        let engine = UnifiedEngine::new(config).expect("Failed to create engine");
        Self {
            engine,
            parser: HtmlParser::new(),
        }
    }

    pub fn index_directory(&mut self, dir: &Path) -> std::io::Result<IndexStats> {
        let documents = self.parser.parse_directory(dir)?;

        let mut indexed = 0;
        for (i, doc) in documents.into_iter().enumerate() {
            self.index_document(i as u64, doc);
            indexed += 1;
        }

        self.engine.flush().ok();

        let stats = self.engine.get_index_stats();
        Ok(IndexStats {
            documents: stats.total_docs,
            indexed,
        })
    }

    fn index_document(&mut self, id: u64, doc: ParsedDocument) {
        let mut fields = HashMap::new();
        fields.insert("url".to_string(), doc.url.clone());
        fields.insert("title".to_string(), doc.title.clone());
        fields.insert("content".to_string(), doc.content.clone());

        self.engine.add_document(id, fields).ok();
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        let result = match self.engine.search(query) {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };

        let mut results = Vec::new();
        for doc_id in result.iter().take(limit) {
            if let Ok(Some(doc)) = self.engine.get_document(*doc_id) {
                let url = doc.get("url").cloned().unwrap_or_default();
                let title = doc.get("title").cloned().unwrap_or_default();
                let content = doc.get("content").cloned().unwrap_or_default();

                let snippet = if content.len() > 100 {
                    format!("{}...", &content[..100])
                } else {
                    content
                };

                results.push(SearchResult {
                    url,
                    title,
                    snippet,
                    score: 1.0,
                });
            }
        }

        results
    }

    pub fn flush(&self) {
        self.engine.flush().ok();
    }

    pub fn get_stats(&self) -> IndexStats {
        let stats = self.engine.get_index_stats();
        IndexStats {
            documents: stats.total_docs,
            indexed: stats.total_docs,
        }
    }
}

impl Default for Indexer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_html(title: &str, content: &str) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
<head><title>{}</title></head>
<body><p>{}</p></body>
</html>"#,
            title, content
        )
    }

    #[test]
    fn test_index_single_document() {
        let mut indexer = Indexer::new();

        let doc = ParsedDocument::new(
            "test.html".to_string(),
            "Test Title".to_string(),
            "Test content".to_string(),
        );

        indexer.index_document(1, doc);
        indexer.flush();

        let results = indexer.search("Test", 10);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_chinese() {
        let mut indexer = Indexer::new();

        let doc = ParsedDocument::new(
            "test.html".to_string(),
            "測試標題".to_string(),
            "這是中文測試內容".to_string(),
        );

        indexer.index_document(1, doc);
        indexer.flush();

        let results = indexer.search("中文", 10);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_partial_match() {
        let mut indexer = Indexer::new();

        let doc = ParsedDocument::new(
            "test.html".to_string(),
            "全文檢索".to_string(),
            "搜尋引擎測試".to_string(),
        );

        indexer.index_document(1, doc);
        indexer.flush();

        let results = indexer.search("搜尋", 10);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_multiple_documents() {
        let mut indexer = Indexer::new();

        indexer.index_document(
            1,
            ParsedDocument::new(
                "doc1.html".to_string(),
                "Rust 程式".to_string(),
                "Rust 是一種程式語言".to_string(),
            ),
        );

        indexer.index_document(
            2,
            ParsedDocument::new(
                "doc2.html".to_string(),
                "Python 程式".to_string(),
                "Python 也是一種程式語言".to_string(),
            ),
        );

        indexer.flush();

        let results = indexer.search("程式", 10);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_get_stats() {
        let mut indexer = Indexer::new();

        indexer.index_document(
            1,
            ParsedDocument::new(
                "test.html".to_string(),
                "Title".to_string(),
                "Content".to_string(),
            ),
        );

        indexer.flush();

        let stats = indexer.get_stats();
        assert_eq!(stats.documents, 1);
    }

    #[test]
    fn test_persistent_indexer() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("test_index.nfts");

        let mut indexer = Indexer::new_persistent(index_path.to_str().unwrap());

        indexer.index_document(
            1,
            ParsedDocument::new(
                "test.html".to_string(),
                "Persistent".to_string(),
                "Persistent content".to_string(),
            ),
        );

        indexer.flush();

        let stats = indexer.get_stats();
        assert_eq!(stats.documents, 1);
    }

    #[test]
    fn test_search_empty_query() {
        let indexer = Indexer::new();
        let results = indexer.search("", 10);
        assert!(results.is_empty());
    }
}
