use crate::indexer::{IndexStats, SearchResult};
use crate::parser::{HtmlParser, ParsedDocument};
use nanofts::{EngineConfig, UnifiedEngine};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub struct Indexer {
    engine: UnifiedEngine,
    parser: HtmlParser,
    documents: HashMap<u32, ParsedDocument>,
    next_id: u32,
    index_path: String,
}

impl Indexer {
    pub fn new() -> Self {
        let config = EngineConfig::memory_only();
        let engine = UnifiedEngine::new(config).expect("Failed to create engine");
        Self {
            engine,
            parser: HtmlParser::new(),
            documents: HashMap::new(),
            next_id: 1,
            index_path: String::new(),
        }
    }

    pub fn new_persistent(path: &str) -> Self {
        let config = EngineConfig::persistent(path);
        let engine = UnifiedEngine::new(config).expect("Failed to create engine");

        let index_path = path.to_string();
        let documents_path = format!("{}.documents.json", path);
        let documents = Self::load_documents(&documents_path);

        let next_id = documents.keys().max().map(|k| k + 1).unwrap_or(1);

        Self {
            engine,
            parser: HtmlParser::new(),
            documents,
            next_id,
            index_path,
        }
    }

    fn load_documents(path: &str) -> HashMap<u32, ParsedDocument> {
        if let Ok(data) = fs::read_to_string(path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            HashMap::new()
        }
    }

    fn save_documents(&self) {
        if !self.index_path.is_empty() {
            let path = format!("{}.documents.json", self.index_path);
            if let Ok(data) = serde_json::to_string_pretty(&self.documents) {
                let _ = fs::write(path, data);
            }
        }
    }

    pub fn index_directory(&mut self, dir: &Path) -> std::io::Result<IndexStats> {
        let documents = self.parser.parse_directory(dir)?;

        let mut indexed = 0;
        for doc in documents {
            self.index_document(doc);
            indexed += 1;
        }

        self.engine.flush().ok();
        self.save_documents();

        Ok(IndexStats {
            documents: self.documents.len(),
            indexed,
        })
    }

    pub fn index_document(&mut self, doc: ParsedDocument) {
        let id = self.next_id;
        self.next_id += 1;

        self.documents.insert(id, doc.clone());

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

        let doc_ids = result.top(limit);
        let mut results = Vec::new();

        for doc_id in doc_ids {
            if let Some(doc) = self.documents.get(&doc_id) {
                let snippet = if doc.content.len() > 100 {
                    let end = doc
                        .content
                        .chars()
                        .take(100)
                        .map(|c| c.len_utf8())
                        .sum::<usize>()
                        .min(doc.content.len());
                    format!("{}...", &doc.content[..end])
                } else {
                    doc.content.clone()
                };

                results.push(SearchResult {
                    url: doc.url.clone(),
                    title: doc.title.clone(),
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
        IndexStats {
            documents: self.documents.len(),
            indexed: self.documents.len(),
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

        indexer.index_document(doc);
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

        indexer.index_document(doc);
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

        indexer.index_document(doc);
        indexer.flush();

        let results = indexer.search("搜尋", 10);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_multiple_documents() {
        let mut indexer = Indexer::new();

        indexer.index_document(ParsedDocument::new(
            "doc1.html".to_string(),
            "Rust 程式".to_string(),
            "Rust 是一種程式語言".to_string(),
        ));

        indexer.index_document(ParsedDocument::new(
            "doc2.html".to_string(),
            "Python 程式".to_string(),
            "Python 也是一種程式語言".to_string(),
        ));

        indexer.flush();

        let results = indexer.search("程式", 10);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_get_stats() {
        let mut indexer = Indexer::new();

        indexer.index_document(ParsedDocument::new(
            "test.html".to_string(),
            "Title".to_string(),
            "Content".to_string(),
        ));

        indexer.flush();

        let stats = indexer.get_stats();
        assert_eq!(stats.documents, 1);
    }

    #[test]
    fn test_persistent_indexer() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("test_index.nfts");

        let mut indexer = Indexer::new_persistent(index_path.to_str().unwrap());

        indexer.index_document(ParsedDocument::new(
            "test.html".to_string(),
            "Persistent".to_string(),
            "Persistent content".to_string(),
        ));

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
