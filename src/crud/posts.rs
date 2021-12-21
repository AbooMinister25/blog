use crate::errors::ApiError;
use crate::models::post::Post;
use crate::schema::posts::dsl::*;
use diesel::prelude::*;

pub fn find_one(conn: &PgConnection, post_id: i32) -> Result<Vec<Post>, ApiError> {
    let posts_vec = posts
        .filter(id.eq(post_id))
        .limit(1)
        .load::<Post>(conn)
        .map_err(ApiError::PostLoadError)?;

    Ok(posts_vec)
}
