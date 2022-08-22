use rocket::http::Status;
use rocket_okapi::openapi;

/// # Health Check
///
/// Return status code 200 (Ok).
#[openapi(tag = "Health Check")]
#[get("/health_check")]
pub async fn health_check() -> Status {
    Status::Ok
}
