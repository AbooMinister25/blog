use crate::crud;
use crate::util::response::{ApiError, ApiResponse};
use crate::DBPool;
use serde_json::json;

#[get("/posts/<id>")]
pub async fn fetch_post(conn: DBPool, id: i32) -> Result<ApiResponse, ApiError> {
    let post = conn.run(move |c| crud::posts::find_one(c, id)).await?;
    Ok(ApiResponse { data: json!(post) })
}
