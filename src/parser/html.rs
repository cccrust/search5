use crate::parser::{HtmlParser, ParsedDocument};
use scraper::{Html, Selector};
use std::path::Path;

pub struct HtmlParser;

impl HtmlParser {
    pub fn new() -> Self {
        Self
    }

    pub fn parse_file(&self, path: &Path) -> std::io::Result<ParsedDocument> {
        let content = std::fs::read_to_string(path)?;
        Ok(self.parse(&content, path.to_string_lossy().as_ref()))
    }

    pub fn parse(&self, html: &str, url: &str) -> ParsedDocument {
        let document = Html::parse_document(html);

        let title = self.extract_title(&document);
        let content = self.extract_content(&document);

        ParsedDocument::new(url.to_string(), title, content)
    }

    fn extract_title(&self, document: &Html) -> String {
        let title_selector = Selector::parse("title, h1").unwrap();

        for element in document.select(&title_selector) {
            let text = element.text().collect::<String>().trim().to_string();
            if !text.is_empty() {
                return text;
            }
        }

        String::from("Untitled")
    }

    fn extract_content(&self, document: &Html) -> String {
        let mut content = String::new();

        let p_selector = Selector::parse("p").unwrap();
        for element in document.select(&p_selector) {
            let text = element.text().collect::<String>().trim().to_string();
            if !text.is_empty() {
                content.push_str(&text);
                content.push(' ');
            }
        }

        content.trim().to_string()
    }

    pub fn parse_directory(&self, dir: &Path) -> std::io::Result<Vec<ParsedDocument>> {
        let mut documents = Vec::new();

        let entries = std::fs::read_dir(dir)?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "html") {
                match self.parse_file(&path) {
                    Ok(doc) => documents.push(doc),
                    Err(e) => eprintln!("Failed to parse {:?}: {}", path, e),
                }
            }
        }

        Ok(documents)
    }
}

impl Default for HtmlParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_html() {
        let parser = HtmlParser::new();
        let html = r#"<!DOCTYPE html>
<html>
<head><title>Test Title</title></head>
<body><p>Hello world</p></body>
</html>"#;

        let doc = parser.parse(html, "test.html");

        assert_eq!(doc.title, "Test Title");
        assert_eq!(doc.url, "test.html");
        assert!(doc.content.contains("Hello world"));
    }

    #[test]
    fn test_extract_title_from_h1() {
        let parser = HtmlParser::new();
        let html = r#"<!DOCTYPE html>
<html>
<head><title>Page Title</title></head>
<body><h1>Heading One</h1></body>
</html>"#;

        let doc = parser.parse(html, "test.html");

        assert_eq!(doc.title, "Heading One");
    }

    #[test]
    fn test_extract_content_multiple_paragraphs() {
        let parser = HtmlParser::new();
        let html = r#"<!DOCTYPE html>
<html>
<body>
<p>First paragraph.</p>
<p>Second paragraph.</p>
<p>Third paragraph.</p>
</body>
</html>"#;

        let doc = parser.parse(html, "test.html");

        assert!(doc.content.contains("First paragraph"));
        assert!(doc.content.contains("Second paragraph"));
        assert!(doc.content.contains("Third paragraph"));
    }

    #[test]
    fn test_default_title() {
        let parser = HtmlParser::new();
        let html = r#"<!DOCTYPE html>
<html><body><p>No title here</p></body></html>"#;

        let doc = parser.parse(html, "test.html");

        assert_eq!(doc.title, "Untitled");
    }

    #[test]
    fn test_chinese_content() {
        let parser = HtmlParser::new();
        let html = r#"<!DOCTYPE html>
<html>
<head><title>測試標題</title></head>
<body><p>這是中文內容測試</p></body>
</html>"#;

        let doc = parser.parse(html, "test.html");

        assert_eq!(doc.title, "測試標題");
        assert!(doc.content.contains("這是中文內容測試"));
    }

    #[test]
    fn test_mixed_chinese_english() {
        let parser = HtmlParser::new();
        let html = r#"<!DOCTYPE html>
<html>
<head><title>Rust Programming</title></head>
<body><p>Rust 是一種安全的程式語言。</p></body>
</html>"#;

        let doc = parser.parse(html, "test.html");

        assert!(doc.content.contains("Rust"));
        assert!(doc.content.contains("程式語言"));
    }
}
