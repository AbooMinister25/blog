use crate::connection::DbConn;
use crate::post::Post;
use chrono::NaiveDateTime;
use color_eyre::eyre::Result;
use lol_html::{element, html_content::TextType, rewrite_str, text, RewriteStrSettings};
use rocket::{fs::NamedFile, response::content::RawHtml, State};
use std::{cell::RefCell, path::Path};
use tera::{Context, Tera};

const DATE_FORMAT: &str = "%b %e, %Y";

#[get("/")]
pub async fn index() -> Option<NamedFile> {
    NamedFile::open(Path::new("./public/index.html")).await.ok()
}

#[get("/posts?<page>")]
pub async fn get_posts(tera: &State<Tera>, conn: DbConn, page: Option<i32>) -> RawHtml<String> {
    let mut stmt = conn
        .prepare(
            "SELECT title, rendered_content, timestamp, tags FROM posts ORDER BY id DESC LIMIT 10",
        )
        .expect("Error when fetching posts from database");
    let posts_iter = stmt
        .query_map([], |row| {
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

    let mut posts = vec![];
    for post in posts_iter {
        posts.push(post.expect("Error while forming post"));
    }

    let mut context = Context::default();
    context.insert("posts", &posts);
    let rendered = tera
        .render("posts.html.tera", &context)
        .expect("Error while rendering template");

    RawHtml(rendered)
}

#[get("/posts/<title>")]
pub async fn get_post(title: &str) -> Option<NamedFile> {
    NamedFile::open(Path::new("./public/").join(format!("{title}.html")))
        .await
        .ok()
}

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
