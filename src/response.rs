use rocket::http::ContentType;
use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use rocket::serde::json::{Json, Value};
use serde::Serialize;

#[derive(Serialize)]
pub struct ApiResponse {
    pub status: &'static str,
    pub data: Value,
}

impl<'r> Responder<'r, 'static> for ApiResponse {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        Response::build_from(Json(self).respond_to(req).unwrap())
            .status(Status::Ok)
            .header(ContentType::JSON)
            .ok()
    }
}
