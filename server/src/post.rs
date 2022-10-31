use crate::connection::DbConn;
use chrono::NaiveDateTime;
use color_eyre::eyre::Result;
use lol_html::{element, html_content::TextType, rewrite_str, text, RewriteStrSettings};
use serde::Serialize;
use std::cell::RefCell;

const DATE_FORMAT: &str = "%b %e, %Y";

#[derive(Serialize)]
pub struct Post {
    pub title: String,
    pub content: String,
    pub summary: String,
    pub timestamp: String,
    pub tags: Vec<String>,
}

impl Post {
    pub fn new(
        title: String,
        content: String,
        summary: String,
        timestamp: String,
        tags: Vec<String>,
    ) -> Self {
        Self {
            title,
            content,
            summary,
            timestamp,
            tags,
        }
    }
}

// Fetch the latest ten posts from the database
pub fn posts_from_database(conn: &DbConn, offset: i32) -> Vec<Post> {
    // Fetch posts from database
    let mut stmt = conn
        .prepare(
            "SELECT title, rendered_content, timestamp, tags FROM posts ORDER BY id DESC LIMIT 10 OFFSET (?)",
        )
        .expect("Error when fetching posts from database");

    // Load into an iterator of `Post`s
    let posts_iter = stmt
        .query_map([offset], |row| {
            let tags_str: String = row.get(3)?;
            let summary_str: String = row.get(1)?;
            let date: NaiveDateTime = row.get(2)?;

            Ok(Post::new(
                row.get(0)?,
                row.get(1)?,
                get_summary(&summary_str).expect("Error while rewriting HTML"),
                date.format(DATE_FORMAT).to_string(),
                serde_json::from_str(&tags_str)
                    .map_err(|_| rusqlite::types::FromSqlError::InvalidType)?,
            ))
        })
        .expect("Error while fetching posts from database");

    // Collect iterator into a vec
    let mut posts = vec![];
    for post in posts_iter {
        posts.push(post.expect("Error while forming post"))
    }

    posts
}

// Truncate first part of post's content to generate summary
fn get_summary(content: &str) -> Result<String> {
    let character_count = RefCell::new(0);
    let mut summary = String::new();
    let mut skip = false;

    let element_content_handlers = vec![
        element!("*", |el| {
            if *character_count.borrow() > 150 {
                skip = true;
            }

            if skip {
                el.remove();
            }

            Ok(())
        }),
        text!("*", |text| {
            if matches!(text.text_type(), TextType::Data) {
                let text_str = text.as_str();
                let mut cc = character_count.borrow_mut();
                *cc += text_str.len();
                summary.push_str(text_str);
            }

            Ok(())
        }),
    ];

    let truncated = rewrite_str(
        content,
        RewriteStrSettings {
            element_content_handlers,
            ..RewriteStrSettings::default()
        },
    )?;

    Ok(truncated)
}
