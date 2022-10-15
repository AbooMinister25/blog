use axum::{http::StatusCode, response::Html};
use std::include_str;

static INDEX_CONTENT: &str = include_str!("../../public/index.html");

// Health check endpoint
pub async fn health_check() -> StatusCode {
    StatusCode::OK
}

// root endpoint
pub async fn index() -> Html<&'static str> {
    Html(INDEX_CONTENT)
}
