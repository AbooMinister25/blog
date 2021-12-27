use crate::auth::AuthenticatedUser;
use crate::crud;
use crate::errors::ApiError;
use crate::models::user::NewUserJson;
use crate::response::ApiResponse;
use crate::DBPool;
use rocket::serde::json::{Json, Value};

#[post("/users", format = "json", data = "<user>")]
pub async fn create_user(
    conn: DBPool,
    user: Json<NewUserJson>,
    auth: Result<AuthenticatedUser, ApiError>,
) -> Result<ApiResponse, ApiError> {
    auth?;

    conn.run(move |c| crud::users::new(c, &user.username, &user.passwd))
        .await?;

    Ok(ApiResponse {
        status: "success",
        data: Value::Null,
    })
}

#[delete("/users/<uid>")]
pub async fn delete_user(
    conn: DBPool,
    uid: i32,
    auth: Result<AuthenticatedUser, ApiError>,
) -> Result<ApiResponse, ApiError> {
    auth?;

    conn.run(move |c| crud::users::delete(c, uid)).await?;

    Ok(ApiResponse {
        status: "success",
        data: Value::Null,
    })
}

#[get("/users/<uid>")]
pub async fn fetch_user(
    conn: DBPool,
    uid: i32,
    auth: Result<AuthenticatedUser, ApiError>,
) -> Result<ApiResponse, ApiError> {
    auth?;

    let user = conn.run(move |c| crud::users::find_one(c, uid)).await?;

    Ok(ApiResponse {
        status: "success",
        data: json!({
            "id": user.id,
            "username": user.username,
        }),
    })
}
