use crate::crud;
use crate::errors::ApiError;
use crate::response::ApiResponse;
use crate::DBPool;

#[get("/posts/<id>")]
pub async fn fetch_post(conn: DBPool, id: i32) -> Result<ApiResponse, ApiError> {
    let mut posts = conn.run(move |c| crud::posts::find_one(c, id)).await?;
    if posts.is_empty() {
        return Err(ApiError::PostNotFound(id));
    }

    let post_json = posts.pop().unwrap().to_json();

    Ok(ApiResponse {
        status: "success",
        data: json!(post_json),
    })
}

