mod endpoints;

use axum::{routing::get, Router};
use endpoints::health_check;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/health-check", get(health_check));
    tracing_subscriber::fmt::init();

    tracing::info!("Listening on http://0.0.0.0:3000");
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
