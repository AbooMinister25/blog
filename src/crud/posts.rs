use crate::errors::ApiError;
use crate::markdown;
use crate::models::post::{NewPost, Post};
use crate::schema::posts::dsl::*;
use crate::DATE_FORMAT;
use chrono::NaiveDateTime;
use diesel::prelude::*;

pub fn find_one(conn: &PgConnection, post_id: i32) -> Result<Post, ApiError> {
    let post = posts
        .filter(id.eq(post_id))
        .limit(1)
        .first::<Post>(conn)
        .optional()
        .map_err(ApiError::PostLoadError)?;

    if let Some(p) = post {
        return Ok(p);
    }

    Err(ApiError::PostNotFound)
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
        if p_date != "any" {
            return Err(ApiError::DateParsingError(p_date));
        }
    }

    let mut query = posts.into_boxed();

    if p_title != "any" {
        query = query.filter(title.eq(p_title))
    }

    if p_date != "any" {
        query = query.filter(published_date.eq(parsed_dt.unwrap()))
    }

    if limit != 1000 {
        query = query.limit(limit);
    }

    query
        .filter(published.eq(p_published))
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
