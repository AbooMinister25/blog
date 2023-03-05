use std::fs;
use std::path::{Path, PathBuf};

use crate::entry::{BuildStatus, Entry};
use crate::markdown::Document;
use crate::sql::{insert_series, insert_tagmaps, insert_tags, MapFor};
use crate::utils::{ensure_directory, get_summary};
use crate::DATE_FORMAT;
use color_eyre::eyre::{ContextCompat, Result};
use rusqlite::Connection;
use tera::{Context, Tera};
use tracing::debug;

// A series of blog posts, represented by a directory inside the `contents/` directory.
#[derive(Debug)]
pub struct Series {
    pub path: PathBuf,
}

impl Entry for Series {
    #[tracing::instrument]
    fn from_file(path: PathBuf) -> Self {
        Self { path }
    }

    #[tracing::instrument]
    fn build_status(&self, conn: &rusqlite::Connection) -> Result<BuildStatus> {
        let markdown_hash = self.hash()?;

        let mut stmt = conn.prepare("SELECT hash FROM series WHERE path = :path")?;
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

        // If the hashes are empty, a new directory was created. If it was different from the
        // new hash, then the file contents changed. Otherwise the file was not changed.
        let build = if hashes.is_empty() {
            BuildStatus::New(markdown_hash)
        } else if hashes[0] != markdown_hash {
            conn.execute(
                "UPDATE series SET hash = (:hash) WHERE path = (:path)",
                &[(":hash", &markdown_hash), (":path", &path_str.to_string())],
            )?;
            BuildStatus::Changed
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

    #[tracing::instrument]
    fn build(&self, conn: &Connection, tera: &Tera) -> Result<()> {
        ensure_directory(Path::new("public/series"))?;
        debug!(
            "Building series at {}",
            self.path.to_str().context("Path should be valid unicode")?
        );

        let parsed_document = Document::from_file(&self.path)?;

        insert_tags(conn, &parsed_document.frontmatter.tags)?;
        insert_series(
            conn,
            &parsed_document.frontmatter.title,
            &self.path,
            &parsed_document.content,
            parsed_document.date,
        )?;
        insert_tagmaps(
            conn,
            &self.path,
            &parsed_document.frontmatter.tags,
            MapFor::Series,
        )?;

        let summary = get_summary(&parsed_document.content)?;
        render_series(tera, &summary, parsed_document)?;
        debug!("Built series");

        Ok(())
    }
}

#[tracing::instrument]
fn render_series(tera: &Tera, summary: &str, document: Document) -> Result<()> {
    // Create the file
    let file = fs::File::create(format!("public/series/{}.html", document.frontmatter.title))?;

    // Insert context for the template
    let mut context = Context::new();
    context.insert("title", &document.frontmatter.title);
    context.insert("tags", &document.frontmatter.tags.join(", "));
    context.insert("date", &document.date.format(DATE_FORMAT).to_string());
    context.insert("markup", &document.content);
    context.insert("summary", summary);

    // Render the template
    tera.render_to("series.html.tera", &context, file)?;

    Ok(())
}
