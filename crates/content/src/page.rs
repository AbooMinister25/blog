use std::{
    fmt::Debug,
    fs,
    hash::Hash,
    path::{Component, Path, PathBuf},
};

use chrono::{DateTime, Utc};
use color_eyre::{eyre::ContextCompat, Result};
use markdown::{Frontmatter, MarkdownRenderer};
use serde::{Deserialize, Serialize};
use sql::RetPostSQL;
use tera::{Context, Tera};
use tracing::trace;
use utils::fs::ensure_directory;

pub const DATE_FORMAT: &str = "%b %e, %Y";

/// Represents a single markdown page in the blog.
#[derive(Debug, Serialize, Deserialize, Eq, Clone)]
pub struct Page {
    pub path: PathBuf,
    pub title: String,
    pub tags: Vec<String>,
    pub permalink: String,
    #[serde(rename = "body")]
    pub raw_content: Option<String>,
    pub content: Option<String>,
    pub frontmatter: Option<Frontmatter>,
    pub toc: Option<Vec<String>>,
    pub summary: String,
    pub hash: String,
    pub date: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    #[serde(skip)]
    pub new: bool,
    pub index: bool,
}

impl From<RetPostSQL> for Page {
    fn from(value: RetPostSQL) -> Self {
        Self::new(
            value.path,
            value.title,
            value.tags,
            value.permalink,
            None,
            None,
            None,
            None,
            value.summary,
            value.hash,
            value.date,
            value.updated,
            value.new,
            false,
        )
    }
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

fn is_special_page<T: AsRef<Path>>(path: T, special_pages: &[String]) -> bool {
    special_pages
        .iter()
        .any(|ending| path.as_ref().ends_with(ending))
}

impl Page {
    #[tracing::instrument]
    pub fn new(
        path: PathBuf,
        title: String,
        tags: Vec<String>,
        permalink: String,
        raw_content: Option<String>,
        content: Option<String>,
        frontmatter: Option<Frontmatter>,
        toc: Option<Vec<String>>,
        summary: String,
        hash: String,
        date: DateTime<Utc>,
        updated: DateTime<Utc>,
        new: bool,
        index: bool,
    ) -> Self {
        Self {
            path,
            title,
            tags,
            permalink,
            raw_content,
            content,
            frontmatter,
            toc,
            summary,
            hash,
            date,
            updated,
            new,
            index,
        }
    }
}

#[tracing::instrument(skip(renderer))]
pub fn render_page<T: AsRef<Path> + Debug>(
    renderer: &MarkdownRenderer,
    url: &str,
    path: T,
    output_directory: T,
    raw_content: String,
    hash: &str,
    new: bool,
    special_pages: &[String],
) -> Result<Page> {
    trace!("Rendering page at {path:?}");

    let document = renderer.render(&raw_content)?;
    let out_path = out_path(&path, &output_directory, &document.frontmatter.title)?;
    ensure_directory(out_path.parent().context("Path should have a parent")?)?;

    let permalink = {
        let mut components = out_path.components();
        for c in components.by_ref() {
            if let Component::Normal(o) = c {
                if output_directory.as_ref().starts_with(o) {
                    break;
                }
            }
        }
        components.next_back();
        format!(
            "{url}{}",
            components
                .as_path()
                .to_str()
                .context("Path should be valid unicode")?,
        )
    };

    Ok(Page::new(
        out_path,
        document.frontmatter.title.clone(),
        document.frontmatter.tags.clone(),
        permalink,
        Some(raw_content),
        Some(document.content),
        Some(document.frontmatter),
        Some(document.toc),
        document.summary,
        hash.to_owned(),
        document.date,
        document.updated,
        new,
        is_special_page(path, special_pages),
    ))
}

#[tracing::instrument(skip(tera))]
pub fn write_page_to_disk(tera: &Tera, page: Page) -> Result<Page> {
    trace!("Rendering template for post at {:?}", page.path);

    let frontmatter = page
        .frontmatter
        .as_ref()
        .context("Page should have frontmatter at this point")?;

    let mut context = Context::new();
    context.insert("title", &frontmatter.title);
    context.insert("tags", &frontmatter.tags.join(", "));
    context.insert("series", &frontmatter.series);
    context.insert("date", &page.date.format(DATE_FORMAT).to_string());
    context.insert("toc", &page.toc);
    context.insert("markup", &page.content);
    context.insert("summary", &page.summary);
    context.insert("frontmatter", &page.frontmatter);

    let template = frontmatter
        .template
        .as_ref()
        .map_or("post.html.tera", |s| s);
    let rendered_html = tera.render(template, &context)?;

    trace!("Rendered template for post at {:?}", page.path);

    trace!("Writing page to {:?}", page.path);
    fs::write(&page.path, rendered_html)?;
    trace!("Wrote page to {:?}", page.path);

    Ok(page)
}

#[tracing::instrument(skip(tera))]
pub fn write_index_to_disk(
    tera: &Tera,
    page: &Page,
    posts: &[Page],
    index_pages: &[Page],
) -> Result<()> {
    trace!("Rendering template for post at {:?}", page.path);

    let frontmatter = page
        .frontmatter
        .as_ref()
        .context("Page should have frontmatter at this point")?;

    let mut context = Context::new();
    context.insert("title", &frontmatter.title);
    context.insert("tags", &frontmatter.tags.join(", "));
    context.insert("series", &frontmatter.series);
    context.insert("date", &page.date.format(DATE_FORMAT).to_string());
    context.insert("toc", &page.toc);
    context.insert("markup", &page.content);
    context.insert("summary", &page.summary);
    context.insert("posts", &posts);
    context.insert("index_pages", &index_pages);
    context.insert("frontmatter", &page.frontmatter);

    let template = frontmatter
        .template
        .as_ref()
        .map_or("post.html.tera", |s| s);
    let rendered_html = tera.render(template, &context)?;

    trace!("Rendered template for post at {:?}", page.path);

    trace!("Writing page to {:?}", page.path);
    fs::write(&page.path, rendered_html)?;
    trace!("Wrote page to {:?}", page.path);

    Ok(())
}

#[tracing::instrument]
fn out_path<T: AsRef<Path> + Debug>(path: T, output_directory: T, title: &str) -> Result<PathBuf> {
    let directory = output_directory.as_ref();

    let ending = if path.as_ref().ends_with("index.md") {
        PathBuf::from("index.html")
    } else {
        PathBuf::from(title.replace(' ', "-")).join("index.html")
    };

    let mut components = path.as_ref().components();
    components.next_back();

    Ok(directory
        .join(components.skip(2).collect::<PathBuf>())
        .join(ending))
}
