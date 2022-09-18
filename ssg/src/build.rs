use crate::markdown::{parse, ParsedPost};
use crate::stylesheets::compile_stylesheets;
use color_eyre::eyre::{eyre, Result};
use ignore::Walk;
use rusqlite::Connection;
use std::fs;
use std::path::PathBuf;
use tera::{Context, Tera};
use tracing::info;

enum ToBuild {
    Nonexistent(String),
    Exist,
    No,
}

#[tracing::instrument(skip(tera))]
pub fn build(conn: Connection, tera: &Tera) -> Result<()> {
    info!("Compile stylesheets");
    compile_stylesheets()?;

    let paths = Walk::new("contents/")
        .filter_map(std::result::Result::ok)
        .map(ignore::DirEntry::into_path)
        .collect::<Vec<PathBuf>>();

    info!("Found {} files in contents/", paths.len());

    let mut rendered = 0;
    let mut skipped = 0;

    for path in paths {
        if path.is_dir() {
            continue;
        }
        let to_build = to_build(&conn, &path)?;

        match to_build {
            ToBuild::Nonexistent(markdown_hash) => {
                let parsed_post = parse_file(&path)?;
                let title = parsed_post.frontmatter.title;
                let file = fs::File::create(format!("public/{}.html", title))?;
                render_template(&parsed_post.content, &title, tera, file)?;

                conn.execute(
                    "INSERT INTO posts
                    (title, path, hash, summary, tags, published, published_at)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                ",
                    (
                        &title,
                        &path
                            .to_str()
                            .ok_or_else(|| eyre!("Error while converting path to string"))?,
                        &markdown_hash,
                        &parsed_post.frontmatter.summary,
                        &serde_json::to_string(&parsed_post.frontmatter.tags)?,
                        true,
                        parsed_post.date,
                    ),
                )?;

                rendered += 1;
            }
            ToBuild::Exist => {
                let parsed_post = parse_file(&path)?;
                let title = parsed_post.frontmatter.title;
                let file = fs::File::create(format!("public/{}.html", title))?;
                render_template(&parsed_post.content, &title, tera, file)?;

                rendered += 1;
            }
            ToBuild::No => skipped += 1,
        }
    }

    info!("Built {rendered} files");
    info!("No changes made to {skipped} files, skipped");

    Ok(())
}

fn parse_file(path: &PathBuf) -> Result<ParsedPost> {
    let markdown = fs::read_to_string(path)?;
    let parsed_post = parse(&markdown)?;

    Ok(parsed_post)
}

fn render_template(markup: &str, title: &str, tera: &Tera, file: fs::File) -> Result<()> {
    let mut context = Context::new();
    context.insert("title", title);
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
        info!("Skipping file, already rendered");
        ToBuild::No
    };

    Ok(build)
}
