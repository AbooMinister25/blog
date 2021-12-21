use rocket::http::ContentType;
use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use rocket::serde::json::Json;
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Post {0} not found")]
    PostNotFound(i32),
    #[error("Error while loading post")]
    PostLoadError(#[from] diesel::result::Error),
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub status: &'static str,
    pub message: String,
}

impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let response = ErrorResponse {
            status: "error",
            message: self.to_string(),
        };

        Response::build_from(Json(response).respond_to(req).unwrap())
            .status(self.status_code())
            .header(ContentType::JSON)
            .ok()
    }
}

impl ApiError {
    fn status_code(&self) -> Status {
        match self {
            ApiError::PostNotFound(_) => Status::NotFound,
            ApiError::PostLoadError(_) => Status::InternalServerError,
        }
    }
}
