use crate::errors::ApiError;
use crate::markdown;
use crate::models::post::{NewPost, Post};
use crate::schema::posts::dsl::*;
use crate::DATE_FORMAT;
use chrono::NaiveDateTime;
use diesel::prelude::*;

pub fn find_one(conn: &PgConnection, post_id: i32) -> Result<Vec<Post>, ApiError> {
    let posts_vec = posts
        .filter(id.eq(post_id))
        .limit(1)
        .load::<Post>(conn)
        .map_err(ApiError::PostLoadError)?;

    Ok(posts_vec)
}

pub fn find_many(
    conn: &PgConnection,
    p_title: String,
    p_published: bool,
    limit: i64,
    p_date: String,
) -> Result<Vec<Post>, ApiError> {
    let parsed_dt = NaiveDateTime::parse_from_str(&p_date, DATE_FORMAT);

    if parsed_dt.is_err() {
        return Err(ApiError::DateParsingError(p_date));
    }

    posts
        .filter(title.eq(p_title))
        .filter(published.eq(p_published))
        .filter(published_date.eq(parsed_dt.unwrap()))
        .limit(limit)
        .load::<Post>(conn)
        .map_err(ApiError::PostLoadError)
}

pub fn new(conn: &PgConnection, p_body: &str, p_summary: &str) -> Result<(), ApiError> {
    let parsed_post = markdown::parse(p_body)?;
    let new_post = NewPost {
        title: parsed_post.title,
        body: parsed_post.content,
        summary: p_summary.to_string(),
        published_date: parsed_post.date,
        published: true,
    };

    diesel::insert_into(posts)
        .values(&new_post)
        .execute(conn)
        .map_err(ApiError::PostInsertionError)?;

    Ok(())
}

pub fn delete(conn: &PgConnection, post_id: i32) -> Result<(), ApiError> {
    diesel::delete(posts.filter(id.eq(post_id)))
        .execute(conn)
        .map_err(ApiError::PostDeletionError)?;

    Ok(())
}
