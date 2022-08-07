use rocket::http::{ContentType, Status};
use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use rocket::serde::{
    json::{Json, Value},
    Serialize,
};
use thiserror::Error;

pub type ApiResult = Result<ApiResponse, ApiError>;

#[derive(Serialize)]
pub struct ApiResponse {
    pub data: Value,
}

#[derive(Serialize, Debug)]
pub struct ApiError {
    #[serde(skip)]
    status: Status,
    message: String,
}

#[derive(Error, Debug, Serialize)]
pub enum ErrorKind {
    #[error("Post not found")]
    PostNotFound,
}

impl<'r> Responder<'r, 'static> for ApiResponse {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        Response::build_from(Json(self).respond_to(req).unwrap())
            .status(Status::Ok)
            .header(ContentType::JSON)
            .ok()
    }
}

impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let status = self.status;

        Response::build_from(Json(self).respond_to(req).unwrap())
            .status(status)
            .header(ContentType::JSON)
            .ok()
    }
}

impl From<ErrorKind> for ApiError {
    fn from(kind: ErrorKind) -> Self {
        Self {
            status: kind.status_code(),
            message: kind.to_string(),
        }
    }
}

impl ErrorKind {
    pub fn status_code(&self) -> Status {
        match self {
            ErrorKind::PostNotFound => Status::NotFound,
        }
    }
}
