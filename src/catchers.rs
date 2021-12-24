use crate::errors::ApiError;
use rocket::Request;

#[catch(404)]
pub fn not_found(req: &Request) -> ApiError {
    ApiError::PageNotFound(req.uri().to_string())
}

#[catch(500)]
pub fn internal_server_error(_: &Request) -> ApiError {
    ApiError::InternalServerError
}
