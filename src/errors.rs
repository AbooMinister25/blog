use rocket::http::ContentType;
use rocket::http::Status;
use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use rocket::serde::json::Json;
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Post not found")]
    PostNotFound,
    #[error("URL {0} not found")]
    PageNotFound(String),
    #[error("Internal Server Error")]
    InternalServerError,
    #[error("Error while parsing date {0}, invalid date format")]
    DateParsingError(String),
    #[error("Error while inserting post")]
    PostInsertionError(#[source] diesel::result::Error),
    #[error("Error while deleting post")]
    PostDeletionError(#[source] diesel::result::Error),
    #[error("Missing required header {0}")]
    MissingHeader(String),
    #[error("Error while loading post")]
    PostLoadError(#[source] diesel::result::Error),
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
            ApiError::PostNotFound => Status::NotFound,
            ApiError::PageNotFound(_) => Status::NotFound,
            ApiError::InternalServerError => Status::InternalServerError,
            ApiError::DateParsingError(_) => Status::BadRequest,
            ApiError::PostInsertionError(_) => Status::InternalServerError,
            ApiError::PostDeletionError(_) => Status::InternalServerError,
            ApiError::MissingHeader(_) => Status::UnprocessableEntity,
            ApiError::PostLoadError(_) => Status::InternalServerError,
        }
    }
}
