use std::{
    fmt::Debug,
    fs,
    hash::Hash,
    path::{Path, PathBuf},
};

use color_eyre::{eyre::ContextCompat, Result};
use markdown::{Frontmatter, MarkdownRenderer};
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};
use tracing::trace;
use utils::fs::ensure_directory;

pub const DATE_FORMAT: &str = "%b %e, %Y";

/// Represents a single markdown page in the blog.
#[derive(Debug, Serialize, Deserialize, Eq, Clone)]
pub struct Page {
    pub path: PathBuf,
    #[serde(rename = "body")]
    pub raw_content: String,
    pub content: String,
    #[serde(flatten)]
    pub frontmatter: Frontmatter,
    pub hash: String,
}

impl Hash for Page {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.path.hash(state);
    }
}

impl PartialEq for Page {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Page {
    #[tracing::instrument]
    pub fn new(
        path: PathBuf,
        raw_content: String,
        content: String,
        frontmatter: Frontmatter,
        hash: String,
    ) -> Self {
        Self {
            path,
            raw_content,
            content,
            frontmatter,
            hash,
        }
    }
}

#[tracing::instrument(skip(renderer))]
pub fn render_page<T: AsRef<Path> + Debug>(
    tera: &Tera,
    renderer: &MarkdownRenderer,
    path: T,
    output_directory: T,
    raw_content: String,
    hash: &str,
) -> Result<Page> {
    trace!("Rendering post at {path:?}");

    let document = renderer.render(&raw_content)?;
    let out_path = out_path(&path, &output_directory, &document.frontmatter.title)?;
    ensure_directory(out_path.parent().context("Path should have a parent")?)?;

    trace!("Rendering post to {out_path:?}");

    let mut context = Context::new();
    context.insert("title", &document.frontmatter.title);
    context.insert("tags", &document.frontmatter.tags.join(", "));
    context.insert("series", &document.frontmatter.series);
    context.insert("date", &document.date.format(DATE_FORMAT).to_string());
    context.insert("toc", &document.toc);
    context.insert("markup", &document.content);
    context.insert("summary", &document.summary);

    let rendered_html = tera.render("post.html.tera", &context)?;
    fs::write(&out_path, &rendered_html)?;

    trace!("Rendered post at {:?}", out_path);

    Ok(Page::new(
        path.as_ref().to_path_buf(),
        raw_content,
        rendered_html,
        document.frontmatter,
        hash.to_owned(),
    ))
}

#[tracing::instrument]
fn out_path<T: AsRef<Path> + Debug>(path: T, output_directory: T, title: &str) -> Result<PathBuf> {
    let parent = path
        .as_ref()
        .parent()
        .context("Path should have a parent")?
        .file_name()
        .context("Path should have a filename")?
        .to_string_lossy();

    let filename = path
        .as_ref()
        .file_name()
        .context("Path should have a filename")?
        .to_string_lossy();

    let directory = output_directory.as_ref();

    match (parent.as_ref(), filename.as_ref()) {
        ("content", "index") => Ok(directory.join("index.html")),
        ("content", _) => Ok(directory.join(format!("{title}.html"))),
        (_, "index") => Ok(directory.join(parent.as_ref()).join("index.html")),
        ("posts", _) => Ok(directory
            .join(parent.as_ref())
            .join(format!("{title}.html"))),
        _ => Ok(directory
            .join("series")
            .join(parent.as_ref())
            .join(format!("{title}.html"))),
    }
}
