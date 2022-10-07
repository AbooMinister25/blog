use crate::assets::process_assets;
use crate::post::build_posts;
use crate::stylesheets::compile_stylesheets;
use color_eyre::eyre::Result;
use lol_html::{element, html_content::TextType, rewrite_str, text, RewriteStrSettings};
use rusqlite::Connection;
use serde::Serialize;
use std::cell::RefCell;
use std::fs;
use std::path::Path;
use tera::{Context, Tera};
use tracing::info;

#[derive(Serialize)]
struct Entry {
    pub title: String,
    pub content: String,
    pub summary: String,
    pub tags: Vec<String>,
}

#[tracing::instrument(skip(
    tera,
    conn,
    output_dir,
    css_output_dir,
    html_input_dir,
    scss_input_dir
))]
pub fn build(
    conn: Connection,
    tera: &Tera,
    output_dir: String,
    css_output_dir: String,
    html_input_dir: String,
    scss_input_dir: String,
) -> Result<()> {
    info!("Creating directories");
    create_directories(&output_dir, &css_output_dir)?;
    info!("Compiling stylesheets");
    compile_stylesheets(&conn, &css_output_dir, &scss_input_dir)?;
    info!("Minimizing assets");
    process_assets(&conn)?;
    info!("Building posts");
    build_posts(&conn, tera, &output_dir, &html_input_dir)?;
    info!("Rendering Index");
    render_index(&conn, tera, &output_dir)?;

    Ok(())
}

fn create_directories(output_dir: &str, css_output_dir: &str) -> Result<()> {
    if !Path::new(output_dir).exists() {
        info!("Creating {output_dir}");
        fs::create_dir(output_dir)?;
    }
    if !Path::new(css_output_dir).exists() {
        info!("Creating {css_output_dir}");
        fs::create_dir(css_output_dir)?;
    }

    Ok(())
}

fn render_index(conn: &Connection, tera: &Tera, output_dir: &str) -> Result<()> {
    // Fetch posts from database
    let mut stmt = conn.prepare("SELECT title, rendered_content, tags FROM posts LIMIT 10")?;
    let posts_iter = stmt.query_map([], |row| {
        let tags_str: String = row.get(2)?;
        let summary_str: String = row.get(1)?;
        Ok(Entry {
            title: row.get(0)?,
            content: row.get(1)?,
            summary: get_summary(&summary_str).expect("Error while writing HTML"),
            tags: serde_json::from_str(&tags_str)
                .map_err(|_| rusqlite::types::FromSqlError::InvalidType)?,
        })
    })?;

    let mut posts = vec![];
    for post in posts_iter {
        posts.push(post?);
    }

    let mut context = Context::new();
    context.insert("posts", &posts);

    let file = fs::File::create(format!("{output_dir}/index.html"))?;
    tera.render_to("index.html", &context, file)?;

    Ok(())
}

fn get_summary(content: &str) -> Result<String> {
    let character_count = RefCell::new(0);
    let mut summary = String::new();
    let mut skip = false;

    let element_content_handlers = vec![
        element!("*", |el| {
            if *character_count.borrow() > 100 {
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
