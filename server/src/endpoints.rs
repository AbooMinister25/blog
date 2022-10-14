use axum::http::StatusCode;

// Health check endpoint
pub async fn health_check() -> StatusCode {
    StatusCode::OK
}
