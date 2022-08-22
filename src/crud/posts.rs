use crate::models::posts::{NewPost, Post};
use crate::schema::posts::dsl::*;
use crate::util::markdown;
use crate::util::response::ErrorKind;
use crate::DATE_FORMAT;
use chrono::NaiveDateTime;
use diesel::prelude::*;

/// Fetch a single post.
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

/// Fetch multiple posts
///
/// Fetch multiple posts based on the given parameters.
///
/// # Errors
/// Returns an `ErrorKind::DateParsingError` if an error occurred while
/// parsing the given date, or an `ErrorKind::PostLoadError` if there was
/// a problem while loading the post from the database.
pub fn find_many(
    conn: &PgConnection,
    post_title: Option<String>,
    post_published: Option<bool>,
    post_timestamp: Option<String>,
    limit: i64,
) -> Result<Vec<Post>, ErrorKind> {
    let mut query = posts.into_boxed();

    if let Some(date) = post_timestamp {
        let parsed_datetime = NaiveDateTime::parse_from_str(&date, DATE_FORMAT)
            .map_err(|_| ErrorKind::DateParsingError(date))?;
        query = query.filter(published_at.eq(parsed_datetime));
    }

    if let Some(t) = post_title {
        query = query.filter(title.eq(t));
    }

    if let Some(p) = post_published {
        query = query.filter(published.eq(p));
    }

    if limit < 1000 {
        query = query.limit(limit);
    }

    query
        .order(id.desc())
        .load::<Post>(conn)
        .map_err(ErrorKind::PostLoadError)
}

/// Create and insert a new post into the database
///
/// # Errors
/// Return an `ErrorKind::PostInsertionError` if an error ocurred while
/// inserting a post into the database. In the event that parsing of the
/// post's markdown fails, any of the errors returned by `blog::util::markdown::parse`
/// can be returned.
pub fn new(conn: &PgConnection, post_body: &str, post_summary: &str) -> Result<(), ErrorKind> {
    let parsed_post = markdown::parse(post_body)?;
    let new_post = NewPost::new(
        parsed_post.title,
        parsed_post.content,
        post_summary.to_string(),
        true,
        parsed_post.date,
        Some(parsed_post.tags),
    );

    diesel::insert_into(posts)
        .values(&new_post)
        .execute(conn)
        .map_err(ErrorKind::PostInsertionError)?;

    Ok(())
}

/// Delete a post from the database
///
/// # Errors
/// Return an `ErrorKind::PostDeletionError` if deleting the post
/// from the database failed.
pub fn delete(conn: &PgConnection, post_id: i32) -> Result<(), ErrorKind> {
    diesel::delete(posts.filter(id.eq(post_id)))
        .execute(conn)
        .map_err(ErrorKind::PostDeletionError)?;
    Ok(())
}
