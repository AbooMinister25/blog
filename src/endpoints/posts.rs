use crate::auth::AuthenticatedUser;
use crate::crud;
use crate::errors::ApiError;
use crate::models::post::{NewPostJson, PostJson};
use crate::response::ApiResponse;
use crate::DBPool;
use rocket::serde::json::{Json, Value};

#[get("/posts/<id>")]
pub async fn fetch_post(conn: DBPool, id: i32) -> Result<ApiResponse, ApiError> {
    let post = conn.run(move |c| crud::posts::find_one(c, id)).await?;

    let post_json = post.to_json();

    Ok(ApiResponse {
        status: "success",
        data: json!(post_json),
    })
}

#[get("/posts?<title>&<published>&<limit>&<published_date>")]
pub async fn fetch_posts(
    conn: DBPool,
    title: String,
    published: bool,
    limit: i64,
    published_date: String,
) -> Result<ApiResponse, ApiError> {
    let posts = conn
        .run(move |c| crud::posts::find_many(c, title, published, limit, published_date))
        .await?;

    if posts.is_empty() {
        return Err(ApiError::PostNotFound);
    }

    let posts_vec = posts
        .into_iter()
        .map(|p| p.to_json())
        .collect::<Vec<PostJson>>();

    Ok(ApiResponse {
        status: "success",
        data: json!(posts_vec),
    })
}

#[post("/posts", format = "json", data = "<post>")]
pub async fn create_post(
    conn: DBPool,
    post: Json<NewPostJson>,
    auth: Result<AuthenticatedUser, ApiError>,
) -> Result<ApiResponse, ApiError> {
    auth?;

    conn.run(move |c| crud::posts::new(c, &post.body, &post.summary))
        .await?;

    Ok(ApiResponse {
        status: "success",
        data: Value::Null,
    })
}

#[delete("/posts/<id>")]
pub async fn delete_post(
    conn: DBPool,
    id: i32,
    auth: Result<AuthenticatedUser, ApiError>,
) -> Result<ApiResponse, ApiError> {
    auth?;

    conn.run(move |c| crud::posts::delete(c, id)).await?;

    Ok(ApiResponse {
        status: "success",
        data: Value::Null,
    })
}
