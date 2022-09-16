use crate::markdown::{parse, HeaderValue, ParsedPost};
use color_eyre::eyre::{eyre, Result};
use ignore::Walk;
use rusqlite::Connection;
use seahash::hash;
use std::fs;
use std::path::{Path, PathBuf};
use tera::{Context, Tera};

#[tracing::instrument]
pub fn build(conn: Connection, tera: &Tera) -> Result<()> {
    let paths = Walk::new("contents/")
        .filter_map(std::result::Result::ok)
        .map(ignore::DirEntry::into_path)
        .collect::<Vec<PathBuf>>();

    for path in paths {
        if path.is_dir() {
            continue;
        }

        let parsed_post = parse_file(path)?;
        let title = parsed_post
            .headers
            .get("title")
            .ok_or_else(|| eyre!("Missing required header `title`"))?;
        let tags = parsed_post
            .headers
            .get("tags")
            .ok_or_else(|| eyre!("Missing required header `tags`"))?;

        let (title, tags) = match (title, tags) {
            (HeaderValue::Single(s), HeaderValue::List(l)) => Ok((s, l)),
            _ => Err(eyre!("Invalid types for headers `title` and `tags`")),
        }?;

        let file = fs::File::create(format!("public/{}.html", title))?;
        render_template(&parsed_post.content, title, tera, file)?;
    }

    Ok(())
}

fn parse_file(path: PathBuf) -> Result<ParsedPost> {
    let markdown = fs::read_to_string(path)?;
    let parsed_post = parse(&markdown)?;

    Ok(parsed_post)
}

fn render_template(markup: &str, title: &str, tera: &Tera, file: fs::File) -> Result<()> {
    let mut context = Context::new();
    context.insert("title", title);
    context.insert("markup", markup);

    tera.render_to("page.html", &context, file)?;
    Ok(())
}
