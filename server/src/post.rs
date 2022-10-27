use serde::Serialize;

#[derive(Serialize)]
pub struct Post {
    pub title: String,
    pub content: String,
    pub summary: String,
    pub timestamp: String,
    pub tags: Vec<String>,
}

impl Post {
    pub fn new(
        title: String,
        content: String,
        summary: String,
        timestamp: String,
        tags: Vec<String>,
    ) -> Self {
        Self {
            title,
            content,
            summary,
            timestamp,
            tags,
        }
    }
}
