mod html;

pub use html::HtmlParser;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ParsedDocument {
    pub url: String,
    pub title: String,
    pub content: String,
}

impl ParsedDocument {
    pub fn new(url: String, title: String, content: String) -> Self {
        Self {
            url,
            title,
            content,
        }
    }
}
