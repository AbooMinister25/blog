use crate::models::posts::Post;
use crate::schema::posts::dsl::*;
use crate::util::response::ErrorKind;
use diesel::prelude::*;

/// Fetch a single post
///
/// Uses the given `post_id` to fetch and retrieve a single
/// post, in the form of the `Post` struct.
///
/// # Errors
/// Return an `ErrorKind::PostLoadError` if an error occurred while loading
/// the post from the database, or `ErrorKind::PostNotFound` if
/// the post was not found.
pub fn find_one(conn: &PgConnection, post_id: i32) -> Result<Post, ErrorKind> {
    let post = posts
        .filter(id.eq(post_id))
        .first::<Post>(conn)
        .optional()
        .map_err(ErrorKind::PostLoadError)?;

    if let Some(p) = post {
        return Ok(p);
    }

    Err(ErrorKind::PostNotFound)
}
