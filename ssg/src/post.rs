use crate::markdown::{parse, Frontmatter, ParsedPost};
use crate::DATE_FORMAT;
use color_eyre::eyre::{eyre, Result};
use ignore::Walk;
use rusqlite::{named_params, Connection};
use std::fs;
use std::path::PathBuf;
use tera::{Context, Tera};
use tracing::info;

// Whether a post should be built
enum ToBuild {
    Nonexistent(String),
    Exist,
    No,
}

pub fn build_posts(
    conn: &Connection,
    tera: &Tera,
    output_dir: &str,
    html_input_dir: &str,
) -> Result<()> {
    let posts_to_build = get_posts_to_build(conn, html_input_dir)?;
    posts_to_build
        .iter()
        .map(|(path, to_build)| Ok((to_build, build_markdown(path, tera, output_dir)?, path)))
        .map(|r| {
            // If building the file wasn't an error, go ahead and insert it into the database.
            r.and_then(|(to_build, parsed_post, path)| {
                match to_build {
                    ToBuild::Nonexistent(markdown_hash) => {
                        conn.execute(
                            "INSERT INTO posts
                            (title, path, hash, rendered_content, timestamp, tags)
                            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                        ",
                            (
                                &parsed_post.frontmatter.title,
                                &path.to_str().ok_or_else(|| {
                                    eyre!("Error while converting path to string")
                                })?,
                                &markdown_hash,
                                &parsed_post.content,
                                &parsed_post.date,
                                &serde_json::to_string(&parsed_post.frontmatter.tags)?,
                            ),
                        )?;
                    }
                    ToBuild::Exist => {
                        conn.execute(
                            "
                        UPDATE posts 
                        SET title = (:title),
                            rendered_content = (:content),
                            timestamp = (:timestamp)
                            tags = (:tags)
                        WHERE path = (:path)
                        ",
                        named_params! {
                            ":title": &parsed_post.frontmatter.title,
                            ":content": &parsed_post.content,
                            ":timestamp": &parsed_post.date,
                            ":tags": &serde_json::to_string(&parsed_post.frontmatter.tags)?,
                            ":path": &path.to_str().ok_or_else(|| eyre!("Error while converting path to string"))?
                        }
                        )?;
                    }
                    ToBuild::No => unreachable!("Should never be ToBuild::No"),
                }

                Ok(())
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(())
}

fn get_posts_to_build(conn: &Connection, input_dir: &str) -> Result<Vec<(PathBuf, ToBuild)>> {
    // Collect all paths in the input directory folder, filter out directories.
    let paths = Walk::new(input_dir)
        .filter_map(std::result::Result::ok)
        .map(ignore::DirEntry::into_path)
        .filter(|path| !path.is_dir())
        .collect::<Vec<_>>();

    info!("Found {} files in {}", paths.len(), input_dir);

    let mut rendering = 0;
    let mut skipping = 0;

    // Push any paths which have either been new or edited.
    let mut posts_to_build = vec![];
    for path in paths {
        let to_build = to_build(conn, &path)?;
        if matches!(to_build, ToBuild::Nonexistent(_) | ToBuild::Exist) {
            posts_to_build.push((path, to_build));
            rendering += 1;
        } else {
            skipping += 1;
        }
    }

    info!("Building {rendering} files");
    info!("{skipping} files left unchanged, skipping");

    Ok(posts_to_build)
}

fn to_build(conn: &Connection, path: &PathBuf) -> Result<ToBuild> {
    let raw_markdown = fs::read_to_string(path)?;
    // Hash markdown, format as string.
    let markdown_hash = format!("{:016x}", seahash::hash(raw_markdown.as_bytes()));
    let mut stmt = conn.prepare("SELECT hash FROM posts WHERE path = :path")?;
    let path_str = path
        .to_str()
        .ok_or_else(|| eyre!("Error while converting path to string"))?;

    // Get the hashes found for this path
    let hashes_iter = stmt.query_map(&[(":path", path_str)], |row| row.get(0))?;
    let mut hashes: Vec<String> = Vec::new();
    for hash in hashes_iter {
        hashes.push(hash?);
    }

    // If the hashes are empty, a new file was created. If it is different from the new
    // hash, then the files contents changed. Otherwise the file was not changed.
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

fn build_markdown(path: &PathBuf, tera: &Tera, output_dir: &str) -> Result<ParsedPost> {
    // Parse the file
    let parsed_post = parse_file(path)?;
    let frontmatter = &parsed_post.frontmatter;
    let file = fs::File::create(format!("{output_dir}/{}.html", frontmatter.title))?;

    render_template(&parsed_post, frontmatter, tera, file)?;
    Ok(parsed_post)
}

fn parse_file(path: &PathBuf) -> Result<ParsedPost> {
    let markdown = fs::read_to_string(path)?;
    let parsed_post = parse(&markdown)?;

    Ok(parsed_post)
}

fn render_template(
    parsed_post: &ParsedPost,
    frontmatter: &Frontmatter,
    tera: &Tera,
    file: fs::File,
) -> Result<()> {
    let mut context = Context::new();
    // Insert context for the template
    context.insert("title", &frontmatter.title);
    context.insert("tags", &frontmatter.tags.join(", "));
    context.insert("date", &parsed_post.date.format(DATE_FORMAT).to_string());
    context.insert("toc", &parsed_post.toc);
    context.insert("markup", &parsed_post.content);
    context.insert("summary", &frontmatter.summary);

    tera.render_to("post.html", &context, file)?;
    Ok(())
}
