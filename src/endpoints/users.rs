use crate::crud;
use crate::errors::ApiError;
use crate::models::user::NewUserJson;
use crate::response::ApiResponse;
use crate::DBPool;
use rocket::serde::json::{Json, Value};

#[post("/users", format = "json", data = "<user>")]
pub async fn create_user(conn: DBPool, user: Json<NewUserJson>) -> Result<ApiResponse, ApiError> {
    conn.run(move |c| crud::users::new(c, &user.username, &user.passwd))
        .await?;

    Ok(ApiResponse {
        status: "success",
        data: Value::Null,
    })
}

#[delete("/users/<username>")]
pub async fn delete_user(conn: DBPool, username: String) -> Result<ApiResponse, ApiError> {
    conn.run(move |c| crud::users::delete(c, &username)).await?;

    Ok(ApiResponse {
        status: "success",
        data: Value::Null,
    })
}
