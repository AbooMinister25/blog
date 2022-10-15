mod endpoints;

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, get_service},
    Router,
};
use axum_extra::routing::SpaRouter;
use endpoints::{health_check, index};
use std::io;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let serve_static = get_service(ServeDir::new("./static")).handle_error(handle_error);
    let styles_router = SpaRouter::new("/styles", "./styles");

    let app = Router::new()
        .route("/", get(index))
        .route("/health-check", get(health_check))
        .merge(styles_router)
        .nest("/static", serve_static);

    tracing_subscriber::fmt::init();
    tracing::info!("Listening on http://0.0.0.0:3000");

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handle_error(_err: io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}
