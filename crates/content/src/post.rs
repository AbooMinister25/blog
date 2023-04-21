use color_eyre::eyre::{ContextCompat, Result};
use entry::filesystem::ensure_directory;
use entry::summary::get_summary;
use entry::{BuildStatus, Entry, DATE_FORMAT};
use markdown::Document;
use rusqlite::Connection;
use sql::{insert_post, insert_tagmaps, insert_tags, update_post, MapFor};
use std::{fs, path::PathBuf};
use tera::{Context, Tera};
use tracing::debug;

// A blog post, represented by a markdown file in the `contents/` directory.
#[derive(Debug)]
pub struct Post {
    pub path: PathBuf,
}

impl Entry for Post {
    #[tracing::instrument]
    fn from_file(path: PathBuf) -> Self {
        Self { path }
    }

    #[tracing::instrument]
    fn build_status(&self, conn: &Connection) -> Result<BuildStatus> {
        let markdown_hash = self.hash()?;

        let mut stmt = conn.prepare("SELECT hash FROM posts WHERE path = :path")?;
        let path_str = self
            .path
            .to_str()
            .context("Error while converting path to string")?;

        // Get the hashes found for this path
        let hashes_iter = stmt.query_map(&[(":path", path_str)], |row| row.get(0))?;
        let mut hashes: Vec<String> = Vec::new();
        for hash in hashes_iter {
            hashes.push(hash?);
        }

        // If the hashes are empty, a new file was created. If it was different from the new
        // hash, then the file contents changed. Otherwise the file was not changed.
        let build = if hashes.is_empty() {
            BuildStatus::New(markdown_hash)
        } else if hashes[0] != markdown_hash {
            BuildStatus::Changed(markdown_hash)
        } else {
            BuildStatus::Unchanged
        };

        Ok(build)
    }

    #[tracing::instrument]
    fn hash(&self) -> Result<String> {
        let raw_markdown = fs::read_to_string(&self.path)?;
        // Hash markdown, format as string
        Ok(format!("{:016x}", seahash::hash(raw_markdown.as_bytes())))
    }

    #[tracing::instrument(skip(tera))]
    fn build(&self, conn: &Connection, tera: &Tera, status: BuildStatus) -> Result<()> {
        ensure_directory("public/posts")?;

        match status {
            BuildStatus::New(markdown_hash) => {
                debug!(
                    "Building post at {}",
                    self.path.to_str().context("Path should be valid unicode")?
                );

                let parsed_document = Document::from_file(&self.path)?;
                insert_tags(conn, &parsed_document.frontmatter.tags)?;
                insert_post(
                    conn,
                    &parsed_document.frontmatter.title,
                    &self.path,
                    &markdown_hash,
                    &parsed_document.content,
                    parsed_document.date,
                )?;
                insert_tagmaps(
                    conn,
                    &self.path,
                    &parsed_document.frontmatter.tags,
                    MapFor::Post,
                )?;

                let summary = get_summary(&parsed_document.content)?;
                render_post(tera, &summary, parsed_document)?;
                debug!("Built post");
            }
            BuildStatus::Changed(markdown_hash) => {
                debug!(
                    "Building post at {}",
                    self.path.to_str().context("Path should be valid unicode")?
                );

                conn.execute(
                    "UPDATE posts SET hash = (:hash) WHERE path = (:path)",
                    &[
                        (":hash", &markdown_hash),
                        (
                            ":path",
                            &self
                                .path
                                .to_str()
                                .context("Path should be valid unicode")?
                                .to_string(),
                        ),
                    ],
                )?;

                let parsed_document = Document::from_file(&self.path)?;
                insert_tags(conn, &parsed_document.frontmatter.tags)?;
                update_post(
                    conn,
                    &parsed_document.frontmatter.title,
                    &parsed_document.content,
                    parsed_document.date,
                    &self.path,
                )?;
                insert_tagmaps(
                    conn,
                    &self.path,
                    &parsed_document.frontmatter.tags,
                    MapFor::Post,
                )?;

                let summary = get_summary(&parsed_document.content)?;
                render_post(tera, &summary, parsed_document)?;
                debug!("Built post");
            }
            BuildStatus::Unchanged => (), // Don't do anything if the file was unchanged
        }

        Ok(())
    }
}

#[tracing::instrument]
fn render_post(tera: &Tera, summary: &str, document: Document) -> Result<()> {
    // Create the file
    let file = fs::File::create(format!("public/posts/{}.html", document.frontmatter.title))?;

    // Insert context for the template
    let mut context = Context::new();
    context.insert("title", &document.frontmatter.title);
    context.insert("tags", &document.frontmatter.tags.join(", "));
    context.insert("date", &document.date.format(DATE_FORMAT).to_string());
    context.insert("toc", &document.toc);
    context.insert("markup", &document.content);
    context.insert("summary", summary);

    // Render the template
    tera.render_to("post.html.tera", &context, file)?;

    Ok(())
}
