#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]

pub mod series_index;

use std::{
    fs,
    path::{Path, PathBuf},
};

use color_eyre::{eyre::ContextCompat, Result};
use entry::{filesystem::ensure_directory, summary::get_summary, BuildStatus, Entry, DATE_FORMAT};
use markdown::Document;
use rusqlite::Connection;
use sql::{
    get_hash, get_posts, insert_content, insert_series, insert_tagmaps, insert_tags,
    update_content, update_hash, For,
};
use tera::{Context, Tera};
use tracing::debug;

// Represents a possible content type
#[derive(Debug)]
pub enum ContentType {
    Post,
    Series,
    Index,
}

impl ContentType {
    pub fn directory(&self) -> PathBuf {
        match self {
            Self::Post => Path::new("posts/").to_owned(),
            Self::Series => Path::new("series/").to_owned(),
            Self::Index => Path::new(".").to_owned(),
        }
    }

    pub const fn template_name(&self) -> &str {
        match self {
            Self::Post => "post.html.tera",
            Self::Series => "series.html.tera",
            Self::Index => "index.html.tera",
        }
    }
}

impl std::fmt::Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Post => write!(f, "post"),
            Self::Series => write!(f, "series"),
            Self::Index => write!(f, "index"),
        }
    }
}

// Represents a blog entry in the `contents/` directory.
#[derive(Debug)]
pub struct BlogContent {
    pub path: PathBuf,
}

impl BlogContent {
    fn content_type(&self) -> Result<ContentType> {
        let parent = self
            .path
            .parent()
            .context("Path should have a parent directory")?;
        let in_subdir = parent != Path::new("./contents");

        if self
            .path
            .file_name()
            .context("Path shouldn't terminate in ..")?
            == "index.md"
        {
            return if in_subdir {
                Ok(ContentType::Series)
            } else {
                Ok(ContentType::Index)
            };
        }

        Ok(ContentType::Post)
    }
}

impl Entry for BlogContent {
    #[tracing::instrument]
    fn from_file(path: PathBuf) -> Self {
        Self { path }
    }

    #[tracing::instrument]
    fn build_status(&self, conn: &Connection) -> Result<BuildStatus> {
        let markdown_hash = self.hash()?;
        let hashes = get_hash(conn, &self.path, For::CONTENT)?;

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
    fn build(&self, conn: &Connection, tera: &tera::Tera, status: BuildStatus) -> Result<()> {
        let content_type = self.content_type()?;
        ensure_directory(Path::new("public/").join(content_type.directory()))?;

        match status {
            BuildStatus::New(markdown_hash) => {
                debug!("Building entry at {:?} (new)", self.path);

                let parsed_document = Document::from_file(&self.path)?;
                insert_tags(conn, &parsed_document.frontmatter.tags)?;
                insert_content(
                    conn,
                    &parsed_document.frontmatter.title,
                    &self.path,
                    &content_type.to_string(),
                    &markdown_hash,
                    &parsed_document.content,
                    parsed_document.date,
                )?;
                insert_tagmaps(conn, &self.path, &parsed_document.frontmatter.tags)?;

                if let ContentType::Series = content_type {
                    insert_series(conn, &parsed_document.frontmatter.title)?;
                }

                let summary = get_summary(&parsed_document.content)?;
                render(conn, tera, &summary, parsed_document, content_type)?;
                debug!("Built entry");
            }
            BuildStatus::Changed(markdown_hash) => {
                debug!("Building entry at {:?} (changed)", self.path);
                update_hash(conn, &markdown_hash, &self.path, For::CONTENT)?;

                let parsed_document = Document::from_file(&self.path)?;
                insert_tags(conn, &parsed_document.frontmatter.tags)?;
                update_content(
                    conn,
                    &parsed_document.frontmatter.title,
                    &parsed_document.content,
                    parsed_document.date,
                    &self.path,
                )?;
                insert_tagmaps(conn, &self.path, &parsed_document.frontmatter.tags)?;

                if let ContentType::Series = content_type {
                    insert_series(conn, &parsed_document.frontmatter.title)?;
                }

                let summary = get_summary(&parsed_document.content)?;
                render(conn, tera, &summary, parsed_document, content_type)?;
                debug!("Built entry");
            }
            BuildStatus::Unchanged => (), // Don't do anything if the file was unchanged
        }

        Ok(())
    }
}

#[tracing::instrument(skip(tera))]
fn render(
    conn: &Connection,
    tera: &Tera,
    summary: &str,
    document: Document,
    content_type: ContentType,
) -> Result<()> {
    // Create the file
    let to_path = Path::new("public/")
        .join(content_type.directory())
        .join(format!("{}.html", document.frontmatter.title));

    let file = fs::File::create(to_path)?;

    // Insert context for the template
    let mut context = Context::new();
    context.insert("title", &document.frontmatter.title);
    context.insert("tags", &document.frontmatter.tags.join(", "));
    context.insert("date", &document.date.format(DATE_FORMAT).to_string());
    context.insert("toc", &document.toc);
    context.insert("markup", &document.content);
    context.insert("summary", summary);

    if matches!(content_type, ContentType::Index) {
        let posts = get_posts(conn, 10, "post")?;
        context.insert("posts", &posts);
    }

    // Render the template
    tera.render_to(content_type.template_name(), &context, file)?;

    Ok(())
}
