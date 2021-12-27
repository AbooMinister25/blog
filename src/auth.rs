use crate::crud;
use crate::errors::ApiError;
use crate::DBPool;
use rocket::form::FromForm;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use serde::Deserialize;

#[derive(FromForm, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

pub struct AuthenticatedUser {
    pub user_id: i32,
}

async fn validate_password(
    conn: DBPool,
    username: String,
    password: String,
) -> Result<bool, ApiError> {
    conn.run(move |c| crud::users::validate(c, &password, &username))
        .await
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = ApiError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let conn = req.guard::<DBPool>().await.succeeded();

        if let Some(cn) = conn {
            let username = req.headers().get_one("username");
            let password = req.headers().get_one("password");

            return match (username, password) {
                (Some(u), Some(p)) => {
                    let validated = validate_password(cn, u.to_string(), p.to_string()).await;

                    if validated.is_err() {
                        let err = validated.err().unwrap();
                        return Outcome::Failure((err.status_code(), err));
                    }

                    if validated.unwrap() {
                        return Outcome::Success(AuthenticatedUser { user_id: 10 });
                    }

                    Outcome::Failure((Status::Unauthorized, ApiError::InvalidCredentials))
                }
                _ => Outcome::Failure((Status::BadRequest, ApiError::MissingData)),
            };
        }

        Outcome::Failure((Status::InternalServerError, ApiError::PoolError))
    }
}
