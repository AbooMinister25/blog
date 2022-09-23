use crate::markdown::{parse, ParsedPost};
use crate::stylesheets::compile_stylesheets;
use chrono::prelude::*;
use color_eyre::eyre::{eyre, Result};
use ignore::Walk;
use rayon::prelude::*;
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;
use tera::{Context, Tera};
use tracing::info;

const DATE_FORMAT: &str = "%b %e, %Y";

enum ToBuild {
    Nonexistent(String),
    Exist,
    No,
}

#[tracing::instrument(skip(tera, conn))]
pub fn build(conn: Connection, tera: &Tera) -> Result<()> {
    info!("Compiling stylesheets");
    compile_stylesheets(&conn)?;

    let paths = Walk::new("contents/")
        .filter_map(std::result::Result::ok)
        .map(ignore::DirEntry::into_path)
        .filter(|path| !path.is_dir())
        .collect::<Vec<PathBuf>>();

    info!("Found {} files in contents/", paths.len());

    let mut rendered = 0;
    let mut skipped = 0;

    for path in paths {
        let to_build = to_build(&conn, &path)?;

        match to_build {
            ToBuild::Nonexistent(markdown_hash) => {
                let parsed_post = build_markdown(&path, tera)?;
                conn.execute(
                    "INSERT INTO posts
                    (title, path, hash, tags)
                    VALUES (?1, ?2, ?3, ?4)
                ",
                    (
                        &parsed_post.frontmatter.title,
                        &path
                            .to_str()
                            .ok_or_else(|| eyre!("Error while converting path to string"))?,
                        &markdown_hash,
                        &serde_json::to_string(&parsed_post.frontmatter.tags)?,
                    ),
                )?;

                rendered += 1;
            }
            ToBuild::Exist => {
                build_markdown(&path, tera)?;
                rendered += 1;
            }
            ToBuild::No => skipped += 1,
        }
    }

    info!("Built {rendered} files");
    info!("{skipped} files left unchanged, skipping");

    Ok(())
}

fn parse_file(path: &PathBuf) -> Result<ParsedPost> {
    let markdown = fs::read_to_string(path)?;
    let parsed_post = parse(&markdown)?;

    Ok(parsed_post)
}

fn build_markdown(path: &PathBuf, tera: &Tera) -> Result<ParsedPost> {
    let parsed_post = parse_file(path)?;
    let frontmatter = &parsed_post.frontmatter;
    let file = fs::File::create(format!("public/{}.html", frontmatter.title))?;

    render_template(
        &parsed_post.content,
        &frontmatter.title,
        &frontmatter.tags,
        parsed_post.date,
        tera,
        file,
    )?;
    Ok(parsed_post)
}

fn render_template(
    markup: &str,
    title: &str,
    tags: &[String],
    date: DateTime<Utc>,
    tera: &Tera,
    file: fs::File,
) -> Result<()> {
    let mut context = Context::new();
    context.insert("title", title);
    context.insert("tags", &tags.join(", "));
    context.insert("date", &date.format(DATE_FORMAT).to_string());
    context.insert("markup", markup);

    tera.render_to("post.html", &context, file)?;
    Ok(())
}

fn to_build(conn: &Connection, path: &PathBuf) -> Result<ToBuild> {
    let raw_markdown = fs::read_to_string(path)?;
    let markdown_hash = format!("{:016x}", seahash::hash(raw_markdown.as_bytes()));
    let mut stmt = conn.prepare("SELECT hash FROM posts WHERE path = :path")?;
    let path_str = path
        .to_str()
        .ok_or_else(|| eyre!("Error while converting path to string"))?;

    let hashes_iter = stmt.query_map(&[(":path", path_str)], |row| row.get(0))?;
    let mut hashes: Vec<String> = Vec::new();
    for hash in hashes_iter {
        hashes.push(hash?);
    }

    let build = if hashes.is_empty() {
        ToBuild::Nonexistent(markdown_hash)
    } else if hashes[0] != markdown_hash {
        conn.execute(
            "UPDATE posts SET hash = (:hash) WHERE path = (:path)",
            &[(":hash", &markdown_hash), (":path", &path_str.to_string())],
        )?;
        ToBuild::Exist
    } else {
        ToBuild::No
    };

    Ok(build)
}
