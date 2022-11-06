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
            "SELECT post_id, title, rendered_content, timestamp FROM posts ORDER BY post_id DESC LIMIT 10 OFFSET (?)",
        )
        .expect("Error when fetching posts from database");
    let mut tags_stmt = conn
        .prepare(
            "
        SELECT tags.name 
        FROM posts 
        INNER JOIN 
            tags_posts ON tags_posts.post_id = posts.post_id 
        INNER JOIN 
            tags ON tags_posts.tag_id = tags.tag_id 
        WHERE posts.post_id = (?)",
        )
        .expect("Error when querying database");

    // Load into an iterator of `Post`s
    let posts_iter = stmt
        .query_map([offset], |row| {
            let id: i32 = row.get(0)?;
            // let tags_str: String = row.get(4)?;
            let summary_str: String = row.get(2)?;
            let date: NaiveDateTime = row.get(3)?;

            // Fetch tags for the post
            let tags_iter = tags_stmt
                .query_map([id], |tags_row| {
                    let tag_name: String = tags_row.get(0)?;
                    Ok(tag_name)
                })
                .expect("Error while fetching posts from database");

            let mut tags = vec![];
            for tag in tags_iter {
                tags.push(tag.expect("Error while collecting tags"))
            }

            Ok(Post::new(
                row.get(1)?,
                row.get(2)?,
                get_summary(&summary_str).expect("Error while rewriting HTML"),
                date.format(DATE_FORMAT).to_string(),
                tags,
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
