use std::{
    fmt::Debug,
    fs,
    hash::Hash,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
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
    pub permalink: String,
    #[serde(rename = "body")]
    pub raw_content: String,
    pub content: String,
    #[serde(flatten)]
    pub frontmatter: Frontmatter,
    pub summary: String,
    pub hash: String,
    pub date: DateTime<Utc>,
    #[serde(skip)]
    pub new: bool,
    pub index: bool,
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
        permalink: String,
        raw_content: String,
        content: String,
        frontmatter: Frontmatter,
        summary: String,
        hash: String,
        date: DateTime<Utc>,
        new: bool,
        index: bool,
    ) -> Self {
        Self {
            path,
            permalink,
            raw_content,
            content,
            frontmatter,
            summary,
            hash,
            date,
            new,
            index,
        }
    }
}

#[tracing::instrument(skip(renderer, tera))]
pub fn render_page<T: AsRef<Path> + Debug>(
    tera: &Tera,
    renderer: &MarkdownRenderer,
    url: &str,
    path: T,
    output_directory: T,
    raw_content: String,
    hash: &str,
    new: bool,
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
        format!("{url}/posts/{}", document.frontmatter.title),
        raw_content,
        rendered_html,
        document.frontmatter,
        document.summary,
        hash.to_owned(),
        document.date,
        new,
        path.as_ref().ends_with("index.md"),
    ))
}

#[tracing::instrument]
fn out_path<T: AsRef<Path> + Debug>(path: T, output_directory: T, title: &str) -> Result<PathBuf> {
    let directory = output_directory.as_ref();

    let filename = if path.as_ref().ends_with("index.md") {
        "index"
    } else {
        title
    };

    let mut components = path.as_ref().components();
    components.next_back();

    Ok(directory
        .join(components.skip(2).collect::<PathBuf>())
        .join(format!("{filename}.html")))
}
